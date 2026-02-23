package novadblite

import (
	"io"
	"os"
)

type FilePager struct {
	f        *os.File
	freelist []PageId
	nextPid  PageId
}

func OpenFilePager(path string) (*FilePager, error) {
	file, err := os.OpenFile(path, os.O_RDWR|os.O_CREATE, 0o666)
	if err != nil {
		return nil, wrapIO(err)
	}
	info, err := file.Stat()
	if err != nil {
		_ = file.Close()
		return nil, wrapIO(err)
	}
	length := info.Size()
	if length%int64(PageSize) != 0 {
		_ = file.Close()
		return nil, newCorruption("db file length is not page-aligned")
	}
	pages := uint32(length / int64(PageSize))
	var nextPid PageId
	if pages == 0 {
		zero := make([]byte, PageSize)
		if _, err := file.Write(zero); err != nil {
			_ = file.Close()
			return nil, wrapIO(err)
		}
		if err := file.Sync(); err != nil {
			_ = file.Close()
			return nil, wrapIO(err)
		}
		nextPid = PageId(1)
	} else {
		nextPid = PageId(pages)
	}
	return &FilePager{
		f:        file,
		freelist: nil,
		nextPid:  nextPid,
	}, nil
}

func (p *FilePager) NumPages() (uint64, error) {
	info, err := p.f.Stat()
	if err != nil {
		return 0, wrapIO(err)
	}
	return uint64(info.Size() / int64(PageSize)), nil
}

func (p *FilePager) seekTo(pid PageId) error {
	off := uint64(pid) * uint64(PageSize)
	if off/uint64(PageSize) != uint64(pid) {
		return newCorruption("page offset overflow")
	}
	if _, err := p.f.Seek(int64(off), io.SeekStart); err != nil {
		return wrapIO(err)
	}
	return nil
}

func (p *FilePager) ReadPage(pid PageId, out []byte) error {
	if len(out) != PageSize {
		return newInvalidArgument("buffer length must equal PAGE_SIZE")
	}
	if pid == PageIdInvalid {
		return newInvalidArgument("invalid page id")
	}
	pages, err := p.NumPages()
	if err != nil {
		return err
	}
	if uint64(pid) >= pages {
		return newInvalidArgument("page id out of range")
	}
	if err := p.seekTo(pid); err != nil {
		return err
	}
	if _, err := io.ReadFull(p.f, out); err != nil {
		return wrapIO(err)
	}
	return nil
}

func (p *FilePager) WritePage(pid PageId, buf []byte) error {
	if len(buf) != PageSize {
		return newInvalidArgument("buffer length must equal PAGE_SIZE")
	}
	if pid == PageIdInvalid {
		return newInvalidArgument("invalid page id")
	}
	pages, err := p.NumPages()
	if err != nil {
		return err
	}
	if uint64(pid) >= pages {
		return newInvalidArgument("page id out of range")
	}
	if err := p.seekTo(pid); err != nil {
		return err
	}
	if _, err := p.f.Write(buf); err != nil {
		return wrapIO(err)
	}
	return nil
}

func (p *FilePager) AllocPage() (PageId, error) {
	if len(p.freelist) > 0 {
		pid := p.freelist[len(p.freelist)-1]
		p.freelist = p.freelist[:len(p.freelist)-1]
		return pid, nil
	}
	pid := p.nextPid
	zero := make([]byte, PageSize)
	if err := p.seekTo(pid); err != nil {
		return 0, err
	}
	if _, err := p.f.Write(zero); err != nil {
		return 0, wrapIO(err)
	}
	p.nextPid++
	return pid, nil
}

func (p *FilePager) FreePage(pid PageId) error {
	if pid == 0 || pid == PageIdInvalid {
		return newInvalidArgument("invalid page id")
	}
	p.freelist = append(p.freelist, pid)
	return nil
}

func (p *FilePager) Flush() error {
	return wrapIO(p.f.Sync())
}
