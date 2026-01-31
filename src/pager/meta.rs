use crate::{DbResult, PageId};

pub struct Meta {
    page_size: u32,
    next_pid: PageId,
    freelist_len: u32,
    //   ... others
}

pub fn encode(meta: &Meta, buf: &mut [u8]) {
    todo!()
}

pub fn decode(buf: &[u8]) -> DbResult<Meta> {
    todo!()
}

pub fn init_default() -> Meta {
    todo!()
}
