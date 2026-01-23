use crate::constants::PAGE_SIZE;
use crate::PageId;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FilePager {}

impl FilePager {
    pub fn open(path: String) {
        todo!()
    }

    pub fn read_page(pid: PageId) -> [u8; PAGE_SIZE] {
        todo!()
    }

    pub fn write_page(pid: PageId, data: &[u8]) {
        todo!()
    }

    pub fn alloc_page() {
        todo!()
    }

    pub fn free_page(pid: PageId) {
        todo!()
    }
}
