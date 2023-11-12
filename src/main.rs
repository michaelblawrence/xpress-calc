use compiler::RecursiveCompiler;
use vm::VM;

mod compiler;
mod parser;
mod tokenizer;
mod vm;

fn main() {
    let mut vm = VM::new();
    loop {
        print!("Enter expression (example: '5 + 2'): ");
        let expression = read_line();
        match compute_expression(&mut vm, &expression) {
            Some(result) => println!("result = {result}"),
            None => println!("result = undefined"),
        }
    }
}

fn compute_expression(vm: &mut VM, input: &str) -> Option<f64> {
    let source = parser::Bite::new(&input).chomp(parser::Chomp::whitespace());
    let token_iter = tokenizer::tokenize_iter(source);

    let tokens: Vec<_> = match token_iter.collect() {
        Ok(x) => x,
        Err(err) => {
            eprintln!("ERROR: could not interpret input tokens: {err}");
            return None;
        }
    };
    let mut compiler = RecursiveCompiler::new(&tokens);
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

    vm.result()
}

fn read_line() -> String {
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer).unwrap();
    buffer.trim_end_matches('\n').to_string()
}

#[cfg(test)]
mod tests {
    use crate::{tokenizer::Token, vm::Instruction};

    use super::*;

    #[test]
    fn can_parse_sin() {
        let mut tokens = tokenizer::tokenize_iter("sin(90)".into());

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
        assert_eq!(0.0, compute_expression(&mut vm, "sin(0)").unwrap().round());
        assert_eq!(1.0, compute_expression(&mut vm, "sin(90)").unwrap().round());
        assert_eq!(
            0.0,
            compute_expression(&mut vm, "sin(180)").unwrap().round()
        );
        assert_eq!(
            -1.0,
            compute_expression(&mut vm, "sin(270)").unwrap().round()
        );

        assert_eq!(
            0.0,
            compute_expression(&mut vm, "sin(90 + 90)").unwrap().round()
        );
        assert_eq!(
            2.0,
            compute_expression(&mut vm, "sin(90) + sin(90)")
                .unwrap()
                .round()
        );
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
        assert_eq!(
            110.0,
            compute_expression(&mut vm, "90 + 20").unwrap().round()
        );
        assert_eq!(3.0, compute_expression(&mut vm, "1 + 2").unwrap().round());
    }

    #[test]
    fn can_compute_add_brackets() {
        let mut vm = VM::new();
        assert_eq!(
            3.0,
            compute_expression(&mut vm, "(1) + (1) + (1)")
                .unwrap()
                .round()
        );
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
        assert_eq!(1.0, compute_expression(&mut vm, "3 - 2").unwrap().round());
        assert_eq!(
            -80.0,
            compute_expression(&mut vm, "20 - 100").unwrap().round()
        );
    }

    #[test]
    fn can_compute_pow() {
        let mut vm = VM::new();
        assert_eq!(9.0, compute_expression(&mut vm, "3 ^ 2").unwrap().round());
        assert_eq!(1024.0, compute_expression(&mut vm, "2^10").unwrap().round());
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
        assert_eq!(
            2.0,
            compute_expression(&mut vm, "3 - sin(90)").unwrap().round()
        );
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
        assert_eq!(
            20.0,
            compute_expression(&mut vm, "2 * (20 - 10)")
                .unwrap()
                .round()
        );
        assert_eq!(
            30.0,
            compute_expression(&mut vm, "(2 * 20) - 10")
                .unwrap()
                .round()
        );
    }

    #[test]
    fn can_compute_variables() {
        let mut vm = VM::new();
        compute_expression(&mut vm, "let x = 1 + 2");
        assert_eq!(5.0, compute_expression(&mut vm, "x + 2").unwrap().round());
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
        let tokens: Result<Vec<_>, _> = tokenizer::tokenize_iter(input.into()).collect();
        let tokens = dbg!(tokens.unwrap());

        let mut compiler = RecursiveCompiler::new(&tokens);
        let instructions = compiler.compile().expect("failed compile");
        instructions
    }
}
