use crate::{ast::*, error_reporter::report_error, file::FilePosition};

pub struct Validator {
    had_error: bool,
    global_data: Vec<String>,
    local_data: Vec<String>,
    loop_count: usize,
}

impl Validator {
    pub fn new() -> Self {
        Validator {
            had_error: false,
            global_data: Vec::new(),
            local_data: Vec::new(),
            loop_count: 0
        }
    }

    pub fn validate_one_decl(&mut self, decl: &Decl) -> bool {
        match &decl.kind {
            DeclKind::External(var) => {
                self.add_global(&var.0, &decl.pos);
                self.validate_iexpr(&var.1);
            },

            DeclKind::Function { name, params, body } => {
                self.add_global(&name, &decl.pos);

                for param in params {
                    self.add_local(param, &decl.pos);
                }

                self.validate_stmt(body);
            }
        }

        let old_error = self.had_error;
        self.clear();
        old_error
    }

    fn validate_stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::Auto(var) => {
                // The scope of a variable begins with its declaration
                // and ends with the end of the function where it was delared.
                self.add_local(&var.0, &stmt.pos);
                self.validate_iexpr(&var.1);
            },

            StmtKind::Extern(name) => {
                self.add_local(name, &stmt.pos)
            },

            StmtKind::Expr(expr) => {
                self.validate_expr(expr)
            }

            StmtKind::Block(stmts) => {
                for stmt in stmts {
                    self.validate_stmt(stmt)
                }
            },

            StmtKind::Break => {
                if self.loop_count == 0 {
                    self.error(&stmt.pos, "break statement appeared outside of loop");
                }
            },

            StmtKind::Continue => {
                if self.loop_count == 0 {
                    self.error(&stmt.pos, "continue statement appeared outside of loop");
                }
            },

            StmtKind::While(cond, body) => {
                self.validate_expr(cond);

                self.loop_count += 1;

                if let Some(body) = body {
                    self.validate_stmt(body);
                }

                self.loop_count -= 1;
            },

            StmtKind::DoWhile(cond, body) => {
                self.loop_count += 1;
                self.validate_stmt(body);
                self.loop_count -= 1;

                self.validate_expr(cond);
            },

            StmtKind::If(cond, then_arm, else_arm) => {
                self.validate_expr(cond);

                self.validate_stmt(then_arm);

                if let Some(else_arm) = else_arm {
                    self.validate_stmt(else_arm);
                }
            },

            StmtKind::Return(expr) => {
                if let Some(expr) = expr {
                    self.validate_expr(expr);
                }
            }
        }
    }

    fn validate_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::IntLit(_) => {},
            ExprKind::StringLit(_) => {},

            ExprKind::Var(name) => {
                if !self.local_data.contains(name) {
                    self.error(&expr.pos, &format!("undefinded reference to '{}'", name));
                }
            },

            ExprKind::UnaryOp(_, _, expr) => {
                self.validate_expr(expr)
            },

            ExprKind::BinOp(left, _, right) => {
                self.validate_expr(left);
                self.validate_expr(right);
            },

            ExprKind::Ternary(cond, then_arm, else_arm) => {
                self.validate_expr(cond);
                self.validate_expr(then_arm);
                self.validate_expr(else_arm);
            }
        }
    }

    fn validate_iexpr(&mut self, expr: &Option<Expr>) {
        match expr {
            None => {},
            Some(e) => match e.kind {
                ExprKind::IntLit(_) => {},
                ExprKind::StringLit(_) => {},

                _ => {
                    self.error(&e.pos, "only integer or string literals are allowed to be iexpr");
                }
            }
        }
    }

    fn add_global(&mut self, name: &String, err_pos: &FilePosition) {
        // Rust doesn't allow me to create function safe_push with mutable vector reference
        // to remove code duplication, because there will be two mutable references:
        // first - in add_global parameter, second - in safe_push parameter.
        if self.global_data.contains(&name) {
            self.error(err_pos, &format!("redefinition of global '{}'", name));
        } else {
            self.global_data.push(name.clone())
        }
    }

    fn add_local(&mut self, name: &String, err_pos: &FilePosition) {
        if self.local_data.contains(&name) {
            self.error(err_pos, &format!("redefinition of local '{}'", name));
        } else {
            self.local_data.push(name.clone())
        }
    }

    fn clear(&mut self) {
        self.had_error = false;
        self.local_data.clear()
    }

    fn error(&mut self, pos: &FilePosition, msg: &str) {
        self.had_error = true;
        report_error(pos, msg);
    }
}
