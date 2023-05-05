use crate::file::FilePosition;

pub fn report_error(pos: &FilePosition, msg: &String) {
    eprintln!("{}:{}: error {}.", pos.file.path, pos.line, msg);
}
