use std::io::{Read, Write};

use crc::crc32;

use crate::archive::FirmwareArchive;
use crate::dfu::{DfuError, DfuRequest, DfuResponse, NoResponse, ObjectType};
use crate::protocol::*;

#[derive(Debug)]
pub enum Error {
    IOError(std::io::Error),
    DfuError(DfuError),
    PingMismatch,
    CrcMismatch,
}

pub struct Updater<'a, T: Read + Write> {
    comm: &'a mut T,
    prn: u16,
    chunk_size: usize,
}

impl<'a, T: Read + Write> Updater<'a, T> {
    pub fn new(comm: &'a mut T) -> Self {
        Self {
            comm,
            prn: 5,
            chunk_size: 0,
        }
    }

    fn request<'de, R: DfuRequest<'de>>(&mut self, request: R) -> Result<R::Response, Error> {
        request.dfu_write(&mut self.comm)?;
        let response = R::Response::dfu_read::<_, R>(&mut self.comm)?;
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

    pub fn update(&mut self, firmware: &FirmwareArchive) -> Result<(), Error> {
        let PingResponse { id } = self.request(PingRequest { id: 0x7F })?;
        if id != 0x7F {
            return Err(Error::PingMismatch);
        }

        self.request(SetReceiptNotifyRequest { target: self.prn })?;

        let GetMtuResponse { mtu } = self.request(GetMtuRequest)?;
        self.chunk_size = ((mtu / 2) - 1) as usize;

        self.transfer_object(ObjectType::Command, firmware.dat.as_slice())?;

        self.transfer_object(ObjectType::Data, firmware.bin.as_slice())?;

        self.request(AbortRequest)?;

        Ok(())
    }
}
