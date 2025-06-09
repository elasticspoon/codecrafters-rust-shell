use std::io::{stderr, stdout, BufRead, BufReader, ErrorKind};
use std::io::{stdin, Read, Write};
use std::{
    env,
    process::{exit, Command},
};

const VALID_COMMANDS: [&str; 4] = ["echo", "type", "exit", "pwd"];

struct Config {
    path: Option<String>,
    home: Option<String>,
    stdout: Box<dyn Write>,
    stdin: Box<dyn Read>,
}

fn main() {
    loop {
        let mut config = Config {
            path: None,
            home: None,
            stdout: Box::new(stdout()),
            stdin: Box::new(stdin()),
        };
        write!(config.stdout, "$ ").expect("failed to write");
        config.stdout.flush().expect("failed to flush");

        let mut buf_reader = BufReader::new(config.stdin);
        let mut input = String::new();
        buf_reader
            .read_line(&mut input)
            .expect("failed to read stdin");
        dbg!(&input);

        if let Ok(v) = env::var("PATH") {
            config.path = Some(v);
        }

        if let Ok(v) = env::var("HOME") {
            config.home = Some(v);
        }

        config.stdin = Box::new(stdin());
        handle_command(input, config);
    }
}

fn type_command(command: &str, config: &mut Config) {
    if VALID_COMMANDS.contains(&command) {
        writeln!(config.stdout, "{} is a shell builtin", command).expect("failed to write");
        config.stdout.flush().expect("failed to flush");
        return;
    }

    let path = config.path.as_deref().unwrap_or(".");
    let exec_path = path
        .split(":")
        .filter_map(|path| std::fs::read_dir(path).ok())
        .flat_map(|entries| entries.into_iter().filter_map(|entry| entry.ok()))
        .find(|value| value.file_name() == command);

    if let Some(path) = exec_path {
        writeln!(config.stdout, "{} is {}", command, path.path().display())
            .expect("failed to write");
        config.stdout.flush().expect("failed to flush");
    } else {
        writeln!(config.stdout, "{}: not found", command).expect("failed to write");
        config.stdout.flush().expect("failed to flush");
    }
}

fn cd_command(path: &str, mut config: Config) {
    let target_path = if path == "~" {
        config.home.as_deref().unwrap_or("")
    } else {
        path
    };

    if let Err(e) = std::env::set_current_dir(target_path) {
        match e.kind() {
            ErrorKind::NotFound => {
                writeln!(config.stdout, "cd: {}: No such file or directory", path)
                    .expect("failed to write");
                config.stdout.flush().expect("failed to flush");
            }
            _ => panic!("unexpected error: {:?}", e),
        }
    }
}

fn handle_command(input: String, mut config: Config) {
    let mut parts = input.trim_end().splitn(2, " ");
    match (parts.next(), parts.next()) {
        (Some("cd"), Some(path)) => {
            cd_command(path, config);
        }
        (Some("pwd"), None) => {
            pwd_command(config);
        }
        (Some("type"), Some(type_input)) => {
            type_command(type_input, &mut config);
        }
        (Some("echo"), Some(echo_input)) => {
            writeln!(config.stdout, "{}", echo_input).expect("failed to write");
            config.stdout.flush().expect("failed to flush");
        }
        (Some("exit"), Some(_exit_code)) => {
            exit(0);
        }
        (Some(command), args) => {
            exec_command(args, command, config);
        }
        _ => {
            writeln!(config.stdout, "command not found").expect("failed to write");
            config.stdout.flush().expect("failed to flush");
        }
    }
}

fn pwd_command(mut config: Config) {
    if let Ok(path) = std::env::current_dir() {
        writeln!(config.stdout, "{}", path.to_str().unwrap()).expect("failed to write");
        config.stdout.flush().expect("failed to flush");
    } else {
        panic!("invalid current_dir")
    }
}

fn exec_command(args: Option<&str>, command: &str, mut config: Config) {
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
            config.stdout.flush().expect("failed to flush");
            stderr()
                .write_all(&result.stderr)
                .expect("Failed write to stderr.");
            stderr().flush().unwrap();
        }
        Err(e) if e.kind() == ErrorKind::NotFound => {
            writeln!(config.stdout, "{}: command not found", command).expect("failed to write");
            config.stdout.flush().expect("failed to flush");
        }
        Err(e) => panic!("{:?} error when executing command", e),
    }
}

#[cfg(test)]
mod tests {
    use crate::{type_command, Config};

    #[test]
    fn test_type_command() {
        let mut res: Vec<u8> = Vec::new();
        let mut config = Config {
            path: Some("/home/bandito/.cargo/bin".to_string()),
            home: None,
            stdout: Box::new(&mut res),
            stdin: Box::new("something".as_bytes()),
        };
        type_command("cargo", &mut config);

        let out = String::from_utf8(res).expect("Invalid UTF-8");
        assert_eq!(out, "echo is a shell builtin\n");
    }
}
