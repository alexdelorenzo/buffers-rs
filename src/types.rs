#![feature(type_alias_impl_trait)]
#![feature(trait_alias)]
use std::io::{Error as IoError, Result as IoResult};
use std::io::{Write, Read, Seek, SeekFrom};

pub trait FileLike = Read + Seek + Write;
// pub trait FileLike: Read + Seek + Write {}
// impl<T: Read + Seek + Write> FileLike for T {}

pub type Byte = u8;
pub type ByteBuf = Vec<Byte>;
pub type ByteResult = IoResult<Byte>;
pub type BufResult = IoResult<ByteBuf>;
pub type Stream<T> = Box<dyn Iterator<Item = T>>;
