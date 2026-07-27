# rsh

A simple Unix-style shell written in Rust.

## Build & run

```
cargo build
cargo run
cargo test          # runs the parser unit tests
```

## Features

- Runs external programs (`fork`/`exec` under the hood via `std::process::Command`)
- Built-in commands: `cd`, `pwd`, `help`, `history`, `exit`
- Quoted and escaped arguments (`echo "hi there"` works correctly)

## Project layout

| File              | Purpose                          |
|-------------------|-----------------------------------|
| `src/main.rs`     | Read-eval loop, command execution |
| `src/parser.rs`   | Tokenizes input into arguments    |
| `src/builtins.rs` | Built-in command implementations  |

## Adding a new builtin

1. Write a function matching this signature in `src/builtins.rs`:

   ```rust
   fn builtin_echo(args: &[String], _state: &mut ShellState) -> ControlFlow {
       println!("{}", args[1..].join(" "));
       ControlFlow::Continue
   }
   ```

2. Register it:

   ```rust
   map.insert("echo", builtin_echo);
   ```

   `help` and dispatch pick it up automatically.

## Roadmap

Rough order of difficulty:

1. **Environment variable expansion** — expand `$NAME` tokens in `parser::split_line`.
2. **I/O redirection** (`cmd > file`, `cmd < file`) — strip redirection tokens in `launch()` and wire up `Command::stdout`/`.stdin`.
3. **Pipelines** (`cmd1 | cmd2`) — split on `|`, connect each `Command`'s stdout to the next one's stdin via `Stdio::piped()`.
4. **Globbing** (`ls *.rs`) — expand wildcards against the filesystem before building `args` (the `glob` crate helps here).
