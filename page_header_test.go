package novadblite

import (
	"testing"
	"unsafe"

	"github.com/stretchr/testify/require"
)

func newPageBuf() []byte {
	return make([]byte, PageSize)
}

func checkHeaderInvariants(t *testing.T, h PageHeaderSnapshot) {
	t.Helper()
	headerSize := uint16(SlottedHeaderSize)
	slotSize := uint16(SlottedSlotSize)
	pageSize := uint16(PageSize)

	require.GreaterOrEqual(t, h.Lower(), headerSize, "lower < header size")
	require.LessOrEqual(t, h.Upper(), pageSize, "upper > page size")
	require.LessOrEqual(t, h.Lower(), h.Upper(), "lower > upper")
	expectedLower := headerSize + h.SlotCount()*slotSize
	require.Equal(t, expectedLower, h.Lower(), "lower formula mismatch")
}

func checkHeaderInvariantsPanic(h PageHeaderSnapshot) {
	headerSize := uint16(SlottedHeaderSize)
	slotSize := uint16(SlottedSlotSize)
	pageSize := uint16(PageSize)

	if h.Lower() < headerSize {
		panic("lower < header size")
	}
	if h.Upper() > pageSize {
		panic("upper > page size")
	}
	if h.Lower() > h.Upper() {
		panic("lower > upper")
	}
	expectedLower := headerSize + h.SlotCount()*slotSize
	if h.Lower() != expectedLower {
		panic("lower formula mismatch")
	}
}

func TestFlagsHelpersAndSetPageType(t *testing.T) {
	flags := setFlag(PageTypeBtreeInternal, FlagHasFreeSlots)
	require.True(t, isPageType(flags, PageTypeBtreeInternal), "expected btree internal page type")
	require.True(t, hasFreeSlots(flags), "expected has free slots")
	flags2 := setPageType(flags, PageTypeHeap)
	require.True(t, isPageType(flags2, PageTypeHeap), "expected heap page type")
	require.True(t, hasFreeSlots(flags2), "expected flags preserved")
}

func TestFlagHelpersGeneric(t *testing.T) {
	var f uint16
	require.False(t, hasFlag(f, FlagHasFreeSlots), "expected flag unset")
	f = setFlag(f, FlagHasFreeSlots)
	require.True(t, hasFlag(f, FlagHasFreeSlots), "expected flag set")
	f = clearFlag(f, FlagHasFreeSlots)
	require.False(t, hasFlag(f, FlagHasFreeSlots), "expected flag cleared")
}

func TestInvariantsOK(t *testing.T) {
	slotCount := uint16(10)
	lowerVal := uint16(SlottedHeaderSize) + slotCount*uint16(SlottedSlotSize)
	h := PageHeaderSnapshot{
		lower:     lowerVal,
		upper:     uint16(PageSize),
		slotCount: slotCount,
		flags:     0,
		reserved:  0,
	}
	checkHeaderInvariants(t, h)
}

func TestInvariantsFailLowerFormula(t *testing.T) {
	defer func() {
		if r := recover(); r == nil {
			t.Fatalf("expected panic")
		}
	}()
	h := PageHeaderSnapshot{
		lower:     uint16(SlottedHeaderSize) + 1,
		upper:     uint16(PageSize),
		slotCount: 0,
		flags:     0,
		reserved:  0,
	}
	checkHeaderInvariantsPanic(h)
}

func TestInitEmptySetsFields(t *testing.T) {
	buf := newPageBuf()
	require.NoError(t, initEmptyHeader(buf, PageTypeBtreeInternal))
	gotLower, _ := lower(buf)
	gotUpper, _ := upper(buf)
	gotSlotCount, _ := slotCount(buf)
	gotFlags, _ := flags(buf)
	gotReserved, _ := reserved(buf)

	require.Equal(t, uint16(SlottedHeaderSize), gotLower, "lower mismatch")
	require.Equal(t, uint16(PageSize), gotUpper, "upper mismatch")
	require.Equal(t, uint16(0), gotSlotCount, "slot_count mismatch")
	require.True(t, isPageType(gotFlags, PageTypeBtreeInternal), "page type mismatch")
	require.Equal(t, uint64(0), gotReserved, "reserved mismatch")
}

func TestHeaderSettersRoundtrip(t *testing.T) {
	buf := newPageBuf()
	require.NoError(t, initEmptyHeader(buf, PageTypeHeap))
	require.NoError(t, setLower(buf, 123))
	require.NoError(t, setUpper(buf, 4000))
	require.NoError(t, setSlotCount(buf, 10))
	require.NoError(t, setFlags(buf, 0x00F2))
	require.NoError(t, setReserved(buf, 0x1122334455667788))

	gotLower, _ := lower(buf)
	gotUpper, _ := upper(buf)
	gotSlotCount, _ := slotCount(buf)
	gotFlags, _ := flags(buf)
	gotReserved, _ := reserved(buf)

	require.Equal(t, uint16(123), gotLower)
	require.Equal(t, uint16(4000), gotUpper)
	require.Equal(t, uint16(10), gotSlotCount)
	require.Equal(t, uint16(0x00F2), gotFlags)
	require.Equal(t, uint64(0x1122334455667788), gotReserved)
}

func TestDecodeInvalidSize(t *testing.T) {
	buf := make([]byte, 100)
	_, err := decodeHeader(buf)
	require.Error(t, err)
}

func TestDecodeRoundtripBasic(t *testing.T) {
	buf := newPageBuf()
	require.NoError(t, initEmptyHeader(buf, PageTypeBtreeLeaf))
	cur, _ := flags(buf)
	require.NoError(t, setFlags(buf, setFlag(cur, FlagHasFreeSlots)))
	require.NoError(t, setReserved(buf, 99))
	h, err := decodeHeader(buf)
	require.NoError(t, err)
	require.Equal(t, uint16(SlottedHeaderSize), h.Lower())
	require.Equal(t, uint16(PageSize), h.Upper())
	require.Equal(t, uint16(0), h.SlotCount())
	require.True(t, isPageType(h.Flags(), PageTypeBtreeLeaf), "page type mismatch")
	require.True(t, hasFreeSlots(h.Flags()), "flag missing")
	require.Equal(t, uint64(99), h.Reserved())
}

func TestStructSizeSanity(t *testing.T) {
	require.Equal(t, uintptr(16), unsafe.Sizeof(PageHeaderSnapshot{}))
}
