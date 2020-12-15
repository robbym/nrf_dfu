use serde::{Deserialize, Serialize};

use crate::dfu::{DfuRequest, DfuResponse, DfuSerialize, NoDataResponse, NoResponse, ObjectType};

// NRF_DFU_OP_PROTOCOL_VERSION
#[derive(Serialize)]
pub struct ProtocolVersionRequest;

impl DfuRequest<'_> for ProtocolVersionRequest {
    const REQUEST_OPCODE: u8 = 0x00;
    type Response = ProtocolVersionResponse;
}

#[derive(Deserialize, Debug)]
pub struct ProtocolVersionResponse {
    pub version: u8,
}

impl DfuResponse<'_> for ProtocolVersionResponse {}

// NRF_DFU_OP_OBJECT_CREATE
#[derive(Serialize)]
pub struct ObjectCreateRequest {
    pub object_type: ObjectType,
    pub object_size: u32,
}

impl DfuRequest<'_> for ObjectCreateRequest {
    const REQUEST_OPCODE: u8 = 0x01;
    type Response = NoDataResponse;
}

// NRF_DFU_OP_RECEIPT_NOTIF_SET
#[derive(Serialize)]
pub struct SetReceiptNotifyRequest {
    pub target: u16,
}

impl DfuRequest<'_> for SetReceiptNotifyRequest {
    const REQUEST_OPCODE: u8 = 0x02;
    type Response = NoDataResponse;
}

// NRF_DFU_OP_CRC_GET
#[derive(Serialize)]
pub struct GetCrcRequest;

impl DfuRequest<'_> for GetCrcRequest {
    const REQUEST_OPCODE: u8 = 0x03;
    type Response = GetCrcResponse;
}

#[derive(Deserialize, Debug)]
pub struct GetCrcResponse {
    pub offset: u32,
    pub crc: u32,
}

impl DfuResponse<'_> for GetCrcResponse {}

// NRF_DFU_OP_OBJECT_EXECUTE
#[derive(Serialize)]
pub struct ObjectExecuteRequest;

impl DfuRequest<'_> for ObjectExecuteRequest {
    const REQUEST_OPCODE: u8 = 0x04;
    type Response = NoDataResponse;
}

// NRF_DFU_OP_OBJECT_SELECT
#[derive(Serialize)]
pub struct ObjectSelectRequest {
    pub object_type: ObjectType,
}

impl DfuRequest<'_> for ObjectSelectRequest {
    const REQUEST_OPCODE: u8 = 0x06;
    type Response = ObjectSelectResponse;
}

#[derive(Deserialize, Debug)]
pub struct ObjectSelectResponse {
    pub max_size: u32,
    pub offset: u32,
    pub crc: u32,
}

impl DfuResponse<'_> for ObjectSelectResponse {}

// NRF_DFU_OP_MTU_GET
#[derive(Serialize)]
pub struct GetMtuRequest;

impl DfuRequest<'_> for GetMtuRequest {
    const REQUEST_OPCODE: u8 = 0x07;
    type Response = GetMtuResponse;
}

#[derive(Deserialize, Debug)]
pub struct GetMtuResponse {
    pub mtu: u16,
}

impl DfuResponse<'_> for GetMtuResponse {}

// NRF_DFU_OP_OBJECT_WRITE
pub struct ObjectWriteRequest<'de, T: DfuResponse<'de> = ObjectWriteResponse> {
    pub data: Vec<u8>,
    phantom: std::marker::PhantomData<&'de T>,
}

impl<'de, T: DfuResponse<'de>> DfuSerialize for ObjectWriteRequest<'de, T> {
    fn serialize(self) -> Vec<u8> {
        self.data
    }
}

impl<'de> DfuRequest<'de> for ObjectWriteRequest<'de, ObjectWriteResponse> {
    const REQUEST_OPCODE: u8 = 0x08;
    const RESPONSE_OPCODE: u8 = 0x03;
    type Response = ObjectWriteResponse;
}

impl<'de> DfuRequest<'de> for ObjectWriteRequest<'de, NoResponse> {
    const REQUEST_OPCODE: u8 = 0x08;
    type Response = NoResponse;
}

impl<'de, T: DfuResponse<'de>> ObjectWriteRequest<'de, T> {
    pub fn new(data: &[u8]) -> ObjectWriteRequest<'de, T> {
        ObjectWriteRequest {
            data: Vec::from(data),
            phantom: std::marker::PhantomData,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct ObjectWriteResponse {
    pub offset: u32,
    pub crc: u32,
}

impl DfuResponse<'_> for ObjectWriteResponse {}

// NRF_DFU_OP_PING
#[derive(Serialize)]
pub struct PingRequest {
    pub id: u8,
}

impl DfuRequest<'_> for PingRequest {
    const REQUEST_OPCODE: u8 = 0x09;
    type Response = PingResponse;
}

#[derive(Deserialize, Debug)]
pub struct PingResponse {
    pub id: u8,
}

impl DfuResponse<'_> for PingResponse {}

// NRF_DFU_OP_HARDWARE_VERSION
#[derive(Serialize)]
pub struct GetHardwareVersionRequest;

impl DfuRequest<'_> for GetHardwareVersionRequest {
    const REQUEST_OPCODE: u8 = 0x0A;
    type Response = GetHardwareVersionResponse;
}

#[derive(Deserialize, Debug)]
pub struct GetHardwareVersionResponse {
    pub part: u32,
    pub variant: u32,
    pub rom_size: u32,
    pub ram_size: u32,
    pub rom_page_size: u32,
}

impl DfuResponse<'_> for GetHardwareVersionResponse {}

// NRF_DFU_OP_FIRMWARE_VERSION
#[derive(Serialize)]
pub struct GetFirmwareVersionRequest {
    pub image: u8,
}

impl DfuRequest<'_> for GetFirmwareVersionRequest {
    const REQUEST_OPCODE: u8 = 0x0B;
    type Response = GetFirmwareVersionResponse;
}

#[derive(Deserialize, Debug)]
pub struct GetFirmwareVersionResponse {
    pub firmware_type: u8,
    pub version: u32,
    pub address: u32,
    pub length: u32,
}

impl DfuResponse<'_> for GetFirmwareVersionResponse {}

// NRF_DFU_OP_ABORT
#[derive(Serialize)]
pub struct AbortRequest;

impl DfuRequest<'_> for AbortRequest {
    const REQUEST_OPCODE: u8 = 0x0C;
    type Response = NoResponse;
}
