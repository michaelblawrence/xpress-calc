use std::fmt::Write;

use crate::compiler::{BinaryOp, Func0Op, Func1Op, RecursiveExpression};

pub(crate) fn pretty_print(program_expression: RecursiveExpression, which: PrettyFormat) -> String {
    let mut pretty_output = String::new();
    delve(&program_expression, None, &mut pretty_output, 0, which);

    fn delve(
        inner: &RecursiveExpression,
        parent: Option<&RecursiveExpression>,
        output: &mut String,
        indent: usize,
        which: PrettyFormat,
    ) {
        match inner {
            RecursiveExpression::Block(statements) => {
                output.push('{');
                which.push_newline(output, indent + 1);

                statements.iter().for_each(|node| {
                    delve(node, Some(inner), output, indent + 1, which);
                    output.push_str(";");
                    which.push_newline(output, indent + 1);
                });

                *output = output
                    .trim_end_matches(|c| matches!(c, ';' | '\n' | ' '))
                    .to_string();

                which.push_newline(output, indent);
                output.push('}');
            }
            RecursiveExpression::FieldAccess(lhs, ident) => {
                delve(lhs, Some(inner), output, indent, which);
                output.push('.');
                output.push_str(ident);
            }
            RecursiveExpression::ObjectLiteral(obj) => {
                output.push('{');
                which.push_newline(output, indent + 1);

                obj.iter().for_each(|(key, node)| {
                    output.push_str(key);
                    output.push_str(": ");
                    delve(node, Some(inner), output, indent + 1, which);
                    output.push(',');
                    which.push_newline(output, indent + 1);
                });

                *output = output
                    .trim_end_matches(|c| matches!(c, '\n' | ' '))
                    .to_string();

                which.push_newline(output, indent);
                output.push('}');
            }
            RecursiveExpression::Literal(x) => write!(output, "{x}").unwrap(),
            RecursiveExpression::Local(ident) => output.push_str(ident),
            RecursiveExpression::FuncDeclaration(params, body) => {
                output.push('(');
                let params_iter = params.iter();
                params_iter.for_each(|ident| {
                    write!(output, "{ident},").unwrap();
                    which.push_space(output);
                });
                *output = output
                    .trim_end_matches(|c| matches!(c, ',' | ' '))
                    .to_string();
                output.push(')');
                which.push_space(output);
                output.push_str("=>");
                which.push_space(output);
                delve(body, Some(inner), output, indent, which);
            }
            RecursiveExpression::If(condition, block) => {
                output.push_str("if (");
                delve(condition, Some(inner), output, indent, which);
                output.push_str(") ");
                delve(block, Some(inner), output, indent, which);
            }
            RecursiveExpression::IfElse(condition, if_block, else_block) => {
                output.push_str("if (");
                delve(condition, Some(inner), output, indent, which);
                output.push_str(") ");
                delve(if_block, Some(inner), output, indent, which);
                output.push_str(" else ");
                delve(else_block, Some(inner), output, indent, which);
            }
            RecursiveExpression::AssignOp(ident, value) => {
                write!(output, "let {ident}").unwrap();
                which.push_space(output);
                output.push('=');
                which.push_space(output);
                delve(value, Some(inner), output, indent, which);
            }
            RecursiveExpression::BinaryOp(lhs, op, rhs) => {
                let requires_parens = match parent {
                    Some(RecursiveExpression::BinaryOp(_, parent_op, _)) => {
                        let precedence = op.precedence();
                        parent_op.precedence() != precedence && precedence < 3
                    }
                    _ => false,
                };
                if requires_parens {
                    output.push('(');
                }
                delve(lhs, Some(inner), output, indent, which);
                let op_str = match op {
                    BinaryOp::Add => " + ",
                    BinaryOp::Sub => " - ",
                    BinaryOp::Div => " / ",
                    BinaryOp::Mul => " * ",
                    BinaryOp::Mod => " % ",
                    BinaryOp::Pow => "^",
                    BinaryOp::EQ => " == ",
                    BinaryOp::NEQ => " != ",
                    BinaryOp::LT => " < ",
                    BinaryOp::LTE => " <= ",
                    BinaryOp::GT => " > ",
                    BinaryOp::GTE => " >= ",
                };
                match which {
                    PrettyFormat::Minified => output.push_str(op_str.trim()),
                    PrettyFormat::Spaced | PrettyFormat::Indented => output.push_str(op_str),
                }
                delve(rhs, Some(inner), output, indent, which);
                if requires_parens {
                    output.push(')');
                }
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
                    Func1Op::Round => output.push_str("round("),
                    Func1Op::Floor => output.push_str("floor("),
                }
                delve(value, Some(inner), output, indent, which);
                output.push(')');
            }
            RecursiveExpression::FuncLocal(ident, args) => {
                write!(output, "{ident}(").unwrap();
                args.iter().for_each(|node| {
                    delve(node, Some(inner), output, indent, which);
                    output.push_str(",");
                    which.push_space(output);
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

#[derive(Debug, Clone, Copy)]
pub enum PrettyFormat {
    Minified,
    Spaced,
    Indented,
}

impl PrettyFormat {
    fn push_newline(&self, output: &mut String, indent: usize) {
        match self {
            PrettyFormat::Minified => (),
            PrettyFormat::Spaced => output.push(' '),
            PrettyFormat::Indented => {
                output.push('\n');
                if indent > 0 {
                    output.push_str(&" ".repeat(indent * 4));
                }
            }
        }
    }
    fn push_space(&self, output: &mut String) {
        match self {
            PrettyFormat::Minified => (),
            PrettyFormat::Spaced => output.push(' '),
            PrettyFormat::Indented => output.push(' '),
        }
    }
}
