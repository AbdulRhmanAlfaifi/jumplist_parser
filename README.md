# Jumplist Parser

[![Crates.io](https://img.shields.io/crates/v/jumplist_parser)](https://crates.io/crates/jumplist_parser)
[![Docs.rs](https://docs.rs/jumplist_parser/badge.svg)](https://docs.rs/jumplist_parser)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/jumplist_parser)](#license)

This repository is a Rust library and CLI tool for parsing **Windows Jumplist artifacts**

## 🔍 What Are Jumplists?

Windows Jumplists are Windows artifacts that provides quick access to recently or frequently used files per application. They are stored under:

```
%APPDATA%\Microsoft\Windows\Recent\AutomaticDestinations\*
%APPDATA%\Microsoft\Windows\Recent\CustomDestinations\*
````

These files contain structured metadata such as:

- File paths and names
- Timestamps (last accessed, modified)
- Hostname where files were opened
- LNK metadata (command-line arguments, working directory, etc)
- Pinned items
- And many more!

Jumplists are extremely useful in **incident response**, **timeline analysis**, and **user activity reconstruction**. If you want to know more about this artifact, I wrote a blog post about its structure here: https://u0041.co

## 📦 Installation

Install the commandline tool using `cargo`:

```bash
cargo install jumplist_parser
````

Once installed, you can run the binary to see available arguments:

```bash
jumplist_parser --help
```

```bash
Created By: AbdulRhman Alfaifi <@A__ALFAIFI>
Version: v0.1.0
Reference: https://u0041.co/posts/articals/jumplist-files-artifacts/

Windows Jumplist Files Parser

Usage: jumplist_parser [OPTIONS]

Options:
  -p, --path <PATH>                    Path(s) to Jumplist files to be parsed - accepts glob (defaults to 'AutomaticDestinations' & 'CustomDestinations' for all users)
  -o, --output <FILE>                  The file path to write the output to [default: stdout]
      --output-format <output-format>  Output format [default: csv] [possible values: csv, jsonl, json]
      --no-headers                     Don't print headers when using CSV as the output format
      --normalize                      Normalize the result to the most important fields
  -h, --help                           Print help
  -V, --version                        Print version
```

Or you can download the latest version from the [release section](https://github.com/AbdulRhmanAlfaifi/jumplist_parser/releases/latest)

## 🧪 Using the Library

### 1️⃣ Add to `Cargo.toml`

```toml
[dependencies]
jumplist_parser = "0.1.0"
```

### 2️⃣ Parse a Jumplist File

```rust
use jumplist_parser::JumplistParser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parser = JumplistParser::from_path("samples/win11/AutomaticDestinations/4cb9c5750d51c07f.automaticDestinations-ms")?;

    println!("App ID: {:?}", parser.app_id);
    println!("{:#?}", parser);

    Ok(())
}
```

## 📝 License

Licensed under either of:

* MIT
* Apache License, Version 2.0

## 📚 References
* https://u0041.co
