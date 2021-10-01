use std::io::{Read, Write};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_repr::*;

use crate::codec::DfuCodec;
use crate::updater::Error;

#[derive(Debug)]
pub enum DfuError {
    InvalidOpcode,
    OpcodeNotSupported,
    InvalidParameter,
    InsufficientResources,
    InvalidObject,
    UnsupportedType,
    OperationNotPermitted,
    OperationFailed,
    ExtendedError,
    UnknownError,
}

#[derive(Serialize_repr, Deserialize_repr, Copy, Clone)]
#[repr(u8)]
pub enum ObjectType {
    Command = 0x01,
    Data = 0x02,
}

impl From<u8> for DfuError {
    fn from(err_code: u8) -> DfuError {
        match err_code {
            0x00 => DfuError::InvalidOpcode,
            0x02 => DfuError::OpcodeNotSupported,
            0x03 => DfuError::InvalidParameter,
            0x04 => DfuError::InsufficientResources,
            0x05 => DfuError::InvalidObject,
            0x06 => DfuError::UnsupportedType,
            0x07 => DfuError::OperationNotPermitted,
            0x08 => DfuError::OperationFailed,
            0x09 => DfuError::ExtendedError,
            _ => DfuError::UnknownError,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::IOError(err)
    }
}

impl From<DfuError> for Error {
    fn from(err: DfuError) -> Error {
        Error::DfuError(err)
    }
}

pub trait DfuSerialize {
    fn serialize(self) -> Vec<u8>;
}

impl<T: Serialize> DfuSerialize for T {
    fn serialize(self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }
}

pub trait DfuRequest<'de>: Sized + DfuSerialize {
    const REQUEST_OPCODE: u8;
    const RESPONSE_OPCODE: u8 = Self::REQUEST_OPCODE;
    type Response: DfuResponse<'de>;

    fn dfu_write<Writer: Write, Codec: DfuCodec>(self, writer: &mut Writer) -> Result<(), Error> {
        let mut request_data = vec![Self::REQUEST_OPCODE];
        request_data.extend_from_slice(&self.serialize());
        Codec::encoded_write(writer, &request_data)?;
        Ok(())
    }
}

pub trait DfuResponse<'de>: Sized + DeserializeOwned {
    fn dfu_read<Reader: Read, Codec: DfuCodec, Request: DfuRequest<'de>>(reader: &mut Reader) -> Result<Self, Error> {
        let response = Codec::decoded_read(reader)?;

        assert!(response.len() >= 2);

        if response[0] != Request::RESPONSE_OPCODE {
            return Err(Error::DfuError(DfuError::InvalidOpcode));
        }
        if response[1] != 1 {
            Err(Error::DfuError(DfuError::from(response[1])))
        } else {
            Ok(bincode::deserialize(&response[2..]).unwrap())
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct NoResponse;

impl<'de> DfuResponse<'de> for NoResponse {
    fn dfu_read<Reader: Read, Codec: DfuCodec, Request: DfuRequest<'de>>(_reader: &mut Reader) -> Result<Self, Error> {
        Ok(NoResponse)
    }
}

#[derive(Deserialize, Debug)]
pub struct NoDataResponse;

impl DfuResponse<'_> for NoDataResponse {}
