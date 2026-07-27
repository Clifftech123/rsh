use std::collections::HashMap;
use std::env;

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
    map
}

// Prints the current directory.
fn builtin_cd(_args: &[String], _state: &mut ShellState) -> ControlFlow {
    match env::current_dir() {
        Ok(path) => println!("{}", path.display()),
        Err(e) => print!("rsh : {}", e),
    }

    ControlFlow::Continue
}

// Prints basic help information and lists available builtin commands.
fn builtin_help(_args: &[String], state: &mut ShellState) -> ControlFlow {
    println!("rsh: a small shell written in Rust");
    println!("Type program names and arguments, and hit enter.");
    println!("The following are built in:");
    for name in register().keys() {
        println!("  {}", name);
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
        Err(e) => eprintln!("rsh: {}", e),
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
                None => eprintln!("rsh: alias: {} not found", arg),
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
