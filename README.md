# reorganize

<img width="957" height="282" alt="Screenshot 2026-09-05 003618" src="https://github.com/user-attachments/assets/42cd15f3-725f-4ac2-87ef-f3caf3f44d81" />

A small Rust CLI tool for organizing files and folders.

> **Status:** Learning project — currently focused on learning Rust fundamentals, filesystem operations, and CLI development.

## What it does

`reorganize` currently allows you to:

* Enter a directory name.
* Resolve the directory from the user's Windows home directory.
* Check whether the directory exists.
* Read the directory contents.
* Detect files and directories.
* Extract file extensions.
* Categorize files.
* Create folders based on file categories.
* Move files into their appropriate folders.
* Organize a custom folder by typing its full path.
* Rename rather than overwrite when a file of the same name already exists.
* Report per-file failures without aborting the rest of the run.
* Handle invalid directory paths gracefully.

### Example

<img width="1535" height="811" alt="Screenshot 2026-09-05 004920" src="https://github.com/user-attachments/assets/07c1b901-7306-4479-988e-be62ca0002af" />


## Technologies

* Rust
* Cargo
* Standard Rust library (`std::io`, `std::env`, `std::fs`, `std::path`)
* [`colored`](https://crates.io/crates/colored) — terminal colors
* [`indicatif`](https://crates.io/crates/indicatif) — spinners and progress bars
* [`inquire`](https://crates.io/crates/inquire) — interactive prompts

## Running the project

Clone the repository and enter the project directory:

```bash
cd reorganize
```

Run the application:

```bash
cargo run
```

Run the test suite:

```bash
cargo test
```

## Current Learning Goals

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
* String formatting with `format!`
* Filesystem operations with `std::fs`
* `Path`
* `Result`
* Error handling with `match`
* `Ok` and `Err`
* `Option` and `ok_or_else`
* Modules and `pub`
* Error propagation with `?`
* Unit tests with `#[cfg(test)]`

## Roadmap

The project is being developed through several learning quests.

### Quest 1 — Core Rust

* [x] Variables
* [x] Input
* [x] Conditions
* [x] Loops
* [x] Strings
* [x] Environment variables
* [x] `Result` / `match`
* [x] Filesystem reading

### Quest 2 — Reorganize Engine

* [x] Detect files/directories
* [x] Extract filenames
* [x] Extract extensions
* [x] Categorize files
* [x] Create folders
* [x] Move files
* [x] Handle filesystem errors safely

### Quest 3 — CLI UI

* [x] Colored output
* [x] Spinners
* [x] Progress bars
* [x] Interactive prompts
* [x] Selection menus
* [x] Confirmation prompts
* [x] Clean success/error states
* [x] ASCII/logo branding

### Quest 4 — AI

* [ ] Local LLM integration
* [ ] AI-assisted file categorization
* [ ] AI-generated organization rules
* [ ] Natural-language commands
* [ ] Explain why files were categorized

## Why this project?

`reorganize` is intentionally being built step-by-step rather than following a tutorial.

Each feature introduces a new Rust concept while contributing to a useful real-world application.

## Project Status

This is currently a learning and experimentation project. The goal is to gradually turn `reorganize` into a polished, practical file-organizing CLI while continuing to learn Rust.
