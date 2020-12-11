use std::io::{Read, Write};
use std::iter;

const SLIP_END: u8 = 0xC0;
const SLIP_ESC: u8 = 0xDB;
const SLIP_ESC_END: [u8; 2] = [0xDB, 0xDC];
const SLIP_ESC_ESC: [u8; 2] = [0xDB, 0xDD];

pub trait SlipEncoder: Read + Write {
    fn slip_read(&mut self) -> std::io::Result<Vec<u8>> {
        let mut data = vec![];
        let mut byte = [0u8; 1];

        self.read_exact(&mut byte)?;
        if byte[0] == 0x60 {
            loop {
                self.read_exact(&mut byte)?;
                match byte[0] {
                    SLIP_ESC => {
                        self.read_exact(&mut byte)?;
                        match byte[0] {
                            0xDC => data.push(SLIP_END),
                            0xDD => data.push(SLIP_ESC),
                            _ => {}
                        }
                    }

                    SLIP_END => {
                        return Ok(data);
                    }

                    x => {
                        data.push(x);
                    }
                }
            }
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Expected byte: 0x60",
            ))
        }
    }

    fn slip_write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let slip_frame = buf
            .iter()
            .flat_map(|x| match *x {
                SLIP_END => Vec::from(SLIP_ESC_END),
                SLIP_ESC => Vec::from(SLIP_ESC_ESC),
                _ => Vec::from([*x]),
            })
            .chain(iter::once(SLIP_END))
            .collect::<Vec<_>>();

        let size = self.write(&slip_frame)?;
        self.flush()?;
        Ok(size)
    }
}

impl<T> SlipEncoder for T where T: Read + Write {}
