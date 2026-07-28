mod builtins;
mod parser;

use builtins::{ControlFlow, ShellState};
use std::collections::{HashMap, HashSet};
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
        Ok(0) => None,      // 0 bytes read = EOF
        Ok(_) => Some(buf), // buf still has the trailing '\n'; we trim it above
        Err(e) => {
            eprintln!("rsh: error reading input: {}", e);
            None
        }
    }
}

fn execute(
    args: &[String],
    builtins: &HashMap<&'static str, builtins::BuiltinFn>,
    state: &mut ShellState,
) -> ControlFlow {
    let expanded_args = match expand_aliases(args, state) {
        Ok(expanded) => expanded,
        Err(e) => {
            eprintln!("rsh: {}", e);
            return ControlFlow::Continue;
        }
    };

    if let Some(cmd) = expanded_args.first() {
        if let Some(func) = builtins.get(cmd.as_str()) {
            return func(&expanded_args, state);
        }
    }

    launch(&expanded_args)
}

/// Expands aliases for the first command token.
///
/// Supports chained aliases (for example: a -> b -> echo hi) and guards
/// against alias loops.
fn expand_aliases(args: &[String], state: &ShellState) -> Result<Vec<String>, String> {
    if args.is_empty() {
        return Ok(Vec::new());
    }

    let mut expanded = args.to_vec();
    let mut seen = HashSet::new();

    loop {
        let Some(cmd) = expanded.first().cloned() else {
            break;
        };

        let Some(alias_value) = state.aliases.get(&cmd) else {
            break;
        };

        if !seen.insert(cmd.clone()) {
            return Err(format!("alias loop detected for '{}'", cmd));
        }

        let mut alias_args = parser::split_line(alias_value)
            .map_err(|e| format!("invalid alias '{}': {}", cmd, e))?;

        if alias_args.is_empty() {
            return Err(format!("alias '{}' expands to an empty command", cmd));
        }

        alias_args.extend(expanded.into_iter().skip(1));
        expanded = alias_args;
    }

    Ok(expanded)
}

fn launch(args: &[String]) -> ControlFlow {
    let (program, rest) = match args.split_first() {
        Some(pair) => pair,
        None => return ControlFlow::Continue, // empty command, same guard as lsh_execute()
    };

    match Command::new(program).args(rest).status() {
        Ok(_status) => {}
        Err(e) => {
            eprintln!("rsh: {}: {}", program, e);
        }
    }

    ControlFlow::Continue
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_single_alias() {
        let mut state = ShellState::new();
        state.aliases.insert("ll".to_string(), "list".to_string());

        let args = vec!["ll".to_string()];
        let expanded = expand_aliases(&args, &state).unwrap();

        assert_eq!(expanded, vec!["list"]);
    }

    #[test]
    fn expands_alias_and_keeps_original_arguments() {
        let mut state = ShellState::new();
        state
            .aliases
            .insert("greet".to_string(), "echo hello".to_string());

        let args = vec!["greet".to_string(), "team".to_string()];
        let expanded = expand_aliases(&args, &state).unwrap();

        assert_eq!(expanded, vec!["echo", "hello", "team"]);
    }

    #[test]
    fn detects_alias_loops() {
        let mut state = ShellState::new();
        state.aliases.insert("a".to_string(), "b".to_string());
        state.aliases.insert("b".to_string(), "a".to_string());

        let args = vec!["a".to_string()];
        let err = expand_aliases(&args, &state).unwrap_err();

        assert!(err.contains("alias loop detected"));
    }
}
