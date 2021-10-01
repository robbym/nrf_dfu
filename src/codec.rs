use std::io::{Read, Write};

pub trait DfuCodec {
    fn decoded_read<T: Read>(reader: &mut T) -> std::io::Result<Vec<u8>>;
    fn encoded_write<T: Write>(writer: &mut T, buf: &[u8]) -> std::io::Result<usize>;
}