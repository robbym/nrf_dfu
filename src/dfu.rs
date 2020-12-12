use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_repr::*;

use crate::slip::SlipEncoder;
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

pub(crate) fn dfu_write_impl<T: SlipEncoder>(
    encoder: &mut T,
    opcode: u8,
    data: &[u8],
) -> Result<(), Error> {
    let mut request_data = vec![opcode];
    request_data.extend_from_slice(data);
    encoder.slip_write(&request_data)?;
    Ok(())
}

pub trait DfuRequest<'de>: Sized + Serialize {
    const OPCODE: u8;
    type Response: DfuResponse<'de>;

    fn dfu_write<T: SlipEncoder>(self, encoder: &mut T) -> Result<(), Error> {
        dfu_write_impl(encoder, Self::OPCODE, &bincode::serialize(&self).unwrap())
    }
}

pub trait DfuResponse<'de>: Sized + DeserializeOwned {
    fn dfu_read<T: SlipEncoder, R: DfuRequest<'de>>(encoder: &mut T) -> Result<Self, Error> {
        let response = encoder.slip_read()?;

        assert!(response.len() >= 2);

        if response[0] != R::OPCODE {
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

impl From<Vec<u8>> for NoResponse {
    fn from(_data: Vec<u8>) -> Self {
        NoResponse
    }
}

impl<'de> DfuResponse<'de> for NoResponse {
    fn dfu_read<T: SlipEncoder, R: DfuRequest<'de>>(_encoder: &mut T) -> Result<Self, Error> {
        Ok(NoResponse)
    }
}

#[derive(Deserialize, Debug)]
pub struct NoDataResponse;

impl From<Vec<u8>> for NoDataResponse {
    fn from(_data: Vec<u8>) -> Self {
        NoDataResponse
    }
}

impl DfuResponse<'_> for NoDataResponse {}
