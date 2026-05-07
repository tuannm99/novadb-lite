use crate::constants::{FLAG_HAS_FREE_SLOTS, PAGE_SIZE, SLOTTED_HEADER_SIZE, SLOTTED_SLOT_SIZE};
use crate::error::{DbError, Result};
use crate::page_header::{
    clear_flag, flags, init_empty_header, lower, set_flag, set_flags, set_lower, set_slot_count,
    set_upper, slot_count, upper, validate_header_layout,
};
use crate::page_slot::{dead_slot, is_dead, read_slot, slot, slot_fields, write_slot};

/// Mutable façade over a single slotted page buffer.
pub struct SlottedPage<'a> {
    buf: &'a mut [u8],
}

impl<'a> SlottedPage<'a> {
    /// Wraps a page-sized byte buffer as a slotted page.
    pub fn new(buf: &'a mut [u8]) -> Result<Self> {
        if buf.len() != PAGE_SIZE {
            return Err(DbError::corruption("buffer length must equal PAGE_SIZE"));
        }
        Ok(Self { buf })
    }

    /// Initializes the page header for an empty page of `page_type`.
    pub fn init(&mut self, page_type: u16) -> Result<()> {
        init_empty_header(self.buf, page_type)
    }

    /// Validates header-level invariants without scanning tuple bodies.
    pub fn validate_header(&self) -> Result<()> {
        validate_header_layout(
            lower(self.buf)? as usize,
            upper(self.buf)? as usize,
            slot_count(self.buf)? as usize,
        )
    }

    /// Validates both the header and all live slot payload boundaries.
    pub fn validate_full(&self) -> Result<()> {
        self.validate_header()?;
        let up = upper(self.buf)? as usize;
        let sc = slot_count(self.buf)?;
        for slot_id in 0..sc {
            let slot = read_slot(self.buf, slot_id)?;
            let (start_u16, len_u16, flags_u16) = slot_fields(slot);
            if !is_dead(flags_u16) {
                let start = start_u16 as usize;
                let length = len_u16 as usize;
                let end = start
                    .checked_add(length)
                    .ok_or_else(|| DbError::corruption("tuple end overflow"))?;
                if end > PAGE_SIZE {
                    return Err(DbError::corruption("corrupt slot: tuple out of bounds"));
                }
                if start < up {
                    return Err(DbError::corruption(
                        "corrupt slot: tuple overlaps free space",
                    ));
                }
            }
        }
        Ok(())
    }

    /// Returns the amount of immediately available free space in bytes.
    pub fn free_space(&self) -> Result<u16> {
        let up = upper(self.buf)?;
        let lo = lower(self.buf)?;
        if up < lo {
            return Err(DbError::corruption("corrupt header: lower > upper"));
        }
        Ok(up - lo)
    }

    /// Returns the payload for a live slot, or `None` for a deleted slot.
    pub fn get(&self, slot_id: u16) -> Result<Option<&[u8]>> {
        self.validate_header()?;
        let sc = slot_count(self.buf)?;
        if slot_id >= sc {
            return Err(DbError::invalid_argument("invalid slot_id"));
        }
        let slot = read_slot(self.buf, slot_id)?;
        let (start_u16, len_u16, flags_u16) = slot_fields(slot);
        if is_dead(flags_u16) {
            return Ok(None);
        }
        let start = start_u16 as usize;
        let up = upper(self.buf)? as usize;
        if start < up {
            return Err(DbError::corruption("tuple overlaps free space"));
        }
        let end = start
            .checked_add(len_u16 as usize)
            .ok_or_else(|| DbError::corruption("tuple end overflow"))?;
        if end > PAGE_SIZE {
            return Err(DbError::corruption("tuple end must be <= PAGE_SIZE"));
        }
        Ok(Some(&self.buf[start..end]))
    }

    /// Inserts a new record and returns its slot id.
    pub fn insert(&mut self, data: &[u8]) -> Result<u16> {
        self.validate_header()?;
        let up = upper(self.buf)?;
        let sc = slot_count(self.buf)?;
        let need_data_len: u16 = data
            .len()
            .try_into()
            .map_err(|_| DbError::corruption("record is too large"))?;

        let reuse_id = self.find_free_slot()?;
        let can_reuse = reuse_id.is_some();
        let slot_id = reuse_id.unwrap_or(sc);
        let need_slot = if can_reuse {
            0
        } else {
            SLOTTED_SLOT_SIZE as u16
        };
        let need_total = usize::from(need_data_len) + usize::from(need_slot);

        if need_total > usize::from(self.free_space()?) {
            return Err(DbError::no_space("not enough space"));
        }
        if up < need_data_len {
            return Err(DbError::corruption("record is too large"));
        }

        let upper_new = up - need_data_len;
        self.buf[upper_new as usize..up as usize].copy_from_slice(data);
        write_slot(self.buf, slot_id, slot(upper_new, need_data_len, 0))?;

        if !can_reuse {
            set_slot_count(self.buf, sc + 1)?;
            let lower_new = SLOTTED_HEADER_SIZE as u16 + (sc + 1) * (SLOTTED_SLOT_SIZE as u16);
            set_lower(self.buf, lower_new)?;
        }
        set_upper(self.buf, upper_new)?;
        Ok(slot_id)
    }

    /// Updates an existing slot.
    ///
    /// Returns `true` when the record had to be relocated within the page.
    pub fn update(&mut self, slot_id: u16, data: &[u8]) -> Result<bool> {
        self.validate_header()?;
        let sc = slot_count(self.buf)?;
        if slot_id >= sc {
            return Err(DbError::invalid_argument("invalid slot_id"));
        }
        let current = read_slot(self.buf, slot_id)?;
        let (offset, old_len, current_flags) = slot_fields(current);
        if is_dead(current_flags) {
            return Err(DbError::corruption("slot is dead"));
        }
        let need: u16 = data
            .len()
            .try_into()
            .map_err(|_| DbError::corruption("record is too large"))?;

        if need <= old_len {
            let start = offset as usize;
            let end_new = start + usize::from(need);
            let end_old = start + usize::from(old_len);
            self.buf[start..end_new].copy_from_slice(data);
            self.buf[end_new..end_old].fill(0);
            write_slot(self.buf, slot_id, slot(offset, need, current_flags))?;
            return Ok(false);
        }

        if need > self.free_space()? {
            return Err(DbError::no_space("not enough space"));
        }
        let up = upper(self.buf)?;
        if up < need {
            return Err(DbError::corruption("record is too large"));
        }
        let upper_new = up - need;
        self.buf[upper_new as usize..up as usize].copy_from_slice(data);
        write_slot(self.buf, slot_id, slot(upper_new, need, current_flags))?;
        set_upper(self.buf, upper_new)?;
        Ok(true)
    }

    /// Marks a slot as deleted so its id can be reused later.
    pub fn delete(&mut self, slot_id: u16) -> Result<()> {
        self.validate_header()?;
        let sc = slot_count(self.buf)?;
        if slot_id >= sc {
            return Err(DbError::invalid_argument("invalid slot_id"));
        }
        let current = read_slot(self.buf, slot_id)?;
        let (_, _, current_flags) = slot_fields(current);
        if is_dead(current_flags) {
            return Ok(());
        }
        write_slot(self.buf, slot_id, dead_slot(current))?;
        let page_flags = flags(self.buf)?;
        set_flags(self.buf, set_flag(page_flags, FLAG_HAS_FREE_SLOTS))
    }

    fn find_free_slot(&mut self) -> Result<Option<u16>> {
        let page_flags = flags(self.buf)?;
        if (page_flags & FLAG_HAS_FREE_SLOTS) == 0 {
            return Ok(None);
        }
        let sc = slot_count(self.buf)?;
        for slot_id in 0..sc {
            let current = read_slot(self.buf, slot_id)?;
            let (_, _, current_flags) = slot_fields(current);
            if is_dead(current_flags) {
                return Ok(Some(slot_id));
            }
        }
        set_flags(self.buf, clear_flag(page_flags, FLAG_HAS_FREE_SLOTS))?;
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::PAGE_TYPE_HEAP;
    use crate::page_header::{decode_header, flags, lower};

    fn make_page(buf: &mut [u8]) -> SlottedPage<'_> {
        let mut page = SlottedPage::new(buf).unwrap();
        page.init(PAGE_TYPE_HEAP).unwrap();
        page
    }

    #[test]
    fn new_rejects_wrong_size() {
        let mut buf = vec![0; 15];
        assert!(SlottedPage::new(&mut buf).is_err());
    }

    #[test]
    fn new_accepts_page_size() {
        let mut buf = vec![0; PAGE_SIZE];
        assert!(SlottedPage::new(&mut buf).is_ok());
    }

    #[test]
    fn slotted_page_validate() {
        let mut buf = vec![0; PAGE_SIZE];
        let page = make_page(&mut buf);
        assert_eq!(
            (PAGE_SIZE - SLOTTED_HEADER_SIZE) as u16,
            page.free_space().unwrap()
        );
        page.validate_full().unwrap();
    }

    #[test]
    fn slotted_page_get() {
        let mut buf = vec![0; PAGE_SIZE];
        let mut page = make_page(&mut buf);
        let data1 = b"Hello, world";
        let id0 = page.insert(data1).unwrap();
        assert_eq!(0, id0);
        let data2 = b"Hello, world.. TUANNM";
        let id1 = page.insert(data2).unwrap();
        assert_eq!(1, id1);
        let header = decode_header(&buf).unwrap();
        assert_eq!((SLOTTED_HEADER_SIZE + 6 + 6) as u16, header.lower());
        assert_eq!(
            PAGE_SIZE - data1.len() - data2.len(),
            header.upper() as usize
        );
        assert_eq!(2, header.slot_count());
    }

    #[test]
    fn find_free_slot() {
        let mut buf = vec![0; PAGE_SIZE];
        let mut page = make_page(&mut buf);
        assert_eq!(None, page.find_free_slot().unwrap());
        let id0 = page.insert(b"Hello, world").unwrap();
        assert_eq!(0, id0);
        assert_eq!(
            SLOTTED_HEADER_SIZE + SLOTTED_SLOT_SIZE,
            lower(page.buf).unwrap() as usize
        );
        assert_eq!(None, page.find_free_slot().unwrap());
        page.delete(0).unwrap();
        assert_eq!(Some(0), page.find_free_slot().unwrap());
        let id_reuse = page.insert(b"Hello, ").unwrap();
        assert_eq!(0, id_reuse);
        assert_eq!(None, page.find_free_slot().unwrap());
        page.delete(0).unwrap();
        let id_reuse2 = page.insert(b"Hello, Tuannm string larger").unwrap();
        assert_eq!(0, id_reuse2);
        assert_ne!(0, flags(page.buf).unwrap() & FLAG_HAS_FREE_SLOTS);
        assert_eq!(None, page.find_free_slot().unwrap());
        assert_eq!(0, flags(page.buf).unwrap() & FLAG_HAS_FREE_SLOTS);
    }

    #[test]
    fn slotted_page_insert() {
        let mut buf = vec![0; PAGE_SIZE];
        let mut page = make_page(&mut buf);
        assert_eq!(
            (PAGE_SIZE - SLOTTED_HEADER_SIZE) as u16,
            page.free_space().unwrap()
        );
        let d1 = b"abc";
        let id0 = page.insert(d1).unwrap();
        assert_eq!(0, id0);
        assert_eq!(
            SLOTTED_HEADER_SIZE + SLOTTED_SLOT_SIZE,
            lower(page.buf).unwrap() as usize
        );
        assert_eq!(PAGE_SIZE - d1.len(), upper(page.buf).unwrap() as usize);
        assert_eq!(Some(d1.as_slice()), page.get(id0).unwrap());

        let d2 = b"hello world";
        let id1 = page.insert(d2).unwrap();
        assert_eq!(1, id1);
        assert_eq!(
            SLOTTED_HEADER_SIZE + 2 * SLOTTED_SLOT_SIZE,
            lower(page.buf).unwrap() as usize
        );
        assert_eq!(
            PAGE_SIZE - d1.len() - d2.len(),
            upper(page.buf).unwrap() as usize
        );
        assert_eq!(Some(d2.as_slice()), page.get(id1).unwrap());

        let huge = vec![0; usize::from(page.free_space().unwrap()) + 1];
        assert!(page.insert(&huge).is_err());
        page.validate_header().unwrap();
        page.validate_full().unwrap();
    }

    #[test]
    fn slotted_page_update() {
        let mut buf = vec![0; PAGE_SIZE];
        let mut page = make_page(&mut buf);
        let id = page.insert(b"hello world").unwrap();
        assert_eq!(0, id);
        let moved = page.update(id, b"hi").unwrap();
        assert!(!moved);
        assert_eq!(Some("hi".as_bytes()), page.get(id).unwrap());
        let up_after_inplace = upper(page.buf).unwrap();
        let big = b"this is a longer string than before";
        let moved2 = page.update(id, big).unwrap();
        assert!(moved2);
        assert_eq!(Some(big.as_slice()), page.get(id).unwrap());
        let up_after_move = upper(page.buf).unwrap();
        assert!(up_after_move < up_after_inplace);
        assert!(page.update(99, b"x").is_err());
        page.delete(id).unwrap();
        assert!(page.update(id, b"x").is_err());
        page.validate_header().unwrap();
        page.validate_full().unwrap();
    }

    #[test]
    fn slotted_page_delete() {
        let mut buf = vec![0; PAGE_SIZE];
        let mut page = make_page(&mut buf);
        let id0 = page.insert(b"a").unwrap();
        let id1 = page.insert(b"b").unwrap();
        assert_eq!(0, id0);
        assert_eq!(1, id1);
        page.delete(id0).unwrap();
        assert_eq!(None, page.get(id0).unwrap());
        assert_eq!(Some("b".as_bytes()), page.get(id1).unwrap());
        page.delete(id0).unwrap();
        assert_ne!(0, flags(page.buf).unwrap() & FLAG_HAS_FREE_SLOTS);
        assert!(page.delete(99).is_err());
        page.validate_header().unwrap();
        page.validate_full().unwrap();
    }

    #[test]
    fn slotted_page_roundtrip() {
        let mut buf = vec![0; PAGE_SIZE];
        let mut page = make_page(&mut buf);
        let id0 = page.insert(b"r0").unwrap();
        let id1 = page.insert(b"record-1").unwrap();
        let id2 = page.insert(b"record-2222").unwrap();
        let id3 = page.insert(b"r3").unwrap();
        assert_eq!(0, id0);
        assert_eq!(1, id1);
        assert_eq!(2, id2);
        assert_eq!(3, id3);

        assert!(!page.update(id1, b"X").unwrap());
        assert_eq!(Some("X".as_bytes()), page.get(id1).unwrap());
        let big = b"this update will move because it's longer than before";
        assert!(page.update(id0, big).unwrap());
        assert_eq!(Some(big.as_slice()), page.get(id0).unwrap());

        page.delete(id2).unwrap();
        page.delete(id3).unwrap();
        assert_eq!(None, page.get(id2).unwrap());
        assert_eq!(None, page.get(id3).unwrap());

        let id_reuse = page.insert(b"reuse").unwrap();
        assert!(id_reuse == id2 || id_reuse == id3);
        assert_eq!(Some("reuse".as_bytes()), page.get(id_reuse).unwrap());
        page.validate_header().unwrap();
        page.validate_full().unwrap();
        assert_eq!(Some(big.as_slice()), page.get(id0).unwrap());
        assert_eq!(Some("X".as_bytes()), page.get(id1).unwrap());
        let other_dead = if id_reuse == id2 { id3 } else { id2 };
        assert_eq!(None, page.get(other_dead).unwrap());
    }
}
