use crate::{tokenizer::Token, vm::Instruction};


#[derive(Default)]
pub struct RecursiveCompiler<'a> {
    position: usize,
    program: &'a [Token],
}

#[derive(Debug)]
enum RecursiveExpression {
    Literal(f64),
    BinaryOp(Box<RecursiveExpression>, BinaryOp, Box<RecursiveExpression>),
    Func(FuncOp, Box<RecursiveExpression>),
}
#[derive(Debug)]
enum BinaryOp {
    Add,
    Sub,
    Div,
    Mul,
    Mod,
}
#[derive(Debug)]
enum FuncOp {
    Sin,
    Cos,
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
                RecursiveExpression::BinaryOp(lhs, op, rhs) => {
                    delve(lhs, stream);
                    delve(rhs, stream);
                    match op {
                        BinaryOp::Add => stream.push(Instruction::Add),
                        BinaryOp::Sub => stream.push(Instruction::Sub),
                        BinaryOp::Div => stream.push(Instruction::Div),
                        BinaryOp::Mul => stream.push(Instruction::Mul),
                        BinaryOp::Mod => stream.push(Instruction::Mod),
                    }
                }
                RecursiveExpression::Func(op, value) => {
                    delve(value, stream);
                    match op {
                        FuncOp::Sin => stream.push(Instruction::Sine),
                        FuncOp::Cos => stream.push(Instruction::Cosine),
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
            Some(Token::Sine | Token::Cosine) => self.compile_func_like(),
            Some(Token::OpenParen) => self.compile_parens_expression(),
            Some(Token::LiteralNum(_)) => self.compile_literal_expression(),
            _ => None,
        };

        match (expression, self.peek()) {
            (Some(lhs), Some(Token::Plus | Token::Sub | Token::Mul | Token::Div | Token::Mod)) => {
                self.compile_binary_op(lhs)
            }
            (expression, _) => expression,
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

    fn compile_parens_expression(&mut self) -> Option<RecursiveExpression> {
        self.try_consume(&Token::OpenParen)?;
        let expression = self.compile_expression()?;
        self.try_consume(&Token::CloseParen)?;
        Some(expression)
    }

    fn compile_func_like(&mut self) -> Option<RecursiveExpression> {
        let func_op = match self.peek()? {
            Token::Sine => Some(FuncOp::Sin),
            Token::Cosine => Some(FuncOp::Cos),
            _ => None,
        }?;
        self.consume()?;
        self.try_consume(&Token::OpenParen)?;
        let expression = self.compile_expression()?;
        self.try_consume(&Token::CloseParen)?;
        Some(RecursiveExpression::Func(func_op, Box::new(expression)))
    }

    fn compile_binary_op(&mut self, lhs: RecursiveExpression) -> Option<RecursiveExpression> {
        let binary_op = match self.peek()? {
            Token::Plus => Some(BinaryOp::Add),
            Token::Sub => Some(BinaryOp::Sub),
            Token::Mul => Some(BinaryOp::Mul),
            Token::Div => Some(BinaryOp::Div),
            Token::Mod => Some(BinaryOp::Mod),
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
