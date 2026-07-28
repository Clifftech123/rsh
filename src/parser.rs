
pub fn split_line(line: &str) -> Result<Vec<String>, String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_token = false;

    let mut chars = line.chars().peekable();
    let mut quote: Option<char> = None;

    while let Some(c) = chars.next() {
        match quote {
            Some(q) => {
                // We are inside a quoted section; only the matching quote closes it.
                if c == q {
                    quote = None;
                } else {
                    current.push(c);
                }
                in_token = true;
            }
            None => match c {
                '\'' | '"' => {
                    quote = Some(c);
                    in_token = true;
                }
                '\\' => {
                    // Backslash escapes the next character, if any.
                    if let Some(next) = chars.next() {
                        current.push(next);
                        in_token = true;
                    }
                }
                c if c.is_whitespace() => {
                    if in_token {
                        tokens.push(std::mem::take(&mut current));
                        in_token = false;
                    }
                }
                c => {
                    current.push(c);
                    in_token = true;
                }
            },
        }
    }

    if quote.is_some() {
        return Err("unmatched quote".to_string());
    }

    if in_token {
        tokens.push(current);
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_on_whitespace() {
        assert_eq!(split_line("ls -la").unwrap(), vec!["ls", "-la"]);
    }

    #[test]
    fn handles_double_quotes() {
        assert_eq!(
            split_line("echo \"hi there\"").unwrap(),
            vec!["echo", "hi there"]
        );
    }

    #[test]
    fn handles_empty_line() {
        assert_eq!(split_line("   ").unwrap(), Vec::<String>::new());
    }

    #[test]
    fn rejects_unterminated_quote() {
        assert!(split_line("echo \"oops").is_err());
    }
}
