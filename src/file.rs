use std::rc::Rc;

#[derive(Debug)]
pub struct File {
    pub path: String,
    pub data: Vec<u8>,
}

pub fn read_file(path: &String) -> Result<Rc<File>, std::io::Error> {
    let file_data = std::fs::read_to_string(path)?;

    Ok(Rc::new(File { path: path.clone(), data: file_data.into_bytes() }))
}

#[derive(Clone, Debug)]
pub struct FilePosition {
    pub file: Rc<File>,
    pub line: usize
}
