use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};

use crate::constants::PAGE_SIZE;
use crate::{DbError, DbResult, PageId};

use super::pager::Pager;

pub struct FilePager {
    f: File,
    freelist: Vec<PageId>,
    next_pid: PageId, // nếu freelist trống, lấy id page kế tiếp
}

impl Pager for FilePager {
    fn num_pages(&mut self) -> DbResult<u64> {
        let len = self.f.metadata()?.len();
        Ok(len / PAGE_SIZE as u64)
    }

    fn read_page(&mut self, pid: PageId, out: &mut [u8]) -> DbResult<()> {
        todo!()
    }

    fn write_page(&mut self, pid: PageId, buf: &[u8]) -> DbResult<()> {
        todo!()
    }

    fn alloc_page(&mut self) -> DbResult<PageId> {
        todo!()
    }

    fn free_page(&mut self, pid: PageId) -> DbResult<()> {
        todo!()
    }

    fn flush(&mut self) -> DbResult<()> {
        // gọi fsync xuống disk
        self.f.sync_data()?;
        Ok(())
    }
}

impl FilePager {
    pub fn open(path: String) -> DbResult<Self> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        let len = file.metadata()?.len();
        if len % (PAGE_SIZE as u64) != 0 {
            return Err(DbError::Corruption("db file length is not page-aligned"));
        }

        let pages = (len / PAGE_SIZE as u64) as u32;

        // Reserve page 0 cho meta
        // Nếu chưa tồn tại file, chắc chắn page meta (0) tồn tại
        let next_pid = if pages == 0 {
            let zero = [0u8; PAGE_SIZE];
            file.write_all(&zero)?;
            file.flush()?;
            PageId(1)
        } else {
            PageId(pages)
        };

        Ok(Self {
            f: file,
            freelist: Vec::new(),
            next_pid,
        })
    }

    #[inline]
    pub fn seek_to(&mut self, pid: PageId) -> DbResult<()> {
        // move pointer đến page tương ứng -> pid * PAGE_SIZE
        let off = (pid.as_u64())
            .checked_mul(PAGE_SIZE as u64)
            .ok_or(DbError::Corruption("page offset overflow"))?;
        self.f.seek(SeekFrom::Start(off))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {}
