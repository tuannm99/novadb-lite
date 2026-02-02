use crate::{DbError, DbResult, PageId};

pub trait Pager {
    fn read_page(&mut self, pid: PageId, out: &mut [u8]) -> DbResult<()>;
    fn write_page(&mut self, pid: PageId, buf: &[u8]) -> DbResult<()>;
    fn alloc_page(&mut self) -> DbResult<PageId>;
    fn free_page(&mut self, pid: PageId) -> DbResult<()>;
    fn flush(&mut self) -> DbResult<()>;
    fn num_pages(&mut self) -> DbResult<u64>;
}
