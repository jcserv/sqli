# sqli

sqli is a sleek SQL client, used as a terminal UI or as a command line tool, to help you query your databases!

inspired by tools like [curl](https://github.com/curl/curl), [posting](https://github.com/darrenburns/posting), and [bruno](https://github.com/usebruno/bruno).

## features 🚀

- 🧪 simple syntax for ad-hoc queries 
- 📁 collections are stored in your local file system
  - repo-level collections are stored in `./sqli` - add these to your source control to share with others!
  - user-level settings & collections are stored in `<CONFIG_DIR>/sqli/collections`
- 🦀 written in rust btw 😎  

## installation 📦

### cargo

`cargo binstall sqli` ([cargo-binstall](https://github.com/cargo-bins/cargo-binstall?tab=readme-ov-file#installation))

or

`cargo install sqli`

## usage ⚙️ 

### tui 🖥️

1. `sqli` - open the TUI

### cli ▶️

1. ad-hoc queries: `sqli query --url postgres://user:password@host:port/database --sql "SELECT * FROM table;"`
2. configure a connection: `sqli config set --name local --url postgres://user:password@host:port/database`

<!-- 
1. `sqli config set --name local --url postgres://user:password@host:port/database` - add a new connection
2. `sqli query --connection local --sql "SELECT * FROM book;"` - use a pre-configured connection
3. `sqli query --connection local --file path/to/file.sql` - execute a sql query from a file 
-->

## references 📚

- [curl](https://github.com/curl/curl)
- [posting](https://github.com/darrenburns/posting)
- [bruno](https://github.com/usebruno/bruno)

<!-- gitingest . -e /target/ -e /LICENSE -e /Cargo.lock -e /digest.txt -->