#![feature(type_alias_impl_trait)]
#![feature(trait_alias)]
#![feature(extend_one)]

mod buf;
mod chunk;
mod constants;
mod types;
mod utils;

pub use buf::StreamBuffer;

#[cfg(test)]
mod tests {
    use std::io::{Read, SeekFrom};
    use std::fs::File;
    use std::iter::repeat;

    use itertools::Itertools;
    use tempfile::{SpooledTempFile};

    use constants::{ZERO_BYTE, MAX_SIZE};
    use buf::{StreamBuffer, ByteStreamBuf, BufferRead, BufferCreate};
    use types::*;

    use super::*;

    const START: usize = 0;
    const LEN: usize = 25;
    const TEST_FILE: &str = "test.txt";
    const BUF_SIZE: usize = 5;

    #[test]
    fn create() {
        let file = File::open(TEST_FILE).unwrap();
        let bytes = Box::new(file.bytes());

        let _stream_buffer = 
          ByteStreamBuf::new(bytes, LEN);
    }

    #[test]
    fn chunk_from_0() {
        let file = File::open(TEST_FILE).unwrap();
        let bytes = Box::new(file.bytes());

        let mut stream_buffer = 
          StreamBuffer::new(bytes, LEN);
    
        let result = stream_buffer.read(START, BUF_SIZE).unwrap();
        assert_eq!(result, b"0\n1\n2");
    }

    #[test]
    fn forward() {
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
    fn stutter() {
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
        stream_buffer.file.seek(SeekFrom::Start(0u64)).unwrap();
        // assert_eq!(stream_buffer.index, 8);

        let result = stream_buffer.read(4, 4).unwrap();
        // test type casting as well
        stream_buffer.file.seek(SeekFrom::Start(0usize as u64)).unwrap();
        assert_eq!(stream_buffer.index, 8);

        let mut buf = String::new();
        stream_buffer.file.read_to_string(&mut buf).unwrap();
        let s = "0\n1\n2\n3\n".to_string();
        assert_eq!(buf, s);

        assert_eq!(result, slice);
    }

    #[test]
    fn inject_file_dependency() {
        let temp = SpooledTempFile::new(MAX_SIZE);

        let file = File::open(TEST_FILE).unwrap();
        let bytes = Box::new(file.bytes());

        let _buf = StreamBuffer::from_file(bytes, LEN, temp);
    }

    #[test]
    fn backwards() {
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
    fn same_chunk_from_0() {
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
    fn forward_then_extend_from_0() {
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
    fn forward_skip_ahead() {
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
    fn tempfile() {
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
    fn vec_iter() {
        let file = File::open(TEST_FILE).unwrap();
        let bytes = Box::new(file.bytes());
        let _vec: Vec<ByteResult> = bytes.collect(); 
    }

    // moved main() to a test
    #[test]
    fn old_main_fn() {
        let file = File::open(TEST_FILE).unwrap();
        let bytes = Box::new(file.bytes());
    
        let mut buffer: StreamBuffer<_, SpooledTempFile> = 
            StreamBuffer::new(bytes, LEN);
    
        let _result = buffer.read(4, 4).unwrap();
    
        let result = buffer.read(START, BUF_SIZE).unwrap();
        assert_eq!(result, b"0\n1\n2");
    
        let result = buffer.read(START, BUF_SIZE).unwrap();
        assert_eq!(result, b"0\n1\n2");
    
        buffer.file.seek(SeekFrom::Start(START as u64)).unwrap();
        let mut buf: Vec<u8> = repeat(ZERO_BYTE).take(10).collect();
        buffer.file.read(&mut buf).unwrap();

        let buf: Vec<u32> = repeat(ZERO_BYTE as u32).take(10).collect();
        let bx = Box::new(buf.into_iter());

        let _sb = StreamBuffer::new(bx, 10);
        // println!("{}", _sb.read(0, 5));
    }
}