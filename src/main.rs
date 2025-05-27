#[allow(unused_imports)]
use std::io::{self, Write};
use std::{env, process::exit};

const VALID_COMMANDS: [&str; 3] = ["echo", "type", "exit"];

struct Config {
    path: Option<String>,
}

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let mut path = None;
        if let Ok(v) = env::var("PATH") {
            path = Some(v);
        }
        let config = Config { path };
        handle_command(input, config);
    }
}

fn type_command(command: &str, path: String) {
    if VALID_COMMANDS.contains(&command) {
        println!("{} is a shell builtin", command);
        return;
    }

    let exec_path = path
        .split(":")
        .filter_map(|path| std::fs::read_dir(path).ok())
        .flat_map(|entries| entries.into_iter().filter_map(|entry| entry.ok()))
        .find(|value| {
            if value.file_name() != command {
                return false;
            };
            value.metadata().map_or_else(|_| false, |v| v.is_file())
        });

    if let Some(path) = exec_path {
        println!("{} is {}", command, path.path().display());
    } else {
        println!("{}: not found", command);
    }
}

fn handle_command(input: String, config: Config) {
    let mut parts = input.trim_end().splitn(2, " ");
    match (parts.next(), parts.next()) {
        (Some("type"), Some(type_input)) => {
            type_command(type_input, config.path.unwrap_or("".to_string()));
        }
        (Some("echo"), Some(echo_input)) => {
            println!("{}", echo_input);
        }
        (Some("exit"), Some(_exit_code)) => {
            exit(0);
        }
        _ => {
            println!("{}: command not found", input.trim_end());
        }
    }
    io::stdout().flush().unwrap();
}
