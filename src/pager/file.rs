use std::fs::{File, OpenOptions};

use crate::constants::PAGE_SIZE;
use crate::{DbError, DbResult, PageId};

pub struct FilePager {
    pub f: File,
}

impl FilePager {
    pub fn open(path: String) -> DbResult<FilePager> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        Ok(FilePager { f: file })
    }

    pub fn read_page(&mut self, pid: PageId) -> [u8; PAGE_SIZE] {
        todo!()
    }

    pub fn write_page(&mut self, pid: PageId, data: &[u8]) {
        todo!()
    }

    pub fn alloc_page(&mut self) {
        todo!()
    }

    pub fn free_page(&mut self, pid: PageId) {
        todo!()
    }

    pub fn flush(&mut self) {
        todo!()
    }

    pub fn num_pages(&self) {
        todo!()
    }
}
