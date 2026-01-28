#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
    Undefined,
    Identifier(String),

    // Keywords
    Var,
    Let,
    Const,
    Function,
    Return,
    If,
    Else,
    While,
    For,
    Break,
    Continue,
    True,
    False,
    This,
    New,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equal,
    EqualEqual,
    EqualEqualEqual,
    BangEqual,
    BangEqualEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
    Bang,
    PlusEqual,
    MinusEqual,
    PlusPlus,
    MinusMinus,

    // Punctuation
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Semicolon,
    Comma,
    Dot,
    Colon,
    Question,

    // Special
    Eof,
}

pub struct Lexer {
    source: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            self.skip_whitespace();
            if self.is_at_end() {
                break;
            }

            if let Some(token) = self.scan_token() {
                tokens.push(token);
            }
        }

        tokens.push(Token::Eof);
        tokens
    }

    fn scan_token(&mut self) -> Option<Token> {
        let c = self.advance();

        match c {
            '(' => Some(Token::LeftParen),
            ')' => Some(Token::RightParen),
            '{' => Some(Token::LeftBrace),
            '}' => Some(Token::RightBrace),
            '[' => Some(Token::LeftBracket),
            ']' => Some(Token::RightBracket),
            ';' => Some(Token::Semicolon),
            ',' => Some(Token::Comma),
            '.' => Some(Token::Dot),
            ':' => Some(Token::Colon),
            '?' => Some(Token::Question),

            '+' => {
                if self.match_char('+') {
                    Some(Token::PlusPlus)
                } else if self.match_char('=') {
                    Some(Token::PlusEqual)
                } else {
                    Some(Token::Plus)
                }
            }
            '-' => {
                if self.match_char('-') {
                    Some(Token::MinusMinus)
                } else if self.match_char('=') {
                    Some(Token::MinusEqual)
                } else {
                    Some(Token::Minus)
                }
            }
            '*' => Some(Token::Star),
            '/' => {
                if self.match_char('/') {
                    while !self.is_at_end() && self.peek() != '\n' {
                        self.advance();
                    }
                    None
                } else if self.match_char('*') {
                    while !self.is_at_end() {
                        if self.peek() == '*' && self.peek_next() == Some('/') {
                            self.advance();
                            self.advance();
                            break;
                        }
                        self.advance();
                    }
                    None
                } else {
                    Some(Token::Slash)
                }
            }
            '%' => Some(Token::Percent),

            '=' => {
                if self.match_char('=') {
                    if self.match_char('=') {
                        Some(Token::EqualEqualEqual)
                    } else {
                        Some(Token::EqualEqual)
                    }
                } else {
                    Some(Token::Equal)
                }
            }
            '!' => {
                if self.match_char('=') {
                    if self.match_char('=') {
                        Some(Token::BangEqualEqual)
                    } else {
                        Some(Token::BangEqual)
                    }
                } else {
                    Some(Token::Bang)
                }
            }
            '<' => {
                if self.match_char('=') {
                    Some(Token::LessEqual)
                } else {
                    Some(Token::Less)
                }
            }
            '>' => {
                if self.match_char('=') {
                    Some(Token::GreaterEqual)
                } else {
                    Some(Token::Greater)
                }
            }
            '&' => {
                if self.match_char('&') {
                    Some(Token::And)
                } else {
                    None
                }
            }
            '|' => {
                if self.match_char('|') {
                    Some(Token::Or)
                } else {
                    None
                }
            }

            '"' | '\'' => Some(self.string(c)),

            _ if c.is_ascii_digit() => Some(self.number(c)),
            _ if c.is_alphabetic() || c == '_' || c == '$' => Some(self.identifier(c)),

            _ => None,
        }
    }

    fn string(&mut self, quote: char) -> Token {
        let mut value = String::new();

        while !self.is_at_end() && self.peek() != quote {
            if self.peek() == '\\' {
                self.advance();
                if !self.is_at_end() {
                    let escaped = self.advance();
                    match escaped {
                        'n' => value.push('\n'),
                        't' => value.push('\t'),
                        'r' => value.push('\r'),
                        '\\' => value.push('\\'),
                        '"' => value.push('"'),
                        '\'' => value.push('\''),
                        _ => value.push(escaped),
                    }
                }
            } else {
                value.push(self.advance());
            }
        }

        if !self.is_at_end() {
            self.advance();
        }

        Token::String(value)
    }

    fn number(&mut self, first: char) -> Token {
        let mut value = String::from(first);

        while !self.is_at_end() && self.peek().is_ascii_digit() {
            value.push(self.advance());
        }

        if !self.is_at_end() && self.peek() == '.' {
            if let Some(next) = self.peek_next() {
                if next.is_ascii_digit() {
                    value.push(self.advance());
                    while !self.is_at_end() && self.peek().is_ascii_digit() {
                        value.push(self.advance());
                    }
                }
            }
        }

        Token::Number(value.parse().unwrap_or(0.0))
    }

    fn identifier(&mut self, first: char) -> Token {
        let mut value = String::from(first);

        while !self.is_at_end() && (self.peek().is_alphanumeric() || self.peek() == '_' || self.peek() == '$') {
            value.push(self.advance());
        }

        match value.as_str() {
            "var" => Token::Var,
            "let" => Token::Let,
            "const" => Token::Const,
            "function" => Token::Function,
            "return" => Token::Return,
            "if" => Token::If,
            "else" => Token::Else,
            "while" => Token::While,
            "for" => Token::For,
            "break" => Token::Break,
            "continue" => Token::Continue,
            "true" => Token::True,
            "false" => Token::False,
            "null" => Token::Null,
            "undefined" => Token::Undefined,
            "this" => Token::This,
            "new" => Token::New,
            _ => Token::Identifier(value),
        }
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.source.len()
    }

    fn peek(&self) -> char {
        self.source.get(self.position).copied().unwrap_or('\0')
    }

    fn peek_next(&self) -> Option<char> {
        self.source.get(self.position + 1).copied()
    }

    fn advance(&mut self) -> char {
        let c = self.source.get(self.position).copied().unwrap_or('\0');
        self.position += 1;
        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        c
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.peek() != expected {
            false
        } else {
            self.position += 1;
            self.column += 1;
            true
        }
    }

    fn skip_whitespace(&mut self) {
        while !self.is_at_end() && self.peek().is_whitespace() {
            self.advance();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let mut lexer = Lexer::new("var x = 42;");
        let tokens = lexer.tokenize();

        assert_eq!(tokens[0], Token::Var);
        assert_eq!(tokens[1], Token::Identifier("x".to_string()));
        assert_eq!(tokens[2], Token::Equal);
        assert_eq!(tokens[3], Token::Number(42.0));
        assert_eq!(tokens[4], Token::Semicolon);
    }

    #[test]
    fn test_string_literal() {
        let mut lexer = Lexer::new(r#""hello world""#);
        let tokens = lexer.tokenize();

        assert_eq!(tokens[0], Token::String("hello world".to_string()));
    }

    #[test]
    fn test_single_quote_string() {
        let mut lexer = Lexer::new("'hello'");
        let tokens = lexer.tokenize();

        assert_eq!(tokens[0], Token::String("hello".to_string()));
    }

    #[test]
    fn test_escape_sequences() {
        let mut lexer = Lexer::new(r#""hello\nworld""#);
        let tokens = lexer.tokenize();

        assert_eq!(tokens[0], Token::String("hello\nworld".to_string()));
    }

    #[test]
    fn test_keywords() {
        let mut lexer = Lexer::new("var let const function return if else while for break continue");
        let tokens = lexer.tokenize();

        assert_eq!(tokens[0], Token::Var);
        assert_eq!(tokens[1], Token::Let);
        assert_eq!(tokens[2], Token::Const);
        assert_eq!(tokens[3], Token::Function);
        assert_eq!(tokens[4], Token::Return);
        assert_eq!(tokens[5], Token::If);
        assert_eq!(tokens[6], Token::Else);
        assert_eq!(tokens[7], Token::While);
        assert_eq!(tokens[8], Token::For);
        assert_eq!(tokens[9], Token::Break);
        assert_eq!(tokens[10], Token::Continue);
    }

    #[test]
    fn test_boolean_literals() {
        let mut lexer = Lexer::new("true false");
        let tokens = lexer.tokenize();

        assert_eq!(tokens[0], Token::True);
        assert_eq!(tokens[1], Token::False);
    }

    #[test]
    fn test_null_undefined() {
        let mut lexer = Lexer::new("null undefined");
        let tokens = lexer.tokenize();

        assert_eq!(tokens[0], Token::Null);
        assert_eq!(tokens[1], Token::Undefined);
    }

    #[test]
    fn test_operators() {
        let mut lexer = Lexer::new("+ - * / % = == === != !== < <= > >=");
        let tokens = lexer.tokenize();

        assert_eq!(tokens[0], Token::Plus);
        assert_eq!(tokens[1], Token::Minus);
        assert_eq!(tokens[2], Token::Star);
        assert_eq!(tokens[3], Token::Slash);
        assert_eq!(tokens[4], Token::Percent);
        assert_eq!(tokens[5], Token::Equal);
        assert_eq!(tokens[6], Token::EqualEqual);
        assert_eq!(tokens[7], Token::EqualEqualEqual);
        assert_eq!(tokens[8], Token::BangEqual);
        assert_eq!(tokens[9], Token::BangEqualEqual);
        assert_eq!(tokens[10], Token::Less);
        assert_eq!(tokens[11], Token::LessEqual);
        assert_eq!(tokens[12], Token::Greater);
        assert_eq!(tokens[13], Token::GreaterEqual);
    }

    #[test]
    fn test_logical_operators() {
        let mut lexer = Lexer::new("&& || !");
        let tokens = lexer.tokenize();

        assert_eq!(tokens[0], Token::And);
        assert_eq!(tokens[1], Token::Or);
        assert_eq!(tokens[2], Token::Bang);
    }

    #[test]
    fn test_increment_decrement() {
        let mut lexer = Lexer::new("++ -- += -=");
        let tokens = lexer.tokenize();

        assert_eq!(tokens[0], Token::PlusPlus);
        assert_eq!(tokens[1], Token::MinusMinus);
        assert_eq!(tokens[2], Token::PlusEqual);
        assert_eq!(tokens[3], Token::MinusEqual);
    }

    #[test]
    fn test_punctuation() {
        let mut lexer = Lexer::new("( ) { } [ ] ; , . : ?");
        let tokens = lexer.tokenize();

        assert_eq!(tokens[0], Token::LeftParen);
        assert_eq!(tokens[1], Token::RightParen);
        assert_eq!(tokens[2], Token::LeftBrace);
        assert_eq!(tokens[3], Token::RightBrace);
        assert_eq!(tokens[4], Token::LeftBracket);
        assert_eq!(tokens[5], Token::RightBracket);
        assert_eq!(tokens[6], Token::Semicolon);
        assert_eq!(tokens[7], Token::Comma);
        assert_eq!(tokens[8], Token::Dot);
        assert_eq!(tokens[9], Token::Colon);
        assert_eq!(tokens[10], Token::Question);
    }

    #[test]
    fn test_float_number() {
        let mut lexer = Lexer::new("3.14159");
        let tokens = lexer.tokenize();

        if let Token::Number(n) = tokens[0] {
            assert!((n - 3.14159).abs() < 0.00001);
        } else {
            panic!("Expected number token");
        }
    }

    #[test]
    fn test_identifiers() {
        let mut lexer = Lexer::new("myVar _private $jquery camelCase");
        let tokens = lexer.tokenize();

        assert_eq!(tokens[0], Token::Identifier("myVar".to_string()));
        assert_eq!(tokens[1], Token::Identifier("_private".to_string()));
        assert_eq!(tokens[2], Token::Identifier("$jquery".to_string()));
        assert_eq!(tokens[3], Token::Identifier("camelCase".to_string()));
    }

    #[test]
    fn test_line_comment() {
        let mut lexer = Lexer::new("var x = 1; // this is a comment\nvar y = 2;");
        let tokens = lexer.tokenize();

        assert_eq!(tokens[0], Token::Var);
        assert_eq!(tokens[1], Token::Identifier("x".to_string()));
        assert_eq!(tokens[2], Token::Equal);
        assert_eq!(tokens[3], Token::Number(1.0));
        assert_eq!(tokens[4], Token::Semicolon);
        assert_eq!(tokens[5], Token::Var);
        assert_eq!(tokens[6], Token::Identifier("y".to_string()));
    }

    #[test]
    fn test_block_comment() {
        let mut lexer = Lexer::new("var x = /* comment */ 1;");
        let tokens = lexer.tokenize();

        assert_eq!(tokens[0], Token::Var);
        assert_eq!(tokens[1], Token::Identifier("x".to_string()));
        assert_eq!(tokens[2], Token::Equal);
        assert_eq!(tokens[3], Token::Number(1.0));
        assert_eq!(tokens[4], Token::Semicolon);
    }

    #[test]
    fn test_this_new() {
        let mut lexer = Lexer::new("this new");
        let tokens = lexer.tokenize();

        assert_eq!(tokens[0], Token::This);
        assert_eq!(tokens[1], Token::New);
    }

    #[test]
    fn test_function_declaration() {
        let mut lexer = Lexer::new("function add(a, b) { return a + b; }");
        let tokens = lexer.tokenize();

        assert_eq!(tokens[0], Token::Function);
        assert_eq!(tokens[1], Token::Identifier("add".to_string()));
        assert_eq!(tokens[2], Token::LeftParen);
        assert_eq!(tokens[3], Token::Identifier("a".to_string()));
        assert_eq!(tokens[4], Token::Comma);
        assert_eq!(tokens[5], Token::Identifier("b".to_string()));
        assert_eq!(tokens[6], Token::RightParen);
        assert_eq!(tokens[7], Token::LeftBrace);
        assert_eq!(tokens[8], Token::Return);
    }

    #[test]
    fn test_empty_input() {
        let mut lexer = Lexer::new("");
        let tokens = lexer.tokenize();

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Eof);
    }
}
