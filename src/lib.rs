use compiler::Compiler;
use vm::VM;

pub mod compiler;
pub mod lexer;
pub mod parser;
pub mod vm;

pub fn compute(vm: &mut VM, input: &str) -> Option<f64> {
    let source = parser::Bite::new(&input).chomp(parser::Chomp::whitespace());
    let tokens = lexer::tokenize(source).collect();

    let tokens: Vec<_> = match tokens {
        Ok(x) => x,
        Err(err) => {
            eprintln!("ERROR: could not interpret input tokens: {err}");
            return None;
        }
    };
    let mut compiler = Compiler::new(&tokens);
    let program = match compiler.compile() {
        Ok(x) => x,
        Err(err) => {
            eprintln!("ERROR: could not compile program: {err}");
            return None;
        }
    };

    match vm.run(&program) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("ERROR: could not compute expression: {err}");
            return None;
        }
    }

    vm.pop_result()
}

#[cfg(test)]
mod tests {
    use crate::{lexer::Token, vm::Instruction};

    use super::*;

    #[test]
    fn can_parse_sin() {
        let mut tokens = lexer::tokenize("sin(90)".into());

        assert_eq!(Some(Ok(Token::Sine)), tokens.next());
        assert_eq!(Some(Ok(Token::OpenParen)), tokens.next());
        assert_eq!(Some(Ok(Token::LiteralNum(90.0))), tokens.next());
        assert_eq!(Some(Ok(Token::CloseParen)), tokens.next());
        assert_eq!(None, tokens.next());
    }

    #[test]
    fn can_compile_sin() {
        let mut instructions = instr_iter("sin(90)").into_iter();

        assert_eq!(Some(Instruction::Push(90.0)), instructions.next());
        assert_eq!(Some(Instruction::Sine), instructions.next());
        assert_eq!(None, instructions.next());
    }

    #[test]
    fn can_compute_sin() {
        let mut vm = VM::new();
        assert_eq!(0.0, compute(&mut vm, "sin(0)").unwrap().round());
        assert_eq!(1.0, compute(&mut vm, "sin(90)").unwrap().round());
        assert_eq!(0.0, compute(&mut vm, "sin(180)").unwrap().round());
        assert_eq!(-1.0, compute(&mut vm, "sin(270)").unwrap().round());

        assert_eq!(0.0, compute(&mut vm, "sin(90 + 90)").unwrap().round());
        assert_eq!(2.0, compute(&mut vm, "sin(90) + sin(90)").unwrap().round());
    }

    #[test]
    fn can_compute_log() {
        let mut vm = VM::new();
        assert_eq!(3.0, compute(&mut vm, "log(1000)").unwrap().round());
    }

    #[test]
    fn can_compile_add() {
        let mut instructions = instr_iter("90 + 20").into_iter();

        assert_eq!(Some(Instruction::Push(90.0)), instructions.next());
        assert_eq!(Some(Instruction::Push(20.0)), instructions.next());
        assert_eq!(Some(Instruction::Add), instructions.next());
        assert_eq!(None, instructions.next());

        let mut instructions = instr_iter("3+2").into_iter();

        assert_eq!(Some(Instruction::Push(3.0)), instructions.next());
        assert_eq!(Some(Instruction::Push(2.0)), instructions.next());
        assert_eq!(Some(Instruction::Add), instructions.next());
        assert_eq!(None, instructions.next());
    }

    #[test]
    fn can_compute_add() {
        let mut vm = VM::new();
        assert_eq!(110.0, compute(&mut vm, "90 + 20").unwrap().round());
        assert_eq!(3.0, compute(&mut vm, "1 + 2").unwrap().round());
    }

    #[test]
    fn can_compute_add_brackets() {
        let mut vm = VM::new();
        assert_eq!(3.0, compute(&mut vm, "(1) + (1) + (1)").unwrap().round());
    }

    #[test]
    fn can_compile_sub() {
        let mut instructions = instr_iter("20 - 10").into_iter();

        assert_eq!(Some(Instruction::Push(20.0)), instructions.next());
        assert_eq!(Some(Instruction::Push(10.0)), instructions.next());
        assert_eq!(Some(Instruction::Sub), instructions.next());
        assert_eq!(None, instructions.next());
    }

    #[test]
    fn can_compute_sub() {
        let mut vm = VM::new();
        assert_eq!(1.0, compute(&mut vm, "3 - 2").unwrap().round());
        assert_eq!(-80.0, compute(&mut vm, "20 - 100").unwrap().round());
    }

    #[test]
    fn can_compute_pow() {
        let mut vm = VM::new();
        assert_eq!(9.0, compute(&mut vm, "3 ^ 2").unwrap().round());
        assert_eq!(1024.0, compute(&mut vm, "2^10").unwrap().round());
    }

    #[test]
    fn can_compute_sqrt() {
        let mut vm = VM::new();
        assert_eq!(10.0, compute(&mut vm, "sqrt(100)").unwrap().round());
    }

    #[test]
    fn can_compile_multiple() {
        let mut instructions = instr_iter("3 - sin(90)").into_iter();

        assert_eq!(Some(Instruction::Push(3.0)), instructions.next());
        assert_eq!(Some(Instruction::Push(90.0)), instructions.next());
        assert_eq!(Some(Instruction::Sine), instructions.next());
        assert_eq!(Some(Instruction::Sub), instructions.next());
        assert_eq!(None, instructions.next());
    }

    #[test]
    fn can_compute_multiple() {
        let mut vm = VM::new();
        assert_eq!(2.0, compute(&mut vm, "3 - sin(90)").unwrap().round());
    }

    #[test]
    fn can_compile_with_basic_parens() {
        let mut instructions = instr_iter("2 + (20 - 10)").into_iter();

        assert_eq!(Some(Instruction::Push(2.0)), instructions.next());
        assert_eq!(Some(Instruction::Push(20.0)), instructions.next());
        assert_eq!(Some(Instruction::Push(10.0)), instructions.next());
        assert_eq!(Some(Instruction::Sub), instructions.next());
        assert_eq!(Some(Instruction::Add), instructions.next());
        assert_eq!(None, instructions.next());
    }

    #[test]
    fn can_compute_with_basic_parens() {
        let mut vm = VM::new();
        assert_eq!(20.0, compute(&mut vm, "2 * (20 - 10)").unwrap().round());
        assert_eq!(30.0, compute(&mut vm, "(2 * 20) - 10").unwrap().round());
    }

    #[test]
    fn can_compute_with_precedence() {
        let mut vm = VM::new();
        assert_eq!(11.0, compute(&mut vm, "2 * 3 + 5").unwrap().round());
        assert_eq!(17.0, compute(&mut vm, "2 + 3 * 5").unwrap().round());
    }

    #[test]
    fn can_compute_variables() {
        let mut vm = VM::new();
        compute(&mut vm, "let x = 1 + 2");
        assert_eq!(5.0, compute(&mut vm, "x + 2").unwrap().round());
    }

    #[test]
    fn can_parse() {
        let source = parser::Bite::new("x + y = z");
        let bite = source;

        let mut bite = bite.chomp(parser::Chomp::whitespace());
        let lhs = bite.nibble(parser::Chomp::alphanumeric()).unwrap();

        let mut bite = bite.chomp(parser::Chomp::whitespace());
        let op = bite
            .nibble(parser::Chomp::char_any(&['+', '-', '*', '/']))
            .unwrap();

        let mut bite = bite.chomp(parser::Chomp::whitespace());
        let rhs = bite.nibble(parser::Chomp::alphanumeric()).unwrap();

        let mut bite = bite.chomp(parser::Chomp::whitespace());
        let eq = bite.nibble(parser::Chomp::char('=')).unwrap();

        let mut bite = bite.chomp(parser::Chomp::whitespace());
        let result = bite.nibble(parser::Chomp::alphanumeric()).unwrap();

        assert_eq!(&[lhs, op, rhs, eq, result], &["x", "+", "y", "=", "z"]);
    }

    fn instr_iter(input: &str) -> Vec<Instruction> {
        let tokens: Result<Vec<_>, _> = lexer::tokenize(input.into()).collect();
        let tokens = dbg!(tokens.unwrap());

        let mut compiler = Compiler::new(&tokens);
        let instructions = compiler.compile().expect("failed compile");
        instructions
    }
}
