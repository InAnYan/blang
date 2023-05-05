use crate::ast::*;

pub fn validate_decl(decl: &Decl) -> bool {
    let mut global_data = Vec::new();
    let mut symbols = Vec::new();

    match &decl.kind {
        DeclKind::Data { name, is_array, size, initial } => {
            // If you write external declaration: 'a  "Hello, World!"', the a variable will have a pointer to the string.
            // If you write: 'a[4] "abc"', then it will be an array of strlen(str) + 1, and it should be.
            // If you write: 'a <expr>', the <expr> should be either string literal, character literal or integer.
            // The same applies to auto statements.
            if is_array {
                validate_array(size, initial)
            }
        }
    }
}
