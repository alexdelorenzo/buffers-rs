use tempfile::SpooledTempFile;
use std::fs::File;


use buffer::{StreamBuffer, BufferRead};
use std::io::{Write, Read, Seek, SeekFrom, Bytes};


fn main() {
    let file = File::open("test.txt").unwrap();
    let bytes = Box::new(file.bytes());

    let mut buffer = StreamBuffer::new(bytes, 21);
    buffer.read(0, 5).unwrap();

}