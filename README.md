# sqli

sqli is a command-line interface sql client that allows you to connect to a database and run queries.

## features 🚀

- simple syntax for ad-hoc queries
- written in rust btw 😎 🦀 

## installation 📦

### cargo

`cargo binstall sqli` ([cargo-binstall](https://github.com/cargo-bins/cargo-binstall?tab=readme-ov-file#installation))

or

`cargo install sqli`

## usage ⚙️ 

### ui 🖥️

1. `sqli` - open the TUI

### cli ▶️

2. `sqli query --url postgres://user:password@host:port/database --sql "SELECT * FROM table;"` - ad-hoc query

<!-- 
1. `sqli config add --name local --url postgres://user:password@host:port/database` - add a new connection
2. `sqli query --conn local --sql "SELECT * FROM book;"` - use a pre-configured connection
3. `sqli query --conn local --file path/to/file.sql` - execute a sql query from a file 
-->

## todo 📆
- collections
  - user/repo-level collections are stored in file system - add these to your source control to share with others! 
- command palette and keyboard navigation
- autocomplete based on tables within a database

## references 📚

- [curl](https://github.com/curl/curl)
- [posting](https://github.com/darrenburns/posting)
- [bruno](https://github.com/usebruno/bruno)

<!-- gitingest . -e /target/ -e /LICENSE -e /Cargo.lock -e /digest.txt -->