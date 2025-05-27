#[allow(unused_imports)]
use std::io::{self, Write};
use std::process::exit;

const VALID_COMMANDS: [&str; 3] = ["echo", "type", "exit"];

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
        Some("type") => {
            if let Some(type_input) = parts.next() {
                if VALID_COMMANDS.contains(&type_input) {
                    println!("{} is a shell builtin", type_input);
                } else {
                    println!("{}: not found", type_input);
                }
            }
        }
        Some("echo") => {
            if let Some(echo) = parts.next() {
                println!("{}", echo);
            }
        }
        Some("exit") => {
            exit(0);
        }
        _ => {
            println!("{}: command not found", input.trim_end());
        }
    }
    io::stdout().flush().unwrap();
}
