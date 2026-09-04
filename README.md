# reorganize
<img width="1535" height="412" alt="Screenshot 2026-09-05 001349" src="https://github.com/user-attachments/assets/8c1ad18b-5a5a-45ce-abd9-3a5e6c33abb1" />

A small Rust CLI tool for organizing files and folders.

> **Status:** Learning project — currently focused on learning Rust fundamentals and filesystem operations.

## What it does

`reorganize` currently allows you to:

* Enter a directory name.
* Resolve the directory from the user's Windows home directory.
* Check whether the directory exists.
* Read the directory contents.
* Print the files found inside it.
* Handle invalid directories without crashing.

### Example

```text
Enter Directory name to organize:
Downloads

Organising your Downloads folder...

0000.jpg
001.jpeg
movie.mp4
...
```

If the directory doesn't exist:

```text
Enter Directory name to organize:
SomethingFake

Directory does not exist.
```

## Technologies

* Rust
* Cargo
* Standard Rust library (`std::io`, `std::env`, `std::fs`)

## Running the project

Clone the repository and enter the project directory:

```bash
cd reorganize
```

Run the application:

```bash
cargo run
```

## Current learning goals

This project is being built incrementally while learning Rust.

Current concepts covered:

* Variables with `let`
* Mutable variables with `mut`
* `String` and `&str`
* Terminal input with `stdin`
* `trim()`
* `if / else if / else`
* `for` loops
* Environment variables
* String interpolation with `format!`
* Filesystem operations with `std::fs`
* `Path`
* `Result`
* Error handling with `match`
* `Ok` and `Err`

## Roadmap

The project will gradually become a real file organizer.

### Planned features include:

Quest 1 — Core Rust
[x] Variables
[x] Input
[x] Conditions
[x] Loops
[x] Strings
[x] Environment variables
[x] Result / match
[x] Filesystem reading

Quest 2 — Reorganize Engine
[x] Detect files/directories
[x] Extract filenames
[x] Extract extensions
[x] Categorize files
[x] Create folders
[x] Move files
[x] Handle failures safely

Quest 3 — CLI UI
[ ] Colored output
[ ] Spinners
[ ] Progress bars
[ ] Interactive prompts
[ ] Selection menus
[ ] Confirmation prompts
[ ] Clean success/error states
[ ] ASCII/logo branding

Quest 4 — AI
[ ] Local LLM integration
[ ] AI-assisted file categorization
[ ] AI-generated organization rules
[ ] Natural-language commands
[ ] Explain why files were categorized


## Why this project?

`reorganize` is intentionally being built step-by-step rather than following a tutorial.

Each feature introduces a new Rust concept while contributing to a useful real-world application.

## License

This project is for learning and experimentation.
