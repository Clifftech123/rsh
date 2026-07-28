use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

pub const RESET: &str = "\x1B[0m";
pub const RED: &str = "\x1B[31m";
pub const GREEN: &str = "\x1B[32m";
pub const YELLOW: &str = "\x1B[33m";
pub const BLUE: &str = "\x1B[34m";
pub const CYAN: &str = "\x1B[36m";

/// Tells the shell loop what to do after a builtin command runs.
pub enum ControlFlow {
    Continue,
    Exit(i32),
}

/// Shared shell state that builtin commands can read or update.
#[derive(Default)]
pub struct ShellState {
    pub history: Vec<String>,
    pub aliases: HashMap<String, String>,
}

impl ShellState {
    /// Creates a new empty shell state.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Function signature every builtin command must match.
pub type BuiltinFn = fn(&[String], &mut ShellState) -> ControlFlow;

/// Builds the command name to builtin function lookup table.
///
/// To add a new builtin, write a function with the `BuiltinFn` signature,
/// then insert it into this map.
pub fn register() -> HashMap<&'static str, BuiltinFn> {
    let mut map: HashMap<&'static str, BuiltinFn> = HashMap::new();
    map.insert("cd", builtin_cd);
    map.insert("help", builtin_help);
    map.insert("exit", builtin_exit);
    map.insert("pwd", builtin_pwd);
    map.insert("history", builtin_history);
    map.insert("alias", builtin_alias);
    map.insert("echo", builtin_echo);
    map.insert("list", builtin_list);
    map.insert("clear", builtin_clear);

    map
}

// Changes the current directory.
fn builtin_cd(args: &[String], _state: &mut ShellState) -> ControlFlow {
    if args.len() > 2 {
        eprintln!("{RED}rsh:{RESET} cd: too many arguments");
        return ControlFlow::Continue;
    }

    let target: PathBuf = if args.len() == 1 || args[1] == "~" {
        match env::var_os("HOME").or_else(|| env::var_os("USERPROFILE")) {
            Some(home) => PathBuf::from(home),
            None => {
                eprintln!("{RED}rsh:{RESET} cd: HOME not set");
                return ControlFlow::Continue;
            }
        }
    } else {
        PathBuf::from(&args[1])
    };

    if let Err(e) = env::set_current_dir(&target) {
        eprintln!("{RED}rsh:{RESET} cd: {}: {}", target.display(), e);
    }

    ControlFlow::Continue
}

// Prints basic help information and lists available builtin commands.
fn builtin_help(_args: &[String], state: &mut ShellState) -> ControlFlow {
    println!("{YELLOW}rsh:{RESET} a small shell written in Rust");
    println!("Type program names and arguments, and hit enter.");
    println!("{YELLOW}The following are built in:{RESET}");
    for name in register().keys() {
        println!("  {GREEN}{}{RESET}", name);
    }
    println!("Use the man command for information on other programs.");
    let _ = state; // not needed here, but shows the signature is uniform
    ControlFlow::Continue
}

// Tells the shell loop to exit with status code 0.
fn builtin_exit(_args: &[String], _state: &mut ShellState) -> ControlFlow {
    ControlFlow::Exit(0)
}

// Prints the current working directory.
fn builtin_pwd(_args: &[String], _state: &mut ShellState) -> ControlFlow {
    match env::current_dir() {
        Ok(path) => println!("{}", path.display()),
        Err(e) => eprintln!("{RED}rsh:{RESET} {}", e),
    }
    ControlFlow::Continue
}

// Prints the saved command history with line numbers.
fn builtin_history(_args: &[String], state: &mut ShellState) -> ControlFlow {
    for (i, line) in state.history.iter().enumerate() {
        println!("{:>4}  {}", i + 1, line);
    }
    ControlFlow::Continue
}

// Shows existing aliases, creates new aliases, or prints one alias by name.
fn builtin_alias(args: &[String], state: &mut ShellState) -> ControlFlow {
    if args.len() == 1 {
        for (name, value) in &state.aliases {
            println!("alias {}='{}'", name, value);
        }
        return ControlFlow::Continue;
    }

    for arg in &args[1..] {
        if let Some((name, value)) = arg.split_once('=') {
            state.aliases.insert(name.to_string(), value.to_string());
        } else {
            match state.aliases.get(arg) {
                Some(value) => println!("alias {}='{}'", arg, value),
                None => eprintln!("{RED}rsh:{RESET} alias: {} not found", arg),
            }
        }
    }

    ControlFlow::Continue
}

// Prints all arguments after `echo`, separated by spaces.
fn builtin_echo(args: &[String], _state: &mut ShellState) -> ControlFlow {
    println!("{}", args[1..].join(" "));
    ControlFlow::Continue
}

// Lists files and folders in the target directory (or current directory).
fn builtin_list(args: &[String], _state: &mut ShellState) -> ControlFlow {
    let target = args.get(1).map(String::as_str).unwrap_or(".");

    let read_dir = match fs::read_dir(target) {
        Ok(iter) => iter,
        Err(e) => {
            eprintln!("{RED}rsh:{RESET} list: {}: {}", target, e);
            return ControlFlow::Continue;
        }
    };

    let mut entries: Vec<(String, bool)> = Vec::new();
    for entry in read_dir {
        match entry {
            Ok(item) => {
                let name = item.file_name().to_string_lossy().into_owned();
                let is_dir = item.path().is_dir();
                entries.push((name, is_dir));
            }
            Err(e) => eprintln!("{RED}rsh:{RESET} list: {}", e),
        }
    }

    entries.sort_by(|a, b| a.0.cmp(&b.0));
    for (name, is_dir) in entries {
        if is_dir {
            println!("{BLUE}{}{RESET}", name);
        } else {
            println!("{}", name);
        }
    }

    ControlFlow::Continue
}

// Clears the terminal screen and moves cursor to top-left.
fn builtin_clear(_args: &[String], _state: &mut ShellState) -> ControlFlow {
    print!("\x1B[2J\x1B[H");
    let _ = io::stdout().flush();
    ControlFlow::Continue
}
