use crate::builtins::{CYAN, RED, RESET};
use std::io::{self, Write};

/// Prints the prompt and reads one line from stdin.
///
/// Returns `None` on EOF or input error.
pub fn prompt_and_read_line() -> Option<String> {
    print_prompt();

    let mut buf = String::new();
    match io::stdin().read_line(&mut buf) {
        Ok(0) => None,
        Ok(_) => Some(buf),
        Err(e) => {
            eprintln!("{RED}rsh:{RESET} error reading input: {}", e);
            None
        }
    }
}

fn print_prompt() {
    print!("{CYAN}> {RESET}");
    // Make sure the prompt appears before blocking on input.
    io::stdout().flush().ok();
}
