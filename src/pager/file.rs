pub trait PageIO {
    fn seek();
    fn read();
    fn sync();
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Pager {}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Segnment {}
