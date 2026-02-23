package novadblite

import "fmt"

type ErrorKind string

const (
	ErrIO              ErrorKind = "io"
	ErrOutOfBounds     ErrorKind = "out_of_bounds"
	ErrCorruption      ErrorKind = "corruption"
	ErrNoSpace         ErrorKind = "no_space"
	ErrInvalidArgument ErrorKind = "invalid_argument"
)

type DbError struct {
	Kind ErrorKind
	Off  int
	Size int
	Len  int
	Msg  string
	Err  error
}

func (e *DbError) Error() string {
	switch e.Kind {
	case ErrIO:
		if e.Err != nil {
			return fmt.Sprintf("io error: %v", e.Err)
		}
		return "io error"
	case ErrOutOfBounds:
		return fmt.Sprintf("out of bounds: off=%d size=%d len=%d", e.Off, e.Size, e.Len)
	case ErrCorruption:
		return fmt.Sprintf("corruption: %s", e.Msg)
	case ErrNoSpace:
		return fmt.Sprintf("no space: %s", e.Msg)
	case ErrInvalidArgument:
		return fmt.Sprintf("invalid args: %s", e.Msg)
	default:
		if e.Msg != "" {
			return e.Msg
		}
		return "db error"
	}
}

func (e *DbError) Unwrap() error {
	return e.Err
}

func newOutOfBounds(off, size, length int) error {
	return &DbError{
		Kind: ErrOutOfBounds,
		Off:  off,
		Size: size,
		Len:  length,
	}
}

func newCorruption(msg string) error {
	return &DbError{Kind: ErrCorruption, Msg: msg}
}

func newNoSpace(msg string) error {
	return &DbError{Kind: ErrNoSpace, Msg: msg}
}

func newInvalidArgument(msg string) error {
	return &DbError{Kind: ErrInvalidArgument, Msg: msg}
}

func wrapIO(err error) error {
	if err == nil {
		return nil
	}
	return &DbError{Kind: ErrIO, Err: err}
}
