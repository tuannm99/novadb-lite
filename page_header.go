package novadblite

const (
	SlottedHeaderSize = 16
	SlottedSlotSize   = 6
)

const (
	offLower     = 0
	offUpper     = 2
	offSlotCount = 4
	offFlags     = 6
	offReserved  = 8
)

const (
	PageTypeHeap          uint16 = 0
	PageTypeBtreeLeaf     uint16 = 1
	PageTypeBtreeInternal uint16 = 2
	PageTypeBtreeOverflow uint16 = 3
)

const (
	FlagHasFreeSlotsBit  uint16 = 4
	FlagIsCompressedBit  uint16 = 5
	FlagIsChecksummedBit uint16 = 6

	FlagHasFreeSlots  uint16 = 1 << FlagHasFreeSlotsBit
	FlagIsCompressed  uint16 = 1 << FlagIsCompressedBit
	FlagIsChecksummed uint16 = 1 << FlagIsChecksummedBit
)

type PageHeaderSnapshot struct {
	/// lower >= HEADER_SIZE (16)
	/// upper <= PAGE_SIZE
	/// lower <= upper

	lower uint16
	upper uint16

	/// slot_count * SLOT_SIZE + HEADER_SIZE == lower
	/// slot_count là số slot đã cấp phát (không giảm), slot_id < slot_count
	slotCount uint16

	/// flags: bitmask trạng thái ở cấp PAGE
	///
	/// - Bits 0..3  : page_type (0=heap, 1=btree_leaf, 2=btree_internal, 3=overflow, 4..15 reserved)
	/// - Bit  4     : HAS_FREE_SLOTS (trang có slot tombstone để reuse)
	/// - Bit  5     : IS_COMPRESSED (nếu sau này có nén)
	/// - Bit  6     : IS_CHECKSUMMED (nếu bật checksum)
	/// - Bit  7     : RESERVED
	/// - Bits 8..15 : mở rộng sau
	flags uint16

	/// special field, mở rộng sau này (lsn, checksum, future metadata...)
	reserved uint64
}

func (h PageHeaderSnapshot) Upper() uint16 {
	return h.upper
}

func (h PageHeaderSnapshot) Lower() uint16 {
	return h.lower
}

func (h PageHeaderSnapshot) Flags() uint16 {
	return h.flags
}

func (h PageHeaderSnapshot) SlotCount() uint16 {
	return h.slotCount
}

func (h PageHeaderSnapshot) Reserved() uint64 {
	return h.reserved
}

func decodeHeader(buf []byte) (*PageHeaderSnapshot, error) {
	if len(buf) != PageSize {
		return nil, newCorruption("buffer length must equal PAGE_SIZE")
	}
	lower, err := readU16LE(buf, offLower)
	if err != nil {
		return nil, err
	}
	upper, err := readU16LE(buf, offUpper)
	if err != nil {
		return nil, err
	}
	slotCount, err := readU16LE(buf, offSlotCount)
	if err != nil {
		return nil, err
	}
	flags, err := readU16LE(buf, offFlags)
	if err != nil {
		return nil, err
	}
	reserved, err := readU64LE(buf, offReserved)
	if err != nil {
		return nil, err
	}
	return &PageHeaderSnapshot{
		lower:     lower,
		upper:     upper,
		slotCount: slotCount,
		flags:     flags,
		reserved:  reserved,
	}, nil
}

func initEmptyHeader(buf []byte, pageType uint16) error {
	if len(buf) != PageSize {
		return newCorruption("buffer length must equal PAGE_SIZE")
	}
	flags := pageType & 0x000F
	if err := setLower(buf, uint16(SlottedHeaderSize)); err != nil {
		return err
	}
	if err := setUpper(buf, uint16(PageSize)); err != nil {
		return err
	}
	if err := setSlotCount(buf, 0); err != nil {
		return err
	}
	if err := setFlags(buf, flags); err != nil {
		return err
	}
	if err := setReserved(buf, 0); err != nil {
		return err
	}
	return nil
}

func lower(buf []byte) (uint16, error) {
	return readU16LE(buf, offLower)
}

func setLower(buf []byte, v uint16) error {
	return writeU16LE(buf, offLower, v)
}

func upper(buf []byte) (uint16, error) {
	return readU16LE(buf, offUpper)
}

func setUpper(buf []byte, v uint16) error {
	return writeU16LE(buf, offUpper, v)
}

func slotCount(buf []byte) (uint16, error) {
	return readU16LE(buf, offSlotCount)
}

func setSlotCount(buf []byte, v uint16) error {
	return writeU16LE(buf, offSlotCount, v)
}

func flags(buf []byte) (uint16, error) {
	return readU16LE(buf, offFlags)
}

func setFlags(buf []byte, v uint16) error {
	return writeU16LE(buf, offFlags, v)
}

func reserved(buf []byte) (uint64, error) {
	return readU64LE(buf, offReserved)
}

func setReserved(buf []byte, v uint64) error {
	return writeU64LE(buf, offReserved, v)
}

func isPageType(f, t uint16) bool {
	return (f & 0x000F) == (t & 0x000F)
}

func setPageType(f, t uint16) uint16 {
	return (f & ^uint16(0x000F)) | (t & 0x000F)
}

func hasFreeSlots(f uint16) bool {
	return (f & FlagHasFreeSlots) != 0
}

func setFlag(f, mask uint16) uint16 {
	return f | mask
}

func clearFlag(f, mask uint16) uint16 {
	return f & ^mask
}

func hasFlag(f, mask uint16) bool {
	return (f & mask) != 0
}
