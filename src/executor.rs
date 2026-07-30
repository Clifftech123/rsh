use crate::builtins::{BuiltinFn, ControlFlow, ShellState, RED, RESET};
use crate::parser;
use std::collections::{HashMap, HashSet};
use std::process::Command;

/// Executes a single (non-pipeline) command line.
pub fn execute(
    args: &[String],
    builtins: &HashMap<&'static str, BuiltinFn>,
    state: &mut ShellState,
) -> ControlFlow {
    let expanded_args = match expand_aliases(args, state) {
        Ok(expanded) => expanded,
        Err(e) => {
            eprintln!("{RED}rsh:{RESET} {}", e);
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

/// Prepares parsed pipeline stages for execution.
///
/// This expands aliases per stage and rejects builtins in pipelines.
pub fn prepare_pipeline_stages(
    stages: Vec<Vec<String>>,
    builtins: &HashMap<&'static str, BuiltinFn>,
    state: &ShellState,
) -> Result<Vec<Vec<String>>, String> {
    let mut expanded_stages = Vec::with_capacity(stages.len());

    for stage in stages {
        let expanded = expand_aliases(&stage, state)?;
        let Some(cmd) = expanded.first() else {
            return Err("empty command in pipeline".to_string());
        };

        if builtins.contains_key(cmd.as_str()) {
            return Err(format!("builtin '{}' cannot be used in a pipeline", cmd));
        }

        expanded_stages.push(expanded);
    }

    Ok(expanded_stages)
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

    while let Some(cmd) = expanded.first().cloned() {
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
        None => return ControlFlow::Continue,
    };

    match Command::new(program).args(rest).status() {
        Ok(_status) => {}
        Err(e) => {
            eprintln!("{RED}rsh:{RESET} {}: {}", program, e);
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

    #[test]
    fn expands_aliases_in_pipeline_stage() {
        let builtins = crate::builtins::register();
        let mut state = ShellState::new();
        state
            .aliases
            .insert("grep_r".to_string(), "findstr r".to_string());

        let stages = vec![vec!["grep_r".to_string(), "main".to_string()]];
        let prepared = prepare_pipeline_stages(stages, &builtins, &state).unwrap();

        assert_eq!(prepared, vec![vec!["findstr", "r", "main"]]);
    }

    #[test]
    fn rejects_builtins_in_pipeline_even_via_alias() {
        let builtins = crate::builtins::register();
        let mut state = ShellState::new();
        state.aliases.insert("ll".to_string(), "list".to_string());

        let stages = vec![vec!["ll".to_string()]];
        let err = prepare_pipeline_stages(stages, &builtins, &state).unwrap_err();

        assert!(err.contains("cannot be used in a pipeline"));
    }
}
