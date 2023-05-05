use std::rc::Rc;

#[derive(Debug)]
pub struct File {
    pub path: String,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct FilePosition {
    pub file: Rc<File>,
    pub line: usize
}
