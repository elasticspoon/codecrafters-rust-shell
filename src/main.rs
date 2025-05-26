#[allow(unused_imports)]
use std::io::{self, Write};
use std::process::exit;

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        handle_command(input);
    }
}

fn handle_command(input: String) {
    let mut parts = input.trim_end().splitn(2, " ");
    let command = parts.next();
    match command {
        Some("echo") => {
            if let Some(echo) = parts.next() {
                println!("{}", echo);
                io::stdout().flush().unwrap();
            }
        }
        Some("exit") => {
            exit(0);
        }
        _ => {
            println!("{}: command not found", input.trim_end());
            io::stdout().flush().unwrap();
        }
    }
}
