use xpress_calc::vm::VM;

fn main() {
    let mut vm = VM::new();
    loop {
        print!("Enter expression (example: '5 + 2'): ");
        let expression = read_line();
        match xpress_calc::compute(&mut vm, &expression) {
            Some(result) => println!("{result}"),
            None => println!("<undefined>"),
        }
    }
}

fn read_line() -> String {
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer).unwrap();
    buffer.trim_end_matches('\n').to_string()
}
