use crate::{DbError, DbResult};

use crate::constants::PAGE_SIZE;
use crate::page::header::{self, flags};
use crate::page::slot::{is_dead, read_slot, slot_off, write_slot, Slot};

use super::raw::write_u64_le;
use super::{SLOTTED_HEADER_SIZE, SLOTTED_SLOT_SIZE};

/// SlottedPage là API cấp cao thao tác trên 1 page bytes theo layout slotted-page.
/// - Header ở đầu page (fixed 16 bytes)
/// - Slot directory grow từ thấp lên (lower tăng dần)
/// - Tuple/data grow từ cao xuống (upper giảm dần)
pub struct SlottedPage<'a> {
    buf: &'a mut [u8],
}

impl<'a> SlottedPage<'a> {
    /// Tạo wrapper trên buffer page.
    pub fn new(buf: &'a mut [u8]) -> DbResult<Self> {
        if buf.len() != PAGE_SIZE {
            return Err(DbError::Corruption("buffer length must equal PAGE_SIZE"));
        }
        Ok(SlottedPage { buf })
    }

    /// Khởi tạo page rỗng.
    /// - lower = HEADER_SIZE
    /// - upper = PAGE_SIZE
    /// - slot_count = 0
    /// - flags = page_type (bits 0..3)
    pub fn init(&mut self, page_type: u16) -> DbResult<()> {
        header::init_empty(self.buf, page_type)?;
        Ok(())
    }

    /// Validate page header + basic invariants.
    pub fn validate(&self) -> DbResult<()> {
        todo!()
    }

    /// Số slot đã cấp phát (không giảm).
    pub fn slot_count(&self) -> DbResult<u16> {
        header::slot_count(self.buf)
    }

    /// Free space hiện tại trong page (upper - lower).
    pub fn free_space(&self) -> DbResult<u16> {
        Ok(header::upper(self.buf)? - header::lower(self.buf)?)
    }

    /// Insert record bytes vào page.
    /// Ý tưởng thuật toán:
    /// 1) Đọc lower/upper/slot_count.
    /// 2) Tìm slot tombstone để reuse (nếu muốn reuse), hoặc cấp slot_id mới.
    /// 3) Tính upper_new = upper - data.len()
    /// 4) Check đủ chỗ:
    ///    - Nếu cấp slot mới: cần thêm SLOT_SIZE bytes cho slot directory (lower tăng)
    ///    - Nếu reuse slot: không tăng lower
    /// 5) Copy data vào vùng [upper_new..upper)
    /// 6) Ghi slot entry: offset=upper_new, len=data.len, flags=0
    /// 7) Update header: upper=upper_new, lower/slot_count nếu slot mới
    pub fn insert(&mut self, data: &[u8]) -> DbResult<u16> {
        // PAGE_LAYOUT: <Header 16bytes> <Lower|slot1,slot2,...> .... <Upper|dataN,data2,data1>
        //                                grows ->                      grows <-
        //                                        <---- free space ---->

        let up = header::upper(self.buf)?;
        let slot_count = header::slot_count(self.buf)?;

        let need_data_len: u16 = data
            .len()
            .try_into()
            .map_err(|_| DbError::Corruption("record is too large"))?;

        let reuse_id = self.find_free_slot()?;
        let can_reuse = reuse_id.is_some();
        let slot_id = reuse_id.unwrap_or(slot_count);

        let need_slot = if can_reuse {
            0
        } else {
            SLOTTED_SLOT_SIZE as u16
        };
        if need_data_len + need_slot > self.free_space()? {
            return Err(DbError::Corruption("not enough space"));
        }

        let upper_new = up
            .checked_sub(need_data_len)
            .ok_or(DbError::Corruption("record is too large"))?;
        let upper_new_usize = upper_new as usize;
        let up_usize = up as usize;
        self.buf[upper_new_usize..up_usize].copy_from_slice(data);

        write_slot(
            self.buf,
            slot_id,
            &Slot {
                offset: upper_new,
                len: need_data_len,
                flags: 0,
            },
        )?;
        if !can_reuse {
            header::set_slot_count(self.buf, slot_count + 1)?;
            let lower_new =
                SLOTTED_HEADER_SIZE as u16 + (slot_count + 1) * SLOTTED_SLOT_SIZE as u16;
            header::set_lower(self.buf, lower_new)?;
        } else {
            // CLEAR HAS_FREE_SLOTS
        }

        header::set_upper(self.buf, upper_new)?;
        Ok(slot_id)
    }

    /// Lấy record bytes theo slot_id.
    /// Trả None nếu slot DEAD.
    /// Các check cần có:
    /// - slot_id < slot_count
    /// - slot.offset + slot.len <= PAGE_SIZE
    pub fn get<'b>(&'b self, slot_id: u16) -> DbResult<Option<&'b [u8]>> {
        todo!()
    }

    /// Delete slot_id: set flag DEAD, không reclaim data ngay (tombstone).
    /// Nếu bạn muốn reuse slot:
    /// - set page header flag HAS_FREE_SLOTS (bit 4)
    pub fn delete(&mut self, slot_id: u16) -> DbResult<()> {
        todo!()
    }

    /// (Optional) Tìm slot tombstone để reuse.
    /// Nếu page header có HAS_FREE_SLOTS thì scan slot directory, return slot_id đầu tiên DEAD.
    fn find_free_slot(&self) -> DbResult<Option<u16>> {
        // let flags = flags(self.buf)?;
        todo!()
        // if flags >> 4 & 1 {
        // } else {
        //
        // }
    }
}

#[cfg(test)]
mod tests {}
