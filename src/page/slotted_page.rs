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
    pub fn init(self, page_type: u16) -> DbResult<Self> {
        header::init_empty(self.buf, page_type)?;
        Ok(self)
    }

    #[cfg(debug_assertions)]
    pub fn validate_full(&self) -> DbResult<()> {
        self.validate_header()?;

        let up = header::upper(self.buf)? as usize;
        let sc = header::slot_count(self.buf)? as usize;

        for slot_id in 0..sc {
            let s = slot::read_slot(self.buf, slot_id as u16)?;
            if !slot::is_dead(s.flags()) {
                let start = s.offset() as usize;
                let len = s.len() as usize;
                let end = start
                    .checked_add(len)
                    .ok_or(DbError::Corruption("tuple end overflow"))?;
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

    pub fn validate_header(&self) -> DbResult<()> {
        // header fields
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
        let slot_bytes = sc
            .checked_mul(SLOTTED_SLOT_SIZE)
            .ok_or(DbError::Corruption("corrupt header: slot_count overflow"))?;

        let expected_lo = SLOTTED_HEADER_SIZE
            .checked_add(slot_bytes)
            .ok_or(DbError::Corruption("corrupt header: lower overflow"))?;

        if expected_lo > PAGE_SIZE {
            return Err(DbError::Corruption(
                "corrupt header: slot directory out of page",
            ));
        }
        if lo != expected_lo {
            return Err(DbError::Corruption(
                "corrupt header: lower != header_size + slot_count*slot_size",
            ));
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

    /// Lấy record bytes theo slot_id.
    /// Trả None nếu slot DEAD.
    /// Các check cần có:
    /// - slot_id < slot_count
    /// - slot.offset + slot.len <= PAGE_SIZE
    pub fn get(&self, slot_id: u16) -> DbResult<Option<&[u8]>> {
        // pub fn get<'b>(&'b self, slot_id: u16) -> DbResult<Option<&'b [u8]>> {
        self.validate_header()?;

        let sc = header::slot_count(self.buf)?;
        if slot_id >= sc {
            return Err(DbError::InvalidArgument("invalid slot_id"));
        }

        let slot = slot::read_slot(self.buf, slot_id)?;
        if slot::is_dead(slot.flags()) {
            return Ok(None);
        }

        let start = slot.offset() as usize;
        let up = header::upper(self.buf)? as usize;
        if start < up {
            return Err(DbError::Corruption("tuple overlaps free space"));
        }

        let len = slot.len() as usize;
        let end = start
            .checked_add(len)
            .ok_or(DbError::Corruption("tuple end overflow"))?;
        if end > PAGE_SIZE {
            return Err(DbError::Corruption("tuple end must be <= PAGE_SIZE"));
        }

        Ok(Some(&self.buf[start..end]))
    }

    /// Insert record bytes vào page.
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
        self.validate_header()?;

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

        let need_total = need_data_len
            .checked_add(need_slot)
            .ok_or(DbError::Corruption("need size overflow"))?;

        if need_total > self.free_space()? {
            return Err(DbError::NoSpace("not enough space"));
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
        }

        header::set_upper(self.buf, upper_new)?;
        Ok(slot_id)
    }

    /// Update record bytes tại slot_id.
    ///
    /// Có 3 case:
    /// 1) Slot DEAD -> return error.
    /// 2) data.len() <= old_len:
    ///    - update in-place tại vùng tuple hiện tại
    ///    - (tuỳ chọn) zero phần thừa để debug
    ///    - update slot.len = data.len()
    ///    - return Ok(false)  // moved = false
    /// 3) data.len() > old_len:
    ///    - allocate vùng data mới ở phía upper (giống insert, nhưng reuse slot entry)
    ///    - copy data mới vào [upper_new..upper)
    ///    - update slot.offset = upper_new, slot.len = data.len()
    ///    - update header.upper = upper_new
    ///    - data cũ trở thành garbage, sẽ được reclaim khi vacuum/compact
    ///    - return Ok(true)   // moved = true
    ///
    /// Return:
    /// - Ok(false) => in-place (case 2)
    /// - Ok(true)  => moved (case 3)
    pub fn update(&mut self, slot_id: u16, data: &[u8]) -> DbResult<bool> {
        self.validate_header()?;

        let sc = header::slot_count(self.buf)?;
        if slot_id >= sc {
            return Err(DbError::InvalidArgument("invalid slot_id"));
        }

        let slot = slot::read_slot(self.buf, slot_id)?;
        if slot::is_dead(slot.flags()) {
            return Err(DbError::Corruption("slot is dead"));
        }

        let need: u16 = data
            .len()
            .try_into()
            .map_err(|_| DbError::Corruption("record is too large"))?;

        let old_len = slot.len();

        // Case 2: in-place
        if need <= old_len {
            let start = slot.offset() as usize;
            let end_new = start + need as usize;
            let end_old = start + old_len as usize;

            self.buf[start..end_new].copy_from_slice(data);

            // zero phần thừa
            self.buf[end_new..end_old].fill(0);

            slot::write_slot(
                self.buf,
                slot_id,
                &slot::Slot::new(slot.offset(), need, slot.flags()),
            )?;
            return Ok(false);
        }

        // Case 3: move tuple (reuse same slot_id)
        let free = self.free_space()?;
        if need > free {
            return Err(DbError::NoSpace("not enough space"));
        }

        let up = header::upper(self.buf)?;
        let upper_new = up
            .checked_sub(need)
            .ok_or(DbError::Corruption("record is too large"))?;

        let upper_new_usize = upper_new as usize;
        let up_usize = up as usize;

        self.buf[upper_new_usize..up_usize].copy_from_slice(data);

        slot::write_slot(
            self.buf,
            slot_id,
            &slot::Slot::new(upper_new, need, slot.flags()),
        )?;
        header::set_upper(self.buf, upper_new)?;

        Ok(true)
    }

    /// Delete slot_id: set flag DEAD, không reclaim data ngay (tombstone).
    /// để reuse slot:
    /// - set page header flag HAS_FREE_SLOTS (bit 4)
    pub fn delete(&mut self, slot_id: u16) -> DbResult<()> {
        self.validate_header()?;

        let sc = header::slot_count(self.buf)?;
        if slot_id >= sc {
            return Err(DbError::InvalidArgument("invalid slot_id"));
        }

        let mut slot = slot::read_slot(self.buf, slot_id)?;
        if slot::is_dead(slot.flags()) {
            return Ok(());
        }
        slot.mark_flags_dead();
        slot::write_slot(self.buf, slot_id, &slot)?;

        let page_flags = header::flags(self.buf)?;
        let new_flags = header::set_flag(page_flags, header::FLAG_HAS_FREE_SLOTS);
        header::set_flags(self.buf, new_flags)?;

        Ok(())
    }

    /// Tìm slot tombstone để reuse.
    /// Nếu page header có HAS_FREE_SLOTS thì scan slot directory, return slot_id đầu tiên DEAD.
    fn find_free_slot(&mut self) -> DbResult<Option<u16>> {
        let page_flags = header::flags(self.buf)?;
        if (page_flags & (header::FLAG_HAS_FREE_SLOTS)) == 0 {
            return Ok(None);
        }

        let sc = header::slot_count(self.buf)?;
        for i in 0..sc {
            let slot = slot::read_slot(self.buf, i)?;
            if slot::is_dead(slot.flags()) {
                return Ok(Some(i));
            }
        }

        let new_flags = header::clear_flag(page_flags, header::FLAG_HAS_FREE_SLOTS);
        header::set_flags(self.buf, new_flags)?;

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page::header::{FLAG_HAS_FREE_SLOTS, PAGE_TYPE_HEAP};

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

    #[test]
    fn test_slotted_page_validate() {
        let mut buf = vec![0u8; PAGE_SIZE];
        let slotted_page = SlottedPage::new(&mut buf)
            .unwrap()
            .init(PAGE_TYPE_HEAP)
            .unwrap();

        let free = slotted_page.free_space().unwrap();
        assert_eq!(free, (PAGE_SIZE - SLOTTED_HEADER_SIZE) as u16);

        slotted_page.validate_full().unwrap();
    }

    #[test]
    fn test_slotted_page_get() {
        let mut buf = vec![0u8; PAGE_SIZE];
        let mut slotted_page = SlottedPage::new(&mut buf)
            .unwrap()
            .init(PAGE_TYPE_HEAP)
            .unwrap();

        // insert 2 records
        let data1 = "Hello, world".as_bytes();
        let page_id = slotted_page.insert(data1).unwrap();
        assert_eq!(page_id, 0);

        let data2 = "Hello, world.. TUANNM".as_bytes();
        let page_id = slotted_page.insert(data2).unwrap();
        assert_eq!(page_id, 1);

        let page_header_snapshot = header::decode(slotted_page.buf).unwrap();
        assert_eq!(
            page_header_snapshot.lower(),
            SLOTTED_HEADER_SIZE as u16 + 6 + 6
        );

        assert_eq!(
            page_header_snapshot.upper() as usize,
            PAGE_SIZE - data1.len() - data2.len()
        );

        assert_eq!(page_header_snapshot.slot_count() as usize, 2);
    }

    #[test]
    fn test_slotted_page_insert() {}

    #[test]
    fn test_slotted_page_update() {}

    #[test]
    fn test_slotted_page_delete() {}

    #[test]
    fn test_slotted_page_roundtrip() {}
}
