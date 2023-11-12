use std::cell::RefCell;

#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
    Add,
    Sub,
    Sine,
    Cosine,
    Push(f64),
    PushRandom,
    Mul,
    Mod,
    Div,
}

#[derive(Debug, Default)]
pub struct VM {
    stack: Vec<f64>,
    rng: Rand,
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
                Instruction::PushRandom => self.push(self.rng.rand()),
                Instruction::Mul => self.binary_op(|lhs, rhs| lhs * rhs),
                Instruction::Div => self.binary_op(|lhs, rhs| lhs / rhs),
                Instruction::Mod => self.binary_op(|lhs, rhs| lhs % rhs),
            }
        }

        Ok(())
    }
    pub fn result(&mut self) -> Option<f64> {
        match self.stack.pop() {
            Some(result) => Some(result),
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

pub struct Rand(RefCell<tiny_rng::Rng>);

impl Rand {
    fn rand(&self) -> f64 {
        use tiny_rng::Rand;
        self.0.borrow_mut().rand_f64()
    }
}

impl Default for Rand {
    fn default() -> Self {
        Self(RefCell::new(tiny_rng::Rand::from_seed(0)))
    }
}

impl std::fmt::Debug for Rand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Rand").finish()
    }
}
