# rsh

A small Unix-style shell written in Rust.

## Build, run, and test

```sh
cargo build
cargo run
cargo test
```

## Features

- Interactive REPL prompt (`>`) with ANSI colors and command history in shell state
- Builtin-command dispatch through a name → function map
- Runs external programs with `std::process::Command`
- Basic pipelines with quote-aware parsing (`cmd1 | cmd2`)
- Argument parsing with support for:
  - quoted strings (`"hi there"`)
  - escaped characters (`\ `, `\"`, etc.)
- Shared shell state (`ShellState`) for history and aliases

## Builtin commands

Current builtins implemented in `src/builtins.rs`:

- `help` : print colorized help and list builtin names
- `exit` : exit the shell
- `pwd` : print current working directory
- `cd [path]` : change current working directory (`cd` / `cd ~` uses home)
- `history` : print command history with line numbers
- `echo [args...]` : print arguments joined by spaces
- `alias` : list aliases, set aliases (`name=value`), or show one alias
- `list [path]` : list directory entries (defaults to current directory, directories in blue)
- `clear` : clear terminal screen

> Alias expansion is supported for command dispatch (e.g. `alias ll=list`, then `ll`).
> Pipeline stages also expand aliases, but builtins are currently rejected inside pipelines.


## Quick demo

After `cargo run`, try:

```text
help
echo hello from rsh
list
alias ll=list
alias
history
clear
exit
```



## Project layout

- `src/main.rs` : top-level shell loop and command routing
- `src/repl.rs` : prompt rendering and input reading
- `src/executor.rs` : command execution, alias expansion, pipeline stage preparation
- `src/pipeline.rs` : pipeline parsing and process piping
- `src/parser.rs` : tokenization with quotes/escapes
- `src/builtins.rs` : builtin implementations and shell state

## Adding a new builtin

1. Add a function in `src/builtins.rs` with this signature:

   ```rust
   fn builtin_name(args: &[String], state: &mut ShellState) -> ControlFlow {
       // implementation
       ControlFlow::Continue
   }
   ```

2. Register it in `register()`:

   ```rust
   map.insert("name", builtin_name);
   ```

`help` and builtin dispatch will pick it up automatically.

## Troubleshooting (Windows)

If `cargo run` fails with:

```text
failed to remove file ... target\debug\rsh.exe
Access is denied. (os error 5)
```

another running `rsh.exe` is still locking the file. Close active shell sessions or run:

```sh
taskkill /F /IM rsh.exe
cargo run
```

## Roadmap

1. Environment variable expansion (`$NAME`) in `parser::split_line`
2. I/O redirection (`>`, `<`) in launch/execution path
3. Builtin support inside pipelines (where practical)
4. Globbing (`*.rs`) before command execution
5. Persistent aliases/history across sessions
