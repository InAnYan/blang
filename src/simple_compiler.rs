use crate::ast::Decl;

pub struct Compiler {
    code: String
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            code: String::new() // TODO: Add init code.
        }
    }

    pub fn compile_one_decl(&mut self, _decl: &Decl) {
        
    }

    pub fn get_code(&self) -> &String {
        &self.code
    }
}
