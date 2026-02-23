package novadblite

type PageId uint32

const (
	PageSize  = 4096
	DbMagic   = "NOVADBLITE\x00\x00"
	DbVersion = uint16(1)
)

const PageIdInvalid PageId = ^PageId(0)

func (p PageId) AsU32() uint32 {
	return uint32(p)
}

func (p PageId) AsU64() uint64 {
	return uint64(p)
}

func (p PageId) AsInt() int {
	return int(p)
}
