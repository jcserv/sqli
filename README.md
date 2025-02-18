# sqli

sqli (see-quer-li) is a command-line interface sql client that allows you to connect to a database and run queries.

## features 🚀

- simple syntax for ad-hoc queries
- user/repo-level collections are stored in file system - add these to your source control to share with others! 
- command palette and keyboard navigation
- collections
- autocomplete based on tables within a database
- written in rust btw 😎 🦀 

## installation 📦

## usage ⚙️ 

### ui 🖥️

1. `sqli` - open the TUI

### cli ▶️
1. `sqli config add --name local --url postgres://user:password@host:port/database` - add a new profile
2. `sqli query --url postgres://user:password@host:port/database "SELECT * FROM table;"` - ad-hoc query
3. `sqli query --profile local --query "SELECT * FROM table;"` - use a pre-configured profile
4. `sqli query --profile local --file path/to/file.sql` - execute a sql query from a file

## references 📚

### inspo 💡
- [curl](https://github.com/curl/curl)
- [posting](https://github.com/darrenburns/posting)
- [bruno](https://github.com/usebruno/bruno)