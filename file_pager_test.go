package novadblite

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestFilePagerAllocWriteRead(t *testing.T) {
	dir := t.TempDir()
	path := filepath.Join(dir, "test.db")
	pager, err := OpenFilePager(path)
	require.NoError(t, err)
	defer pager.f.Close()

	pid, err := pager.AllocPage()
	require.NoError(t, err)
	buf := make([]byte, PageSize)
	for i := range buf {
		buf[i] = byte(i % 255)
	}
	require.NoError(t, pager.WritePage(pid, buf))
	out := make([]byte, PageSize)
	require.NoError(t, pager.ReadPage(pid, out))
	require.Equal(t, string(buf), string(out))
}

func TestFilePagerFreeReuse(t *testing.T) {
	dir := t.TempDir()
	path := filepath.Join(dir, "test2.db")
	pager, err := OpenFilePager(path)
	require.NoError(t, err)
	defer pager.f.Close()

	pid1, err := pager.AllocPage()
	require.NoError(t, err)
	pid2, err := pager.AllocPage()
	require.NoError(t, err)
	require.NoError(t, pager.FreePage(pid2))
	pidReuse, err := pager.AllocPage()
	require.NoError(t, err)
	require.Equal(t, pid2, pidReuse, "expected reuse pid %d, got %d (pid1=%d)", pid2, pidReuse, pid1)
}

func TestFilePagerRejectsMisalignedFile(t *testing.T) {
	dir := t.TempDir()
	path := filepath.Join(dir, "bad.db")
	require.NoError(t, os.WriteFile(path, []byte("bad"), 0o644))
	_, err := OpenFilePager(path)
	require.Error(t, err)
}
