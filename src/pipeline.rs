use crate::builtins::{RED, RESET};
use std::process::{Child, Command, Stdio};

pub fn parse_pipeline(line: &str) -> Result<Vec<Vec<String>>, String> {
    let stage_texts = split_pipeline_stages(line)?;
    stage_texts
        .into_iter()
        .map(|stage| crate::parser::split_line(stage.trim()))
        .collect()
}

fn split_pipeline_stages(line: &str) -> Result<Vec<String>, String> {
    let mut stages = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;

    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match quote {
            Some(q) => {
                current.push(c);
                if c == q {
                    quote = None;
                }
            }
            None => match c {
                '\'' | '"' => {
                    quote = Some(c);
                    current.push(c);
                }
                '\\' => {
                    current.push(c);
                    if let Some(next) = chars.next() {
                        current.push(next);
                    }
                }
                '|' => {
                    let stage = current.trim();
                    if stage.is_empty() {
                        return Err("empty command in pipeline".to_string());
                    }
                    stages.push(stage.to_string());
                    current.clear();
                }
                _ => current.push(c),
            },
        }
    }

    if quote.is_some() {
        return Err("unmatched quote".to_string());
    }

    let stage = current.trim();
    if stage.is_empty() {
        return Err("empty command in pipeline".to_string());
    }
    stages.push(stage.to_string());

    Ok(stages)
}

pub fn run_pipeline(stages: &[Vec<String>]) {
    if stages.is_empty() || stages.iter().any(|s| s.is_empty()) {
        eprintln!("{RED}rsh:{RESET} empty command in pipeline");
        return;
    }

    let mut children: Vec<Child> = Vec::with_capacity(stages.len());
    // Holds the previous stage's stdout, ready to become this stage's stdin.
    let mut prev_stdout: Option<Stdio> = None;

    for (i, args) in stages.iter().enumerate() {
        let (program, rest) = args.split_first().unwrap();
        let is_last = i == stages.len() - 1;

        let mut cmd = Command::new(program);
        cmd.args(rest);

        if let Some(stdin_source) = prev_stdout.take() {
            cmd.stdin(stdin_source);
        }
        if !is_last {
            // Capture this stage's stdout so the next stage can read it.
            cmd.stdout(Stdio::piped());
        }

        match cmd.spawn() {
            Ok(mut child) => {
                // Hand this child's stdout to the next loop iteration.
                prev_stdout = child.stdout.take().map(Stdio::from);
                children.push(child);
            }
            Err(e) => {
                eprintln!("{RED}rsh:{RESET} {}: {}", program, e);
                break; // don't run later stages against a broken pipeline
            }
        }
    }

    for mut child in children {
        let _ = child.wait();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_stages_on_pipe() {
        let stages = parse_pipeline("echo hi | wc -l").unwrap();
        assert_eq!(stages, vec![vec!["echo", "hi"], vec!["wc", "-l"]]);
    }

    #[test]
    fn keeps_pipe_inside_quotes() {
        let stages = parse_pipeline("echo \"a|b\" | wc -c").unwrap();
        assert_eq!(stages, vec![vec!["echo", "a|b"], vec!["wc", "-c"]]);
    }

    #[test]
    fn keeps_escaped_pipe_inside_stage() {
        let stages = parse_pipeline("echo a\\|b | wc -c").unwrap();
        assert_eq!(stages, vec![vec!["echo", "a|b"], vec!["wc", "-c"]]);
    }

    #[test]
    fn rejects_empty_stage() {
        assert!(parse_pipeline("echo hi || wc -l").is_err());
        assert!(parse_pipeline("| wc -l").is_err());
        assert!(parse_pipeline("echo hi |").is_err());
    }

    #[test]
    fn single_stage_is_one_element() {
        let stages = parse_pipeline("ls -la").unwrap();
        assert_eq!(stages, vec![vec!["ls", "-la"]]);
    }
}
