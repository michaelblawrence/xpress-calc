use std::cell::RefCell;

#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
    Add,
    Sub,
    Sine,
    Cosine,
    Log,
    Push(f64),
    Assign(String),
    LoadLocal(String),
    CallRoutine,
    PushRoutine(Vec<Instruction>),
    PushRandom,
    Mul,
    Mod,
    Div,
    Pow,
}

#[derive(Debug, Clone)]
enum Value {
    Number(f64),
    Routine(Vec<Instruction>),
}

impl Value {
    fn as_number(&self) -> f64 {
        match self {
            Self::Number(v) => *v,
            Self::Routine(routine) if !routine.is_empty() => 1.0,
            Self::Routine(_) => 0.0,
        }
    }
}

impl From<Vec<Instruction>> for Value {
    fn from(v: Vec<Instruction>) -> Self {
        Self::Routine(v)
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Self::Number(v)
    }
}

#[derive(Debug, Default)]
pub struct VM {
    stack: Vec<Value>,
    locals: Vec<(String, Value)>,
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
                Instruction::Log => self.uanry_op(|x| x.log10()),
                Instruction::Push(x) => self.push(*x),
                Instruction::LoadLocal(ident) => self.load_local(&ident),
                Instruction::Assign(ident) => self.assign(ident),
                Instruction::CallRoutine => self.call_routine()?,
                Instruction::PushRoutine(routine) => self.push(routine.to_vec()),
                Instruction::PushRandom => self.push(self.rng.rand()),
                Instruction::Mul => self.binary_op(|lhs, rhs| lhs * rhs),
                Instruction::Div => self.binary_op(|lhs, rhs| lhs / rhs),
                Instruction::Mod => self.binary_op(|lhs, rhs| lhs % rhs),
                Instruction::Pow => self.binary_op(|lhs, rhs| lhs.powf(rhs)),
            }
        }

        Ok(())
    }
    pub fn pop_result(&mut self) -> Option<f64> {
        match self.stack.pop() {
            Some(result) => Some(result.as_number()),
            _ => {
                dbg!(self);
                None
            }
        }
    }

    fn uanry_op(&mut self, op: impl FnOnce(f64) -> f64) {
        let result = op(self.stack.pop().expect("missing operand").as_number());
        self.stack.push(result.into());
    }

    fn binary_op(&mut self, op: impl FnOnce(f64, f64) -> f64) {
        let rhs = self.stack.pop().expect("missing rhs").as_number();
        let lhs = self.stack.pop().expect("missing lhs").as_number();
        let result = op(lhs, rhs);
        self.stack.push(result.into());
    }

    fn push(&mut self, x: impl Into<Value>) {
        self.stack.push(x.into());
    }

    fn load_local(&mut self, identifier: &str) {
        let x = self
            .locals
            .iter()
            .find(|(ident, _)| ident == identifier)
            .map(|(_, x)| x.clone());

        let x = x.unwrap_or_else(|| {
            eprintln!("WARN: missing variable '{identifier}'");
            0.0.into()
        });

        self.stack.push(x.into());
    }

    fn assign(&mut self, identifier: &str) {
        let value = self.stack.pop().expect("missing assignment value");
        if let Some((_, x)) = self
            .locals
            .iter_mut()
            .find(|(ident, _)| ident == identifier)
        {
            *x = value;
            return;
        }

        self.locals.push((identifier.to_string(), value));
    }

    fn call_routine(&mut self) -> Result<(), String> {
        match self.stack.pop() {
            Some(Value::Routine(routine)) => self.run(&routine),
            Some(x) => {
                eprintln!("WARN: current value is not callable '{x:?}'");
                Ok(())
            }
            None => {
                eprintln!("WARN: no current value to call");
                Ok(())
            }
        }
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
