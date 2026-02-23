package novadblite

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func makePage(t *testing.T, buf []byte) *SlottedPage {
	t.Helper()
	p, err := NewSlottedPage(buf)
	require.NoError(t, err)
	p, err = p.Init(PageTypeHeap)
	require.NoError(t, err)
	return p
}

func TestNewRejectsWrongSize(t *testing.T) {
	buf := make([]byte, 15)
	_, err := NewSlottedPage(buf)
	require.Error(t, err)
}

func TestNewAcceptsPageSize(t *testing.T) {
	buf := make([]byte, PageSize)
	_, err := NewSlottedPage(buf)
	require.NoError(t, err)
}

func TestSlottedPageValidate(t *testing.T) {
	buf := make([]byte, PageSize)
	p := makePage(t, buf)
	free, err := p.FreeSpace()
	require.NoError(t, err)
	require.Equal(t, uint16(PageSize-SlottedHeaderSize), free)
	require.NoError(t, p.ValidateFull())
}

func TestSlottedPageGet(t *testing.T) {
	buf := make([]byte, PageSize)
	p := makePage(t, buf)
	data1 := []byte("Hello, world")
	id0, err := p.Insert(data1)
	require.NoError(t, err)
	require.Equal(t, uint16(0), id0)
	data2 := []byte("Hello, world.. TUANNM")
	id1, err := p.Insert(data2)
	require.NoError(t, err)
	require.Equal(t, uint16(1), id1)
	headerSnap, err := decodeHeader(buf)
	require.NoError(t, err)
	require.Equal(t, uint16(SlottedHeaderSize+6+6), headerSnap.Lower())
	require.Equal(t, PageSize-len(data1)-len(data2), int(headerSnap.Upper()))
	require.Equal(t, uint16(2), headerSnap.SlotCount())
}

func TestFindFreeSlot(t *testing.T) {
	buf := make([]byte, PageSize)
	p := makePage(t, buf)
	if got, err := p.findFreeSlot(); err != nil || got != nil {
		require.Fail(t, "expected no free slot")
	}
	id0, err := p.Insert([]byte("Hello, world"))
	require.NoError(t, err)
	require.Equal(t, uint16(0), id0)
	sc, _ := slotCount(buf)
	require.Equal(t, uint16(1), sc)
	lo, _ := lower(buf)
	require.Equal(t, SlottedHeaderSize+SlottedSlotSize, int(lo))
	if got, err := p.findFreeSlot(); err != nil || got != nil {
		require.Fail(t, "expected no free slot after insert")
	}
	require.NoError(t, p.Delete(0))
	if got, err := p.findFreeSlot(); err != nil || got == nil {
		require.Fail(t, "expected free slot")
	}
	idReuse, err := p.Insert([]byte("Hello, "))
	require.NoError(t, err)
	require.Equal(t, uint16(0), idReuse)
	sc, _ = slotCount(buf)
	require.Equal(t, uint16(1), sc)
	if got, err := p.findFreeSlot(); err != nil || got != nil {
		require.Fail(t, "expected no free slot after reuse")
	}
	require.NoError(t, p.Delete(0))
	idReuse2, err := p.Insert([]byte("Hello, Tuannm string larger"))
	require.NoError(t, err)
	require.Equal(t, uint16(0), idReuse2)
	fl, _ := flags(buf)
	require.NotZero(t, fl&FlagHasFreeSlots)
	if got, err := p.findFreeSlot(); err != nil || got != nil {
		require.Fail(t, "expected no free slot after scan")
	}
	fl, _ = flags(buf)
	require.Zero(t, fl&FlagHasFreeSlots)
}

func TestSlottedPageInsert(t *testing.T) {
	buf := make([]byte, PageSize)
	p := makePage(t, buf)
	free0, _ := p.FreeSpace()
	require.Equal(t, PageSize-SlottedHeaderSize, int(free0))
	d1 := []byte("abc")
	id0, err := p.Insert(d1)
	require.NoError(t, err)
	require.Equal(t, uint16(0), id0)
	sc, _ := slotCount(buf)
	require.Equal(t, uint16(1), sc)
	lo, _ := lower(buf)
	require.Equal(t, SlottedHeaderSize+SlottedSlotSize, int(lo))
	up, _ := upper(buf)
	require.Equal(t, PageSize-len(d1), int(up))
	got, ok, err := p.Get(id0)
	require.NoError(t, err)
	require.True(t, ok)
	require.Equal(t, string(d1), string(got))
	d2 := []byte("hello world")
	id1, err := p.Insert(d2)
	require.NoError(t, err)
	require.Equal(t, uint16(1), id1)
	sc, _ = slotCount(buf)
	require.Equal(t, uint16(2), sc)
	lo, _ = lower(buf)
	require.Equal(t, SlottedHeaderSize+2*SlottedSlotSize, int(lo))
	up, _ = upper(buf)
	require.Equal(t, PageSize-len(d1)-len(d2), int(up))
	got2, ok, err := p.Get(id1)
	require.NoError(t, err)
	require.True(t, ok)
	require.Equal(t, string(d2), string(got2))
	free, _ := p.FreeSpace()
	huge := make([]byte, int(free)+1)
	_, err = p.Insert(huge)
	require.Error(t, err)
	require.NoError(t, p.ValidateHeader())
	require.NoError(t, p.ValidateFull())
}

func TestSlottedPageUpdate(t *testing.T) {
	buf := make([]byte, PageSize)
	p := makePage(t, buf)
	id, err := p.Insert([]byte("hello world"))
	require.NoError(t, err)
	require.Equal(t, uint16(0), id)
	moved, err := p.Update(id, []byte("hi"))
	require.NoError(t, err)
	require.False(t, moved)
	got, ok, err := p.Get(id)
	require.NoError(t, err)
	require.True(t, ok)
	require.Equal(t, "hi", string(got))
	upAfterInplace, _ := upper(buf)
	big := []byte("this is a longer string than before")
	moved2, err := p.Update(id, big)
	require.NoError(t, err)
	require.True(t, moved2)
	got2, ok, err := p.Get(id)
	require.NoError(t, err)
	require.True(t, ok)
	require.Equal(t, string(big), string(got2))
	upAfterMove, _ := upper(buf)
	require.Less(t, upAfterMove, upAfterInplace, "upper should decrease after move")
	_, err = p.Update(99, []byte("x"))
	require.Error(t, err)
	require.NoError(t, p.Delete(id))
	_, err = p.Update(id, []byte("x"))
	require.Error(t, err)
	require.NoError(t, p.ValidateHeader())
	require.NoError(t, p.ValidateFull())
}

func TestSlottedPageDelete(t *testing.T) {
	buf := make([]byte, PageSize)
	p := makePage(t, buf)
	id0, err := p.Insert([]byte("a"))
	require.NoError(t, err)
	require.Equal(t, uint16(0), id0)
	id1, err := p.Insert([]byte("b"))
	require.NoError(t, err)
	require.Equal(t, uint16(1), id1)
	require.NoError(t, p.Delete(id0))
	_, ok, _ := p.Get(id0)
	require.False(t, ok, "expected nil for deleted slot")
	got1, ok, err := p.Get(id1)
	require.NoError(t, err)
	require.True(t, ok)
	require.Equal(t, "b", string(got1))
	require.NoError(t, p.Delete(id0))
	fl, _ := flags(buf)
	require.NotZero(t, fl&FlagHasFreeSlots)
	require.Error(t, p.Delete(99))
	require.NoError(t, p.ValidateHeader())
	require.NoError(t, p.ValidateFull())
}

func TestSlottedPageRoundtrip(t *testing.T) {
	buf := make([]byte, PageSize)
	p := makePage(t, buf)
	id0, _ := p.Insert([]byte("r0"))
	id1, _ := p.Insert([]byte("record-1"))
	id2, _ := p.Insert([]byte("record-2222"))
	id3, _ := p.Insert([]byte("r3"))
	require.Equal(t, uint16(0), id0)
	require.Equal(t, uint16(1), id1)
	require.Equal(t, uint16(2), id2)
	require.Equal(t, uint16(3), id3)
	moved, _ := p.Update(id1, []byte("X"))
	require.False(t, moved)
	got, ok, _ := p.Get(id1)
	require.True(t, ok)
	require.Equal(t, "X", string(got))
	big := []byte("this update will move because it's longer than before")
	moved, _ = p.Update(id0, big)
	require.True(t, moved)
	got, ok, _ = p.Get(id0)
	require.True(t, ok)
	require.Equal(t, string(big), string(got))
	_ = p.Delete(id2)
	_ = p.Delete(id3)
	_, ok, _ = p.Get(id2)
	require.False(t, ok)
	_, ok, _ = p.Get(id3)
	require.False(t, ok)
	idReuse, err := p.Insert([]byte("reuse"))
	require.NoError(t, err)
	require.True(t, idReuse == id2 || idReuse == id3, "expected reuse dead slot")
	got, ok, _ = p.Get(idReuse)
	require.True(t, ok)
	require.Equal(t, "reuse", string(got))
	require.NoError(t, p.ValidateHeader())
	require.NoError(t, p.ValidateFull())
	got, ok, _ = p.Get(id0)
	require.True(t, ok)
	require.Equal(t, string(big), string(got))
	got, ok, _ = p.Get(id1)
	require.True(t, ok)
	require.Equal(t, "X", string(got))
	otherDead := id2
	if idReuse == id2 {
		otherDead = id3
	}
	_, ok, _ = p.Get(otherDead)
	require.False(t, ok, "expected other dead slot to be nil")
}
