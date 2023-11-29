use compiler::Compiler;
use vm::VM;

pub mod compiler;
pub mod lexer;
pub mod parser;
pub mod pretty;
pub mod vm;

pub fn compute(vm: &mut VM, input: &str) -> Option<f64> {
    let program = match compile(input) {
        Ok(value) => value,
        Err(msg) => {
            eprintln!("{}", msg);
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

pub fn compile(input: &str) -> Result<Vec<vm::Instruction>, String> {
    let tokens = tokenize(input)?;
    let mut compiler = Compiler::new(&tokens);
    let program = match compiler.compile() {
        Ok(x) => x,
        Err(err) => {
            return Err(format!("ERROR: could not compile program: {err}"));
        }
    };
    Ok(program)
}

pub fn format(input: &str) -> Result<String, String> {
    let tokens = tokenize(input)?;
    let mut compiler = Compiler::new(&tokens);
    let expression_tree = compiler.compile_expression_tree()?;
    let formatted = pretty::pretty_print(expression_tree);
    Ok(formatted)
}

fn tokenize(input: &str) -> Result<Vec<lexer::Token>, String> {
    let source = parser::Bite::new(&input).chomp(parser::Chomp::whitespace());
    let tokens = lexer::tokenize(source).collect();
    match tokens {
        Ok(x) => Ok(x),
        Err(err) => Err(format!("ERROR: could not interpret input tokens: {err}")),
    }
}

#[cfg(test)]
mod tests {
    use crate::{lexer::Token, tests::helpers::ToFixedPrecision, vm::Instruction};

    use super::*;

    #[test]
    fn can_parse_negatives_and_decimals() {
        let mut tokens = lexer::tokenize("-90".into());
        assert_eq!(Some(Ok(Token::LiteralNum(-90.0))), tokens.next());
        assert_eq!(None, tokens.next());

        let mut tokens = lexer::tokenize("0.3".into());
        assert_eq!(Some(Ok(Token::LiteralNum(0.3))), tokens.next());
        assert_eq!(None, tokens.next());

        let mut tokens = lexer::tokenize("0.3 - 0.2".into());
        assert_eq!(Some(Ok(Token::LiteralNum(0.3))), tokens.next());
        assert_eq!(Some(Ok(Token::Sub)), tokens.next());
        assert_eq!(Some(Ok(Token::LiteralNum(0.2))), tokens.next());
        assert_eq!(None, tokens.next());

        let mut tokens = lexer::tokenize("0.3 -0.2".into());
        assert_eq!(Some(Ok(Token::LiteralNum(0.3))), tokens.next());
        assert_eq!(Some(Ok(Token::Sub)), tokens.next());
        assert_eq!(Some(Ok(Token::LiteralNum(0.2))), tokens.next());
        assert_eq!(None, tokens.next());

        let mut tokens = lexer::tokenize("0.3 + -0.2".into());
        assert_eq!(Some(Ok(Token::LiteralNum(0.3))), tokens.next());
        assert_eq!(Some(Ok(Token::Plus)), tokens.next());
        assert_eq!(Some(Ok(Token::LiteralNum(-0.2))), tokens.next());
        assert_eq!(None, tokens.next());
    }

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
    fn can_compile_define_fn() {
        let mut instructions = instr_iter("let s = (x) => sin(x) + x").into_iter();

        assert_eq!(
            Some(Instruction::PushRoutine(vec![
                Instruction::ShadowAssign(String::from("x")),
                Instruction::LoadLocal(String::from("x")),
                Instruction::Sine,
                Instruction::LoadLocal(String::from("x")),
                Instruction::Add
            ])),
            instructions.next()
        );
        assert_eq!(
            Some(Instruction::Assign(String::from("s"))),
            instructions.next()
        );
        assert_eq!(None, instructions.next());
    }

    mod helpers {
        pub trait ToFixedPrecision: Copy {
            fn to_fixed(self, decimals: usize) -> Self;
        }

        impl ToFixedPrecision for f64 {
            fn to_fixed(self, decimals: usize) -> Self {
                let factor = 10i32.pow(decimals as u32) as f64;
                (self * factor).round() / factor
            }
        }
    }

    #[test]
    fn can_compute_define_fn() {
        let mut vm = VM::new();
        assert_eq!(None, compute(&mut vm, "let s = (x) => (x - 3) * (x - 2)"));
        assert_eq!(0.0, compute(&mut vm, "s(3)").unwrap().round());
        assert_eq!(0.0, compute(&mut vm, "s(2)").unwrap().round());
        assert_eq!(6.0, compute(&mut vm, "s(0)").unwrap().round());
        assert_eq!(6.0, compute(&mut vm, "s(s(3) + s(2))").unwrap().round());

        assert_eq!(None, compute(&mut vm, "let s1 = (x, y) => x^2 + y^2"));
        assert_eq!(
            1.0,
            compute(&mut vm, "s1(1/sqrt(2), 1/sqrt(2))")
                .unwrap()
                .to_fixed(2)
        );
        assert_eq!(
            1.0,
            compute(&mut vm, "s1(sin(40), cos(40))")
                .unwrap()
                .to_fixed(2)
        );
        assert_eq!(
            1.0,
            compute(&mut vm, "s1(sin(90), cos(90))")
                .unwrap()
                .to_fixed(2)
        );
        assert_eq!(
            None,
            compute(&mut vm, "let s2 = (x, y, z) => (x - y) mod z")
        );
        assert_eq!(1.0, compute(&mut vm, "s2(10, 1, 4)").unwrap().round());
        assert_eq!(2.0, compute(&mut vm, "s2(20, 9, 3)").unwrap().round());
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
        assert_eq!(
            10.0,
            compute(&mut vm, "( 1 ) + (2) + (3 ) + ( 4)")
                .unwrap()
                .round()
        );
        assert_eq!(16.0, compute(&mut vm, "( 8 + 8 )").unwrap().round());
        assert_eq!(4.0, compute(&mut vm, " ( 2 + 2 ) ").unwrap().round());
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
    fn can_compute_lt() {
        let mut vm = VM::new();
        assert_eq!(0.0, compute(&mut vm, "3 < 2").unwrap().round());
        assert_eq!(0.0, compute(&mut vm, "2 < 2").unwrap().round());
        assert_eq!(1.0, compute(&mut vm, "1 < 2").unwrap().round());

        assert_eq!(0.0, compute(&mut vm, "3 <= 2").unwrap().round());
        assert_eq!(1.0, compute(&mut vm, "2 <= 2").unwrap().round());
        assert_eq!(1.0, compute(&mut vm, "1 <= 2").unwrap().round());
    }

    #[test]
    fn can_compute_gt() {
        let mut vm = VM::new();
        assert_eq!(1.0, compute(&mut vm, "3 > 2").unwrap().round());
        assert_eq!(0.0, compute(&mut vm, "2 > 2").unwrap().round());
        assert_eq!(0.0, compute(&mut vm, "1 > 2").unwrap().round());

        assert_eq!(1.0, compute(&mut vm, "3 >= 2").unwrap().round());
        assert_eq!(1.0, compute(&mut vm, "2 >= 2").unwrap().round());
        assert_eq!(0.0, compute(&mut vm, "1 >= 2").unwrap().round());
    }

    #[test]
    fn can_compute_if_statements() {
        let mut vm = VM::new();
        assert_eq!(
            0.0,
            compute(&mut vm, "{let x = 0; if (3 < 2) { let x = 3 }; x}")
                .unwrap()
                .round()
        );
        assert_eq!(
            3.0,
            compute(&mut vm, "{let x = 0; if (3 > 2) { let x = 3 }; x}")
                .unwrap()
                .round()
        );
    }

    #[test]
    fn can_compute_if_else_expressions() {
        let mut vm = VM::new();
        assert_eq!(
            5.0,
            compute(&mut vm, "if (1) { 5 } else { 8 }").unwrap().round()
        );
        assert_eq!(
            8.0,
            compute(&mut vm, "if (0) { 5 } else { 8 }").unwrap().round()
        );
    }

    #[test]
    fn can_compute_loop_program() {
        let mut vm = VM::new();
        assert_eq!(
            None,
            compute(
                &mut vm,
                "let loop = (i, n, f) => { if (i < n) { f(); loop(i + 1, n, f); } else {} }"
            )
        );
        assert_eq!(None, compute(&mut vm, "let y = 0"));
        assert_eq!(None, compute(&mut vm, "loop(0, 9, () => let y = y + 1)"));
        assert_eq!(9.0, compute(&mut vm, "y").unwrap().round());
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
    fn can_compute_with_implicit_multiplication() {
        let mut vm = VM::new();
        assert_eq!(20.0, compute(&mut vm, "2(20 - 10)").unwrap().round());
        assert_eq!(None, compute(&mut vm, "let x = 3"));
        assert_eq!(16.0, compute(&mut vm, "(x)(x) + 2x + 1").unwrap().round());
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
    fn can_compute_blocks() {
        let mut vm = VM::new();
        compute(&mut vm, "let x = { let y = 1 + 2; y + 3 }");
        assert_eq!(6.0, compute(&mut vm, "x").unwrap().round());
    }

    #[test]
    fn can_compute_block_fn() {
        let mut vm = VM::new();
        compute(
            &mut vm,
            "let calc = (x) => { let y = 2(x + 1); y^2 + y + 3 }",
        );
        assert_eq!(45.0, compute(&mut vm, "calc(2)").unwrap().round());

        compute(&mut vm, "let y = 15");
        compute(&mut vm, "let calc2 = ( x ) => { let y = y * x }");
        assert_eq!(None, compute(&mut vm, "calc2(3)"));
        assert_eq!(45.0, compute(&mut vm, "y").unwrap().round());

        compute(&mut vm, "let calc = (x, y) => { x^2 + y^2 }");
        assert_eq!(13.0, compute(&mut vm, "calc(2, 3)").unwrap().round());

        compute(&mut vm, "let calc = () => {}");
        compute(&mut vm, "calc()");
        assert!(vm.pop_result().is_none());
    }

    #[test]
    fn can_pretty_print() {
        let formatted = super::format("let calc= ( x, ) =>sin (90)").unwrap();
        assert_eq!("let calc = (x) => sin(90)", formatted);
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

    pub fn compute(vm: &mut VM, input: &str) -> Option<f64> {
        let program = match compile(input) {
            Ok(value) => value,
            Err(msg) => {
                panic!("{}", msg);
            }
        };

        match vm.run(&program) {
            Ok(_) => {}
            Err(err) => {
                panic!("ERROR: could not compute expression: {err}");
            }
        }

        vm.pop_result()
    }

    fn instr_iter(input: &str) -> Vec<Instruction> {
        let tokens: Result<Vec<_>, _> = lexer::tokenize(input.into()).collect();
        let tokens = dbg!(tokens.unwrap());

        let mut compiler = Compiler::new(&tokens);
        let instructions = compiler.compile().expect("failed compile");
        instructions
    }
}
