# sqli

![downloads](https://img.shields.io/crates/d/sqli) [![code coverage](https://coveralls.io/repos/github/jcserv/sqli/badge.svg?branch=main)](https://coveralls.io/github/jcserv/sqli?branch=main)

sqli (as in, "sql" + "cli" = "sqli") is a simple & sleek SQL client, used as a terminal UI or as a command line tool, to help you query your Postgres database!

heavily inspired by tools like [posting](https://github.com/darrenburns/posting), [curl](https://github.com/curl/curl), and [bruno](https://github.com/usebruno/bruno).

## features 🚀

- 📊 view query results directly in the terminal
  - or pipe it into other tools like `jq`
- 🧪 simple syntax for ad-hoc queries from the terminal
- 🔄 save and reuse database connections
- 📁 collections are stored in your local file system
  - repo-level collections are stored in `./sqli` - add these to your source control to share with others!
  - user-level settings & collections are stored in `<CONFIG_DIR>/sqli`
- 🦀 written in rust btw 😎

![Product demo](https://imgur.com/ff3hcNB.gif)

## installation 📦

### homebrew

`brew tap jcserv/cask`

`brew install sqli`

### cargo

`cargo binstall sqli` ([cargo-binstall](https://github.com/cargo-bins/cargo-binstall?tab=readme-ov-file#installation))

or

`cargo install sqli`

## usage ⚙️ 

### tui 🖥️

1. `sqli` - open the TUI

Keybindings:

| Key          | Action                     |
|--------------|----------------------------|
| Tab          | Switch between panels (when in nav mode)      |
| Arrow keys   | Switch between panels (when in nav mode)     |
| Space/Enter  | Focus on a pane                |
| Ctrl+N       | Create new file/folder     |
| Ctrl+E       | Edit selected file/folder  |
| Ctrl+S       | Save current file          |
| Ctrl+Space   | Run SQL query              |
| Esc          | Exit edit mode             |
| Ctrl+C       | Quit application           |

### cli ▶️

1. ad-hoc queries:
- `sqli query --url postgres://user:password@host:port/database --sql "SELECT * FROM table;"`
2. configure a connection:
  - `sqli config set --name local --url postgres://user:password@host:port/database`
3. query using a pre-configured connection:
  - `sqli query --conn local --sql "SELECT * FROM table;"`
4. query using a file:
  - `sqli query --conn local --sql path/to/file.sql`

## references 📚

- [posting](https://github.com/darrenburns/posting)
- [curl](https://github.com/curl/curl)
- [bruno](https://github.com/usebruno/bruno)
