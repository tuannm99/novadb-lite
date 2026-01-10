use super::{slot, SLOTTED_HEADER_SIZE, SLOTTED_SLOT_SIZE};
use crate::page::header::{self};
use crate::{constants::PAGE_SIZE, DbError, DbResult};

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

    pub fn validate(&self) -> DbResult<()> {
        // 1) buffer size
        header::validate(self.buf)?;

        // 2) header fields
        let lo = header::lower(self.buf)? as usize;
        let up = header::upper(self.buf)? as usize;
        let sc = header::slot_count(self.buf)? as usize;

        // lower phải >= header size
        if lo < SLOTTED_HEADER_SIZE {
            return Err(DbError::Corruption("corrupt header: lower < header size"));
        }

        // upper không vượt page size
        if up > PAGE_SIZE {
            return Err(DbError::Corruption("corrupt header: upper > PAGE_SIZE"));
        }

        // lower <= upper
        if lo > up {
            return Err(DbError::Corruption("corrupt header: lower > upper"));
        }

        // lower phải đúng công thức slot directory
        let expected_lo = SLOTTED_HEADER_SIZE + sc * SLOTTED_SLOT_SIZE;
        if lo != expected_lo {
            return Err(DbError::Corruption(
                "corrupt header: lower != header_size + slot_count*slot_size",
            ));
        }

        // scan slot bounds để bắt corruption sớm
        for slot_id in 0..sc {
            let s = slot::read_slot(self.buf, slot_id as u16)?;
            if !slot::is_dead(s.flags()) {
                let start = s.offset() as usize;
                let end = start + s.len() as usize;
                if end > PAGE_SIZE {
                    return Err(DbError::Corruption("corrupt slot: tuple out of bounds"));
                }
                if start < up {
                    return Err(DbError::Corruption(
                        "corrupt slot: tuple overlaps free space",
                    ));
                }
            }
        }

        Ok(())
    }

    /// Free space hiện tại trong page (upper - lower).
    pub fn free_space(&self) -> DbResult<u16> {
        let up = header::upper(self.buf)?;
        let lo = header::lower(self.buf)?;
        up.checked_sub(lo)
            .ok_or(DbError::Corruption("corrupt header: lower > upper"))
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
        self.validate()?;

        let up = header::upper(self.buf)?;
        let slot_count = header::slot_count(self.buf)?;

        let need_data_len: u16 = data
            .len()
            .try_into()
            .map_err(|_| DbError::Corruption("record is too large"))?;

        let reuse_id = self.find_free_slot()?;
        let can_reuse = reuse_id.is_some();

        // slot_id sẽ là tổng slot hiện tại (slot_count) hoặc tombstone id(nếu thỏa mãn)
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

        slot::write_slot(
            self.buf,
            slot_id,
            &slot::Slot::new(upper_new, need_data_len, 0),
        )?;

        // insert mới nếu k tìm thấy tombstone (deleted)
        if !can_reuse {
            header::set_slot_count(self.buf, slot_count + 1)?;
            let lower_new =
                SLOTTED_HEADER_SIZE as u16 + (slot_count + 1) * SLOTTED_SLOT_SIZE as u16;
            header::set_lower(self.buf, lower_new)?;
        } else {
            // CLEAR HAS_FREE_SLOTS
            // "bit này chỉ là optimization, hiện tại không clear để đơn giản;
            // find_free_slot sẽ tự scan và return None nếu hết tombstone."
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
        self.validate()?;

        let sc = header::slot_count(self.buf)?;
        if slot_id >= sc {
            return Err(DbError::Corruption("invalid slot_id"));
        }

        let slot = slot::read_slot(self.buf, slot_id)?;
        if slot::is_dead(slot.flags()) {
            return Ok(None);
        }

        let start = slot.offset() as usize;
        let end = (slot.offset() + slot.len()) as usize;
        if end > PAGE_SIZE {
            return Err(DbError::Corruption("tuple end must be <= PAGE_SIZE"));
        }

        Ok(Some(&self.buf[start..end]))
    }

    /// Delete slot_id: set flag DEAD, không reclaim data ngay (tombstone).
    /// để reuse slot:
    /// - set page header flag HAS_FREE_SLOTS (bit 4)
    pub fn delete(&mut self, slot_id: u16) -> DbResult<()> {
        self.validate()?;

        let sc = header::slot_count(self.buf)?;
        if slot_id >= sc {
            return Err(DbError::Corruption("invalid slot_id"));
        }

        let mut slot = slot::read_slot(self.buf, slot_id)?;
        if slot::is_dead(slot.flags()) {
            return Ok(());
        }
        slot.mark_flags_dead();
        slot::write_slot(self.buf, slot_id, &slot)?;

        let page_flags = header::flags(self.buf)?;
        header::set_flags(self.buf, page_flags | (1 << 4))?;

        Ok(())
    }

    /// Tìm slot tombstone để reuse.
    /// Nếu page header có HAS_FREE_SLOTS thì scan slot directory, return slot_id đầu tiên DEAD.
    fn find_free_slot(&self) -> DbResult<Option<u16>> {
        let page_flags = header::flags(self.buf)?;
        if (page_flags & (1 << 4)) == 0 {
            return Ok(None);
        }

        let sc = header::slot_count(self.buf)?;
        for i in 0..sc {
            let slot = slot::read_slot(self.buf, i)?;
            if slot::is_dead(slot.flags()) {
                return Ok(Some(i));
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_rejects_wrong_size() {
        let mut buf = [0u8; 15];
        let got = SlottedPage::new(&mut buf);
        assert!(got.is_err(), "new() must reject non-PAGE_SIZE buffers");
    }

    #[test]
    fn test_new_accepts_page_size() {
        let mut buf = vec![0u8; PAGE_SIZE];
        let got = SlottedPage::new(&mut buf);
        assert!(got.is_ok(), "new() must accept PAGE_SIZE buffers");
    }
}
