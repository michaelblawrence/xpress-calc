use crate::{tokenizer::Token, vm::Instruction};

#[derive(Debug)]
pub enum InstructionStreamItem {
    Single(Instruction),
    Double(Instruction, Instruction),
    Many(Vec<Instruction>),
    Skip,
    Err(String),
}

#[derive(Default)]
enum State {
    #[default]
    Empty,
    ExpectOpen,
    ExpectClose(InstructionStreamItem),
}

#[derive(Default)]
pub struct Compiler {
    stack: Stack,
    state: State,
}

impl Compiler {
    pub fn iter<'a>(
        &'a mut self,
        tokens: impl Iterator<Item = Token> + 'a,
    ) -> impl Iterator<Item = Instruction> + 'a {
        enum IterState {
            Empty,
            Single(Instruction),
            Multi(Vec<Instruction>, usize),
        }

        let mut tokens = tokens.into_iter().chain(std::iter::once(Token::EOF));
        let mut current = IterState::Empty;
        let mut done = false;

        std::iter::from_fn(move || {
            match &mut current {
                IterState::Multi(many, idx) => {
                    if *idx >= many.len() {
                        current = IterState::Empty;
                    } else {
                        let next = many.get(*idx).cloned();
                        *idx += 1;
                        return next;
                    }
                }
                IterState::Single(next) => {
                    let next = next.clone();
                    current = IterState::Empty;
                    return Some(next);
                }
                IterState::Empty => (),
            }

            if done {
                return None;
            }

            while let Some(token) = tokens.next() {
                if matches!(token, Token::EOF) {
                    done = true;
                }
                match self.next_impl(token) {
                    InstructionStreamItem::Single(single) => return Some(single),
                    InstructionStreamItem::Double(first, second) => {
                        current = IterState::Single(second);
                        return Some(first);
                    }
                    InstructionStreamItem::Many(many) => {
                        let first = many.first().cloned();
                        current = IterState::Multi(many, 1);
                        return first;
                    }
                    InstructionStreamItem::Skip => continue,
                    InstructionStreamItem::Err(err) => {
                        panic!("Error processing instruction stream {err}");
                    }
                }
            }

            None
        })
        .filter(|x| !matches!(x, Instruction::Noop))
    }

    fn next_impl<'a>(&'a mut self, token: Token) -> InstructionStreamItem {
        let next = match std::mem::replace(&mut self.state, State::Empty) {
            State::ExpectOpen => match token {
                Token::OpenParen => InstructionStreamItem::Skip,
                _ => InstructionStreamItem::Err("Expected '('".into()),
            },
            State::ExpectClose(item) => match token {
                Token::CloseParen => item,
                _ => InstructionStreamItem::Err("Expected ')'".into()),
            },
            State::Empty => match token {
                Token::LiteralNum(x) => Self::pop_stack(
                    Instruction::Push(x as f64),
                    &mut self.stack,
                    &mut self.state,
                ),
                Token::Plus => {
                    self.stack.push(Instruction::Add);
                    InstructionStreamItem::Skip
                }
                Token::Sub => {
                    self.stack.push(Instruction::Sub);
                    InstructionStreamItem::Skip
                }
                Token::Mul => {
                    self.stack.push(Instruction::Mul);
                    InstructionStreamItem::Skip
                }
                Token::Div => {
                    self.stack.push(Instruction::Div);
                    InstructionStreamItem::Skip
                }
                Token::Sine => {
                    self.stack.push(Instruction::Routine(
                        Some(Box::new(Instruction::Sine)),
                        vec![],
                    ));
                    self.state = State::ExpectOpen;
                    InstructionStreamItem::Skip
                }
                Token::Cosine => {
                    self.stack.push(Instruction::Routine(
                        Some(Box::new(Instruction::Cosine)),
                        vec![],
                    ));
                    self.state = State::ExpectOpen;
                    InstructionStreamItem::Skip
                }
                Token::OpenParen => {
                    self.stack.push(Instruction::Routine(None, vec![]));
                    InstructionStreamItem::Skip
                }
                Token::CloseParen => {
                    if matches!(self.stack.last(), Some(Instruction::Routine(_, _))) {
                        Self::pop_stack(Instruction::Noop, &mut self.stack, &mut self.state);
                        match self.stack.pop() {
                            Some(Instruction::Routine(Some(root), mut many)) => {
                                many.push(*root);
                                InstructionStreamItem::Many(many)
                            }
                            Some(Instruction::Routine(None, many)) => {
                                InstructionStreamItem::Many(many)
                            }
                            _ => unreachable!(),
                        }
                    } else {
                        InstructionStreamItem::Err("Unexpected ')'".into())
                    }
                }
                Token::Pow => todo!(),
                Token::Mod => todo!(),
                Token::EOF => Self::pop_stack(Instruction::Noop, &mut self.stack, &mut self.state),
            },
        };

        match next {
            InstructionStreamItem::Single(next) => match self.stack.try_add(next) {
                Ok(()) => InstructionStreamItem::Skip,
                Err(next) => InstructionStreamItem::Single(next),
            },
            InstructionStreamItem::Double(first, second) => {
                match self.stack.try_add_many([first, second].into_iter()) {
                    Ok(()) => InstructionStreamItem::Skip,
                    Err(next) => InstructionStreamItem::Many(next),
                }
            }
            InstructionStreamItem::Many(next) => match self.stack.try_add_many(next.into_iter()) {
                Ok(()) => InstructionStreamItem::Skip,
                Err(next) => InstructionStreamItem::Many(next),
            },
            next => next,
        }
    }

    fn pop_stack(
        instr: Instruction,
        stack: &mut Stack,
        state: &mut State,
    ) -> InstructionStreamItem {
        if let Some(Instruction::Routine(_, many)) = stack.last_mut() {
            many.push(instr);
            return InstructionStreamItem::Skip;
        }
        match stack.pop() {
            Some(Instruction::Sine) => {
                *state =
                    State::ExpectClose(InstructionStreamItem::Double(instr, Instruction::Sine));
                InstructionStreamItem::Skip
            }
            Some(Instruction::Cosine) => {
                *state =
                    State::ExpectClose(InstructionStreamItem::Double(instr, Instruction::Cosine));
                InstructionStreamItem::Skip
            }
            Some(Instruction::Add) => InstructionStreamItem::Double(instr, Instruction::Add),
            Some(Instruction::Sub) => InstructionStreamItem::Double(instr, Instruction::Sub),
            Some(Instruction::Mul) => InstructionStreamItem::Double(instr, Instruction::Mul),
            Some(Instruction::Div) => InstructionStreamItem::Double(instr, Instruction::Div),
            None => InstructionStreamItem::Single(instr),

            Some(Instruction::Routine(_, _)) => unreachable!(),
            Some(Instruction::Noop) => unreachable!(),
            Some(Instruction::Push(_)) => unreachable!(),
        }
    }
}

#[derive(Default)]
struct Stack(Vec<Instruction>);

impl Stack {
    pub fn push(&mut self, value: Instruction) {
        self.0.push(value)
    }

    pub fn pop(&mut self) -> Option<Instruction> {
        self.0.pop()
    }

    pub fn last(&self) -> Option<&Instruction> {
        self.0.last()
    }

    pub fn last_mut(&mut self) -> Option<&mut Instruction> {
        self.0.last_mut()
    }

    pub fn try_add(&mut self, value: Instruction) -> Result<(), Instruction> {
        self.try_add_many(std::iter::once(value))
            .map_err(|x| x.into_iter().next().unwrap())
    }

    pub fn try_add_many(
        &mut self,
        value: impl Iterator<Item = Instruction>,
    ) -> Result<(), Vec<Instruction>> {
        fn inner_impl(
            inner: &mut Vec<Instruction>,
            value: impl Iterator<Item = Instruction>,
            level: usize,
        ) -> Result<(), Vec<Instruction>> {
            let routine = inner
                .iter_mut()
                .rfind(|x| matches!(x, Instruction::Routine(_, _)));

            match (level, routine) {
                (0, Some(Instruction::Routine(_, many))) => {
                    for item in value {
                        many.push(item);
                    }
                    Ok(())
                }
                (level, Some(Instruction::Routine(_, many))) => inner_impl(many, value, level + 1),
                (level, _) if level > 0 => {
                    for item in value {
                        inner.push(item);
                    }
                    Ok(())
                }
                _ => Err(value.collect()),
            }
        }

        let inner = &mut self.0;
        inner_impl(inner, value, 0)
    }
}
