package novadblite

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestSlotReadWriteRoundtrip(t *testing.T) {
	buf := make([]byte, PageSize)
	slot := Slot{
		offset: 123,
		length: 45,
		flags:  0x0002,
	}
	require.NoError(t, writeSlot(buf, 0, slot))
	got, err := readSlot(buf, 0)
	require.NoError(t, err)
	require.Equal(t, slot, got)
}

func TestSlotOutOfBounds(t *testing.T) {
	buf := make([]byte, PageSize)
	slot := Slot{offset: 1, length: 1, flags: 0}
	require.Error(t, writeSlot(buf, ^uint16(0), slot))
	_, err := readSlot(buf, ^uint16(0))
	require.Error(t, err)
}

func TestIsDead(t *testing.T) {
	require.True(t, isDead(1<<0))
	require.False(t, isDead(0))
}

func TestFlagsHelpers(t *testing.T) {
	require.True(t, isRedirected(1<<1))
	require.True(t, isOverflow(1<<2))
	require.False(t, isRedirected(0))
	require.False(t, isOverflow(0))
}
