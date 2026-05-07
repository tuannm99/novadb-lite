use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use crate::constants::PAGE_SIZE;
use crate::error::{DbError, Result};
use crate::pager::Pager;
use crate::types::PageId;

/// Pager implementation backed by a single database file.
pub struct FilePager {
    file: File,
    freelist: Vec<PageId>,
    next_pid: PageId,
}

impl FilePager {
    /// Opens or creates a file-backed pager.
    ///
    /// A new file is initialized with page `0` reserved and zero-filled.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)
            .map_err(DbError::wrap_io)?;
        let info = file.metadata().map_err(DbError::wrap_io)?;
        let length = info.len();
        if length % PAGE_SIZE as u64 != 0 {
            return Err(DbError::corruption("db file length is not page-aligned"));
        }
        let pages = (length / PAGE_SIZE as u64) as u32;
        let next_pid = if pages == 0 {
            let zero = [0u8; PAGE_SIZE];
            file.write_all(&zero).map_err(DbError::wrap_io)?;
            file.sync_all().map_err(DbError::wrap_io)?;
            PageId(1)
        } else {
            PageId(pages)
        };
        Ok(Self {
            file,
            freelist: Vec::new(),
            next_pid,
        })
    }

    fn seek_to(&mut self, pid: PageId) -> Result<()> {
        let off = pid
            .as_u64()
            .checked_mul(PAGE_SIZE as u64)
            .ok_or_else(|| DbError::corruption("page offset overflow"))?;
        self.file
            .seek(SeekFrom::Start(off))
            .map(|_| ())
            .map_err(DbError::wrap_io)
    }
}

impl Pager for FilePager {
    fn read_page(&mut self, pid: PageId, out: &mut [u8]) -> Result<()> {
        if out.len() != PAGE_SIZE {
            return Err(DbError::invalid_argument(
                "buffer length must equal PAGE_SIZE",
            ));
        }
        if pid == PageId::INVALID {
            return Err(DbError::invalid_argument("invalid page id"));
        }
        if pid.as_u64() >= self.num_pages()? {
            return Err(DbError::invalid_argument("page id out of range"));
        }
        self.seek_to(pid)?;
        self.file.read_exact(out).map_err(DbError::wrap_io)
    }

    fn write_page(&mut self, pid: PageId, buf: &[u8]) -> Result<()> {
        if buf.len() != PAGE_SIZE {
            return Err(DbError::invalid_argument(
                "buffer length must equal PAGE_SIZE",
            ));
        }
        if pid == PageId::INVALID {
            return Err(DbError::invalid_argument("invalid page id"));
        }
        if pid.as_u64() >= self.num_pages()? {
            return Err(DbError::invalid_argument("page id out of range"));
        }
        self.seek_to(pid)?;
        self.file.write_all(buf).map_err(DbError::wrap_io)
    }

    fn alloc_page(&mut self) -> Result<PageId> {
        if let Some(pid) = self.freelist.pop() {
            return Ok(pid);
        }
        let pid = self.next_pid;
        let zero = [0u8; PAGE_SIZE];
        self.seek_to(pid)?;
        self.file.write_all(&zero).map_err(DbError::wrap_io)?;
        self.next_pid = PageId(self.next_pid.0 + 1);
        Ok(pid)
    }

    fn free_page(&mut self, pid: PageId) -> Result<()> {
        if pid.0 == 0 || pid == PageId::INVALID {
            return Err(DbError::invalid_argument("invalid page id"));
        }
        self.freelist.push(pid);
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        self.file.sync_all().map_err(DbError::wrap_io)
    }

    fn num_pages(&self) -> Result<u64> {
        self.file
            .metadata()
            .map(|info| info.len() / PAGE_SIZE as u64)
            .map_err(DbError::wrap_io)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_db_path(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "novadb-storage-{name}-{}-{unique}.db",
            std::process::id()
        ))
    }

    #[test]
    fn file_pager_alloc_write_read() {
        let path = temp_db_path("rw");
        let mut pager = FilePager::open(&path).unwrap();

        let pid = pager.alloc_page().unwrap();
        let mut buf = vec![0; PAGE_SIZE];
        for (i, byte) in buf.iter_mut().enumerate() {
            *byte = (i % 255) as u8;
        }
        pager.write_page(pid, &buf).unwrap();
        let mut out = vec![0; PAGE_SIZE];
        pager.read_page(pid, &mut out).unwrap();
        assert_eq!(buf, out);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn file_pager_free_reuse() {
        let path = temp_db_path("reuse");
        let mut pager = FilePager::open(&path).unwrap();

        let pid1 = pager.alloc_page().unwrap();
        let pid2 = pager.alloc_page().unwrap();
        pager.free_page(pid2).unwrap();
        let pid_reuse = pager.alloc_page().unwrap();
        assert_eq!(pid2, pid_reuse);
        assert_ne!(pid1, pid2);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn file_pager_rejects_misaligned_file() {
        let path = temp_db_path("bad");
        std::fs::write(&path, b"bad").unwrap();
        assert!(FilePager::open(&path).is_err());
        let _ = std::fs::remove_file(path);
    }
}
