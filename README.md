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
* Handle invalid directory paths gracefully.

### Example

<img width="1535" height="810" alt="Screenshot 2026-09-05 003630" src="https://github.com/user-attachments/assets/4078a7f3-1484-4075-9bd7-054a2c53df11" />


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

* [ ] Colored output
* [ ] Spinners
* [ ] Progress bars
* [ ] Interactive prompts
* [ ] Selection menus
* [ ] Confirmation prompts
* [ ] Clean success/error states
* [ ] ASCII/logo branding

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
