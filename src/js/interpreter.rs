use super::parser::{BinaryOp, Expr, Stmt, UnaryOp};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Value {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Object(Rc<RefCell<JsObject>>),
    Array(Rc<RefCell<Vec<Value>>>),
    Function(JsFunction),
    NativeFunction(String),
}

#[derive(Debug, Clone)]
pub struct JsObject {
    pub properties: HashMap<String, Value>,
    pub prototype: Option<Rc<RefCell<JsObject>>>,
}

impl JsObject {
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
            prototype: None,
        }
    }

    pub fn get(&self, key: &str) -> Value {
        if let Some(value) = self.properties.get(key) {
            value.clone()
        } else if let Some(proto) = &self.prototype {
            proto.borrow().get(key)
        } else {
            Value::Undefined
        }
    }

    pub fn set(&mut self, key: String, value: Value) {
        self.properties.insert(key, value);
    }
}

impl Default for JsObject {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct JsFunction {
    pub name: Option<String>,
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
    pub closure: Rc<RefCell<Environment>>,
}

#[derive(Debug, Clone)]
pub struct Environment {
    pub variables: HashMap<String, Value>,
    pub parent: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            parent: None,
        }
    }

    pub fn with_parent(parent: Rc<RefCell<Environment>>) -> Self {
        Self {
            variables: HashMap::new(),
            parent: Some(parent),
        }
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(value) = self.variables.get(name) {
            Some(value.clone())
        } else if let Some(parent) = &self.parent {
            parent.borrow().get(name)
        } else {
            None
        }
    }

    pub fn set(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    pub fn assign(&mut self, name: &str, value: Value) -> bool {
        if self.variables.contains_key(name) {
            self.variables.insert(name.to_string(), value);
            true
        } else if let Some(parent) = &self.parent {
            parent.borrow_mut().assign(name, value)
        } else {
            false
        }
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

pub enum ControlFlow {
    None,
    Return(Value),
    Break,
    Continue,
}

pub struct Interpreter {
    pub global: Rc<RefCell<Environment>>,
    pub current_env: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        let global = Rc::new(RefCell::new(Environment::new()));

        global.borrow_mut().set("console".to_string(), Value::NativeFunction("console".to_string()));
        global.borrow_mut().set("document".to_string(), Value::NativeFunction("document".to_string()));

        Self {
            global: global.clone(),
            current_env: global,
        }
    }

    pub fn execute(&mut self, statements: &[Stmt]) -> Value {
        let mut result = Value::Undefined;

        for stmt in statements {
            match self.execute_statement(stmt) {
                ControlFlow::Return(value) => return value,
                ControlFlow::Break | ControlFlow::Continue => break,
                ControlFlow::None => {}
            }
            if let Stmt::Expr(expr) = stmt {
                result = self.evaluate(expr);
            }
        }

        result
    }

    fn execute_statement(&mut self, stmt: &Stmt) -> ControlFlow {
        match stmt {
            Stmt::Expr(expr) => {
                self.evaluate(expr);
                ControlFlow::None
            }

            Stmt::Var(name, init) => {
                let value = init.as_ref().map(|e| self.evaluate(e)).unwrap_or(Value::Undefined);
                self.current_env.borrow_mut().set(name.clone(), value);
                ControlFlow::None
            }

            Stmt::Let(name, init) => {
                let value = init.as_ref().map(|e| self.evaluate(e)).unwrap_or(Value::Undefined);
                self.current_env.borrow_mut().set(name.clone(), value);
                ControlFlow::None
            }

            Stmt::Const(name, init) => {
                let value = self.evaluate(init);
                self.current_env.borrow_mut().set(name.clone(), value);
                ControlFlow::None
            }

            Stmt::If(condition, then_branch, else_branch) => {
                let cond_value = self.evaluate(condition);
                if self.is_truthy(&cond_value) {
                    self.execute_statement(then_branch)
                } else if let Some(else_branch) = else_branch {
                    self.execute_statement(else_branch)
                } else {
                    ControlFlow::None
                }
            }

            Stmt::While(condition, body) => {
                loop {
                    let cond_value = self.evaluate(condition);
                    if !self.is_truthy(&cond_value) {
                        break;
                    }
                    match self.execute_statement(body) {
                        ControlFlow::Break => break,
                        ControlFlow::Continue => continue,
                        ControlFlow::Return(v) => return ControlFlow::Return(v),
                        ControlFlow::None => {}
                    }
                }
                ControlFlow::None
            }

            Stmt::For(init, condition, update, body) => {
                if let Some(init) = init {
                    self.execute_statement(init);
                }

                loop {
                    if let Some(cond) = condition {
                        let cond_value = self.evaluate(cond);
                        if !self.is_truthy(&cond_value) {
                            break;
                        }
                    }

                    match self.execute_statement(body) {
                        ControlFlow::Break => break,
                        ControlFlow::Continue => {}
                        ControlFlow::Return(v) => return ControlFlow::Return(v),
                        ControlFlow::None => {}
                    }

                    if let Some(update) = update {
                        self.evaluate(update);
                    }
                }
                ControlFlow::None
            }

            Stmt::Block(statements) => {
                let new_env = Rc::new(RefCell::new(Environment::with_parent(self.current_env.clone())));
                let old_env = self.current_env.clone();
                self.current_env = new_env;

                let mut result = ControlFlow::None;
                for stmt in statements {
                    result = self.execute_statement(stmt);
                    match &result {
                        ControlFlow::Return(_) | ControlFlow::Break | ControlFlow::Continue => break,
                        ControlFlow::None => {}
                    }
                }

                self.current_env = old_env;
                result
            }

            Stmt::Return(value) => {
                let val = value.as_ref().map(|e| self.evaluate(e)).unwrap_or(Value::Undefined);
                ControlFlow::Return(val)
            }

            Stmt::Break => ControlFlow::Break,
            Stmt::Continue => ControlFlow::Continue,

            Stmt::Function(name, params, body) => {
                let func = JsFunction {
                    name: Some(name.clone()),
                    params: params.clone(),
                    body: body.clone(),
                    closure: self.current_env.clone(),
                };
                self.current_env.borrow_mut().set(name.clone(), Value::Function(func));
                ControlFlow::None
            }
        }
    }

    pub fn evaluate(&mut self, expr: &Expr) -> Value {
        match expr {
            Expr::Number(n) => Value::Number(*n),
            Expr::String(s) => Value::String(s.clone()),
            Expr::Boolean(b) => Value::Boolean(*b),
            Expr::Null => Value::Null,
            Expr::Undefined => Value::Undefined,
            Expr::This => Value::Undefined,

            Expr::Identifier(name) => {
                self.current_env.borrow().get(name).unwrap_or(Value::Undefined)
            }

            Expr::Binary(left, op, right) => {
                let left_val = self.evaluate(left);
                let right_val = self.evaluate(right);
                self.binary_op(&left_val, *op, &right_val)
            }

            Expr::Unary(op, operand) => {
                let val = self.evaluate(operand);
                self.unary_op(*op, &val)
            }

            Expr::Assignment(target, value) => {
                let val = self.evaluate(value);
                match target.as_ref() {
                    Expr::Identifier(name) => {
                        if !self.current_env.borrow_mut().assign(name, val.clone()) {
                            self.current_env.borrow_mut().set(name.clone(), val.clone());
                        }
                    }
                    Expr::Member(obj, prop) => {
                        if let Value::Object(obj) = self.evaluate(obj) {
                            obj.borrow_mut().set(prop.clone(), val.clone());
                        }
                    }
                    Expr::Index(obj, index) => {
                        let obj_val = self.evaluate(obj);
                        let idx_val = self.evaluate(index);
                        if let (Value::Array(arr), Value::Number(n)) = (obj_val, idx_val) {
                            let idx = n as usize;
                            let mut arr = arr.borrow_mut();
                            if idx < arr.len() {
                                arr[idx] = val.clone();
                            }
                        }
                    }
                    _ => {}
                }
                val
            }

            Expr::Call(callee, args) => {
                let callee_val = self.evaluate(callee);
                let arg_vals: Vec<Value> = args.iter().map(|a| self.evaluate(a)).collect();

                match callee_val {
                    Value::Function(func) => self.call_function(&func, arg_vals),
                    Value::NativeFunction(name) => self.call_native(&name, arg_vals),
                    _ => Value::Undefined,
                }
            }

            Expr::Member(obj, prop) => {
                let obj_val = self.evaluate(obj);
                match obj_val {
                    Value::NativeFunction(name) => {
                        // Handle console.log, document.getElementById, etc.
                        Value::NativeFunction(format!("{}.{}", name, prop))
                    }
                    Value::Object(obj) => obj.borrow().get(prop),
                    Value::String(s) if prop == "length" => Value::Number(s.len() as f64),
                    Value::Array(arr) if prop == "length" => Value::Number(arr.borrow().len() as f64),
                    _ => Value::Undefined,
                }
            }

            Expr::Index(obj, index) => {
                let obj_val = self.evaluate(obj);
                let idx_val = self.evaluate(index);
                match (obj_val, idx_val) {
                    (Value::Array(arr), Value::Number(n)) => {
                        let idx = n as usize;
                        arr.borrow().get(idx).cloned().unwrap_or(Value::Undefined)
                    }
                    (Value::Object(obj), Value::String(key)) => obj.borrow().get(&key),
                    _ => Value::Undefined,
                }
            }

            Expr::Object(properties) => {
                let obj = JsObject::new();
                let obj_ref = Rc::new(RefCell::new(obj));
                for (key, value) in properties {
                    let val = self.evaluate(value);
                    obj_ref.borrow_mut().set(key.clone(), val);
                }
                Value::Object(obj_ref)
            }

            Expr::Array(elements) => {
                let vals: Vec<Value> = elements.iter().map(|e| self.evaluate(e)).collect();
                Value::Array(Rc::new(RefCell::new(vals)))
            }

            Expr::Function(name, params, body) => {
                Value::Function(JsFunction {
                    name: name.clone(),
                    params: params.clone(),
                    body: body.clone(),
                    closure: self.current_env.clone(),
                })
            }

            Expr::New(callee, args) => {
                let callee_val = self.evaluate(callee);
                let arg_vals: Vec<Value> = args.iter().map(|a| self.evaluate(a)).collect();

                if let Value::Function(func) = callee_val {
                    let obj = Rc::new(RefCell::new(JsObject::new()));
                    let new_env = Rc::new(RefCell::new(Environment::with_parent(func.closure.clone())));

                    new_env.borrow_mut().set("this".to_string(), Value::Object(obj.clone()));

                    for (param, arg) in func.params.iter().zip(arg_vals) {
                        new_env.borrow_mut().set(param.clone(), arg);
                    }

                    let old_env = self.current_env.clone();
                    self.current_env = new_env;

                    for stmt in &func.body {
                        if let ControlFlow::Return(_) = self.execute_statement(stmt) {
                            break;
                        }
                    }

                    self.current_env = old_env;
                    Value::Object(obj)
                } else {
                    Value::Undefined
                }
            }
        }
    }

    fn binary_op(&self, left: &Value, op: BinaryOp, right: &Value) -> Value {
        match op {
            BinaryOp::Add => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Value::Number(a + b),
                (Value::String(a), Value::String(b)) => Value::String(format!("{}{}", a, b)),
                (Value::String(a), b) => Value::String(format!("{}{}", a, self.to_string_value(b))),
                (a, Value::String(b)) => Value::String(format!("{}{}", self.to_string_value(a), b)),
                _ => Value::Number(f64::NAN),
            },
            BinaryOp::Sub => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Value::Number(a - b),
                _ => Value::Number(f64::NAN),
            },
            BinaryOp::Mul => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Value::Number(a * b),
                _ => Value::Number(f64::NAN),
            },
            BinaryOp::Div => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Value::Number(a / b),
                _ => Value::Number(f64::NAN),
            },
            BinaryOp::Mod => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Value::Number(a % b),
                _ => Value::Number(f64::NAN),
            },
            BinaryOp::Eq | BinaryOp::StrictEq => Value::Boolean(self.values_equal(left, right)),
            BinaryOp::Ne | BinaryOp::StrictNe => Value::Boolean(!self.values_equal(left, right)),
            BinaryOp::Lt => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Value::Boolean(a < b),
                _ => Value::Boolean(false),
            },
            BinaryOp::Le => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Value::Boolean(a <= b),
                _ => Value::Boolean(false),
            },
            BinaryOp::Gt => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Value::Boolean(a > b),
                _ => Value::Boolean(false),
            },
            BinaryOp::Ge => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Value::Boolean(a >= b),
                _ => Value::Boolean(false),
            },
            BinaryOp::And => {
                if self.is_truthy(left) {
                    right.clone()
                } else {
                    left.clone()
                }
            }
            BinaryOp::Or => {
                if self.is_truthy(left) {
                    left.clone()
                } else {
                    right.clone()
                }
            }
        }
    }

    fn unary_op(&self, op: UnaryOp, operand: &Value) -> Value {
        match op {
            UnaryOp::Not => Value::Boolean(!self.is_truthy(operand)),
            UnaryOp::Neg => match operand {
                Value::Number(n) => Value::Number(-n),
                _ => Value::Number(f64::NAN),
            },
            UnaryOp::Typeof => Value::String(match operand {
                Value::Undefined => "undefined",
                Value::Null => "object",
                Value::Boolean(_) => "boolean",
                Value::Number(_) => "number",
                Value::String(_) => "string",
                Value::Object(_) => "object",
                Value::Array(_) => "object",
                Value::Function(_) | Value::NativeFunction(_) => "function",
            }.to_string()),
        }
    }

    fn is_truthy(&self, value: &Value) -> bool {
        match value {
            Value::Undefined | Value::Null => false,
            Value::Boolean(b) => *b,
            Value::Number(n) => *n != 0.0 && !n.is_nan(),
            Value::String(s) => !s.is_empty(),
            _ => true,
        }
    }

    fn values_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Undefined, Value::Undefined) => true,
            (Value::Null, Value::Null) => true,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            _ => false,
        }
    }

    fn to_string_value(&self, value: &Value) -> String {
        match value {
            Value::Undefined => "undefined".to_string(),
            Value::Null => "null".to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Number(n) => n.to_string(),
            Value::String(s) => s.clone(),
            Value::Object(_) => "[object Object]".to_string(),
            Value::Array(arr) => {
                let items: Vec<String> = arr.borrow().iter().map(|v| self.to_string_value(v)).collect();
                items.join(",")
            }
            Value::Function(_) | Value::NativeFunction(_) => "[Function]".to_string(),
        }
    }

    fn call_function(&mut self, func: &JsFunction, args: Vec<Value>) -> Value {
        let new_env = Rc::new(RefCell::new(Environment::with_parent(func.closure.clone())));

        for (param, arg) in func.params.iter().zip(args) {
            new_env.borrow_mut().set(param.clone(), arg);
        }

        let old_env = self.current_env.clone();
        self.current_env = new_env;

        let mut result = Value::Undefined;
        for stmt in &func.body {
            match self.execute_statement(stmt) {
                ControlFlow::Return(v) => {
                    result = v;
                    break;
                }
                _ => {}
            }
        }

        self.current_env = old_env;
        result
    }

    fn call_native(&mut self, name: &str, args: Vec<Value>) -> Value {
        match name {
            "console.log" => {
                let output: Vec<String> = args.iter().map(|a| self.to_string_value(a)).collect();
                println!("{}", output.join(" "));
                Value::Undefined
            }
            _ => Value::Undefined,
        }
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::js::lexer::Lexer;
    use crate::js::parser::Parser;

    fn run(code: &str) -> Value {
        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let stmts = parser.parse();
        let mut interpreter = Interpreter::new();
        interpreter.execute(&stmts)
    }

    fn get_var(interpreter: &Interpreter, name: &str) -> Value {
        interpreter.current_env.borrow().get(name).unwrap_or(Value::Undefined)
    }

    fn run_and_get_var(code: &str, var_name: &str) -> Value {
        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let stmts = parser.parse();
        let mut interpreter = Interpreter::new();
        interpreter.execute(&stmts);
        get_var(&interpreter, var_name)
    }

    #[test]
    fn test_number_literal() {
        let result = run("42");
        assert!(matches!(result, Value::Number(n) if n == 42.0));
    }

    #[test]
    fn test_string_literal() {
        let result = run("\"hello\"");
        assert!(matches!(result, Value::String(ref s) if s == "hello"));
    }

    #[test]
    fn test_boolean_literal() {
        let result = run("true");
        assert!(matches!(result, Value::Boolean(true)));
    }

    #[test]
    fn test_variable_declaration() {
        let result = run_and_get_var("var x = 10;", "x");
        assert!(matches!(result, Value::Number(n) if n == 10.0));
    }

    #[test]
    fn test_let_declaration() {
        let result = run_and_get_var("let y = 20;", "y");
        assert!(matches!(result, Value::Number(n) if n == 20.0));
    }

    #[test]
    fn test_const_declaration() {
        let result = run_and_get_var("const PI = 3.14;", "PI");
        if let Value::Number(n) = result {
            assert!((n - 3.14).abs() < 0.001);
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_addition() {
        let result = run("1 + 2");
        assert!(matches!(result, Value::Number(n) if n == 3.0));
    }

    #[test]
    fn test_subtraction() {
        let result = run("5 - 3");
        assert!(matches!(result, Value::Number(n) if n == 2.0));
    }

    #[test]
    fn test_multiplication() {
        let result = run("4 * 3");
        assert!(matches!(result, Value::Number(n) if n == 12.0));
    }

    #[test]
    fn test_division() {
        let result = run("10 / 2");
        assert!(matches!(result, Value::Number(n) if n == 5.0));
    }

    #[test]
    fn test_modulo() {
        let result = run("7 % 3");
        assert!(matches!(result, Value::Number(n) if n == 1.0));
    }

    #[test]
    fn test_string_concatenation() {
        let result = run("\"hello\" + \" \" + \"world\"");
        assert!(matches!(result, Value::String(ref s) if s == "hello world"));
    }

    #[test]
    fn test_comparison_less() {
        let result = run("3 < 5");
        assert!(matches!(result, Value::Boolean(true)));
    }

    #[test]
    fn test_comparison_greater() {
        let result = run("5 > 3");
        assert!(matches!(result, Value::Boolean(true)));
    }

    #[test]
    fn test_comparison_equal() {
        let result = run("5 == 5");
        assert!(matches!(result, Value::Boolean(true)));
    }

    #[test]
    fn test_comparison_not_equal() {
        let result = run("5 != 3");
        assert!(matches!(result, Value::Boolean(true)));
    }

    #[test]
    fn test_logical_and() {
        let result = run("true && false");
        assert!(matches!(result, Value::Boolean(false)));
    }

    #[test]
    fn test_logical_or() {
        let result = run("true || false");
        assert!(matches!(result, Value::Boolean(true)));
    }

    #[test]
    fn test_logical_not() {
        let result = run("!true");
        assert!(matches!(result, Value::Boolean(false)));
    }

    #[test]
    fn test_unary_negation() {
        let result = run("-5");
        assert!(matches!(result, Value::Number(n) if n == -5.0));
    }

    #[test]
    fn test_if_true_branch() {
        let result = run_and_get_var("var x = 0; if (true) { x = 1; }", "x");
        assert!(matches!(result, Value::Number(n) if n == 1.0));
    }

    #[test]
    fn test_if_false_branch() {
        let result = run_and_get_var("var x = 0; if (false) { x = 1; } else { x = 2; }", "x");
        assert!(matches!(result, Value::Number(n) if n == 2.0));
    }

    #[test]
    fn test_while_loop() {
        let result = run_and_get_var("var i = 0; while (i < 5) { i = i + 1; }", "i");
        assert!(matches!(result, Value::Number(n) if n == 5.0));
    }

    #[test]
    fn test_for_loop() {
        let result = run_and_get_var("var sum = 0; for (var i = 0; i < 5; i = i + 1) { sum = sum + i; }", "sum");
        assert!(matches!(result, Value::Number(n) if n == 10.0));
    }

    #[test]
    fn test_break_statement() {
        let result = run_and_get_var("var i = 0; while (true) { i = i + 1; if (i == 3) { break; } }", "i");
        assert!(matches!(result, Value::Number(n) if n == 3.0));
    }

    #[test]
    fn test_function_declaration_and_call() {
        let result = run_and_get_var("function double(x) { return x * 2; } var result = double(5);", "result");
        assert!(matches!(result, Value::Number(n) if n == 10.0));
    }

    #[test]
    fn test_function_with_multiple_params() {
        let result = run_and_get_var("function add(a, b) { return a + b; } var result = add(3, 4);", "result");
        assert!(matches!(result, Value::Number(n) if n == 7.0));
    }

    #[test]
    fn test_recursive_function() {
        let result = run_and_get_var(
            "function factorial(n) { if (n <= 1) { return 1; } return n * factorial(n - 1); } var result = factorial(5);",
            "result"
        );
        assert!(matches!(result, Value::Number(n) if n == 120.0));
    }

    #[test]
    fn test_array_creation() {
        let result = run("[1, 2, 3]");
        if let Value::Array(arr) = result {
            assert_eq!(arr.borrow().len(), 3);
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_array_index_access() {
        let result = run_and_get_var("var arr = [10, 20, 30]; var x = arr[1];", "x");
        assert!(matches!(result, Value::Number(n) if n == 20.0));
    }

    #[test]
    fn test_array_length() {
        let result = run_and_get_var("var arr = [1, 2, 3, 4, 5]; var len = arr.length;", "len");
        assert!(matches!(result, Value::Number(n) if n == 5.0));
    }

    #[test]
    fn test_object_creation() {
        let result = run_and_get_var("var obj = { a: 1, b: 2 };", "obj");
        if let Value::Object(obj) = result {
            let obj = obj.borrow();
            assert!(matches!(obj.get("a"), Value::Number(n) if n == 1.0));
            assert!(matches!(obj.get("b"), Value::Number(n) if n == 2.0));
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_object_member_access() {
        let result = run_and_get_var("var obj = { x: 42 }; var val = obj.x;", "val");
        assert!(matches!(result, Value::Number(n) if n == 42.0));
    }

    #[test]
    fn test_object_member_assignment() {
        let result = run_and_get_var("var obj = { x: 1 }; obj.x = 99; var val = obj.x;", "val");
        assert!(matches!(result, Value::Number(n) if n == 99.0));
    }

    #[test]
    fn test_string_length() {
        let result = run_and_get_var("var s = \"hello\"; var len = s.length;", "len");
        assert!(matches!(result, Value::Number(n) if n == 5.0));
    }

    #[test]
    fn test_truthy_number() {
        let result = run_and_get_var("var x = 0; if (1) { x = 1; }", "x");
        assert!(matches!(result, Value::Number(n) if n == 1.0));
    }

    #[test]
    fn test_falsy_zero() {
        let result = run_and_get_var("var x = 1; if (0) { x = 2; }", "x");
        assert!(matches!(result, Value::Number(n) if n == 1.0));
    }

    #[test]
    fn test_falsy_empty_string() {
        let result = run_and_get_var("var x = 1; if (\"\") { x = 2; }", "x");
        assert!(matches!(result, Value::Number(n) if n == 1.0));
    }

    #[test]
    fn test_truthy_non_empty_string() {
        let result = run_and_get_var("var x = 0; if (\"hello\") { x = 1; }", "x");
        assert!(matches!(result, Value::Number(n) if n == 1.0));
    }

    #[test]
    fn test_null_value() {
        let result = run("null");
        assert!(matches!(result, Value::Null));
    }

    #[test]
    fn test_undefined_value() {
        let result = run("undefined");
        assert!(matches!(result, Value::Undefined));
    }

    #[test]
    fn test_function_expression() {
        let result = run_and_get_var("var double = function(x) { return x * 2; }; var result = double(4);", "result");
        assert!(matches!(result, Value::Number(n) if n == 8.0));
    }

    #[test]
    fn test_nested_function_calls() {
        let result = run_and_get_var(
            "function add(a, b) { return a + b; } function mul(a, b) { return a * b; } var result = add(mul(2, 3), mul(4, 5));",
            "result"
        );
        assert!(matches!(result, Value::Number(n) if n == 26.0));
    }

    #[test]
    fn test_block_scope() {
        let result = run_and_get_var("var x = 1; { var y = 2; x = x + y; }", "x");
        assert!(matches!(result, Value::Number(n) if n == 3.0));
    }

    #[test]
    fn test_operator_precedence() {
        let result = run("2 + 3 * 4");
        assert!(matches!(result, Value::Number(n) if n == 14.0));
    }

    #[test]
    fn test_parentheses() {
        let result = run("(2 + 3) * 4");
        assert!(matches!(result, Value::Number(n) if n == 20.0));
    }

    #[test]
    fn test_continue_statement() {
        let result = run_and_get_var(
            "var sum = 0; for (var i = 0; i < 5; i = i + 1) { if (i == 2) { continue; } sum = sum + i; }",
            "sum"
        );
        assert!(matches!(result, Value::Number(n) if n == 8.0));
    }

    #[test]
    fn test_string_number_concatenation() {
        let result = run("\"value: \" + 42");
        assert!(matches!(result, Value::String(ref s) if s == "value: 42"));
    }

    #[test]
    fn test_comparison_less_equal() {
        let result = run("5 <= 5");
        assert!(matches!(result, Value::Boolean(true)));
    }

    #[test]
    fn test_comparison_greater_equal() {
        let result = run("5 >= 3");
        assert!(matches!(result, Value::Boolean(true)));
    }
}
