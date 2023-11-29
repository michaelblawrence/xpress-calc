use crate::{lexer::Token, vm::Instruction};

#[derive(Default)]
pub struct Compiler<'a> {
    position: usize,
    program: &'a [Token],
}

#[derive(Debug)]
pub(crate) enum RecursiveExpression {
    Block(Vec<RecursiveExpression>),
    Literal(f64),
    Local(String),
    FuncDeclaration(Vec<String>, Box<RecursiveExpression>),
    If(Box<RecursiveExpression>, Box<RecursiveExpression>),
    IfElse(
        Box<RecursiveExpression>,
        Box<RecursiveExpression>,
        Box<RecursiveExpression>,
    ),
    AssignOp(String, Box<RecursiveExpression>),
    BinaryOp(Box<RecursiveExpression>, BinaryOp, Box<RecursiveExpression>),
    Func0(Func0Op),
    Func1(Func1Op, Box<RecursiveExpression>),
    FuncLocal(String, Vec<RecursiveExpression>),
}
#[derive(Debug, Clone, Copy)]
pub(crate) enum BinaryOp {
    Add,
    Sub,
    Div,
    Mul,
    Mod,
    Pow,
    EQ,
    NEQ,
    LT,
    LTE,
    GT,
    GTE,
}

impl BinaryOp {
    fn precedence(&self) -> usize {
        match self {
            Self::Pow | Self::Mod => 3,
            Self::Mul | Self::Div => 2,
            Self::Add | Self::Sub => 1,
            Self::EQ | Self::NEQ | Self::LT | Self::LTE | Self::GT | Self::GTE => 0,
        }
    }
}

#[derive(Debug)]
pub(crate) enum Func0Op {
    Rand,
}
#[derive(Debug)]
pub(crate) enum Func1Op {
    Sin,
    Cos,
    Sqrt,
    Log,
}

impl<'a> Compiler<'a> {
    pub fn new(program: &'a [Token]) -> Self {
        Self {
            position: Default::default(),
            program: program,
        }
    }

    fn reset(&mut self) -> usize {
        let last_pos = self.position;
        self.position = 0;
        last_pos
    }

    pub fn compile(&mut self) -> Result<Vec<Instruction>, String> {
        fn delve(node: &RecursiveExpression, stream: &mut Vec<Instruction>) {
            match node {
                RecursiveExpression::Block(statements) => {
                    stream.push(Instruction::Enter);
                    statements.iter().for_each(|node| delve(node, stream));
                    stream.push(Instruction::Leave);
                }
                RecursiveExpression::Literal(x) => stream.push(Instruction::Push(*x)),
                RecursiveExpression::Local(ident) => {
                    stream.push(Instruction::LoadLocal(ident.clone()))
                }
                RecursiveExpression::FuncDeclaration(params, body) => {
                    let mut routine = vec![];
                    delve(body, &mut routine);
                    let routine = params
                        .iter()
                        .map(|ident| Instruction::ShadowAssign(ident.clone()))
                        .chain(routine.into_iter())
                        .collect();
                    stream.push(Instruction::PushRoutine(routine));
                }
                RecursiveExpression::If(condition, block) => {
                    delve(condition, stream);
                    let mut routine = vec![];
                    delve(block, &mut routine);
                    stream.push(Instruction::SkipIfNot(routine));
                }
                RecursiveExpression::IfElse(condition, if_block, else_block) => {
                    delve(condition, stream);
                    let mut if_routine = vec![];
                    delve(if_block, &mut if_routine);
                    let mut else_routine = vec![];
                    delve(else_block, &mut else_routine);
                    stream.push(Instruction::IfElse(if_routine, else_routine));
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
                        BinaryOp::EQ => stream.push(Instruction::CmpEQ),
                        BinaryOp::NEQ => stream.push(Instruction::CmpNEQ),
                        BinaryOp::LT => stream.push(Instruction::CmpLT),
                        BinaryOp::LTE => stream.push(Instruction::CmpLTE),
                        BinaryOp::GT => stream.push(Instruction::CmpGT),
                        BinaryOp::GTE => stream.push(Instruction::CmpGTE),
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
                RecursiveExpression::FuncLocal(ident, args) => {
                    args.iter().rev().for_each(|node| delve(node, stream));
                    stream.push(Instruction::LoadLocal(ident.clone()));
                    stream.push(Instruction::CallRoutine);
                }
            }
        }

        let mut instruction_stream = vec![];

        let program_expression = self.compile_expression_tree()?;
        delve(&program_expression, &mut instruction_stream);

        Ok(instruction_stream)
    }

    pub(crate) fn compile_expression_tree(&mut self) -> Result<RecursiveExpression, String> {
        let program_expression = self.parse_expression();
        let last_pos = self.reset();
        if last_pos != self.program.len() {
            let err_msg = if program_expression.is_some() {
                format!(
                    "failed to compile remaining tokens: {:?}",
                    &self.program[last_pos..]
                )
            } else {
                format!(
                    "failed to parse expression, unexpected token sequence: {:?}",
                    &self.program[last_pos..]
                )
            };
            return Err(err_msg);
        }
        let program_expression =
            program_expression.ok_or_else(|| format!("invalid program expression."))?;
        Ok(program_expression)
    }

    fn parse_expression(&mut self) -> Option<RecursiveExpression> {
        let expression = self.parse_primary_expression();

        match (expression, self.peek_binary_op()) {
            (Some(lhs), Some(_)) => self.parse_binary_op(lhs, 0),
            (expression, _) => expression,
        }
    }

    fn parse_primary_expression(&mut self) -> Option<RecursiveExpression> {
        match self.peek() {
            Some(Token::OpenCurly) => self.parse_block(),
            Some(Token::OpenParen) => self.parse_parens_expression(),
            Some(Token::Let) => self.parse_assignment_expression(),
            Some(Token::If) => self.parse_if_expression(),
            Some(Token::Pi | Token::E) => self.parse_const_expression(),
            Some(Token::LiteralNum(_)) => self.parse_literal_expression(),
            Some(Token::Identifier(_)) => self.parse_var_expression(),
            _ => {
                if let Some(_) = self.peek_func_0_op() {
                    self.parse_func_0()
                } else if let Some(_) = self.peek_func_1_op() {
                    self.parse_func_1()
                } else if let Some(_) = self.peek_const_literal() {
                    self.parse_const_expression()
                } else {
                    None
                }
            }
        }
    }

    fn parse_const_expression(&mut self) -> Option<RecursiveExpression> {
        let x = self.peek_const_literal()?;
        self.consume();
        Some(RecursiveExpression::Literal(x))
    }

    fn parse_literal_expression(&mut self) -> Option<RecursiveExpression> {
        match *self.peek()? {
            Token::LiteralNum(x) => {
                self.consume();
                Some(RecursiveExpression::Literal(x))
            }
            _ => None,
        }
    }

    fn parse_var_expression(&mut self) -> Option<RecursiveExpression> {
        match self.peek()? {
            Token::Identifier(ident) => {
                let ident = ident.clone();
                self.consume()?;
                match self.peek() {
                    Some(Token::OpenParen) => {
                        self.consume()?;
                        let args = self.parse_func_argument_list()?;
                        self.try_consume(&Token::CloseParen)?;
                        Some(RecursiveExpression::FuncLocal(ident, args))
                    }
                    _ => Some(RecursiveExpression::Local(ident)),
                }
            }
            _ => None,
        }
    }

    fn parse_parens_expression(&mut self) -> Option<RecursiveExpression> {
        if let Some(fn_expression) = self.try_or_revert(Self::parse_func_expression) {
            return Some(fn_expression);
        }

        self.try_consume(&Token::OpenParen)?;
        let expression = self.parse_expression()?;
        self.try_consume(&Token::CloseParen)?;
        Some(expression)
    }

    fn parse_block(&mut self) -> Option<RecursiveExpression> {
        self.try_consume(&Token::OpenCurly)?;
        if let Some(_) = self.try_consume(&Token::CloseCurly) {
            return Some(RecursiveExpression::Block(vec![]));
        }
        let expression = self.parse_expression()?;
        let mut statements = vec![expression];
        if let Some(_) = self.try_consume(&Token::Semicolon) {
            while let Some(expression) = self.parse_expression() {
                statements.push(expression);
                if let None = self.try_consume(&Token::Semicolon) {
                    break;
                }
            }
        }
        self.try_consume(&Token::CloseCurly)?;
        Some(RecursiveExpression::Block(statements))
    }

    fn parse_func_expression(&mut self) -> Option<RecursiveExpression> {
        self.try_consume(&Token::OpenParen)?;
        let parameters = self.parse_func_params()?;
        self.try_consume(&Token::CloseParen)?;
        self.try_consume(&Token::LeftArrow)?;
        let body = self.parse_expression()?;
        Some(RecursiveExpression::FuncDeclaration(
            parameters,
            Box::new(body),
        ))
    }

    fn parse_assignment_expression(&mut self) -> Option<RecursiveExpression> {
        self.try_consume(&Token::Let)?;
        let identifier = match self.peek()? {
            Token::Identifier(ident) => Some(ident.clone()),
            _ => None,
        }?;
        self.consume()?;
        self.try_consume(&Token::Equals)?;
        let expression = self.parse_expression()?;
        Some(RecursiveExpression::AssignOp(
            identifier,
            Box::new(expression),
        ))
    }

    fn parse_if_expression(&mut self) -> Option<RecursiveExpression> {
        self.try_consume(&Token::If)?;
        self.try_consume(&Token::OpenParen)?;
        let expression = self.parse_expression()?;
        self.try_consume(&Token::CloseParen)?;
        let block = self.parse_block()?;
        match self.try_consume(&Token::Else) {
            Some(_) => {
                let else_block = self.parse_block()?;
                Some(RecursiveExpression::IfElse(
                    Box::new(expression),
                    Box::new(block),
                    Box::new(else_block),
                ))
            }
            None => Some(RecursiveExpression::If(
                Box::new(expression),
                Box::new(block),
            )),
        }
    }

    fn parse_func_0(&mut self) -> Option<RecursiveExpression> {
        let func_op = self.peek_func_0_op()?;
        self.consume()?;
        self.try_consume(&Token::OpenParen)?;
        self.try_consume(&Token::CloseParen)?;
        Some(RecursiveExpression::Func0(func_op))
    }

    fn parse_func_1(&mut self) -> Option<RecursiveExpression> {
        let func_op = self.peek_func_1_op()?;
        self.consume()?;
        self.try_consume(&Token::OpenParen)?;
        let expression = self.parse_expression()?;
        self.try_consume(&Token::CloseParen)?;
        Some(RecursiveExpression::Func1(func_op, Box::new(expression)))
    }

    fn parse_binary_op(
        &mut self,
        mut lhs: RecursiveExpression,
        min_precedence: usize,
    ) -> Option<RecursiveExpression> {
        while let Some(op) = self
            .peek_binary_op()
            .filter(|op| op.precedence() >= min_precedence)
        {
            match self.peek() {
                Some(Token::OpenParen) => {
                    // handles implicit multiplication by parentheses (example: '(x+1)(x-2)')
                }
                Some(Token::Identifier(_)) if matches!(lhs, RecursiveExpression::Literal(_)) => {
                    // handles implicit multiplication by literal (example: '3x')
                }
                Some(Token::Identifier(_)) => return None, // otherwise, token was not expected
                _ => {
                    self.consume()?;
                }
            }

            let mut rhs = self.parse_primary_expression()?;

            while let Some(_) = self
                .peek_binary_op()
                .filter(|next_op| next_op.precedence() > op.precedence())
            {
                rhs = self.parse_binary_op(rhs, op.precedence() + 1)?;
            }

            lhs = RecursiveExpression::BinaryOp(Box::new(lhs), op, Box::new(rhs))
        }

        Some(lhs)
    }

    fn parse_func_params(&mut self) -> Option<Vec<String>> {
        let mut idents = vec![];
        while let Some(Token::Identifier(ident)) = self.peek() {
            idents.push(ident.clone());
            self.consume()?;
            if let None = self.try_consume(&Token::Comma) {
                break;
            }
        }
        Some(idents)
    }

    fn parse_func_argument_list(&mut self) -> Option<Vec<RecursiveExpression>> {
        let mut idents = vec![];
        while let Some(expression) = self.parse_expression() {
            idents.push(expression);
            if let None = self.try_consume(&Token::Comma) {
                break;
            }
        }
        Some(idents)
    }

    fn peek(&self) -> Option<&Token> {
        self.program.get(self.position)
    }

    fn peek_binary_op(&mut self) -> Option<BinaryOp> {
        match self.peek()? {
            Token::Plus => Some(BinaryOp::Add),
            Token::Sub => Some(BinaryOp::Sub),
            Token::Mul => Some(BinaryOp::Mul),
            Token::Div => Some(BinaryOp::Div),
            Token::Mod => Some(BinaryOp::Mod),
            Token::Pow => Some(BinaryOp::Pow),
            Token::Eq => Some(BinaryOp::EQ),
            Token::NotEq => Some(BinaryOp::NEQ),
            Token::LessThan => Some(BinaryOp::LT),
            Token::LessThanEquals => Some(BinaryOp::LTE),
            Token::GreaterThan => Some(BinaryOp::GT),
            Token::GreaterThanEquals => Some(BinaryOp::GTE),

            Token::OpenParen => Some(BinaryOp::Mul),
            Token::Identifier(_) => Some(BinaryOp::Mul),
            _ => None,
        }
    }

    fn peek_func_0_op(&mut self) -> Option<Func0Op> {
        match self.peek()? {
            Token::Rand => Some(Func0Op::Rand),
            _ => None,
        }
    }

    fn peek_func_1_op(&mut self) -> Option<Func1Op> {
        match self.peek()? {
            Token::Sine => Some(Func1Op::Sin),
            Token::Cosine => Some(Func1Op::Cos),
            Token::Sqrt => Some(Func1Op::Sqrt),
            Token::Log => Some(Func1Op::Log),
            _ => None,
        }
    }

    fn peek_const_literal(&mut self) -> Option<f64> {
        match self.peek()? {
            Token::Pi => Some(std::f64::consts::PI),
            Token::E => Some(std::f64::consts::E),
            _ => None,
        }
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

    fn try_or_revert(
        &mut self,
        mut parse_fn: impl FnMut(&mut Self) -> Option<RecursiveExpression>,
    ) -> Option<RecursiveExpression> {
        let initial_position = self.position;
        let parse_result = parse_fn(self);
        if parse_result.is_none() {
            self.position = initial_position;
        }
        parse_result
    }
}
