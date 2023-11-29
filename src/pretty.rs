use std::fmt::Write;

use crate::compiler::{BinaryOp, Func0Op, Func1Op, RecursiveExpression};

pub(crate) fn pretty_print(program_expression: RecursiveExpression) -> String {
    let mut pretty_output = String::new();
    delve(&program_expression, &mut pretty_output, 0);

    fn delve(node: &RecursiveExpression, output: &mut String, indent: usize) {
        match node {
            RecursiveExpression::Block(statements) => {
                output.push('{');
                output.push('\n');

                statements.iter().for_each(|node| {
                    delve(node, output, indent + 1);
                    output.push_str(";\n");
                });
                *output = output
                    .trim_end_matches(|c| matches!(c, ';' | '\n'))
                    .to_string();
                output.push_str("\n}");
            }
            RecursiveExpression::Literal(x) => write!(output, "{x}").unwrap(),
            RecursiveExpression::Local(ident) => output.push_str(ident),
            RecursiveExpression::FuncDeclaration(params, body) => {
                output.push('(');
                let params_iter = params.iter();
                params_iter.for_each(|ident| write!(output, "{ident}, ").unwrap());
                *output = output
                    .trim_end_matches(|c| matches!(c, ',' | ' '))
                    .to_string();
                output.push(')');
                output.push_str(" => ");
                delve(body, output, indent + 1);
            }
            RecursiveExpression::If(condition, block) => {
                output.push_str("if (");
                delve(condition, output, indent + 1);
                output.push_str(") ");
                delve(block, output, indent + 1);
            }
            RecursiveExpression::IfElse(condition, if_block, else_block) => {
                output.push_str("if (");
                delve(condition, output, indent + 1);
                output.push_str(") ");
                delve(if_block, output, indent + 1);
                output.push_str(" else ");
                delve(else_block, output, indent + 1);
            }
            RecursiveExpression::AssignOp(ident, value) => {
                write!(output, "let {ident} = ").unwrap();
                delve(value, output, indent + 1);
            }
            RecursiveExpression::BinaryOp(lhs, op, rhs) => {
                delve(lhs, output, indent);
                match op {
                    BinaryOp::Add => output.push_str(" + "),
                    BinaryOp::Sub => output.push_str(" - "),
                    BinaryOp::Div => output.push_str(" / "),
                    BinaryOp::Mul => output.push_str(" * "),
                    BinaryOp::Mod => output.push_str(" % "),
                    BinaryOp::Pow => output.push_str(" ^"),
                    BinaryOp::LT => output.push_str(" < "),
                    BinaryOp::LTE => output.push_str(" <= "),
                    BinaryOp::GT => output.push_str(" > "),
                    BinaryOp::GTE => output.push_str(" >= "),
                }
                delve(rhs, output, indent);
            }
            RecursiveExpression::Func0(op) => match op {
                Func0Op::Rand => output.push_str("rand()"),
            },
            RecursiveExpression::Func1(op, value) => {
                match op {
                    Func1Op::Sin => output.push_str("sin("),
                    Func1Op::Cos => output.push_str("cos("),
                    Func1Op::Sqrt => output.push_str("sqrt("),
                    Func1Op::Log => output.push_str("log("),
                }
                delve(value, output, indent + 1);
                output.push(')');
            }
            RecursiveExpression::FuncLocal(ident, args) => {
                write!(output, "{ident}(").unwrap();
                args.iter()
                    .rev()
                    .for_each(|node| delve(node, output, indent + 1));
                let args_iter = args.iter();
                args_iter.for_each(|node| {
                    delve(node, output, indent + 1);
                    output.push_str(", ");
                });
                *output = output
                    .trim_end_matches(|c| matches!(c, ',' | ' '))
                    .to_string();
                output.push(')');
            }
        }
    }

    pretty_output
}
