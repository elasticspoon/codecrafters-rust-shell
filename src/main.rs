#[allow(unused_imports)]
use std::io::{self, Write};
use std::{
    env,
    io::{stderr, stdout, ErrorKind},
    process::{exit, Command},
};

const VALID_COMMANDS: [&str; 4] = ["echo", "type", "exit", "pwd"];

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
        .find(|value| value.file_name() == command);

    if let Some(path) = exec_path {
        println!("{} is {}", command, path.path().display());
    } else {
        println!("{}: not found", command);
    }
}

fn handle_command(input: String, config: Config) {
    let mut parts = input.trim_end().splitn(2, " ");
    match (parts.next(), parts.next()) {
        (Some("pwd"), None) => {
            if let Ok(path) = std::env::current_dir() {
                println!("{}", path.to_str().unwrap());
            } else {
                panic!("invalid current_dir")
            }
        }
        (Some("type"), Some(type_input)) => {
            type_command(type_input, config.path.unwrap_or(".".to_string()));
        }
        (Some("echo"), Some(echo_input)) => {
            println!("{}", echo_input);
        }
        (Some("exit"), Some(_exit_code)) => {
            exit(0);
        }
        (Some(command), args) => {
            let output = if let Some(command_args) = args {
                Command::new(command).args(command_args.split(" ")).output()
            } else {
                Command::new(command).output()
            };
            match output {
                Ok(result) => {
                    stdout()
                        .write_all(&result.stdout)
                        .expect("Failed write to stdout.");
                    stderr()
                        .write_all(&result.stderr)
                        .expect("Failed write to stderr.")
                }
                Err(e) if e.kind() == ErrorKind::NotFound => {
                    println!("{}: command not found", command)
                }
                Err(e) => panic!("{:?} error when executing command", e),
            }
        }
        _ => {
            println!("command not found");
        }
    }
    io::stdout().flush().unwrap();
}
