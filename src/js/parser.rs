use super::lexer::Token;

#[derive(Debug, Clone)]
pub enum Expr {
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
    Undefined,
    Identifier(String),
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
    Unary(UnaryOp, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Member(Box<Expr>, String),
    Index(Box<Expr>, Box<Expr>),
    Assignment(Box<Expr>, Box<Expr>),
    Object(Vec<(String, Expr)>),
    Array(Vec<Expr>),
    Function(Option<String>, Vec<String>, Vec<Stmt>),
    This,
    New(Box<Expr>, Vec<Expr>),
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    StrictEq,
    Ne,
    StrictNe,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOp {
    Not,
    Neg,
    Typeof,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(Expr),
    Var(String, Option<Expr>),
    Let(String, Option<Expr>),
    Const(String, Expr),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    While(Expr, Box<Stmt>),
    For(Option<Box<Stmt>>, Option<Expr>, Option<Expr>, Box<Stmt>),
    Block(Vec<Stmt>),
    Return(Option<Expr>),
    Break,
    Continue,
    Function(String, Vec<String>, Vec<Stmt>),
}

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, position: 0 }
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            }
        }

        statements
    }

    fn parse_statement(&mut self) -> Option<Stmt> {
        match self.peek() {
            Token::Var => self.parse_var_declaration(),
            Token::Let => self.parse_let_declaration(),
            Token::Const => self.parse_const_declaration(),
            Token::If => self.parse_if_statement(),
            Token::While => self.parse_while_statement(),
            Token::For => self.parse_for_statement(),
            Token::Function => self.parse_function_declaration(),
            Token::Return => self.parse_return_statement(),
            Token::Break => {
                self.advance();
                self.consume_semicolon();
                Some(Stmt::Break)
            }
            Token::Continue => {
                self.advance();
                self.consume_semicolon();
                Some(Stmt::Continue)
            }
            Token::LeftBrace => self.parse_block(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_var_declaration(&mut self) -> Option<Stmt> {
        self.advance(); // consume 'var'
        let name = self.expect_identifier()?;
        let initializer = if self.match_token(&Token::Equal) {
            self.parse_expression()
        } else {
            None
        };
        self.consume_semicolon();
        Some(Stmt::Var(name, initializer))
    }

    fn parse_let_declaration(&mut self) -> Option<Stmt> {
        self.advance(); // consume 'let'
        let name = self.expect_identifier()?;
        let initializer = if self.match_token(&Token::Equal) {
            self.parse_expression()
        } else {
            None
        };
        self.consume_semicolon();
        Some(Stmt::Let(name, initializer))
    }

    fn parse_const_declaration(&mut self) -> Option<Stmt> {
        self.advance(); // consume 'const'
        let name = self.expect_identifier()?;
        self.expect_token(&Token::Equal)?;
        let initializer = self.parse_expression()?;
        self.consume_semicolon();
        Some(Stmt::Const(name, initializer))
    }

    fn parse_if_statement(&mut self) -> Option<Stmt> {
        self.advance(); // consume 'if'
        self.expect_token(&Token::LeftParen)?;
        let condition = self.parse_expression()?;
        self.expect_token(&Token::RightParen)?;
        let then_branch = Box::new(self.parse_statement()?);
        let else_branch = if self.match_token(&Token::Else) {
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };
        Some(Stmt::If(condition, then_branch, else_branch))
    }

    fn parse_while_statement(&mut self) -> Option<Stmt> {
        self.advance(); // consume 'while'
        self.expect_token(&Token::LeftParen)?;
        let condition = self.parse_expression()?;
        self.expect_token(&Token::RightParen)?;
        let body = Box::new(self.parse_statement()?);
        Some(Stmt::While(condition, body))
    }

    fn parse_for_statement(&mut self) -> Option<Stmt> {
        self.advance(); // consume 'for'
        self.expect_token(&Token::LeftParen)?;

        let init = if self.match_token(&Token::Semicolon) {
            None
        } else if self.peek() == Token::Var {
            let stmt = self.parse_var_declaration()?;
            Some(Box::new(stmt))
        } else if self.peek() == Token::Let {
            let stmt = self.parse_let_declaration()?;
            Some(Box::new(stmt))
        } else {
            let expr = self.parse_expression()?;
            self.expect_token(&Token::Semicolon)?;
            Some(Box::new(Stmt::Expr(expr)))
        };

        let condition = if self.peek() != Token::Semicolon {
            self.parse_expression()
        } else {
            None
        };
        self.expect_token(&Token::Semicolon)?;

        let update = if self.peek() != Token::RightParen {
            self.parse_expression()
        } else {
            None
        };
        self.expect_token(&Token::RightParen)?;

        let body = Box::new(self.parse_statement()?);
        Some(Stmt::For(init, condition, update, body))
    }

    fn parse_function_declaration(&mut self) -> Option<Stmt> {
        self.advance(); // consume 'function'
        let name = self.expect_identifier()?;
        self.expect_token(&Token::LeftParen)?;
        let params = self.parse_parameters();
        self.expect_token(&Token::RightParen)?;
        self.expect_token(&Token::LeftBrace)?;
        let body = self.parse_block_statements();
        self.expect_token(&Token::RightBrace)?;
        Some(Stmt::Function(name, params, body))
    }

    fn parse_return_statement(&mut self) -> Option<Stmt> {
        self.advance(); // consume 'return'
        let value = if self.peek() != Token::Semicolon && self.peek() != Token::RightBrace {
            self.parse_expression()
        } else {
            None
        };
        self.consume_semicolon();
        Some(Stmt::Return(value))
    }

    fn parse_block(&mut self) -> Option<Stmt> {
        self.advance(); // consume '{'
        let statements = self.parse_block_statements();
        self.expect_token(&Token::RightBrace)?;
        Some(Stmt::Block(statements))
    }

    fn parse_block_statements(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();
        while !self.is_at_end() && self.peek() != Token::RightBrace {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            }
        }
        statements
    }

    fn parse_expression_statement(&mut self) -> Option<Stmt> {
        let expr = self.parse_expression()?;
        self.consume_semicolon();
        Some(Stmt::Expr(expr))
    }

    fn parse_expression(&mut self) -> Option<Expr> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Option<Expr> {
        let expr = self.parse_or()?;

        if self.match_token(&Token::Equal) {
            let value = self.parse_assignment()?;
            return Some(Expr::Assignment(Box::new(expr), Box::new(value)));
        }

        Some(expr)
    }

    fn parse_or(&mut self) -> Option<Expr> {
        let mut expr = self.parse_and()?;

        while self.match_token(&Token::Or) {
            let right = self.parse_and()?;
            expr = Expr::Binary(Box::new(expr), BinaryOp::Or, Box::new(right));
        }

        Some(expr)
    }

    fn parse_and(&mut self) -> Option<Expr> {
        let mut expr = self.parse_equality()?;

        while self.match_token(&Token::And) {
            let right = self.parse_equality()?;
            expr = Expr::Binary(Box::new(expr), BinaryOp::And, Box::new(right));
        }

        Some(expr)
    }

    fn parse_equality(&mut self) -> Option<Expr> {
        let mut expr = self.parse_comparison()?;

        loop {
            let op = match self.peek() {
                Token::EqualEqual => BinaryOp::Eq,
                Token::EqualEqualEqual => BinaryOp::StrictEq,
                Token::BangEqual => BinaryOp::Ne,
                Token::BangEqualEqual => BinaryOp::StrictNe,
                _ => break,
            };
            self.advance();
            let right = self.parse_comparison()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }

        Some(expr)
    }

    fn parse_comparison(&mut self) -> Option<Expr> {
        let mut expr = self.parse_term()?;

        loop {
            let op = match self.peek() {
                Token::Less => BinaryOp::Lt,
                Token::LessEqual => BinaryOp::Le,
                Token::Greater => BinaryOp::Gt,
                Token::GreaterEqual => BinaryOp::Ge,
                _ => break,
            };
            self.advance();
            let right = self.parse_term()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }

        Some(expr)
    }

    fn parse_term(&mut self) -> Option<Expr> {
        let mut expr = self.parse_factor()?;

        loop {
            let op = match self.peek() {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_factor()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }

        Some(expr)
    }

    fn parse_factor(&mut self) -> Option<Expr> {
        let mut expr = self.parse_unary()?;

        loop {
            let op = match self.peek() {
                Token::Star => BinaryOp::Mul,
                Token::Slash => BinaryOp::Div,
                Token::Percent => BinaryOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }

        Some(expr)
    }

    fn parse_unary(&mut self) -> Option<Expr> {
        let op = match self.peek() {
            Token::Bang => UnaryOp::Not,
            Token::Minus => UnaryOp::Neg,
            _ => return self.parse_call(),
        };
        self.advance();
        let operand = self.parse_unary()?;
        Some(Expr::Unary(op, Box::new(operand)))
    }

    fn parse_call(&mut self) -> Option<Expr> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.match_token(&Token::LeftParen) {
                let args = self.parse_arguments();
                self.expect_token(&Token::RightParen)?;
                expr = Expr::Call(Box::new(expr), args);
            } else if self.match_token(&Token::Dot) {
                let name = self.expect_identifier()?;
                expr = Expr::Member(Box::new(expr), name);
            } else if self.match_token(&Token::LeftBracket) {
                let index = self.parse_expression()?;
                self.expect_token(&Token::RightBracket)?;
                expr = Expr::Index(Box::new(expr), Box::new(index));
            } else {
                break;
            }
        }

        Some(expr)
    }

    fn parse_primary(&mut self) -> Option<Expr> {
        let token = self.advance();

        match token {
            Token::Number(n) => Some(Expr::Number(n)),
            Token::String(s) => Some(Expr::String(s)),
            Token::True => Some(Expr::Boolean(true)),
            Token::False => Some(Expr::Boolean(false)),
            Token::Null => Some(Expr::Null),
            Token::Undefined => Some(Expr::Undefined),
            Token::This => Some(Expr::This),
            Token::Identifier(name) => Some(Expr::Identifier(name)),

            Token::LeftParen => {
                let expr = self.parse_expression()?;
                self.expect_token(&Token::RightParen)?;
                Some(expr)
            }

            Token::LeftBrace => self.parse_object_literal(),

            Token::LeftBracket => self.parse_array_literal(),

            Token::Function => self.parse_function_expression(),

            Token::New => {
                let callee = self.parse_call()?;
                if let Expr::Call(callee, args) = callee {
                    Some(Expr::New(callee, args))
                } else {
                    Some(Expr::New(Box::new(callee), Vec::new()))
                }
            }

            _ => None,
        }
    }

    fn parse_object_literal(&mut self) -> Option<Expr> {
        let mut properties = Vec::new();

        while self.peek() != Token::RightBrace {
            let key = match self.advance() {
                Token::Identifier(name) => name,
                Token::String(s) => s,
                _ => return None,
            };

            self.expect_token(&Token::Colon)?;
            let value = self.parse_expression()?;
            properties.push((key, value));

            if !self.match_token(&Token::Comma) {
                break;
            }
        }

        self.expect_token(&Token::RightBrace)?;
        Some(Expr::Object(properties))
    }

    fn parse_array_literal(&mut self) -> Option<Expr> {
        let mut elements = Vec::new();

        while self.peek() != Token::RightBracket {
            elements.push(self.parse_expression()?);
            if !self.match_token(&Token::Comma) {
                break;
            }
        }

        self.expect_token(&Token::RightBracket)?;
        Some(Expr::Array(elements))
    }

    fn parse_function_expression(&mut self) -> Option<Expr> {
        let name = if let Token::Identifier(_) = self.peek() {
            Some(self.expect_identifier()?)
        } else {
            None
        };

        self.expect_token(&Token::LeftParen)?;
        let params = self.parse_parameters();
        self.expect_token(&Token::RightParen)?;
        self.expect_token(&Token::LeftBrace)?;
        let body = self.parse_block_statements();
        self.expect_token(&Token::RightBrace)?;

        Some(Expr::Function(name, params, body))
    }

    fn parse_parameters(&mut self) -> Vec<String> {
        let mut params = Vec::new();

        while let Token::Identifier(_) = self.peek() {
            params.push(self.expect_identifier().unwrap());
            if !self.match_token(&Token::Comma) {
                break;
            }
        }

        params
    }

    fn parse_arguments(&mut self) -> Vec<Expr> {
        let mut args = Vec::new();

        if self.peek() != Token::RightParen {
            loop {
                if let Some(expr) = self.parse_expression() {
                    args.push(expr);
                }
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }

        args
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len() || self.peek() == Token::Eof
    }

    fn peek(&self) -> Token {
        self.tokens.get(self.position).cloned().unwrap_or(Token::Eof)
    }

    fn advance(&mut self) -> Token {
        let token = self.peek();
        if !self.is_at_end() {
            self.position += 1;
        }
        token
    }

    fn match_token(&mut self, expected: &Token) -> bool {
        if &self.peek() == expected {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect_token(&mut self, expected: &Token) -> Option<()> {
        if self.match_token(expected) {
            Some(())
        } else {
            None
        }
    }

    fn expect_identifier(&mut self) -> Option<String> {
        match self.advance() {
            Token::Identifier(name) => Some(name),
            _ => None,
        }
    }

    fn consume_semicolon(&mut self) {
        self.match_token(&Token::Semicolon);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::js::lexer::Lexer;

    fn parse(code: &str) -> Vec<Stmt> {
        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    #[test]
    fn test_parse_variable() {
        let stmts = parse("var x = 42;");

        assert_eq!(stmts.len(), 1);
        if let Stmt::Var(name, Some(Expr::Number(n))) = &stmts[0] {
            assert_eq!(name, "x");
            assert_eq!(*n, 42.0);
        } else {
            panic!("Expected var declaration");
        }
    }

    #[test]
    fn test_parse_let_declaration() {
        let stmts = parse("let y = 10;");

        assert_eq!(stmts.len(), 1);
        if let Stmt::Let(name, Some(Expr::Number(n))) = &stmts[0] {
            assert_eq!(name, "y");
            assert_eq!(*n, 10.0);
        } else {
            panic!("Expected let declaration");
        }
    }

    #[test]
    fn test_parse_const_declaration() {
        let stmts = parse("const PI = 3.14;");

        assert_eq!(stmts.len(), 1);
        if let Stmt::Const(name, Expr::Number(n)) = &stmts[0] {
            assert_eq!(name, "PI");
            assert!((n - 3.14).abs() < 0.001);
        } else {
            panic!("Expected const declaration");
        }
    }

    #[test]
    fn test_parse_uninitialized_var() {
        let stmts = parse("var x;");

        assert_eq!(stmts.len(), 1);
        if let Stmt::Var(name, None) = &stmts[0] {
            assert_eq!(name, "x");
        } else {
            panic!("Expected uninitialized var declaration");
        }
    }

    #[test]
    fn test_parse_binary_expression() {
        let stmts = parse("var x = 1 + 2 * 3;");

        if let Stmt::Var(_, Some(expr)) = &stmts[0] {
            if let Expr::Binary(left, BinaryOp::Add, right) = expr {
                assert!(matches!(left.as_ref(), Expr::Number(1.0)));
                if let Expr::Binary(l, BinaryOp::Mul, r) = right.as_ref() {
                    assert!(matches!(l.as_ref(), Expr::Number(2.0)));
                    assert!(matches!(r.as_ref(), Expr::Number(3.0)));
                } else {
                    panic!("Expected multiplication");
                }
            } else {
                panic!("Expected addition");
            }
        }
    }

    #[test]
    fn test_parse_function_declaration() {
        let stmts = parse("function add(a, b) { return a + b; }");

        assert_eq!(stmts.len(), 1);
        if let Stmt::Function(name, params, body) = &stmts[0] {
            assert_eq!(name, "add");
            assert_eq!(params.len(), 2);
            assert_eq!(params[0], "a");
            assert_eq!(params[1], "b");
            assert_eq!(body.len(), 1);
        } else {
            panic!("Expected function declaration");
        }
    }

    #[test]
    fn test_parse_if_statement() {
        let stmts = parse("if (x > 0) { return 1; }");

        assert_eq!(stmts.len(), 1);
        if let Stmt::If(cond, then_branch, else_branch) = &stmts[0] {
            assert!(matches!(cond, Expr::Binary(_, BinaryOp::Gt, _)));
            assert!(matches!(then_branch.as_ref(), Stmt::Block(_)));
            assert!(else_branch.is_none());
        } else {
            panic!("Expected if statement");
        }
    }

    #[test]
    fn test_parse_if_else_statement() {
        let stmts = parse("if (x > 0) { return 1; } else { return 0; }");

        if let Stmt::If(_, _, else_branch) = &stmts[0] {
            assert!(else_branch.is_some());
        } else {
            panic!("Expected if-else statement");
        }
    }

    #[test]
    fn test_parse_while_loop() {
        let stmts = parse("while (i < 10) { i = i + 1; }");

        assert_eq!(stmts.len(), 1);
        if let Stmt::While(cond, body) = &stmts[0] {
            assert!(matches!(cond, Expr::Binary(_, BinaryOp::Lt, _)));
            assert!(matches!(body.as_ref(), Stmt::Block(_)));
        } else {
            panic!("Expected while loop");
        }
    }

    #[test]
    fn test_parse_for_loop() {
        let stmts = parse("for (var i = 0; i < 10; i = i + 1) { x = x + i; }");

        assert_eq!(stmts.len(), 1);
        if let Stmt::For(init, cond, update, body) = &stmts[0] {
            assert!(init.is_some());
            assert!(cond.is_some());
            assert!(update.is_some());
            assert!(matches!(body.as_ref(), Stmt::Block(_)));
        } else {
            panic!("Expected for loop");
        }
    }

    #[test]
    fn test_parse_return_statement() {
        let stmts = parse("return 42;");

        if let Stmt::Return(Some(Expr::Number(n))) = &stmts[0] {
            assert_eq!(*n, 42.0);
        } else {
            panic!("Expected return statement");
        }
    }

    #[test]
    fn test_parse_break_continue() {
        let stmts = parse("break; continue;");

        assert_eq!(stmts.len(), 2);
        assert!(matches!(stmts[0], Stmt::Break));
        assert!(matches!(stmts[1], Stmt::Continue));
    }

    #[test]
    fn test_parse_function_call() {
        let stmts = parse("foo(1, 2, 3);");

        if let Stmt::Expr(Expr::Call(callee, args)) = &stmts[0] {
            assert!(matches!(callee.as_ref(), Expr::Identifier(_)));
            assert_eq!(args.len(), 3);
        } else {
            panic!("Expected function call");
        }
    }

    #[test]
    fn test_parse_member_access() {
        let stmts = parse("obj.property;");

        if let Stmt::Expr(Expr::Member(obj, prop)) = &stmts[0] {
            assert!(matches!(obj.as_ref(), Expr::Identifier(_)));
            assert_eq!(prop, "property");
        } else {
            panic!("Expected member access");
        }
    }

    #[test]
    fn test_parse_array_literal() {
        let stmts = parse("var arr = [1, 2, 3];");

        if let Stmt::Var(_, Some(Expr::Array(elements))) = &stmts[0] {
            assert_eq!(elements.len(), 3);
        } else {
            panic!("Expected array literal");
        }
    }

    #[test]
    fn test_parse_object_literal() {
        let stmts = parse("var obj = { a: 1, b: 2 };");

        if let Stmt::Var(_, Some(Expr::Object(props))) = &stmts[0] {
            assert_eq!(props.len(), 2);
            assert_eq!(props[0].0, "a");
            assert_eq!(props[1].0, "b");
        } else {
            panic!("Expected object literal");
        }
    }

    #[test]
    fn test_parse_array_index() {
        let stmts = parse("arr[0];");

        if let Stmt::Expr(Expr::Index(arr, idx)) = &stmts[0] {
            assert!(matches!(arr.as_ref(), Expr::Identifier(_)));
            assert!(matches!(idx.as_ref(), Expr::Number(_)));
        } else {
            panic!("Expected array index");
        }
    }

    #[test]
    fn test_parse_assignment() {
        let stmts = parse("x = 5;");

        if let Stmt::Expr(Expr::Assignment(target, value)) = &stmts[0] {
            assert!(matches!(target.as_ref(), Expr::Identifier(_)));
            assert!(matches!(value.as_ref(), Expr::Number(5.0)));
        } else {
            panic!("Expected assignment");
        }
    }

    #[test]
    fn test_parse_unary_negation() {
        let stmts = parse("var x = -5;");

        if let Stmt::Var(_, Some(Expr::Unary(UnaryOp::Neg, operand))) = &stmts[0] {
            assert!(matches!(operand.as_ref(), Expr::Number(5.0)));
        } else {
            panic!("Expected unary negation");
        }
    }

    #[test]
    fn test_parse_unary_not() {
        let stmts = parse("var x = !true;");

        if let Stmt::Var(_, Some(Expr::Unary(UnaryOp::Not, operand))) = &stmts[0] {
            assert!(matches!(operand.as_ref(), Expr::Boolean(true)));
        } else {
            panic!("Expected unary not");
        }
    }

    #[test]
    fn test_parse_logical_operators() {
        let stmts = parse("var x = a && b || c;");

        if let Stmt::Var(_, Some(expr)) = &stmts[0] {
            assert!(matches!(expr, Expr::Binary(_, BinaryOp::Or, _)));
        } else {
            panic!("Expected logical expression");
        }
    }

    #[test]
    fn test_parse_comparison_operators() {
        let stmts = parse("var x = a == b;");

        if let Stmt::Var(_, Some(Expr::Binary(_, op, _))) = &stmts[0] {
            assert!(matches!(op, BinaryOp::Eq));
        } else {
            panic!("Expected comparison expression");
        }
    }

    #[test]
    fn test_parse_string_literal() {
        let stmts = parse("var s = \"hello\";");

        if let Stmt::Var(_, Some(Expr::String(s))) = &stmts[0] {
            assert_eq!(s, "hello");
        } else {
            panic!("Expected string literal");
        }
    }

    #[test]
    fn test_parse_boolean_literals() {
        let stmts = parse("var t = true; var f = false;");

        assert_eq!(stmts.len(), 2);
        if let Stmt::Var(_, Some(Expr::Boolean(b))) = &stmts[0] {
            assert!(*b);
        }
        if let Stmt::Var(_, Some(Expr::Boolean(b))) = &stmts[1] {
            assert!(!*b);
        }
    }

    #[test]
    fn test_parse_null_undefined() {
        let stmts = parse("var n = null; var u = undefined;");

        assert_eq!(stmts.len(), 2);
        assert!(matches!(&stmts[0], Stmt::Var(_, Some(Expr::Null))));
        assert!(matches!(&stmts[1], Stmt::Var(_, Some(Expr::Undefined))));
    }

    #[test]
    fn test_parse_function_expression() {
        let stmts = parse("var f = function(x) { return x * 2; };");

        if let Stmt::Var(_, Some(Expr::Function(name, params, body))) = &stmts[0] {
            assert!(name.is_none());
            assert_eq!(params.len(), 1);
            assert!(!body.is_empty());
        } else {
            panic!("Expected function expression");
        }
    }

    #[test]
    fn test_parse_nested_blocks() {
        let stmts = parse("{ { var x = 1; } }");

        if let Stmt::Block(inner) = &stmts[0] {
            if let Stmt::Block(innermost) = &inner[0] {
                assert_eq!(innermost.len(), 1);
            } else {
                panic!("Expected nested block");
            }
        } else {
            panic!("Expected block");
        }
    }

    #[test]
    fn test_parse_new_expression() {
        let stmts = parse("var obj = new Foo();");

        if let Stmt::Var(_, Some(Expr::New(_, args))) = &stmts[0] {
            assert!(args.is_empty());
        } else {
            panic!("Expected new expression");
        }
    }

    #[test]
    fn test_parse_chained_member_access() {
        let stmts = parse("a.b.c;");

        if let Stmt::Expr(Expr::Member(inner, c)) = &stmts[0] {
            assert_eq!(c, "c");
            if let Expr::Member(_, b) = inner.as_ref() {
                assert_eq!(b, "b");
            } else {
                panic!("Expected nested member access");
            }
        } else {
            panic!("Expected member access");
        }
    }
}
