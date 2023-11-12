use crate::{tokenizer::Token, vm::Instruction};

#[derive(Default)]
pub struct RecursiveCompiler<'a> {
    position: usize,
    program: &'a [Token],
}

#[derive(Debug)]
enum RecursiveExpression {
    Literal(f64),
    Local(String),
    AssignOp(String, Box<RecursiveExpression>),
    BinaryOp(Box<RecursiveExpression>, BinaryOp, Box<RecursiveExpression>),
    Func0(Func0Op),
    Func1(Func1Op, Box<RecursiveExpression>),
}
#[derive(Debug)]
enum BinaryOp {
    Add,
    Sub,
    Div,
    Mul,
    Mod,
    Pow,
}
#[derive(Debug)]
enum Func0Op {
    Rand,
}
#[derive(Debug)]
enum Func1Op {
    Sin,
    Cos,
    Sqrt,
    Log,
}

impl<'a> RecursiveCompiler<'a> {
    pub fn new(program: &'a [Token]) -> Self {
        Self {
            position: Default::default(),
            program: program,
        }
    }

    pub fn compile(&mut self) -> Result<Vec<Instruction>, String> {
        let program_expression = self
            .compile_expression()
            .ok_or_else(|| format!("empty program expression!"))?;

        fn delve(node: &RecursiveExpression, stream: &mut Vec<Instruction>) {
            match node {
                RecursiveExpression::Literal(x) => stream.push(Instruction::Push(*x)),
                RecursiveExpression::Local(ident) => {
                    stream.push(Instruction::LoadLocal(ident.clone()))
                }
                RecursiveExpression::AssignOp(ident, value) => {
                    delve(value, stream);
                    stream.push(Instruction::Assign(ident.clone()));
                }
                RecursiveExpression::BinaryOp(lhs, op, rhs) => {
                    delve(lhs, stream);
                    delve(rhs, stream);
                    match op {
                        BinaryOp::Add => stream.push(Instruction::Add),
                        BinaryOp::Sub => stream.push(Instruction::Sub),
                        BinaryOp::Div => stream.push(Instruction::Div),
                        BinaryOp::Mul => stream.push(Instruction::Mul),
                        BinaryOp::Mod => stream.push(Instruction::Mod),
                        BinaryOp::Pow => stream.push(Instruction::Pow),
                    }
                }
                RecursiveExpression::Func0(op) => match op {
                    Func0Op::Rand => stream.push(Instruction::PushRandom),
                },
                RecursiveExpression::Func1(op, value) => {
                    delve(value, stream);
                    match op {
                        Func1Op::Sin => stream.push(Instruction::Sine),
                        Func1Op::Cos => stream.push(Instruction::Cosine),
                        Func1Op::Sqrt => {
                            stream.push(Instruction::Push(0.5));
                            stream.push(Instruction::Pow);
                        }
                        Func1Op::Log => stream.push(Instruction::Log),
                    }
                }
            }
        }

        let mut instruction_stream = vec![];

        delve(&program_expression, &mut instruction_stream);

        Ok(instruction_stream)
    }

    fn compile_expression(&mut self) -> Option<RecursiveExpression> {
        let expression = match self.peek() {
            Some(Token::Sine | Token::Cosine | Token::Log | Token::Sqrt | Token::Rand) => {
                self.compile_func_like()
            }
            Some(Token::OpenParen) => self.compile_parens_expression(),
            Some(Token::Let) => self.compile_assignment_expression(),
            Some(Token::Pi | Token::E) => self.compile_const_expression(),
            Some(Token::LiteralNum(_)) => self.compile_literal_expression(),
            Some(Token::Identifier(_)) => self.compile_var_expression(),
            _ => None,
        };

        match (expression, self.peek()) {
            (
                Some(lhs),
                Some(Token::Plus | Token::Sub | Token::Mul | Token::Div | Token::Mod | Token::Pow),
            ) => self.compile_binary_op(lhs),
            (expression, _) => expression,
        }
    }

    fn compile_const_expression(&mut self) -> Option<RecursiveExpression> {
        match *self.peek()? {
            Token::Pi => {
                self.consume();
                Some(RecursiveExpression::Literal(std::f64::consts::PI))
            }
            Token::E => {
                self.consume();
                Some(RecursiveExpression::Literal(std::f64::consts::E))
            }
            _ => None,
        }
    }

    fn compile_literal_expression(&mut self) -> Option<RecursiveExpression> {
        match *self.peek()? {
            Token::LiteralNum(x) => {
                self.consume();
                Some(RecursiveExpression::Literal(x))
            }
            _ => None,
        }
    }

    fn compile_var_expression(&mut self) -> Option<RecursiveExpression> {
        match self.peek()? {
            Token::Identifier(ident) => {
                let ident = ident.clone();
                self.consume();
                Some(RecursiveExpression::Local(ident))
            }
            _ => None,
        }
    }

    fn compile_parens_expression(&mut self) -> Option<RecursiveExpression> {
        self.try_consume(&Token::OpenParen)?;
        let expression = self.compile_expression()?;
        self.try_consume(&Token::CloseParen)?;
        Some(expression)
    }

    fn compile_assignment_expression(&mut self) -> Option<RecursiveExpression> {
        self.try_consume(&Token::Let)?;
        let identifier = match self.peek()? {
            Token::Identifier(ident) => Some(ident.clone()),
            _ => None,
        }?;
        self.consume()?;
        self.try_consume(&Token::Equals)?;
        let expression = self.compile_expression()?;
        Some(RecursiveExpression::AssignOp(
            identifier,
            Box::new(expression),
        ))
    }

    fn compile_func_like(&mut self) -> Option<RecursiveExpression> {
        if let Some(expression) = self.compile_func_0() {
            Some(expression)
        } else if let Some(expression) = self.compile_func_1() {
            Some(expression)
        } else {
            None
        }
    }

    fn compile_func_0(&mut self) -> Option<RecursiveExpression> {
        let func_op = match self.peek()? {
            Token::Rand => Some(Func0Op::Rand),
            _ => None,
        }?;
        self.consume()?;
        self.try_consume(&Token::OpenParen)?;
        self.try_consume(&Token::CloseParen)?;
        Some(RecursiveExpression::Func0(func_op))
    }
    fn compile_func_1(&mut self) -> Option<RecursiveExpression> {
        let func_op = match self.peek()? {
            Token::Sine => Some(Func1Op::Sin),
            Token::Cosine => Some(Func1Op::Cos),
            Token::Sqrt => Some(Func1Op::Sqrt),
            Token::Log => Some(Func1Op::Log),
            _ => None,
        }?;
        self.consume()?;
        self.try_consume(&Token::OpenParen)?;
        let expression = self.compile_expression()?;
        self.try_consume(&Token::CloseParen)?;
        Some(RecursiveExpression::Func1(func_op, Box::new(expression)))
    }

    fn compile_binary_op(&mut self, lhs: RecursiveExpression) -> Option<RecursiveExpression> {
        let binary_op = match self.peek()? {
            Token::Plus => Some(BinaryOp::Add),
            Token::Sub => Some(BinaryOp::Sub),
            Token::Mul => Some(BinaryOp::Mul),
            Token::Div => Some(BinaryOp::Div),
            Token::Mod => Some(BinaryOp::Mod),
            Token::Pow => Some(BinaryOp::Pow),
            _ => None,
        }?;
        self.consume()?;
        let rhs = self.compile_expression()?;
        Some(RecursiveExpression::BinaryOp(
            Box::new(lhs),
            binary_op,
            Box::new(rhs),
        ))
    }

    fn peek(&self) -> Option<&Token> {
        self.program.get(self.position)
    }

    fn consume(&mut self) -> Option<&Token> {
        let token = self.program.get(self.position);
        self.position += 1;
        token
    }

    fn try_consume(&mut self, token: &Token) -> Option<&Token> {
        let next_token = self.program.get(self.position)?;
        if token == next_token {
            self.position += 1;
            Some(next_token)
        } else {
            None
        }
    }
}
