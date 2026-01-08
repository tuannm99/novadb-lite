/// page header fixed 16 bytes, trong đó có 8 bytes là reserved
#[derive(Debug)]
pub struct PageHeader {
    /// lower >= HEADER_SIZE (16)
    /// upper <= PAGE_SIZE
    /// lower <= upper
    lower: u16,
    upper: u16,

    /// slot_count * SLOT_SIZE + HEADER_SIZE == lower
    /// slot_count là số slot đã cấp phát (không giảm), slot_id < slot_count
    slot_count: u16,

    /// flags: bitmask trạng thái ở cấp PAGE
    ///
    /// - Bits 0..3  : page_type (0=heap, 1=btree_leaf, 2=btree_internal, 3=overflow, 4..15 reserved)
    /// - Bit  4     : HAS_FREE_SLOTS (trang có slot tombstone để reuse)
    /// - Bit  5     : IS_COMPRESSED (nếu sau này có nén)
    /// - Bit  6     : IS_CHECKSUMMED (nếu bật checksum)
    /// - Bit  7     : RESERVED
    /// - Bits 8..15 : mở rộng sau
    flags: u16,

    /// special field, mở rộng sau này (lsn, checksum, page_type)
    reserved: u64,
}

impl PageHeader {
    pub fn upper(&self) -> u16 {
        return self.upper;
    }

    pub fn lower(&self) -> u16 {
        return self.lower;
    }

    pub fn flags(&self) -> u16 {
        return self.flags;
    }
    pub fn slot_count(&self) -> u16 {
        return self.slot_count;
    }

    pub fn reserved(&self) -> u64 {
        return self.reserved;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::PAGE_SIZE;

    // Giữ các hằng số này thống nhất với thiết kế module page của bạn.
    const HEADER_SIZE: u16 = 16;
    const SLOT_SIZE: u16 = 6; // offset:u16 + len:u16 + flags:u16

    /// Kiểm tra các invariant cơ bản của PageHeader.
    /// Lưu ý: kiểm tra ở cấp struct (snapshot), chưa liên quan đến on-disk bytes.
    fn check_invariants(h: &PageHeader) {
        // lower phải bắt đầu sau header
        assert!(h.lower() >= HEADER_SIZE, "lower must be >= HEADER_SIZE");

        // upper không được vượt quá kích thước page
        assert!(h.upper() <= PAGE_SIZE as u16, "upper must be <= PAGE_SIZE");

        // lower luôn nằm trước hoặc bằng upper (free space = upper - lower)
        assert!(h.lower() <= h.upper(), "lower must be <= upper");

        // Nếu bạn chọn quy ước: lower = HEADER_SIZE + slot_count * SLOT_SIZE
        // thì công thức này phải đúng (slot_count là số slot đã cấp phát).
        let expected_lower = HEADER_SIZE + h.slot_count() * SLOT_SIZE;
        assert_eq!(
            h.lower(),
            expected_lower,
            "lower must equal HEADER_SIZE + slot_count*SLOT_SIZE"
        );
    }

    #[test]
    fn test_getters() {
        let h = PageHeader {
            lower: HEADER_SIZE,
            upper: PAGE_SIZE as u16,
            slot_count: 0,
            flags: 0,
            reserved: 0,
        };

        assert_eq!(h.lower(), HEADER_SIZE);
        assert_eq!(h.upper(), PAGE_SIZE as u16);
        assert_eq!(h.slot_count(), 0);
        assert_eq!(h.flags(), 0);
        assert_eq!(h.reserved(), 0);
    }

    #[test]
    fn test_flags_bits() {
        // sample
        // - page_type = 2 (btree_internal) ở bits 0..3
        // - HAS_FREE_SLOTS ở bit 4
        let page_type: u16 = 2;
        let has_free_slots: u16 = 1 << 4;

        let h = PageHeader {
            lower: HEADER_SIZE,
            upper: PAGE_SIZE as u16,
            slot_count: 0,
            flags: (page_type & 0x000F) | has_free_slots,
            reserved: 0,
        };

        let flags = h.flags();
        let extracted_page_type = flags & 0x000F;

        // Kiểm tra bit HAS_FREE_SLOTS
        let extracted_has_free = (flags & (1 << 4)) != 0;

        assert_eq!(extracted_page_type, 2);
        assert!(extracted_has_free);
    }

    #[test]
    fn test_invariants_ok() {
        // valid header theo công thức lower = HEADER + slot_count*SLOT_SIZE
        let slot_count = 10;
        let lower = HEADER_SIZE + slot_count * SLOT_SIZE;

        let h = PageHeader {
            lower,
            upper: PAGE_SIZE as u16,
            slot_count,
            flags: 0,
            reserved: 0,
        };

        check_invariants(&h);
    }

    #[test]
    #[should_panic]
    fn test_invariants_fail_lower_formula() {
        // header sai cần đảm bảo invariant sẽ "bắt lỗi"
        let h = PageHeader {
            lower: HEADER_SIZE + 1, // sai công thức
            upper: PAGE_SIZE as u16,
            slot_count: 0,
            flags: 0,
            reserved: 0,
        };

        check_invariants(&h);
    }

    #[test]
    fn test_struct_size_sanity() {
        // sanity check: struct có đúng 16 bytes trong build hiện tại.
        // KHÔNG được dựa vào layout struct để serialize trực tiếp ra file.
        assert_eq!(std::mem::size_of::<PageHeader>(), 16);
    }
}
