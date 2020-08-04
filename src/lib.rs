#![feature(extend_one)]
use std::io::{self, Write, Read, Seek, SeekFrom, Bytes};
use std::fs::File;

use bytes::{BytesMut, BufMut, Buf, Bytes as BytesConst};
use tempfile::SpooledTempFile;


const MAX_SIZE: usize = 5 * 1_024 * 1_024;  // bytes
const START_INDEX: usize = 0;

type IoError = io::Error;
type Stream<T, E> = Box<dyn Iterator<Item = Result<T, E>>>;

pub struct StreamBuffer<T, E> {
    pub size: usize,
    pub index: usize,
    stream: Stream<T, E>,
    temp: SpooledTempFile,
}

impl<T, E> StreamBuffer<T, E> {
    fn new(
        stream: Stream<T, E>, 
        size: usize,
    ) -> StreamBuffer<T, E> {
        StreamBuffer {
            size,
            index: START_INDEX,
            stream,
            temp: SpooledTempFile::new(MAX_SIZE),
        }
    }
}

pub trait Buffer<I, O, E> {
    fn read(&mut self, offset: usize, size: usize) -> O;
}

impl<E> Buffer<u8, BytesMut, E> for StreamBuffer<u8, E> {
    fn read(&mut self, offset: usize, size: usize) -> BytesMut {
        let end = offset + size;
        let mut buf = BytesMut::new();

        let seek_offset = SeekFrom::Start(offset as u64);
        let seek_index = SeekFrom::Start(self.index as u64);
        
        self.temp.seek(seek_offset).unwrap();

        if offset < self.index && end <= self.index {
            self.temp.read(&mut buf).unwrap();
        } else if offset == self.index {
            while let Some(Ok(byte)) = self.stream.next() {
                let bytes = &[byte];
                self.index += bytes.len();

                self.temp.write(bytes).unwrap();
                buf.extend_one(byte);

                // if buf.len() >= size {
                //     buf.truncate(size);
                //     return buf;
                // }
            }
        } else if self.index < offset && offset <= self.size {            
            while let Some(Ok(byte)) = self.stream.next()  {
                let bytes = &[byte];
                self.index += bytes.len();

                self.temp.write(bytes).unwrap();

                if self.index >= offset {
                    buf.extend_one(byte);
                }

                // if buf.len() >= size {
                //     buf.truncate(size);
                //     return buf;
                // }
            }
        } else if offset < self.index && self.index < end {
            // self.temp.seek(seek_offset).unwrap();
            self.temp.read(&mut buf).unwrap();
        }

        buf.truncate(size);
        buf
    }
}


#[cfg(test)]
mod tests {
    use std::io::{Read, Bytes};
    use std::fs::File;
    
    use super::*;

    #[test]
    fn test_create() {
        let mut file = File::open("test.txt").unwrap();
        let bytes = Box::new(file.bytes());

        let mut stream_buffer = 
          StreamBuffer::new(bytes, 25);
    }

    #[test]
    fn test_forward() {
        let mut file = File::open("test.txt").unwrap();
        let bytes = Box::new(file.bytes());

        let mut stream_buffer = 
          StreamBuffer::new(bytes, 25);
    
        let result = stream_buffer.read(0, 5);
        assert_eq!(result.bytes(), b"0\n1\n2");

        let result = stream_buffer.read(0, 5);
        assert_eq!(result.bytes(), b"\n3\n4\n");
    }

    // #[test]
    fn test_backward() {
        let mut file = File::open("test.txt").unwrap();
        let bytes = Box::new(file.bytes());

        let mut stream_buffer = 
          StreamBuffer::new(bytes, 25);
    
        let result = stream_buffer.read(0, 5);
        assert_eq!(result.bytes(), b"0\n1\n2");

        let result = stream_buffer.read(5, 5);
        assert_eq!(result.bytes(), b"\n3\n4\n");

        let result = stream_buffer.read(0, 5);
        assert_eq!(result.bytes(), b"0\n1\n2");
    }
}
