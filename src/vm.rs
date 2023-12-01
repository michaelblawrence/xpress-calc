use std::{cell::RefCell, rc::Rc};

#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
    Add,
    Sub,
    Sine,
    Cosine,
    Log,
    Round,
    Floor,
    Push(f64),
    Assign(String),
    ShadowAssign(String),
    LoadLocal(String),
    CallRoutine,
    PushRoutine(Vec<Instruction>),
    SkipIfNot(Vec<Instruction>),
    IfElse(Vec<Instruction>, Vec<Instruction>),
    PushRandom,
    Mul,
    Mod,
    Div,
    Pow,
    CmpEQ,
    CmpNEQ,
    CmpLT,
    CmpLTE,
    CmpGT,
    CmpGTE,
    Enter,
    Leave,
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

#[derive(Debug, Default, Clone)]
pub struct VM {
    stack: Vec<Value>,
    scopes: ScopeStack,
    rng: Rc<Rand>,
}

impl VM {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn run(&mut self, program: &[Instruction]) -> Result<(), String> {
        for instruction in program {
            match instruction {
                Instruction::Add => self.binary_op(|lhs, rhs| lhs + rhs)?,
                Instruction::Sub => self.binary_op(|lhs, rhs| lhs - rhs)?,
                Instruction::Sine => self.unary_op(|x| x.to_radians().sin())?,
                Instruction::Cosine => self.unary_op(|x| x.to_radians().cos())?,
                Instruction::Log => self.unary_op(|x| x.log10())?,
                Instruction::Round => self.unary_op(|x| x.round())?,
                Instruction::Floor => self.unary_op(|x| x.floor())?,
                Instruction::Push(x) => self.push(*x),
                Instruction::LoadLocal(ident) => self.load_local(&ident),
                Instruction::Assign(ident) => self.assign(ident)?,
                Instruction::ShadowAssign(ident) => self.shadow_assign(ident)?,
                Instruction::CallRoutine => self.call_routine()?,
                Instruction::PushRoutine(routine) => self.push(routine.to_vec()),
                Instruction::SkipIfNot(block) => self.conditional(|x| x != 0.0, block)?,
                Instruction::IfElse(if_block, else_block) => {
                    let operand = self.stack.pop();
                    let operand = operand
                        .ok_or_else(|| String::from("missing operand"))?
                        .as_number();
                    if operand != 0.0 {
                        self.run(if_block)?;
                    } else {
                        self.run(else_block)?;
                    }
                }
                Instruction::PushRandom => self.push(self.rng.rand()),
                Instruction::Mul => self.binary_op(|lhs, rhs| lhs * rhs)?,
                Instruction::Div => self.binary_op(|lhs, rhs| lhs / rhs)?,
                Instruction::Mod => self.binary_op(|lhs, rhs| lhs % rhs)?,
                Instruction::Pow => self.binary_op(|lhs, rhs| lhs.powf(rhs))?,
                Instruction::CmpEQ => self.binary_op(|lhs, rhs| (lhs == rhs) as u8 as f64)?,
                Instruction::CmpNEQ => self.binary_op(|lhs, rhs| (lhs != rhs) as u8 as f64)?,
                Instruction::CmpLT => self.binary_op(|lhs, rhs| (lhs < rhs) as u8 as f64)?,
                Instruction::CmpLTE => self.binary_op(|lhs, rhs| (lhs <= rhs) as u8 as f64)?,
                Instruction::CmpGT => self.binary_op(|lhs, rhs| (lhs > rhs) as u8 as f64)?,
                Instruction::CmpGTE => self.binary_op(|lhs, rhs| (lhs >= rhs) as u8 as f64)?,
                Instruction::Enter => self.scopes.push(),
                Instruction::Leave => self.scopes.pop(),
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

    pub fn peek_routine(&mut self) -> Option<&[Instruction]> {
        match self.stack.last() {
            Some(Value::Routine(routine)) => Some(routine.as_slice()),
            _ => None,
        }
    }

    fn unary_op(&mut self, op: impl FnOnce(f64) -> f64) -> Result<(), String> {
        let operand = self.stack.pop();
        let operand = operand
            .ok_or_else(|| String::from("missing operand"))?
            .as_number();
        let result = op(operand);
        self.stack.push(result.into());
        Ok(())
    }

    fn binary_op(&mut self, op: impl FnOnce(f64, f64) -> f64) -> Result<(), String> {
        let rhs = self.stack.pop();
        let rhs = rhs.ok_or_else(|| String::from("missing rhs"))?.as_number();
        let lhs = self.stack.pop();
        let lhs = lhs.ok_or_else(|| String::from("missing lhs"))?.as_number();
        let result = op(lhs, rhs);
        self.stack.push(result.into());
        Ok(())
    }

    fn conditional(
        &mut self,
        op: impl FnOnce(f64) -> bool,
        block: &[Instruction],
    ) -> Result<(), String> {
        let operand = self.stack.pop();
        let operand = operand
            .ok_or_else(|| String::from("missing operand"))?
            .as_number();
        if op(operand) {
            self.run(block)?;
        }
        Ok(())
    }

    fn push(&mut self, x: impl Into<Value>) {
        self.stack.push(x.into());
    }

    fn load_local(&mut self, identifier: &str) {
        let x = self.scopes.get(identifier).map(|(_, x)| x.clone());

        let x = x.unwrap_or_else(|| {
            eprintln!("WARN: missing variable '{identifier}'");
            0.0.into()
        });

        self.stack.push(x.into());
    }

    fn assign(&mut self, identifier: &str) -> Result<(), String> {
        let value = self.stack.pop();
        let value = value.ok_or_else(|| String::from("missing assignment value"))?;
        if let Some((_, x)) = self.scopes.get_mut(identifier) {
            *x = value;
            return Ok(());
        }

        self.scopes
            .put(identifier.to_string(), value)
            .expect("failed to put local");
        Ok(())
    }

    fn shadow_assign(&mut self, identifier: &str) -> Result<(), String> {
        let value = self.stack.pop();
        let value = value.ok_or_else(|| String::from("missing assignment value"))?;

        self.scopes
            .put(identifier.to_string(), value)
            .expect("failed to put local");
        Ok(())
    }

    fn call_routine(&mut self) -> Result<(), String> {
        match self.stack.pop() {
            Some(Value::Routine(routine)) => {
                self.scopes.push();
                let result = self.run(&routine);
                self.scopes.pop();
                result
            }
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

#[derive(Debug, Default, Clone)]
struct LocalScope(Vec<(String, Value)>);

#[derive(Debug, Clone)]
struct ScopeStack(Vec<LocalScope>);

impl Default for ScopeStack {
    fn default() -> Self {
        Self(vec![Default::default()])
    }
}

impl ScopeStack {
    fn get(&self, name: &str) -> Option<&(String, Value)> {
        let (layer_idx, local_idx) = self.position(name)?;
        let locals = self.0.get(layer_idx)?;
        locals.0.get(local_idx)
    }
    fn get_mut(&mut self, name: &str) -> Option<&mut (String, Value)> {
        let (layer_idx, local_idx) = self.position(name)?;
        let locals = self.0.get_mut(layer_idx)?;
        locals.0.get_mut(local_idx)
    }
    fn position(&self, name: &str) -> Option<(usize, usize)> {
        for (layer_idx, locals) in self.0.iter().enumerate().rev() {
            if let Some(local_idx) = locals.0.iter().position(|(x, _)| &*x == name) {
                return Some((layer_idx, local_idx));
            }
        }
        None
    }
    pub fn push(&mut self) {
        self.0.push(Default::default())
    }
    pub fn pop(&mut self) {
        self.0.pop();
    }
    pub fn put(&mut self, name: String, value: Value) -> Result<bool, ()> {
        let locals = self.0.last_mut().ok_or(())?;
        if let Some((_, x)) = locals.0.iter_mut().find(|(x, _)| x == &name) {
            *x = value;
            Ok(true)
        } else {
            locals.0.push((name, value));
            Ok(false)
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
