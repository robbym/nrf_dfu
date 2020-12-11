use crate::dfu::{DfuRequest, DfuResponse, NoDataResponse, NoResponse, ObjectType};
use crate::slip::SlipEncoder;
use crate::updater::Error;

// NRF_DFU_OP_PROTOCOL_VERSION
pub struct ProtocolVersionRequest;
impl Into<Vec<u8>> for ProtocolVersionRequest {
    fn into(self: Self) -> Vec<u8> {
        vec![]
    }
}
impl DfuRequest for ProtocolVersionRequest {
    const OPCODE: u8 = 0x00;
    type Response = ProtocolVersionResponse;
}
#[derive(Debug)]
pub struct ProtocolVersionResponse {
    pub version: u8,
}
impl From<Vec<u8>> for ProtocolVersionResponse {
    fn from(data: Vec<u8>) -> Self {
        ProtocolVersionResponse { version: data[0] }
    }
}
impl DfuResponse for ProtocolVersionResponse {}

// NRF_DFU_OP_OBJECT_CREATE
pub struct ObjectCreateRequest {
    pub object_type: ObjectType,
    pub object_size: u32,
}
impl Into<Vec<u8>> for ObjectCreateRequest {
    fn into(self: Self) -> Vec<u8> {
        let mut data = vec![];
        data.extend(&(self.object_type as u8).to_le_bytes());
        data.extend(&self.object_size.to_le_bytes());
        data
    }
}
impl DfuRequest for ObjectCreateRequest {
    const OPCODE: u8 = 0x01;
    type Response = NoDataResponse;
}

// NRF_DFU_OP_RECEIPT_NOTIF_SET
pub struct SetReceiptNotifyRequest {
    pub target: u16,
}
impl Into<Vec<u8>> for SetReceiptNotifyRequest {
    fn into(self: Self) -> Vec<u8> {
        let mut data = vec![];
        data.extend(&self.target.to_le_bytes());
        data
    }
}
impl DfuRequest for SetReceiptNotifyRequest {
    const OPCODE: u8 = 0x02;
    type Response = NoDataResponse;
}

// NRF_DFU_OP_CRC_GET
pub struct GetCrcRequest;

impl Into<Vec<u8>> for GetCrcRequest {
    fn into(self: Self) -> Vec<u8> {
        vec![]
    }
}
impl DfuRequest for GetCrcRequest {
    const OPCODE: u8 = 0x03;
    type Response = GetCrcResponse;
}
#[derive(Debug)]
pub struct GetCrcResponse {
    pub offset: u32,
    pub crc: u32,
}
impl From<Vec<u8>> for GetCrcResponse {
    fn from(data: Vec<u8>) -> Self {
        assert_eq!(data.len(), 8);

        let mut offset = [0u8; 4];
        let mut crc = [0u8; 4];

        offset.copy_from_slice(&data[0..4]);
        crc.copy_from_slice(&data[4..8]);

        let offset = u32::from_le_bytes(offset);
        let crc = u32::from_le_bytes(crc);

        GetCrcResponse { offset, crc }
    }
}
impl DfuResponse for GetCrcResponse {}

// NRF_DFU_OP_OBJECT_EXECUTE
pub struct ObjectExecuteRequest;

impl Into<Vec<u8>> for ObjectExecuteRequest {
    fn into(self: Self) -> Vec<u8> {
        vec![]
    }
}
impl DfuRequest for ObjectExecuteRequest {
    const OPCODE: u8 = 0x04;
    type Response = NoDataResponse;
}

// NRF_DFU_OP_OBJECT_SELECT
pub struct ObjectSelectRequest {
    pub object_type: ObjectType,
}

impl Into<Vec<u8>> for ObjectSelectRequest {
    fn into(self: Self) -> Vec<u8> {
        let mut data = vec![];
        data.extend(&(self.object_type as u8).to_le_bytes());
        data
    }
}
impl DfuRequest for ObjectSelectRequest {
    const OPCODE: u8 = 0x06;
    type Response = ObjectSelectResponse;
}
#[derive(Debug)]
pub struct ObjectSelectResponse {
    pub max_size: u32,
    pub offset: u32,
    pub crc: u32,
}
impl From<Vec<u8>> for ObjectSelectResponse {
    fn from(data: Vec<u8>) -> Self {
        assert_eq!(data.len(), 12);

        let mut max_size = [0u8; 4];
        let mut offset = [0u8; 4];
        let mut crc = [0u8; 4];

        max_size.copy_from_slice(&data[0..4]);
        offset.copy_from_slice(&data[4..8]);
        crc.copy_from_slice(&data[8..12]);

        let max_size = u32::from_le_bytes(max_size);
        let offset = u32::from_le_bytes(offset);
        let crc = u32::from_le_bytes(crc);

        ObjectSelectResponse {
            max_size,
            offset,
            crc,
        }
    }
}
impl DfuResponse for ObjectSelectResponse {}

// NRF_DFU_OP_MTU_GET
pub struct GetMtuRequest;

impl Into<Vec<u8>> for GetMtuRequest {
    fn into(self: Self) -> Vec<u8> {
        vec![]
    }
}
impl DfuRequest for GetMtuRequest {
    const OPCODE: u8 = 0x07;
    type Response = GetMtuResponse;
}
#[derive(Debug)]
pub struct GetMtuResponse {
    pub mtu: u16,
}
impl From<Vec<u8>> for GetMtuResponse {
    fn from(data: Vec<u8>) -> Self {
        assert_eq!(data.len(), 2);

        let mut mtu = [0u8; 2];

        mtu.copy_from_slice(&data[0..2]);

        let mtu = u16::from_le_bytes(mtu);

        GetMtuResponse { mtu }
    }
}
impl DfuResponse for GetMtuResponse {}

// NRF_DFU_OP_OBJECT_WRITE
pub struct ObjectWriteRequest<T: DfuResponse = ObjectWriteResponse> {
    pub data: Vec<u8>,
    phantom: std::marker::PhantomData<T>,
}
impl<T: DfuResponse> Into<Vec<u8>> for ObjectWriteRequest<T> {
    fn into(self: Self) -> Vec<u8> {
        self.data
    }
}
impl DfuRequest for ObjectWriteRequest<ObjectWriteResponse> {
    const OPCODE: u8 = 0x03;
    type Response = ObjectWriteResponse;

    fn dfu_write<T: SlipEncoder>(self, encoder: &mut T) -> Result<(), Error> {
        let mut request_data: Vec<u8> = vec![0x08];
        let data: Vec<u8> = self.into();
        request_data.extend(data);
        encoder.slip_write(&request_data)?;
        Ok(())
    }
}
impl DfuRequest for ObjectWriteRequest<NoResponse> {
    const OPCODE: u8 = 0x08;
    type Response = NoResponse;
}
impl<T: DfuResponse> ObjectWriteRequest<T> {
    pub fn new(data: &[u8]) -> ObjectWriteRequest<T> {
        ObjectWriteRequest {
            data: Vec::from(data),
            phantom: std::marker::PhantomData,
        }
    }
}
#[derive(Debug)]
pub struct ObjectWriteResponse {
    pub offset: u32,
    pub crc: u32,
}
impl From<Vec<u8>> for ObjectWriteResponse {
    fn from(data: Vec<u8>) -> Self {
        assert_eq!(data.len(), 8);

        let mut offset = [0u8; 4];
        let mut crc = [0u8; 4];

        offset.copy_from_slice(&data[0..4]);
        crc.copy_from_slice(&data[4..8]);

        let offset = u32::from_le_bytes(offset);
        let crc = u32::from_le_bytes(crc);

        ObjectWriteResponse { offset, crc }
    }
}
impl DfuResponse for ObjectWriteResponse {}

// NRF_DFU_OP_PING
pub struct PingRequest {
    pub id: u8,
}
impl Into<Vec<u8>> for PingRequest {
    fn into(self: Self) -> Vec<u8> {
        vec![self.id]
    }
}
impl DfuRequest for PingRequest {
    const OPCODE: u8 = 0x09;
    type Response = PingResponse;
}
#[derive(Debug)]
pub struct PingResponse {
    pub id: u8,
}
impl From<Vec<u8>> for PingResponse {
    fn from(data: Vec<u8>) -> Self {
        PingResponse { id: data[0] }
    }
}
impl DfuResponse for PingResponse {}

// NRF_DFU_OP_HARDWARE_VERSION
pub struct GetHardwareVersionRequest;

impl Into<Vec<u8>> for GetHardwareVersionRequest {
    fn into(self: Self) -> Vec<u8> {
        vec![]
    }
}
impl DfuRequest for GetHardwareVersionRequest {
    const OPCODE: u8 = 0x0A;
    type Response = GetHardwareVersionResponse;
}
#[derive(Debug)]
pub struct GetHardwareVersionResponse {
    pub part: u32,
    pub variant: u32,
    pub rom_size: u32,
    pub ram_size: u32,
    pub rom_page_size: u32,
}
impl From<Vec<u8>> for GetHardwareVersionResponse {
    fn from(data: Vec<u8>) -> Self {
        assert_eq!(data.len(), 20);

        let mut part = [0u8; 4];
        let mut variant = [0u8; 4];
        let mut rom_size = [0u8; 4];
        let mut ram_size = [0u8; 4];
        let mut rom_page_size = [0u8; 4];

        part.copy_from_slice(&data[0..4]);
        variant.copy_from_slice(&data[4..8]);
        rom_size.copy_from_slice(&data[8..12]);
        ram_size.copy_from_slice(&data[12..16]);
        rom_page_size.copy_from_slice(&data[16..20]);

        let part = u32::from_le_bytes(part);
        let variant = u32::from_le_bytes(variant);
        let rom_size = u32::from_le_bytes(rom_size);
        let ram_size = u32::from_le_bytes(ram_size);
        let rom_page_size = u32::from_le_bytes(rom_page_size);

        GetHardwareVersionResponse {
            part,
            variant,
            rom_size,
            ram_size,
            rom_page_size,
        }
    }
}
impl DfuResponse for GetHardwareVersionResponse {}

// NRF_DFU_OP_FIRMWARE_VERSION
pub struct GetFirmwareVersionRequest {
    pub image: u8,
}

impl Into<Vec<u8>> for GetFirmwareVersionRequest {
    fn into(self: Self) -> Vec<u8> {
        vec![self.image]
    }
}
impl DfuRequest for GetFirmwareVersionRequest {
    const OPCODE: u8 = 0x0B;
    type Response = GetFirmwareVersionResponse;
}
#[derive(Debug)]
pub struct GetFirmwareVersionResponse {
    pub firmware_type: u8,
    pub version: u32,
    pub address: u32,
    pub length: u32,
}
impl From<Vec<u8>> for GetFirmwareVersionResponse {
    fn from(data: Vec<u8>) -> Self {
        assert_eq!(data.len(), 13);

        let mut firmware_type = [0u8; 1];
        let mut version = [0u8; 4];
        let mut address = [0u8; 4];
        let mut length = [0u8; 4];

        firmware_type.copy_from_slice(&data[0..1]);
        version.copy_from_slice(&data[1..5]);
        address.copy_from_slice(&data[5..9]);
        length.copy_from_slice(&data[9..13]);

        let firmware_type = u8::from_le_bytes(firmware_type);
        let version = u32::from_le_bytes(version);
        let address = u32::from_le_bytes(address);
        let length = u32::from_le_bytes(length);

        GetFirmwareVersionResponse {
            firmware_type,
            version,
            address,
            length,
        }
    }
}
impl DfuResponse for GetFirmwareVersionResponse {}

// NRF_DFU_OP_ABORT
pub struct AbortRequest;

impl Into<Vec<u8>> for AbortRequest {
    fn into(self: Self) -> Vec<u8> {
        vec![]
    }
}
impl DfuRequest for AbortRequest {
    const OPCODE: u8 = 0x0C;
    type Response = NoResponse;
}
