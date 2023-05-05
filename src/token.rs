use crate::file::FilePosition;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum TokenType {
    KeywordReturn,
    KeywordIf,
    KeywordElse,
    KeywordAuto,
    KeywordWhile,
    KeywordDo,
    KeywordBreak,
    KeywordContinue,
    KeywordExtern,

    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,

    Semicolon,
    Colon,
    QuestionMark,
    Comma,
    
    Plus,
    PlusPlus,
    Minus,
    MinusMinus,
    Star,
    Slash,
    Percent,

    Bang,
    Tilda,
    
    Equal,

    EqualEqual,
    BangEqual,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,

    Bar,
    BarBar,
    Ampersand,
    AmpersandAmpersand,
    UpArrow,

    GreaterGreater,
    LessLess,

    Identifier,
    IntLiteral,
    CharLiteral,
    StringLiteral,

    EndOfFile,
    Error,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenType,
    pub pos: FilePosition,
    pub data: String
}
