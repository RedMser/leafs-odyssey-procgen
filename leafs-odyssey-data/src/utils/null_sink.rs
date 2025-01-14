use std::io::{self, Seek, SeekFrom, Write};

pub struct NullSink {
    position: u64,
}

impl NullSink {
    pub fn new() -> Self {
        Self { position: 0 }
    }
}

impl Write for NullSink {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.position += buf.len() as u64;
        Ok(buf.len()) // Pretend to write all data
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(()) // No-op
    }
}

impl Seek for NullSink {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        match pos {
            SeekFrom::Start(offset) => {
                self.position = offset;
            }
            SeekFrom::End(offset) => {
                if offset < 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Negative offset from end is not supported in NullSink",
                    ));
                }
                self.position = offset as u64;
            }
            SeekFrom::Current(offset) => {
                if offset < 0 {
                    let abs_offset = (-offset) as u64;
                    if abs_offset > self.position {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "Seeking before the start of the stream is not allowed",
                        ));
                    }
                    self.position -= abs_offset;
                } else {
                    self.position += offset as u64;
                }
            }
        }
        Ok(self.position)
    }
}