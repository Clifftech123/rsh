

mod builtins;
mod parser;

use builtins::{ControlFlow, ShellState};
use std::io::{self, Write};
use std::process::Command;

fn main() {
    let builtins = builtins::register();
    let mut state = ShellState::new();

    loop {
        print!("> ");
        // Make sure the prompt actually appears before we block on input.
        io::stdout().flush().ok();

        let line = match read_line() {
            Some(l) => l,
            None => break, 
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        state.history.push(trimmed.to_string());

        let args = match parser::split_line(trimmed) {
            Ok(args) => args,
            Err(e) => {
                eprintln!("rsh: {}", e);
                continue;
            }
        };

        match execute(&args, &builtins, &mut state) {
            ControlFlow::Continue => continue,
            ControlFlow::Exit(code) => std::process::exit(code),
        }
    }
}

/// Read one line from stdin. `None` means EOF was hit (Ctrl-D), which is
/// `lsh_read_line()`.
fn read_line() -> Option<String> {
    let mut buf = String::new();
    match io::stdin().read_line(&mut buf) {
        Ok(0) => None,       // 0 bytes read = EOF
        Ok(_) => Some(buf),  // buf still has the trailing '\n'; we trim it above
        Err(e) => {
            eprintln!("rsh: error reading input: {}", e);
            None
        }
    }
}


fn execute(
    args: &[String],
    builtins: &std::collections::HashMap<&'static str, builtins::BuiltinFn>,
    state: &mut ShellState,
) -> ControlFlow {
    if let Some(cmd) = args.first() {
        if let Some(func) = builtins.get(cmd.as_str()) {
            return func(args, state);
        }
    }
    launch(args)
}

fn launch(args: &[String]) -> ControlFlow {
    let (program, rest) = match args.split_first() {
        Some(pair) => pair,
        None => return ControlFlow::Continue, // empty command, same guard as lsh_execute()
    };

    match Command::new(program).args(rest).status() {
        Ok(_status) => {
       
        }
        Err(e) => {
            eprintln!("rsh: {}: {}", program, e);
        }
    }

    ControlFlow::Continue
}
