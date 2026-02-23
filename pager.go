package novadblite

type Pager interface {
	ReadPage(pid PageId, out []byte) error
	WritePage(pid PageId, buf []byte) error
	AllocPage() (PageId, error)
	FreePage(pid PageId) error
	Flush() error
	NumPages() (uint64, error)
}
