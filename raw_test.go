package novadblite

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestReadWriteU32(t *testing.T) {
	buf := make([]byte, 16)
	require.NoError(t, writeU32LE(buf, 4, 0x11223344))
	v, err := readU32LE(buf, 4)
	require.NoError(t, err)
	require.Equal(t, uint32(0x11223344), v)
}

func TestReadWriteU64(t *testing.T) {
	buf := make([]byte, 32)
	require.NoError(t, writeU64LE(buf, 8, 0x1122334455667788))
	v, err := readU64LE(buf, 8)
	require.NoError(t, err)
	require.Equal(t, uint64(0x1122334455667788), v)
}

func TestOutOfBounds(t *testing.T) {
	buf := make([]byte, 8)
	err := writeU64LE(buf, 4, 1)
	require.Error(t, err)
	_, ok := err.(*DbError)
	require.True(t, ok, "expected DbError, got %T", err)
}
