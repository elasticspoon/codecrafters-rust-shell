use std::io::{stderr, stdout, ErrorKind};
use std::io::{stdin, Write};
use std::{
    env,
    process::{exit, Command},
};

const VALID_COMMANDS: [&str; 4] = ["echo", "type", "exit", "pwd"];

struct Config<'a> {
    path: Option<String>,
    home: Option<String>,
    stdout: Box<dyn Write + 'a>,
}

impl Config<'_> {
    fn println(&mut self, value: &str) {
        writeln!(self.stdout, "{}", value).expect("failed to write");
        self.stdout.flush().expect("failed to flush");
    }
}

fn main() {
    loop {
        let mut config = Config {
            path: None,
            home: None,
            stdout: Box::new(stdout()),
        };
        config.println("$ ");

        let mut input = String::new();

        stdin().read_line(&mut input).unwrap();

        if let Ok(v) = env::var("PATH") {
            config.path = Some(v);
        }

        if let Ok(v) = env::var("HOME") {
            config.home = Some(v);
        }

        handle_command(input, config);
    }
}

fn type_command(command: &str, config: &mut Config) {
    if VALID_COMMANDS.contains(&command) {
        let output = format!("{} is a shell builtin", command);
        config.println(output.as_str());
        return;
    }

    let path = config.path.as_deref().unwrap_or(".");
    let exec_path = path
        .split(":")
        .filter_map(|path| std::fs::read_dir(path).ok())
        .flat_map(|entries| entries.into_iter().filter_map(|entry| entry.ok()))
        .find(|value| value.file_name() == command);

    if let Some(path) = exec_path {
        let output = format!("{} is {}", command, path.path().display());
        config.println(output.as_str());
    } else {
        let output = format!("{}: not found", command);
        config.println(output.as_str());
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
                let output = format!("{} is {}", "cd: {}: No such file or directory", path);
                config.println(output.as_str());
            }
            _ => panic!("unexpected error: {:?}", e),
        }
    }
}

fn parse_input(input: &String) -> Vec<String> {
    let v: Vec<String> = input.trim_end().split(" ").map(|s| s.to_string()).collect();
    let f = v.get(0).map(|f| f.as_str());
    let rest = v.get(1..).map(|v| v.join(" "));
}

fn handle_command(input: String, mut config: Config) {
    let mut parts = input.trim_end().splitn(2, " ");

    // match (parts.get(0), parts.get(1..)) {
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
            echo_command(echo_input, &mut config);
        }
        (Some("exit"), Some(_exit_code)) => {
            exit(0);
        }
        (Some(command), args) => {
            exec_command(args, command, config);
        }
        _ => {
            config.println("command not found");
        }
    }
}

fn echo_command(echo_input: &str, config: &mut Config) {
    config.println(echo_input);
}

fn pwd_command(mut config: Config) {
    if let Ok(path) = std::env::current_dir() {
        config.println(path.to_str().unwrap());
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
            config
                .stdout
                .write_all(&result.stdout)
                .expect("Failed write to stdout.");
            config.stdout.flush().expect("failed to flush");
            stderr()
                .write_all(&result.stderr)
                .expect("Failed write to stderr.");
            stderr().flush().unwrap();
        }
        Err(e) if e.kind() == ErrorKind::NotFound => {
            let output = format!("{}: command not found", command);
            config.println(output.as_str());
        }
        Err(e) => panic!("{:?} error when executing command", e),
    }
}

#[cfg(test)]
mod tests {
    use std::io::ErrorKind;

    use crate::{echo_command, exec_command, type_command, Config};

    #[test]
    fn test_echo_command() {
        let mut res: Vec<u8> = Vec::new();
        // this is a block like this because we need to ensure the borrow
        // checker knows that the borrow of res is dropped after the function
        // under test is done.
        {
            let mut config = Config {
                path: None,
                home: None,
                stdout: Box::new(&mut res),
            };
            echo_command("123", &mut config);
        }

        let out = String::from_utf8(res).unwrap();
        assert_eq!(out, "123\n");
    }

    #[test]
    fn test_echo_command_presenves_spacing_woth_single_quotes() {
        let mut res: Vec<u8> = Vec::new();
        {
            let mut config = Config {
                path: None,
                home: None,
                stdout: Box::new(&mut res),
            };
            echo_command("'a'", &mut config);
        }

        let out = String::from_utf8(res).unwrap();
        assert_eq!(out, "a\n");
    }

    #[test]
    fn test_exec_command_found() {
        let mut res: Vec<u8> = Vec::new();
        {
            let config = Config {
                path: None,
                home: None,
                stdout: Box::new(&mut res),
            };
            exec_command(Some("alice"), "./test/custom_exe", config);
        }

        let out = String::from_utf8(res).unwrap();
        assert_eq!(
            out,
            concat!(
                "Program was passed 2 args (including program name).\n",
                "Arg #0 (program name): custom_exe\n",
                "Arg #1: alice\n"
            )
        )
    }

    #[test]
    fn test_exec_command_not_found() {
        let mut res: Vec<u8> = Vec::new();
        {
            let config = Config {
                path: None,
                home: None,
                stdout: Box::new(&mut res),
            };
            exec_command(Some("alice"), "./test/missing", config);
        }

        let out = String::from_utf8(res).unwrap();
        assert_eq!(out, "./test/missing: command not found\n",)
    }

    #[test]
    fn test_type_command_builtin() {
        let mut res: Vec<u8> = Vec::new();
        {
            let mut config = Config {
                path: None,
                home: None,
                stdout: Box::new(&mut res),
            };
            type_command("echo", &mut config);
        }

        let out = String::from_utf8(res).unwrap();
        assert_eq!(out, "echo is a shell builtin\n");
    }

    #[test]
    fn test_type_command_not_found() {
        let mut res: Vec<u8> = Vec::new();
        {
            let mut config = Config {
                path: None,
                home: None,
                stdout: Box::new(&mut res),
            };
            type_command("not_found", &mut config);
        }

        let out = String::from_utf8(res).unwrap();
        assert_eq!(out, "not_found: not found\n");
    }

    #[test]
    fn test_type_command_path() {
        if let Err(res) = std::fs::create_dir("tmp") {
            if res.kind() != ErrorKind::AlreadyExists {
                panic!("{:?}", res);
            }
        }
        std::fs::write("./tmp/test", "test").expect("failed to write file");
        let mut res: Vec<u8> = Vec::new();
        {
            let mut config = Config {
                path: Some("./tmp".to_string()),
                home: None,
                stdout: Box::new(&mut res),
            };
            type_command("test", &mut config);
        }

        let out = String::from_utf8(res).unwrap();
        assert_eq!(out, "test is ./tmp/test\n");
    }
}
