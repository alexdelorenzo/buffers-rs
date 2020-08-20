pub use std::io::{Write, Read, Seek, SeekFrom};
pub use std::io::{Error as IoError, Result as IoResult};

pub trait FileLike = Read + Seek + Write;
// pub trait FileLike: Read + Seek + Write {}
// impl<T: Read + Seek + Write> FileLike for T {}

pub type Byte = u8;
pub type ByteBuf = Vec<Byte>;
pub type ByteResult = IoResult<Byte>;
pub type BufResult = IoResult<ByteBuf>;
pub type Stream<T> = Box<dyn Iterator<Item = T>>;
// pub type IntoStream<T> = 
//   Box<dyn IntoIterator<Item = T, IntoIter = dyn Iterator<Item = T>>>;

pub trait IntoStream<T> = IntoIterator<Item = T>;

fn test<T: IntoStream<u8>>(items: T) {
    for item in items {
        // snip
    }
}