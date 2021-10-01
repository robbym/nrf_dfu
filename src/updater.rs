use std::thread;
use std::time::Duration;
use std::io::{Read, Write};

use crc::crc32;

use crate::archive::{FirmwareArchive, FirmwareData};
use crate::codec::DfuCodec;
use crate::dfu::{DfuError, DfuRequest, DfuResponse, NoResponse, ObjectType};
use crate::protocol::*;

#[derive(Debug)]
pub enum Error {
    IOError(std::io::Error),
    DfuError(DfuError),
    PingMismatch,
    CrcMismatch,
}

pub enum ResetMode {
    Bootloader,
    Application,
}

pub trait NordicDevice: Read + Write {
    type Codec: DfuCodec;
    fn reset(&mut self, mode: ResetMode);
}

pub struct Updater<'a, T: NordicDevice> {
    comm: &'a mut T,
    prn: u16,
    chunk_size: usize,
    force: bool,
}

impl<'a, T: NordicDevice> Updater<'a, T> {
    pub fn new(comm: &'a mut T, force: bool) -> Self {
        Self {
            comm,
            prn: 5,
            chunk_size: 0,
            force,
        }
    }

    fn request<'de, Request: DfuRequest<'de>>(&mut self, request: Request) -> Result<Request::Response, Error> {
        request.dfu_write::<T, T::Codec>(self.comm)?;
        let response = Request::Response::dfu_read::<T, T::Codec, Request>(self.comm)?;
        Ok(response)
    }

    fn write_object(&mut self, mut object_crc: u32, data: &[u8]) -> Result<u32, Error> {
        let mut prn_count = 0;

        for chunk in data.chunks(self.chunk_size) {
            object_crc = crc32::update(object_crc, &crc32::IEEE_TABLE, chunk);

            if self.prn > 0 {
                if prn_count < self.prn - 1 {
                    prn_count += 1;
                    self.request(ObjectWriteRequest::<NoResponse>::new(chunk))?;
                } else {
                    prn_count = 0;
                    let ObjectWriteResponse { offset: _, crc } =
                        self.request(ObjectWriteRequest::<ObjectWriteResponse>::new(chunk))?;
                    if crc != object_crc {
                        return Err(Error::CrcMismatch);
                    }
                }
            } else {
                self.request(ObjectWriteRequest::<NoResponse>::new(chunk))?;
            }
        }

        Ok(object_crc)
    }

    fn transfer_object(&mut self, object_type: ObjectType, data: &[u8]) -> Result<(), Error> {
        let ObjectSelectResponse {
            max_size,
            offset,
            crc,
        } = self.request(ObjectSelectRequest { object_type })?;

        let object_max_size = max_size as usize;
        let mut object_offset = offset as usize;
        let mut object_crc = crc;
        let firmware_crc = crc32::checksum_ieee(data);

        if self.force {
            object_offset = 0;
            object_crc = 0;
        }

        loop {
            if (object_offset > 0 && (object_offset % object_max_size) == 0)
                || (object_offset == data.len() && object_crc == firmware_crc)
            {
                self.request(ObjectExecuteRequest)?;

                if object_offset == data.len() {
                    break;
                }
            }

            let mut object_end =
                object_offset - (object_offset % object_max_size) + object_max_size;
            if object_end > data.len() {
                object_end = data.len();
            }

            if (object_offset % object_max_size) == 0
                || object_crc != crc32::checksum_ieee(&data[0..object_offset])
            {
                self.request(ObjectCreateRequest {
                    object_type,
                    object_size: (object_end - object_offset) as u32,
                })?;
            }

            object_crc = self.write_object(object_crc, &data[object_offset..object_end])?;

            let GetCrcResponse { offset, crc } = self.request(GetCrcRequest)?;
            object_offset = offset as usize;
            if crc != object_crc {
                return Err(Error::CrcMismatch);
            }
        }

        Ok(())
    }

    fn update_module(&mut self, firmware: &FirmwareData) -> Result<(), Error> {
        match self.request(PingRequest { id: 0x7F }) {
            Ok(PingResponse { id }) => {
                if id != 0x7F {
                    return Err(Error::PingMismatch);
                }
            }
            Err(Error::DfuError(DfuError::OpcodeNotSupported)) => {}
            Err(err) => {
                return Err(err);
            }
        }

        self.request(SetReceiptNotifyRequest { target: self.prn })?;

        match self.request(GetMtuRequest) {
            Ok(GetMtuResponse { mtu }) => {
                self.chunk_size = ((mtu / 2) - 1) as usize;
            }
            Err(Error::DfuError(DfuError::OpcodeNotSupported)) => {
                self.chunk_size = 223;
            }
            Err(err) => {
                return Err(err);
            }
        }

        self.transfer_object(ObjectType::Command, firmware.dat.as_slice())?;

        self.transfer_object(ObjectType::Data, firmware.bin.as_slice())?;

        Ok(())
    }

    pub fn update(&mut self, firmware: &FirmwareArchive) -> Result<(), Error> {
        if let Some(softdevice_bootloader) = &firmware.softdevice_bootloader {
            if let Err(err) = self.update_module(&softdevice_bootloader) {
                self.request(AbortRequest)?;
                return Err(err);
            }
            thread::sleep(Duration::from_millis(1000));
            self.comm.reset(ResetMode::Bootloader);
        } else if let Some(bootloader) = &firmware.bootloader {
            if let Err(err) = self.update_module(&bootloader) {
                self.request(AbortRequest)?;
                return Err(err);
            }
            thread::sleep(Duration::from_millis(500));
            self.comm.reset(ResetMode::Bootloader);
        }

        if let Some(application) = &firmware.application {
            if let Err(err) = self.update_module(&application) {
                self.request(AbortRequest)?;
                return Err(err);
            }
            thread::sleep(Duration::from_millis(500));
        }

        Ok(())
    }
    pub fn get_firmware_version(&mut self) -> Result<u32, Error> {
        let GetFirmwareVersionResponse {
            firmware_type: _,
            version,
            address: _,
            length: _,
        } = self.request(GetFirmwareVersionRequest { image: 2 })?;
        return Ok(version);
    }
}
