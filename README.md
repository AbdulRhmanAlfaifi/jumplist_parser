# Jumplist Parser
<div align="center">
  <a href="https://crates.io/crates/lnk_parser">
    <img src="https://img.shields.io/crates/v/lnk_parser" alt="Crates.io">
  </a>
  <a href="https://docs.rs/lnk_parser">
    <img src="https://docs.rs/lnk_parser/badge.svg" alt="Docs.rs">
  </a>
  <a href="#license">
    <img src="https://img.shields.io/crates/l/lnk_parser" alt="License">
  </a>
</div>

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

Jumplists are extremely useful in **incident response**, **timeline analysis**, and **user activity reconstruction**. If you want to know more about this artifact, I wrote a blog post about its structure here: [u0041.co](https://u0041.co/posts/articals/jumplist-files-artifacts/)

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
* [My blog post for Jumplist artifact - u0041.co](https://u0041.co/posts/articals/jumplist-files-artifacts/)
* [LNK Strcuture](https://u0041.co/posts/articals/lnk-files-artifact/)
* [LNK Parser](https://github.com/AbdulRhmanAlfaifi/lnk_parser)
