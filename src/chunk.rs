use std::io::{SeekFrom};

use super::buf::{StreamBuffer, ByteStreamBuf};
use super::constants::{ZERO_BYTE, INCREMENT};
use super::types::{FileLike, BufResult};
use super::utils::{no_capacity_vec, sized_vec};


pub enum Location {
    BeforeIndex,
    Bisected,
    AtIndex,
    AfterIndex,
}

pub trait ChunkLocation {
    fn _chunk_location(&self, offset: usize, size: usize) -> Location;
}

pub trait ChunkRead<T> {
    fn _chunk_before_index(&mut self, offset: usize, size: usize) -> T;
    fn _chunk_bisected_by_index(&mut self,  offset: usize, size: usize) -> T;
    fn _chunk_at_index(&mut self, size: usize) -> T;
    fn _chunk_after_index(&mut self, offset: usize, size: usize) -> T;
}

impl<T, F: FileLike> ChunkLocation for StreamBuffer<T ,F> {
    fn _chunk_location(&self, offset: usize, size: usize) -> Location {
        let end = offset + size;

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

impl<F: FileLike> ChunkRead<BufResult> for ByteStreamBuf<F> {
    fn _chunk_before_index(&mut self, offset: usize, size: usize) -> BufResult {
        let seek_offset = SeekFrom::Start(offset as u64);
        let mut buf = sized_vec(size, ZERO_BYTE);

        self.file.seek(seek_offset)?;
        self.file.read(&mut buf)?;

        Ok(buf)
    }

    fn _chunk_bisected_by_index(&mut self, offset: usize, size: usize) -> BufResult {
        let existing_size = self.index - offset;
        let mut buf = 
            self._chunk_before_index(offset, existing_size)?;
        
        let new_size = size - buf.len();
        let new_buf = self._chunk_at_index(new_size)?;
        buf.extend(new_buf);

        Ok(buf)
    }

    fn _chunk_at_index(&mut self, size: usize) -> BufResult {
        let seek_offset = SeekFrom::Start(self.index as u64);
        self.file.seek(seek_offset)?;

        let mut buf = no_capacity_vec();

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

        Ok(buf)
    }

    fn _chunk_after_index(&mut self, offset: usize, size: usize) -> BufResult {
        let seek_index = SeekFrom::Start(self.index as u64);
        self.file.seek(seek_index)?;

        let mut buf = no_capacity_vec();

        while let Some(Ok(byte)) = self.stream.next()  {
            let bytes = &[byte];
            self.file.write(bytes)?;
            self.index += INCREMENT;

            if self.index >= offset {
                buf.extend_one(byte);
            }

            if buf.len() >= size {
                buf.truncate(size);
                break;
            }
        }

        Ok(buf)
    }
}
