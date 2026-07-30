mod builtins;
mod executor;
mod parser;
mod pipeline;
mod repl;

use builtins::{ControlFlow, ShellState, RED, RESET};

fn main() {
    let builtins = builtins::register();
    let mut state = ShellState::new();

    while let Some(line) = repl::prompt_and_read_line() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        state.history.push(trimmed.to_string());

        if trimmed.contains('|') {
            match pipeline::parse_pipeline(trimmed)
                .and_then(|stages| executor::prepare_pipeline_stages(stages, &builtins, &state))
            {
                Ok(stages) => pipeline::run_pipeline(&stages),
                Err(e) => eprintln!("{RED}rsh:{RESET} {}", e),
            }
            continue;
        }

        let args = match parser::split_line(trimmed) {
            Ok(args) => args,
            Err(e) => {
                eprintln!("{RED}rsh:{RESET} {}", e);
                continue;
            }
        };

        match executor::execute(&args, &builtins, &mut state) {
            ControlFlow::Continue => continue,
            ControlFlow::Exit(code) => std::process::exit(code),
        }
    }
}
