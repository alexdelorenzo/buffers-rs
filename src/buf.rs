// use super::*;
use tempfile::{SpooledTempFile};
use itertools::Itertools;

use super::constants::*;
use super::types::*;
use super::chunk::*;


pub trait Buffer {}

pub type ByteStreamBuf<F> = StreamBuffer<ByteResult, F>;


pub struct StreamBuffer<T, F: FileLike> {
    pub size: usize,
    pub index: usize,
    pub stream: Stream<T>,
    pub file: F,
    // reader: BufReader<Box<FileType>>,
    // writer: BufWriter<Box<FileType>>,
}

pub trait BufferCreate<T, F: FileLike>: Buffer {
    fn from_file(stream: Stream<T>, size: usize, file: F) -> StreamBuffer<T, F>;
}

pub trait BufferRead<T>: Buffer {
    fn read(&mut self, offset: usize, size: usize) -> T;
}

impl<T, F: FileLike> Buffer for StreamBuffer<T, F> {}

impl<I, T: ChunkRead<I> + ChunkLocation + Buffer> BufferRead<I> for T {
    fn read(&mut self, offset: usize, size: usize) -> I {
        match self._chunk_location(offset, size) {
            Location::BeforeIndex => self._chunk_before_index(offset, size),
            Location::Bisected => self._chunk_bisected_by_index(offset, size),
            Location::AtIndex => self._chunk_at_index(size),
            Location::AfterIndex => self._chunk_after_index(offset, size)
        }
    }
}

impl<T> StreamBuffer<T, SpooledTempFile> {
    pub fn new(stream: Stream<T>, size: usize) -> StreamBuffer<T, SpooledTempFile> {
        let file = SpooledTempFile::new(MAX_SIZE);
        StreamBuffer { size, stream, file, index: START_INDEX }
    }
}

impl<T, F: FileLike> BufferCreate<T, F> for StreamBuffer<T, F> {
    fn from_file(stream: Stream<T>, size: usize, file: F) -> StreamBuffer<T, F> {
        StreamBuffer { size, stream, file, index: START_INDEX }
    }
}