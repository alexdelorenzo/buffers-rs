use std::fs::File;

use buffer::{StreamBuffer};
use std::io::{Read};


fn main() {
    let file = File::open("test.txt").unwrap();
    let bytes = Box::new(file.bytes());

    let _buffer = StreamBuffer::new(bytes, 21);
    // buffer.read(0, 5).unwrap();

}