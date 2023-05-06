use crate::file::FilePosition;

pub fn report_error(pos: &FilePosition, msg: &str) {
    eprintln!("{}:{}: error {}.", pos.file.path, pos.line, msg);
}
