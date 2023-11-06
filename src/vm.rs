#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
    Add,
    Sub,
    Sine,
    Cosine,
    Push(f64),
    Mul,
    Div,
    Routine(Option<Box<Instruction>>, Vec<Instruction>),
    Noop,
}

#[derive(Debug, Default)]
pub struct VM {
    stack: Vec<f64>,
}

impl VM {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn run(&mut self, program: &[Instruction]) -> Result<(), String> {
        for instruction in program {
            match instruction {
                Instruction::Add => self.binary_op(|lhs, rhs| lhs + rhs),
                Instruction::Sub => self.binary_op(|lhs, rhs| lhs - rhs),
                Instruction::Sine => self.uanry_op(|x| x.to_radians().sin()),
                Instruction::Cosine => self.uanry_op(|x| x.to_radians().cos()),
                Instruction::Push(x) => self.push(*x),
                Instruction::Mul => self.binary_op(|lhs, rhs| lhs * rhs),
                Instruction::Div => self.binary_op(|lhs, rhs| lhs / rhs),
                Instruction::Routine(None, routine) => self.run(&routine)?,
                Instruction::Routine(Some(root), routine) => self.run(&{
                    let mut v = Vec::with_capacity(routine.len() + 1);
                    v.push(*root.clone());
                    for item in routine {
                        v.push(item.clone());
                    }
                    v
                })?,
                Instruction::Noop => {}
            }
        }

        Ok(())
    }
    pub fn result(&self) -> Option<f64> {
        match &self.stack[..] {
            &[result] => Some(result),
            _ => {
                dbg!(self);
                None
            }
        }
    }

    fn uanry_op(&mut self, op: impl FnOnce(f64) -> f64) {
        let result = op(self.stack.pop().expect("missing operand"));
        self.stack.push(result);
    }

    fn binary_op(&mut self, op: impl FnOnce(f64, f64) -> f64) {
        let rhs = self.stack.pop().expect("missing rhs");
        let lhs = self.stack.pop().expect("missing lhs");
        let result = op(lhs, rhs);
        self.stack.push(result);
    }

    fn push(&mut self, x: f64) {
        self.stack.push(x);
    }
}
