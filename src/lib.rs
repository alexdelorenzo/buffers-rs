#![feature(extend_one)]
// #![feature(trait_alias)]
use std::io::prelude::*;
use std::io::{BufWriter, BufReader, BufRead};
use std::io::{Write, Read, Seek, SeekFrom, Bytes};
use std::io::{Error as IoError};
use std::ops::Range;

// use bufstream::{BufStream, };
use tempfile::{SpooledTempFile};

const START_INDEX: usize = 0;
const NO_CAPACITY: usize = 0;
const ZERO_BYTE: u8 = 0u8;
const INCREMENT: usize = 1;

const MAX_SIZE: usize = 5 * 1_024 * 1_024;  // bytes

enum Location {
    BeforeIndex,
    Bisected,
    AtIndex,
    AfterIndex,
}

// pub trait FileLike = Read + Seek + Write;
pub trait FileLike: Read + Seek + Write {}
impl<T: Read + Seek + Write> FileLike for T {}
type FileType = Box<dyn FileLike>;

type Byte = u8;
type ByteBuf = Vec<Byte>;
type ByteBufResult = Result<ByteBuf, IoError>;
type ByteStreamBuf<E, F> = StreamBuffer<Byte, E, F>;
type Stream<T, E> = Box<dyn Iterator<Item = Result<T, E>>>;

pub trait Buffer {}

pub trait BufferCreate<T, E, F: FileLike>: Buffer {
    fn from_file(
        stream: Stream<T, E>, 
        size: usize, 
        file: F
    ) -> StreamBuffer<T, E, F>;
}

pub trait BufferRead<O, E>: Buffer {
    fn read(&mut self, offset: usize, size: usize) -> Result<O, E>;
}

trait ChunkLocation {
    fn _chunk_location(&self, offset: usize, end: usize) -> Location;
}

trait ChunkRead {
    fn _chunk_before_index(&mut self, offset: usize, size: usize) -> ByteBufResult;
    fn _chunk_bisected_by_index(&mut self,  offset: usize, size: usize) -> ByteBufResult;
    fn _chunk_at_index(&mut self, size: usize) -> ByteBufResult;
    fn _chunk_after_index(&mut self, offset: usize, size: usize) -> ByteBufResult;
}

pub struct StreamBuffer<T, E, F: FileLike> {
    pub size: usize,
    pub index: usize,
    pub stream: Stream<T, E>,
    pub file: F,
}

impl<T, E> StreamBuffer<T, E, FileType> {
    pub fn new(
        stream: Stream<T, E>, 
        size: usize,
    ) -> StreamBuffer<T, E, SpooledTempFile> {
        let file = SpooledTempFile::new(MAX_SIZE);

        StreamBuffer {
            size,
            stream,
            file: file,
            index: START_INDEX,
        }
    }
}

impl<T, E, F: FileLike> Buffer for StreamBuffer<T, E, F> {}

impl<T, E, F: FileLike> BufferCreate<T, E, F> for StreamBuffer<T, E, F> {
    fn from_file(
        stream: Stream<T, E>, 
        size: usize,
        file: F,
    ) -> StreamBuffer<T, E, F> {
        StreamBuffer {
            size,
            stream,
            file,
            index: START_INDEX,
        }
    }
}

impl<E, F: FileLike> ChunkRead for ByteStreamBuf<E, F> {
    fn _chunk_before_index(&mut self, offset: usize, size: usize) -> ByteBufResult {
        let seek_offset = SeekFrom::Start(offset as u64);
        let mut buf = get_sized_vec(size);

        self.file.seek(seek_offset)?;
        self.file.read(&mut buf)?;

        return Ok(buf);
    }

    fn _chunk_bisected_by_index(&mut self,  offset: usize, size: usize) -> ByteBufResult {
        let existing_chunk = self.index - offset;
        let mut buf = 
            self._chunk_before_index(offset, existing_chunk)?;
        
        let new_chunk = size - buf.len();
        let new_buf = self._chunk_at_index(new_chunk)?;
        buf.extend(new_buf);

        return Ok(buf);
    }

    fn _chunk_at_index(&mut self, size: usize) -> ByteBufResult {
        let mut buf = get_no_capacity_vec();

        while let Some(Ok(byte)) = self.stream.next() {
            let bytes = &[byte];
            self.file.write(bytes)?;
            self.index += INCREMENT;

            buf.extend_one(byte);

            if buf.len() >= size {
                buf.truncate(size);
                break;
            }
        }

        return Ok(buf);
    }

    fn _chunk_after_index(&mut self, offset: usize, size: usize) -> ByteBufResult {
        let mut buf = get_no_capacity_vec();
        let seek_index = SeekFrom::Start(self.index as u64);
        self.file.seek(seek_index)?;

        while let Some(Ok(byte)) = self.stream.next()  {
            let bytes = &[byte];
            self.file.write(bytes)?;
            self.index += INCREMENT;

            if self.index >= offset {
                buf.extend_one(byte);
            }

            if buf.len() >= size {
                buf.truncate(size);
                return Ok(buf);
            }
        }

        return Ok(buf);
    }
}

impl<E, F: FileLike> ChunkLocation for ByteStreamBuf<E, F> {
    fn _chunk_location(&self, offset: usize, end: usize) -> Location {
        if offset < self.index && end <= self.index {
            Location::BeforeIndex
        } else if offset < self.index && self.index < end {
            Location::Bisected
        } else if offset == self.index {
            Location::AtIndex
        } else {
            Location::AfterIndex
        }
    }
}

impl<E, F: FileLike> BufferRead<ByteBuf, IoError> for ByteStreamBuf<E, F> {
    fn read(&mut self, offset: usize, size: usize) -> ByteBufResult {
        let end = offset + size;

        match self._chunk_location(offset, end) {
            Location::BeforeIndex => self._chunk_before_index(offset, size),
            Location::Bisected => self._chunk_bisected_by_index(offset, size),
            Location::AtIndex => self._chunk_at_index(size),
            Location::AfterIndex => self._chunk_after_index(offset, size)
        }
    }
}

fn get_sized_vec(size: usize) -> Vec<Byte> {
    let mut vec = Vec::with_capacity(size);
    vec.resize(size, ZERO_BYTE);
    vec
}

fn get_no_capacity_vec<T>() -> Vec<T> {
    Vec::with_capacity(NO_CAPACITY)
}

#[cfg(test)]
mod tests {
    use tempfile::{SpooledTempFile};
    use std::io::{Read, Bytes};
    use std::fs::File;
    use std::iter::repeat;
    use super::*;

    const START: usize = 0;
    const LEN: usize = 25;
    const TEST_FILE: &str = "test.txt";
    const BUF_SIZE: usize = 5;

    #[test]
    fn test_create() {
        let file = File::open(TEST_FILE).unwrap();
        let bytes = Box::new(file.bytes());

        let stream_buffer = 
          ByteStreamBuf::new(bytes, LEN);
    }

    #[test]
    fn test_chunk_from_0() {
        let file = File::open(TEST_FILE).unwrap();
        let bytes = Box::new(file.bytes());

        let mut stream_buffer = 
          StreamBuffer::new(bytes, LEN);
    
        let result = stream_buffer.read(START, BUF_SIZE).unwrap();
        assert_eq!(result, b"0\n1\n2");
    }

    #[test]
    fn test_forward() {
        let file = File::open(TEST_FILE).unwrap();
        let bytes = Box::new(file.bytes());

        let mut stream_buffer = 
          StreamBuffer::new(bytes, LEN);
    
        let result = stream_buffer.read(START, BUF_SIZE).unwrap();
        assert_eq!(result, b"0\n1\n2");

        let result = stream_buffer.read(5, BUF_SIZE).unwrap();
        assert_eq!(result, b"\n3\n4\n");
    }
    #[test]
    fn test_vec_iter() {
        let file = File::open(TEST_FILE).unwrap();
        let bytes = Box::new(file.bytes());
        let vec: Vec<Result<u8, IoError>> = bytes.collect();
        // let mut stream_buffer = 
        //   StreamBuffer::new(Box::new(vec.item()), LEN);    
    }

    #[test]
    fn test_stutter() {
        let file = File::open(TEST_FILE).unwrap();
        let bytes = Box::new(file.bytes());

        let mut stream_buffer = 
          StreamBuffer::new(bytes, LEN);
    
        let result = stream_buffer.read(START, BUF_SIZE).unwrap();
        assert_eq!(result, b"0\n1\n2");
        assert_eq!(stream_buffer.index, BUF_SIZE);

        let slice =  b"2\n3\n";
        let result = stream_buffer.read(4, 4).unwrap();
        assert_eq!(result, slice);

        // let result = stream_buffer.read(4, 4).unwrap();
        stream_buffer.file.seek(SeekFrom::Start(0usize as u64)).unwrap();
        // assert_eq!(stream_buffer.index, 8);

        let result = stream_buffer.read(4, 4).unwrap();
        stream_buffer.file.seek(SeekFrom::Start(0usize as u64)).unwrap();
        assert_eq!(stream_buffer.index, 8);

        let mut buf = String::new();
        stream_buffer.file.read_to_string(&mut buf).unwrap();
        let s = "0\n1\n2\n3\n".to_string();
        assert_eq!(buf, s);

        assert_eq!(result, slice);
    }

    #[test]
    fn test_inject_file_dependency() {
        let temp = SpooledTempFile::new(MAX_SIZE);

        let file = File::open(TEST_FILE).unwrap();
        let bytes = Box::new(file.bytes());

        let buf = StreamBuffer::from_file(bytes, LEN, temp);
    }

    #[test]
    fn test_backward() {
        let file = File::open(TEST_FILE).unwrap();
        let bytes = Box::new(file.bytes());

        let mut stream_buffer = 
          StreamBuffer::new(bytes, LEN);
    
        let result = 
            stream_buffer.read(START, BUF_SIZE).unwrap();
        assert_eq!(result, b"0\n1\n2");

        let result = stream_buffer.read(5, BUF_SIZE).unwrap();
        assert_eq!(result, b"\n3\n4\n");

        let result = stream_buffer.read(START, BUF_SIZE).unwrap();
        assert_eq!(result, b"0\n1\n2");
    }

    #[test]
    fn test_same_chunk_from_0() {
        let file = File::open(TEST_FILE).unwrap();
        let bytes = Box::new(file.bytes());

        let mut stream_buffer = 
            StreamBuffer::new(bytes, LEN);
    
        let result = stream_buffer.read(START, BUF_SIZE).unwrap();
        assert_eq!(result, b"0\n1\n2");

        let result = stream_buffer.read(START, BUF_SIZE).unwrap();
        assert_eq!(result, b"0\n1\n2");
    }

    #[test]
    fn test_forward_then_extend_from_0() {
        let file = File::open(TEST_FILE).unwrap();
        let bytes = Box::new(file.bytes());
    
        let mut buffer = 
            StreamBuffer::new(bytes, LEN);
    
        let result = buffer.read(START, BUF_SIZE).unwrap();
        assert_eq!(result, b"0\n1\n2");
    
        let result = buffer.read(START, BUF_SIZE).unwrap();
        assert_eq!(result, b"0\n1\n2");

        let result = buffer.read(START, 10).unwrap();
        assert_eq!(result, b"0\n1\n2\n3\n4\n");
    }

    #[test]
    fn test_forward_skip_ahead() {
        let file = File::open(TEST_FILE).unwrap();
        let bytes = Box::new(file.bytes());
    
        let mut buffer = 
            StreamBuffer::new(bytes, LEN);
    
        let result = buffer.read(START, BUF_SIZE).unwrap();
        assert_eq!(result, b"0\n1\n2");
        
        let result = buffer.read(8, 3).unwrap();
        assert_eq!(result, b"\n4\n");
    }

    #[test]
    fn test_tempfile() {
        let mut temp = SpooledTempFile::new(MAX_SIZE);
        let data: [u8; 10] = 
          [0, 1, 2, 3, 4,
           5, 6, 7, 8, 9];
        temp.write(&data).unwrap();
        temp.seek(SeekFrom::Start(0u64)).unwrap();
        let mut buf = [0u8; 10];
        temp.read(&mut buf).unwrap();
        assert_eq!(buf, data);
    }

    #[test]
    fn test_old_main_fn() {
        let file = File::open(TEST_FILE).unwrap();
        let bytes = Box::new(file.bytes());
    
        let mut buffer: StreamBuffer<_, _, SpooledTempFile> = 
            StreamBuffer::new(bytes, LEN);
    
        let result = buffer.read(4, 4).unwrap();
    
        let result = buffer.read(START, BUF_SIZE).unwrap();
        assert_eq!(result, b"0\n1\n2");
    
        let result = buffer.read(START, BUF_SIZE).unwrap();
        assert_eq!(result, b"0\n1\n2");
    
        buffer.file.seek(SeekFrom::Start(START as u64)).unwrap();
        let mut buf: Vec<u8> = repeat(ZERO_BYTE).take(10).collect();
    
        buffer.file.read(&mut buf).unwrap();
    }
}