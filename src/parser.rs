use crate::ast::*;
use crate::file::*;
use crate::scanner::*;
use crate::token::*;
use crate::error_reporter::report_error;

use std::rc::Rc;

pub struct Parser<'a> {
    scanner: &'a mut Scanner,
    previous_token: Token,
    current_token: Token,
    had_error: bool
}

struct ParserError {
    pos: FilePosition,
    msg: String
}

macro_rules! parse_expression_type {
    ($name:ident, $inner:ident, $type:expr $(, $types:expr)*) => {
        fn $name(&mut self) -> Result<Expr, ParserError> {
            let mut left = self.$inner()?;

            while self.matching($type) $( || self.matching($types) )* {
                let token = self.previous_token.clone();

                let right = self.$inner()?;

                left = Expr {
                    pos: token.pos,
                    kind: ExprKind::BinOp(Box::new(left), token.kind, Box::new(right))
                }
            }

            Ok(left)
        }
    }
}

impl<'a> Parser<'a> {
    pub fn new(scanner: &mut Scanner) -> Parser {
        let token = scanner.next_token();
        let dummy_token = Token {
            kind: TokenType::Error,
            pos: FilePosition {
                file: Rc::new(File {
                    path: String::from(""),
                    data: Vec::new()
                }),
                line: 0
            },
            data: String::from("")
        };

        Parser {
            scanner,
            previous_token: dummy_token,
            current_token: token,
            had_error: false
        }
    }

    pub fn parse(&mut self) -> Vec<Decl> {
        let mut res = Vec::new();

        while !self.is_at_end() {
            match self.parse_decl() {
                Ok(decl) => res.push(decl),
                Err(e) => {
                    report_error(&e.pos, &e.msg);
                    self.synchronize_decl()
                }
            }
        }

        res
    }

    pub fn had_error(&self) -> bool {
        self.had_error
    }

    fn synchronize_decl(&mut self) {
        self.advance();

        while !self.is_at_end() {
            match self.current_token.kind {
                TokenType::Identifier => break,
                _ => self.advance()
            }
        }
    }

    fn is_at_end(&self) -> bool {
        self.current_token.kind == TokenType::EndOfFile
    }

    fn parse_decl(&mut self) -> Result<Decl, ParserError> {
        if self.matching(TokenType::Identifier) {
            let name = self.previous_token.clone();
            if self.matching(TokenType::LeftParen) {
                self.continue_parse_fn(name)
            } else {
                self.continue_parse_var(name)
            }
        } else {
            Err(self.error_at_current("expected declaration"))
        }
    }

    fn parse_stmt(&mut self) -> Result<Stmt, ParserError> {
        if self.matching(TokenType::LeftBracket) {
            self.parse_block_stmt()
        } else if self.matching(TokenType::KeywordReturn) {
            self.parse_return_stmt()
        } else if self.matching(TokenType::KeywordIf) {
            self.parse_if_stmt()
        } else if self.matching(TokenType::KeywordAuto) {
            self.parse_auto_stmt()
        } else if self.matching(TokenType::KeywordExtern) {
            self.parse_extern_stmt()
        } else if self.matching(TokenType::KeywordWhile) {
            self.parse_while_stmt()
        } else if self.matching(TokenType::KeywordDo) {
            self.parse_do_stmt()
        } else if self.matching(TokenType::KeywordBreak) {
            self.parse_break_stmt()
        } else if self.matching(TokenType::KeywordContinue) {
            self.parse_continue_stmt()
        } else {
            self.parse_expr_stmt()
        }
    }
    
    fn parse_auto_stmt(&mut self) -> Result<Stmt, ParserError> {
        let name = self.require(TokenType::Identifier, "expected variable name")?;

        let var = self.continue_parse_fn(name)?;
        let kind = match var.kind {
            DeclKind::Data { name, count, initial } => StmtKind::Auto(name, count, initial),
            _ => panic!()
        };

        Ok(Stmt {
            pos: var.pos,
            kind
        })
    }

    fn continue_parse_var(&mut self, name: Token) -> Result<Decl, ParserError> {
        let count = if self.matching(TokenType::LeftBrace) {
            let count_token = self.require(TokenType::IntLiteral, "expected size of array")?;
            self.require(TokenType::RightBrace, "expected ']' after array size")?;

            match count_token.data.parse::<i32>() {
                Ok(num) => num,
                Err(e) => return Err(self.error_at_current(&e.to_string()))
            }
        } else {
            1
        };

        let initial = if !self.matching(TokenType::Semicolon) {
            let res = self.parse_expr()?;
            self.require(TokenType::Semicolon, "expected ';' at the end of declaration")?;
            Some(res)
        } else {
            None
        };

        Ok(Decl {
            pos: name.pos,
            kind: DeclKind::Data {
                name: name.data,
                count,
                initial
            }
        })
    }

    fn parse_extern_stmt(&mut self) -> Result<Stmt, ParserError> {
        let token = self.previous_token.clone();

        let name = self.require(TokenType::Identifier, "expected identifier")?;

        self.require(TokenType::Semicolon, "expected ';' after extern statement")?;

        Ok(Stmt {
            pos: token.pos,
            kind: StmtKind::Extern(name.data)
        })
    }

    fn parse_expr(&mut self) -> Result<Expr, ParserError> {
        self.parse_comma_expr()
    }

    parse_expression_type!(parse_comma_expr, parse_assignment, TokenType::Comma);

    fn parse_assignment(&mut self) -> Result<Expr, ParserError> {
        let left = self.parse_ternary()?;
        
        if self.matching(TokenType::Equal) {
            let equal_token = self.previous_token.clone();
            
            let right = self.parse_assignment()?;

            return Ok(Expr {
                pos: equal_token.pos,
                kind: ExprKind::BinOp(Box::new(left), equal_token.kind, Box::new(right))
            })
        }

        Ok(left)
    }

    fn parse_ternary(&mut self) -> Result<Expr, ParserError> {
        let condition = self.parse_logical_or()?;

        if self.matching(TokenType::QuestionMark) {
            let question_mark = self.previous_token.clone();

            let then_arm = self.parse_expr()?;

            self.require(TokenType::Colon, "expected ':' in ternary expression")?;
            
            let else_arm = self.parse_expr()?;

            return Ok(Expr {
                pos: question_mark.pos,
                kind: ExprKind::Ternary(Box::new(condition), Box::new(then_arm), Box::new(else_arm))
            })
        }

        Ok(condition)
    }

    parse_expression_type!(parse_logical_or, parse_logical_and, TokenType::BarBar);
    parse_expression_type!(parse_logical_and, parse_bitwise_or, TokenType::AmpersandAmpersand);
    parse_expression_type!(parse_bitwise_or, parse_bitwise_xor, TokenType::Bar);
    parse_expression_type!(parse_bitwise_xor, parse_bitwise_and, TokenType::UpArrow);
    parse_expression_type!(parse_bitwise_and, parse_equality, TokenType::Ampersand);
    parse_expression_type!(parse_equality, parse_comparison, TokenType::EqualEqual, TokenType::BangEqual);
    parse_expression_type!(parse_comparison, parse_shift, TokenType::Greater, TokenType::Less,
                           TokenType::GreaterEqual, TokenType::LessEqual);
    parse_expression_type!(parse_shift, parse_term, TokenType::GreaterGreater, TokenType::LessLess);
    parse_expression_type!(parse_term, parse_factor, TokenType::Plus, TokenType::Minus);
    parse_expression_type!(parse_factor, parse_unary, TokenType::Star, TokenType::Slash, TokenType::Percent);

    fn parse_unary(&mut self) -> Result<Expr, ParserError> {
        if self.matching(TokenType::Minus) || self.matching(TokenType::Plus) ||
            self.matching(TokenType::Ampersand) || self.matching(TokenType::Star) ||
            self.matching(TokenType::Bang) || self.matching(TokenType::Tilda) ||
            self.matching(TokenType::PlusPlus) || self.matching(TokenType::MinusMinus) {
                let token = self.previous_token.clone();

                let expr = self.parse_unary()?;

                Ok(Expr {
                    pos: token.pos,
                    kind: ExprKind::UnaryOp(token.kind, false, Box::new(expr))
                })
            } else {
                self.parse_primary()
            }
    }
    
    fn parse_primary(&mut self) -> Result<Expr, ParserError> {
        let expr = if self.matching(TokenType::IntLiteral) {
            let token = self.previous_token.clone();

            let number_res = token.data.parse::<i32>();

            match number_res {
                Ok(num) => Ok(Expr {
                    pos: token.pos,
                    kind: ExprKind::IntLit(num)
                }),

                Err(e) => {
                    Err(self.error_at_current(&e.to_string()))
                }
            }

        } else if self.matching(TokenType::Identifier) {
            let token = self.previous_token.clone();
            
            Ok(Expr {
                pos: token.pos,
                kind: ExprKind::Var(token.data.clone())
            })
        } else if self.matching(TokenType::LeftParen) {
            let expr = self.parse_expr()?;
            self.require(TokenType::RightParen, "expected ')' after expression")?;
            Ok(expr)
        } else {
            Err(self.error_at_current("expected expression"))
        }?;

        self.continue_parse_postfix(expr)
    }

    fn continue_parse_postfix(&mut self, expr: Expr) -> Result<Expr, ParserError> {
        let mut res = expr;

        while self.matching(TokenType::PlusPlus) || self.matching(TokenType::MinusMinus) ||
            self.matching(TokenType::LeftBrace) {
                if self.previous_token.kind == TokenType::LeftBrace {
                    let token = self.previous_token.clone();

                    let index = self.parse_expr()?;
                    self.require(TokenType::RightBrace, "expected ']' after index")?;

                    res = Expr {
                        pos: token.pos.clone(),
                        kind: ExprKind::UnaryOp(TokenType::Star, false, Box::new(Expr {
                            pos: token.pos.clone(),
                            kind: ExprKind::BinOp(Box::new(res), TokenType::Plus, Box::new(Expr {
                                pos: token.pos.clone(),
                                kind: ExprKind::BinOp(Box::new(index), TokenType::Star, Box::new(Expr {
                                    pos: token.pos,
                                    kind: ExprKind::IntLit(4)
                                }))
                            }))
                        }))
                    };
                } else {
                    let token = self.previous_token.clone();
                    res = Expr {
                        pos: token.pos,
                        kind: ExprKind::UnaryOp(token.kind, true, Box::new(res))
                    };
                }
            }

        Ok(res)
    }

    fn continue_parse_fn(&mut self, name: Token) -> Result<Decl, ParserError> {
        let params = self.parse_parameters()?;

        self.require(TokenType::LeftBracket, "expected '{' before function body")?;
        let body = self.parse_block_stmt()?;

        Ok(Decl {
            pos: name.pos,
            kind: DeclKind::Function {
                name: name.data,
                params,
                body
            }
        })
    }

    fn parse_parameters(&mut self) -> Result<Vec<String>, ParserError> {
        self.require(TokenType::LeftParen, "expected '(' before function parameters")?;

        let mut res = Vec::new();
        
        if !self.matching(TokenType::RightParen) {
            res.push(self.previous_token.data.clone());

            while self.matching(TokenType::Comma) {
                res.push(self.previous_token.data.clone());
            }
        }

        self.require(TokenType::RightParen, "expected ')' after function parameters")?;

        Ok(res)
    }

    fn parse_block_stmt(&mut self) -> Result<Stmt, ParserError> {
        let left_bracket = self.previous_token.clone();
        
        let mut res = Vec::new();

        while !self.is_at_end() && !self.matching(TokenType::RightBracket) {
            match self.parse_stmt() {
                Ok(stmt) => res.push(stmt),
                Err(e) => {
                    report_error(&e.pos, &e.msg);
                    self.synchronize_stmt()
                } 
            }
        }

        if self.previous_token.kind != TokenType::RightBracket {
            // TODO: At current or at previous?
            Err(self.error_at_current("expected '}'"))
        } else {
            Ok(Stmt {
                pos: left_bracket.pos,
                kind: StmtKind::Block(res)
            })
        }
    }

    fn synchronize_stmt(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous_token.kind == TokenType::Semicolon {
                break;
            }

            match self.current_token.kind {
                TokenType::KeywordReturn => break,
                TokenType::KeywordExtern => break,
                TokenType::KeywordWhile => break,
                TokenType::KeywordDo => break,
                TokenType::KeywordIf => break,
                _ => self.advance()
            }
        }
    }

    fn parse_return_stmt(&mut self) -> Result<Stmt, ParserError> {
        let return_keyword = self.previous_token.clone();
        let expr = if self.matching(TokenType::Semicolon) {
            None
        } else {
            let res = self.parse_expr()?;
            self.require(TokenType::Semicolon, "expected ';' after return statement")?;
            Some(res)
        };

        Ok(Stmt {
            pos: return_keyword.pos,
            kind: StmtKind::Return(expr)
        })
    }
    
    fn parse_expr_stmt(&mut self) -> Result<Stmt, ParserError> {
        let expr = self.parse_expr()?;
        self.require(TokenType::Semicolon, "expected ';' after expression statement")?;

        Ok(Stmt {
            pos: expr.pos.clone(),
            kind: StmtKind::Expr(expr)
        })
    }

    fn parse_if_stmt(&mut self) -> Result<Stmt, ParserError> {
        let if_keyword = self.previous_token.clone();

        self.require(TokenType::LeftParen, "expected '(' before if condition")?;
        let condition = self.parse_expr()?;
        self.require(TokenType::RightParen, "expected ')' after if condition")?;

        let then_arm = self.parse_stmt()?;
        let else_arm = if self.matching(TokenType::KeywordElse) {
            Some(Box::new(self.parse_stmt()?))
        } else {
            None
        };

        Ok(Stmt {
            pos: if_keyword.pos,
            kind: StmtKind::If(condition, Box::new(then_arm), else_arm)
        })
    }
    
    fn parse_while_stmt(&mut self) -> Result<Stmt, ParserError> {
        let while_token = self.previous_token.clone();

        self.require(TokenType::LeftParen, "expected '(' before while loop condition")?;
        let condition = self.parse_expr()?;
        self.require(TokenType::RightParen, "expected ')' after while loop condition")?;

        let body = self.parse_stmt()?;

        Ok(Stmt {
            pos: while_token.pos,
            kind: StmtKind::While(condition, Box::new(body))
        })
    }

    fn parse_do_stmt(&mut self) -> Result<Stmt, ParserError> {
        let do_token = self.previous_token.clone();

        let body = self.parse_stmt()?;

        self.require(TokenType::LeftParen, "expected '(' before do loop condition")?;
        let condition = self.parse_expr()?;
        self.require(TokenType::RightParen, "expected ')' after do loop condition")?;
        self.require(TokenType::Semicolon, "expected ';' after do loop condition")?;

        Ok(Stmt {
            pos: do_token.pos,
            kind: StmtKind::DoWhile(condition, Box::new(body))
        })
    }
    
    fn parse_break_stmt(&mut self) -> Result<Stmt, ParserError> {
        let token = self.previous_token.clone();
        self.require(TokenType::Semicolon, "expected ';' after break statement")?;

        Ok(Stmt {
            pos: token.pos,
            kind: StmtKind::Break
        })
    }
    
    fn parse_continue_stmt(&mut self) -> Result<Stmt, ParserError> {
        let token = self.previous_token.clone();
        self.require(TokenType::Semicolon, "expected ';' after continue statement")?;

        Ok(Stmt {
            pos: token.pos,
            kind: StmtKind::Continue
        })
    }
    
    fn matching(&mut self, kind: TokenType) -> bool {
        if self.check(kind) {
            self.advance();
            return true;
        }

        return false;
    }

    fn require(&mut self, kind: TokenType, error_msg: &str) -> Result<Token, ParserError> {
        if self.matching(kind) {
            Ok(self.previous_token.clone())
        } else {
            Err(self.error_at_current(error_msg))
        }
    }

    fn check(&self, kind: TokenType) -> bool {
        self.current_token.kind == kind
    }

    fn advance(&mut self) {
        self.previous_token = self.current_token.clone();
        // TODO: Scanner error.
        self.current_token = self.scanner.next_token();
    }

    fn error_at_current(&mut self, msg: &str) -> ParserError {
        self.had_error = true;
        ParserError { pos: self.current_token.pos.clone(), msg: String::from(msg) }
    }
}
