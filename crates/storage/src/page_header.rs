use crate::constants::{
    FLAG_HAS_FREE_SLOTS, PAGE_SIZE, SLOTTED_HEADER_SIZE, SLOTTED_SLOT_SIZE,
};
use crate::error::{DbError, Result};
use crate::raw::{read_u16_le, read_u64_le, write_u16_le, write_u64_le};

const OFF_LOWER: usize = 0;
const OFF_UPPER: usize = 2;
const OFF_SLOT_COUNT: usize = 4;
const OFF_FLAGS: usize = 6;
const OFF_RESERVED: usize = 8;

/// Snapshot of a decoded slotted-page header.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PageHeaderSnapshot {
    lower: u16,
    upper: u16,
    slot_count: u16,
    flags: u16,
    reserved: u64,
}

impl PageHeaderSnapshot {
    /// Returns the lower boundary of used header and slot-directory space.
    pub fn lower(self) -> u16 {
        self.lower
    }

    /// Returns the upper boundary of free space.
    pub fn upper(self) -> u16 {
        self.upper
    }

    /// Returns the number of allocated slot entries.
    pub fn slot_count(self) -> u16 {
        self.slot_count
    }

    /// Returns the raw header flag bitmask.
    pub fn flags(self) -> u16 {
        self.flags
    }

    /// Returns the reserved header field.
    pub fn reserved(self) -> u64 {
        self.reserved
    }
}

pub(crate) fn decode_header(buf: &[u8]) -> Result<PageHeaderSnapshot> {
    if buf.len() != PAGE_SIZE {
        return Err(DbError::corruption("buffer length must equal PAGE_SIZE"));
    }
    Ok(PageHeaderSnapshot {
        lower: read_u16_le(buf, OFF_LOWER)?,
        upper: read_u16_le(buf, OFF_UPPER)?,
        slot_count: read_u16_le(buf, OFF_SLOT_COUNT)?,
        flags: read_u16_le(buf, OFF_FLAGS)?,
        reserved: read_u64_le(buf, OFF_RESERVED)?,
    })
}

pub(crate) fn init_empty_header(buf: &mut [u8], page_type: u16) -> Result<()> {
    if buf.len() != PAGE_SIZE {
        return Err(DbError::corruption("buffer length must equal PAGE_SIZE"));
    }
    let page_flags = page_type & 0x000F;
    set_lower(buf, SLOTTED_HEADER_SIZE as u16)?;
    set_upper(buf, PAGE_SIZE as u16)?;
    set_slot_count(buf, 0)?;
    set_flags(buf, page_flags)?;
    set_reserved(buf, 0)?;
    Ok(())
}

pub(crate) fn lower(buf: &[u8]) -> Result<u16> {
    read_u16_le(buf, OFF_LOWER)
}

pub(crate) fn set_lower(buf: &mut [u8], value: u16) -> Result<()> {
    write_u16_le(buf, OFF_LOWER, value)
}

pub(crate) fn upper(buf: &[u8]) -> Result<u16> {
    read_u16_le(buf, OFF_UPPER)
}

pub(crate) fn set_upper(buf: &mut [u8], value: u16) -> Result<()> {
    write_u16_le(buf, OFF_UPPER, value)
}

pub(crate) fn slot_count(buf: &[u8]) -> Result<u16> {
    read_u16_le(buf, OFF_SLOT_COUNT)
}

pub(crate) fn set_slot_count(buf: &mut [u8], value: u16) -> Result<()> {
    write_u16_le(buf, OFF_SLOT_COUNT, value)
}

pub(crate) fn flags(buf: &[u8]) -> Result<u16> {
    read_u16_le(buf, OFF_FLAGS)
}

pub(crate) fn set_flags(buf: &mut [u8], value: u16) -> Result<()> {
    write_u16_le(buf, OFF_FLAGS, value)
}

pub(crate) fn set_flag(flags: u16, mask: u16) -> u16 {
    flags | mask
}

pub(crate) fn clear_flag(flags: u16, mask: u16) -> u16 {
    flags & !mask
}

pub(crate) fn validate_header_layout(lo: usize, up: usize, sc: usize) -> Result<()> {
    if lo < SLOTTED_HEADER_SIZE {
        return Err(DbError::corruption("corrupt header: lower < header size"));
    }
    if up > PAGE_SIZE {
        return Err(DbError::corruption("corrupt header: upper > PAGE_SIZE"));
    }
    if lo > up {
        return Err(DbError::corruption("corrupt header: lower > upper"));
    }
    if sc > (i32::MAX as usize) / SLOTTED_SLOT_SIZE {
        return Err(DbError::corruption("corrupt header: slot_count overflow"));
    }

    let slot_bytes = sc * SLOTTED_SLOT_SIZE;
    let expected_lo = SLOTTED_HEADER_SIZE + slot_bytes;
    if expected_lo > PAGE_SIZE {
        return Err(DbError::corruption(
            "corrupt header: slot directory out of page",
        ));
    }
    if lo != expected_lo {
        return Err(DbError::corruption(
            "corrupt header: lower != header_size + slot_count*slot_size",
        ));
    }
    Ok(())
}

fn reserved(buf: &[u8]) -> Result<u64> {
    read_u64_le(buf, OFF_RESERVED)
}

fn set_reserved(buf: &mut [u8], value: u64) -> Result<()> {
    write_u64_le(buf, OFF_RESERVED, value)
}

fn is_page_type(flags: u16, page_type: u16) -> bool {
    (flags & 0x000F) == (page_type & 0x000F)
}

fn set_page_type(flags: u16, page_type: u16) -> u16 {
    (flags & !0x000F) | (page_type & 0x000F)
}

fn has_free_slots(flags: u16) -> bool {
    (flags & FLAG_HAS_FREE_SLOTS) != 0
}

fn has_flag(flags: u16, mask: u16) -> bool {
    (flags & mask) != 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::{PAGE_TYPE_BTREE_INTERNAL, PAGE_TYPE_BTREE_LEAF, PAGE_TYPE_HEAP};
    use std::mem;

    fn new_page_buf() -> Vec<u8> {
        vec![0; PAGE_SIZE]
    }

    fn check_header_invariants(snapshot: PageHeaderSnapshot) {
        assert!(snapshot.lower() >= SLOTTED_HEADER_SIZE as u16);
        assert!(snapshot.upper() <= PAGE_SIZE as u16);
        assert!(snapshot.lower() <= snapshot.upper());
        assert_eq!(
            SLOTTED_HEADER_SIZE as u16 + snapshot.slot_count() * SLOTTED_SLOT_SIZE as u16,
            snapshot.lower()
        );
    }

    #[test]
    fn flags_helpers_and_set_page_type() {
        let flags = set_flag(PAGE_TYPE_BTREE_INTERNAL, FLAG_HAS_FREE_SLOTS);
        assert!(is_page_type(flags, PAGE_TYPE_BTREE_INTERNAL));
        assert!(has_free_slots(flags));
        let flags2 = set_page_type(flags, PAGE_TYPE_HEAP);
        assert!(is_page_type(flags2, PAGE_TYPE_HEAP));
        assert!(has_free_slots(flags2));
    }

    #[test]
    fn flag_helpers_generic() {
        let mut flags = 0;
        assert!(!has_flag(flags, FLAG_HAS_FREE_SLOTS));
        flags = set_flag(flags, FLAG_HAS_FREE_SLOTS);
        assert!(has_flag(flags, FLAG_HAS_FREE_SLOTS));
        flags = clear_flag(flags, FLAG_HAS_FREE_SLOTS);
        assert!(!has_flag(flags, FLAG_HAS_FREE_SLOTS));
    }

    #[test]
    fn invariants_ok() {
        let snapshot = PageHeaderSnapshot {
            lower: SLOTTED_HEADER_SIZE as u16 + 10 * SLOTTED_SLOT_SIZE as u16,
            upper: PAGE_SIZE as u16,
            slot_count: 10,
            flags: 0,
            reserved: 0,
        };
        check_header_invariants(snapshot);
    }

    #[test]
    #[should_panic]
    fn invariants_fail_lower_formula() {
        let snapshot = PageHeaderSnapshot {
            lower: SLOTTED_HEADER_SIZE as u16 + 1,
            upper: PAGE_SIZE as u16,
            slot_count: 0,
            flags: 0,
            reserved: 0,
        };
        check_header_invariants(snapshot);
    }

    #[test]
    fn init_empty_sets_fields() {
        let mut buf = new_page_buf();
        init_empty_header(&mut buf, PAGE_TYPE_BTREE_INTERNAL).unwrap();
        assert_eq!(SLOTTED_HEADER_SIZE as u16, lower(&buf).unwrap());
        assert_eq!(PAGE_SIZE as u16, upper(&buf).unwrap());
        assert_eq!(0, slot_count(&buf).unwrap());
        assert!(is_page_type(flags(&buf).unwrap(), PAGE_TYPE_BTREE_INTERNAL));
        assert_eq!(0, reserved(&buf).unwrap());
    }

    #[test]
    fn header_setters_roundtrip() {
        let mut buf = new_page_buf();
        init_empty_header(&mut buf, PAGE_TYPE_HEAP).unwrap();
        set_lower(&mut buf, 123).unwrap();
        set_upper(&mut buf, 4000).unwrap();
        set_slot_count(&mut buf, 10).unwrap();
        set_flags(&mut buf, 0x00F2).unwrap();
        set_reserved(&mut buf, 0x1122334455667788).unwrap();

        assert_eq!(123, lower(&buf).unwrap());
        assert_eq!(4000, upper(&buf).unwrap());
        assert_eq!(10, slot_count(&buf).unwrap());
        assert_eq!(0x00F2, flags(&buf).unwrap());
        assert_eq!(0x1122334455667788, reserved(&buf).unwrap());
    }

    #[test]
    fn decode_invalid_size() {
        let buf = vec![0; 100];
        assert!(decode_header(&buf).is_err());
    }

    #[test]
    fn decode_roundtrip_basic() {
        let mut buf = new_page_buf();
        init_empty_header(&mut buf, PAGE_TYPE_BTREE_LEAF).unwrap();
        let current = flags(&buf).unwrap();
        set_flags(&mut buf, set_flag(current, FLAG_HAS_FREE_SLOTS)).unwrap();
        set_reserved(&mut buf, 99).unwrap();
        let snapshot = decode_header(&buf).unwrap();
        assert_eq!(SLOTTED_HEADER_SIZE as u16, snapshot.lower());
        assert_eq!(PAGE_SIZE as u16, snapshot.upper());
        assert_eq!(0, snapshot.slot_count());
        assert!(is_page_type(snapshot.flags(), PAGE_TYPE_BTREE_LEAF));
        assert!(has_free_slots(snapshot.flags()));
        assert_eq!(99, snapshot.reserved());
    }

    #[test]
    fn struct_size_sanity() {
        assert_eq!(16, mem::size_of::<PageHeaderSnapshot>());
    }
}
