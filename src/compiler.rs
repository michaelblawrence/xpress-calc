use crate::{tokenizer::Token, vm::Instruction};

#[derive(Debug)]
pub enum InstructionStreamItem {
    Single(Instruction),
    Many(Vec<Instruction>),
    // Done(Instruction),
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
    stack: Vec<Instruction>,
    state: State,
}

impl Compiler {
    pub fn iter<'a>(
        &'a mut self,
        tokens: impl Iterator<Item = Token> + 'a,
    ) -> impl Iterator<Item = Instruction> + 'a {
        let mut tokens = tokens.into_iter().chain(std::iter::once(Token::EOF));
        let mut current: Option<(Vec<Instruction>, usize)> = None;
        let mut done = false;

        std::iter::from_fn(move || {
            if let Some((many, idx)) = &mut current {
                if *idx >= many.len() {
                    current = None;
                } else {
                    let next = many.get(*idx).cloned();
                    *idx += 1;
                    return next;
                }
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
                    InstructionStreamItem::Many(many) => {
                        let first = many.first().cloned();
                        current = Some((many, 1));
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

        let routine = self
            .stack
            .iter_mut()
            .rfind(|x| matches!(x, Instruction::Routine(_, _)));

        match (routine, next) {
            (Some(Instruction::Routine(_, many)), InstructionStreamItem::Single(next)) => {
                many.push(next);
                InstructionStreamItem::Skip
            }
            (Some(Instruction::Routine(_, many)), InstructionStreamItem::Many(next)) => {
                for item in next {
                    many.push(item);
                }
                InstructionStreamItem::Skip
            }
            (_, next) => next,
        }
    }

    fn pop_stack(
        instr: Instruction,
        stack: &mut Vec<Instruction>,
        state: &mut State,
    ) -> InstructionStreamItem {
        if let Some(Instruction::Routine(_, many)) = stack.last_mut() {
            many.push(instr);
            return InstructionStreamItem::Skip;
        }
        match stack.pop() {
            Some(Instruction::Sine) => {
                *state =
                    State::ExpectClose(InstructionStreamItem::Many(vec![instr, Instruction::Sine]));
                InstructionStreamItem::Skip
            }
            Some(Instruction::Cosine) => {
                *state = State::ExpectClose(InstructionStreamItem::Many(vec![
                    instr,
                    Instruction::Cosine,
                ]));
                InstructionStreamItem::Skip
            }
            Some(Instruction::Add) => InstructionStreamItem::Many(vec![instr, Instruction::Add]),
            Some(Instruction::Sub) => InstructionStreamItem::Many(vec![instr, Instruction::Sub]),
            Some(Instruction::Mul) => InstructionStreamItem::Many(vec![instr, Instruction::Mul]),
            Some(Instruction::Div) => InstructionStreamItem::Many(vec![instr, Instruction::Div]),
            // Some(Instruction::Routine(mut many)) => {
            //     many.push(instr);
            //     InstructionStreamItem::Many(many)
            // }
            None => InstructionStreamItem::Single(instr),

            Some(Instruction::Routine(_, _)) => unreachable!(),
            Some(Instruction::Noop) => unreachable!(),
            Some(Instruction::Push(_)) => unreachable!(),
        }
    }
}
