/// Complete lexer for Axiom language — Final Maturation
/// Supports all keywords (cls, ext, enm, self, out, loc, new, match, go)
/// and @ interpolation inside strings.
use crate::errors::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    If,
    Else,
    While,
    For,
    In,
    Fun,
    Return,
    Let,
    Go,
    Async,
    Await,
    Loc,
    Lib,
    Cls,
    Ext,
    Enm,
    SelfKw,
    Out,
    New,
    Match,
    Els,  // Genesis syntax: wildcard in match

    // Literals
    Number(f64),
    String(String),
    /// An interpolated string is stored as a vec of (is_expr, text) pairs.
    /// is_expr=false → literal segment, is_expr=true → expression source text.
    InterpolatedString(Vec<(bool, String)>),
    True,
    False,
    Ident(String),

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
    Not,
    Assign,

    // Delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Semicolon,
    Colon,
    Dot,
    Arrow,

    // End of input
    Eof,
}

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    source_id: u32,
}

impl Lexer {
    pub fn new(input: &str, source_id: u32) -> Self {
        Lexer {
            input: input.chars().collect(),
            pos: 0,
            source_id,
        }
    }

    fn current(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn peek(&self, offset: usize) -> Option<char> {
        self.input.get(self.pos + offset).copied()
    }

    fn advance(&mut self) -> Option<char> {
        if self.pos < self.input.len() {
            let ch = self.input[self.pos];
            self.pos += 1;
            Some(ch)
        } else {
            None
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.current() {
                Some(ch) if ch.is_whitespace() => {
                    self.advance();
                }
                Some('/') if self.peek(1) == Some('/') => {
                    self.advance();
                    self.advance();
                    while let Some(ch) = self.current() {
                        if ch == '\n' {
                            break;
                        }
                        self.advance();
                    }
                }
                Some('/') if self.peek(1) == Some('*') => {
                    self.advance();
                    self.advance();
                    while let Some(ch) = self.current() {
                        if ch == '*' && self.peek(1) == Some('/') {
                            self.advance();
                            self.advance();
                            break;
                        }
                        self.advance();
                    }
                }
                _ => break,
            }
        }
    }

    fn read_number(&mut self) -> f64 {
        let start = self.pos;

        while let Some(ch) = self.current() {
            if ch.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }

        if self.current() == Some('.') && self.peek(1).map_or(false, |c| c.is_ascii_digit()) {
            self.advance();
            while let Some(ch) = self.current() {
                if ch.is_ascii_digit() {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        let num_str: String = self.input[start..self.pos].iter().collect();
        num_str.parse::<f64>().unwrap_or(0.0)
    }

    /// Read a string literal. If it contains `@var` or `@(expr)`, emit
    /// `Token::InterpolatedString` instead of `Token::String`.
    fn read_string(&mut self, quote: char) -> Token {
        self.advance(); // consume opening quote
        let mut segments: Vec<(bool, String)> = Vec::new();
        let mut current_literal = String::new();
        let mut has_interpolation = false;

        while let Some(ch) = self.current() {
            if ch == quote {
                self.advance();
                break;
            } else if ch == '\\' {
                self.advance();
                match self.current() {
                    Some('n') => current_literal.push('\n'),
                    Some('t') => current_literal.push('\t'),
                    Some('r') => current_literal.push('\r'),
                    Some('\\') => current_literal.push('\\'),
                    Some('"') => current_literal.push('"'),
                    Some('\'') => current_literal.push('\''),
                    Some('@') => current_literal.push('@'),
                    Some(c) => current_literal.push(c),
                    None => break,
                }
                self.advance();
            } else if ch == '@' {
                has_interpolation = true;
                // Flush current literal segment
                if !current_literal.is_empty() {
                    segments.push((false, std::mem::take(&mut current_literal)));
                }
                self.advance(); // consume '@'

                if self.current() == Some('(') {
                    // Expression interpolation: @(expr)
                    self.advance(); // consume '('
                    let mut depth = 1;
                    let mut expr_text = String::new();
                    while let Some(c) = self.current() {
                        if c == '(' {
                            depth += 1;
                        } else if c == ')' {
                            depth -= 1;
                            if depth == 0 {
                                self.advance(); // consume closing ')'
                                break;
                            }
                        }
                        expr_text.push(c);
                        self.advance();
                    }
                    segments.push((true, expr_text));
                } else {
                    // Simple variable interpolation: @varname
                    let mut var_name = String::new();
                    while let Some(c) = self.current() {
                        if c.is_alphanumeric() || c == '_' {
                            var_name.push(c);
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    if !var_name.is_empty() {
                        segments.push((true, var_name));
                    }
                }
            } else {
                current_literal.push(ch);
                self.advance();
            }
        }

        if has_interpolation {
            if !current_literal.is_empty() {
                segments.push((false, current_literal));
            }
            Token::InterpolatedString(segments)
        } else {
            Token::String(current_literal)
        }
    }

    fn read_identifier(&mut self) -> String {
        let start = self.pos;
        while let Some(ch) = self.current() {
            if ch.is_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }
        self.input[start..self.pos].iter().collect()
    }

    pub fn next_token(&mut self) -> (Token, Span) {
        self.skip_whitespace_and_comments();

        let start = self.pos;

        let token = match self.current() {
            None => Token::Eof,
            Some(ch) => {
                if ch.is_ascii_digit() {
                    Token::Number(self.read_number())
                } else if ch == '"' || ch == '\'' {
                    self.read_string(ch)
                } else if ch.is_alphabetic() || ch == '_' {
                    let ident = self.read_identifier();
                    match ident.as_str() {
                        "if" => Token::If,
                        "else" => Token::Else,
                        "while" => Token::While,
                        "for" => Token::For,
                        "in" => Token::In,
                        "fun" => Token::Fun,
                        "ret" => Token::Return,
                        "let" => Token::Let,
                        "go" => Token::Go,
                        "async" => Token::Async,
                        "await" => Token::Await,
                        "loc" => Token::Loc,
                        "lib" => Token::Lib,
                        "cls" => Token::Cls,
                        "ext" => Token::Ext,
                        "enm" => Token::Enm,
                        "self" => Token::SelfKw,
                        "out" => Token::Out,
                        "new" => Token::New,
                        "match" => Token::Match,
                        "els" => Token::Els,
                        "true" => Token::True,
                        "false" => Token::False,
                        _ => Token::Ident(ident),
                    }
                } else {
                    match ch {
                        '+' => {
                            self.advance();
                            Token::Plus
                        }
                        '-' => {
                            self.advance();
                            Token::Minus
                        }
                        '*' => {
                            self.advance();
                            Token::Star
                        }
                        '/' => {
                            self.advance();
                            Token::Slash
                        }
                        '%' => {
                            self.advance();
                            Token::Percent
                        }
                        '=' => {
                            self.advance();
                            match self.current() {
                                Some('=') => {
                                    self.advance();
                                    Token::Equal
                                }
                                Some('>') => {
                                    self.advance();
                                    Token::Arrow
                                }
                                _ => Token::Assign,
                            }
                        }
                        '!' => {
                            self.advance();
                            if self.current() == Some('=') {
                                self.advance();
                                Token::NotEqual
                            } else {
                                Token::Not
                            }
                        }
                        '<' => {
                            self.advance();
                            if self.current() == Some('=') {
                                self.advance();
                                Token::LessEqual
                            } else {
                                Token::Less
                            }
                        }
                        '>' => {
                            self.advance();
                            if self.current() == Some('=') {
                                self.advance();
                                Token::GreaterEqual
                            } else {
                                Token::Greater
                            }
                        }
                        '&' => {
                            self.advance();
                            if self.current() == Some('&') {
                                self.advance();
                            }
                            Token::And
                        }
                        '|' => {
                            self.advance();
                            if self.current() == Some('|') {
                                self.advance();
                            }
                            Token::Or
                        }
                        '(' => {
                            self.advance();
                            Token::LParen
                        }
                        ')' => {
                            self.advance();
                            Token::RParen
                        }
                        '[' => {
                            self.advance();
                            Token::LBracket
                        }
                        ']' => {
                            self.advance();
                            Token::RBracket
                        }
                        '{' => {
                            self.advance();
                            Token::LBrace
                        }
                        '}' => {
                            self.advance();
                            Token::RBrace
                        }
                        ',' => {
                            self.advance();
                            Token::Comma
                        }
                        ';' => {
                            self.advance();
                            Token::Semicolon
                        }
                        ':' => {
                            self.advance();
                            Token::Colon
                        }
                        '.' => {
                            self.advance();
                            Token::Dot
                        }
                        _ => {
                            self.advance();
                            Token::Eof
                        }
                    }
                }
            }
        };

        let span = Span::new(self.source_id, start, self.pos);
        (token, span)
    }

    pub fn tokenize(&mut self) -> Vec<(Token, Span)> {
        let mut tokens = Vec::new();
        loop {
            let (token, span) = self.next_token();
            if token == Token::Eof {
                break;
            }
            tokens.push((token, span));
        }
        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number() {
        let mut lexer = Lexer::new("42", 0);
        let (token, _) = lexer.next_token();
        match token {
            Token::Number(n) => assert_eq!(n, 42.0),
            _ => panic!("Expected number token"),
        }
    }

    #[test]
    fn test_string() {
        let mut lexer = Lexer::new("\"hello\"", 0);
        let (token, _) = lexer.next_token();
        match token {
            Token::String(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected string token"),
        }
    }

    #[test]
    fn test_interpolated_string() {
        let mut lexer = Lexer::new("\"hello @name, val: @(x + 1)\"", 0);
        let (token, _) = lexer.next_token();
        match token {
            Token::InterpolatedString(parts) => {
                assert_eq!(parts.len(), 4);
                assert_eq!(parts[0], (false, "hello ".to_string()));
                assert_eq!(parts[1], (true, "name".to_string()));
                assert_eq!(parts[2], (false, ", val: ".to_string()));
                assert_eq!(parts[3], (true, "x + 1".to_string()));
            }
            _ => panic!("Expected interpolated string token"),
        }
    }

    #[test]
    fn test_keywords() {
        let mut lexer = Lexer::new("cls ext enm self out new match go loc", 0);
        let tokens: Vec<Token> = lexer.tokenize().into_iter().map(|(t, _)| t).collect();
        assert_eq!(
            tokens,
            vec![
                Token::Cls,
                Token::Ext,
                Token::Enm,
                Token::SelfKw,
                Token::Out,
                Token::New,
                Token::Match,
                Token::Go,
                Token::Loc,
            ]
        );
    }

    #[test]
    fn test_identifier() {
        let mut lexer = Lexer::new("foo", 0);
        let (token, _) = lexer.next_token();
        match token {
            Token::Ident(id) => assert_eq!(id, "foo"),
            _ => panic!("Expected identifier token"),
        }
    }
}
