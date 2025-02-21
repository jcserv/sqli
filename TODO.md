# todo

## tui
- [ ] collections
  - [ ] allow clicking to select items in tree 
- [ ] execute query - button & keybind
- [ ] creating a sql file
- [ ] view database schema
- [ ] query: when using a sql file with parameters, prompt for values
- [ ] results pane
  - [ ] add table widget for viewing rows
  - [ ] pagination
  - [ ] display number of rows, time taken 

- [ ] command palette and keyboard navigation
- [ ] autocomplete based on tables within a database
- [ ] syntax highlighting
- [ ] sql formatting
- [ ] find/replace
- [ ] keybind configuration

widgets:
- tabs
- button
- select

## cli
- [ ] collections
- [ ] query: when using a sql file with parameters, prompt for values
- [ ] server_ca/client_cert/client_key

## bugs

## refactors
- [ ] file manager

## nice to have
- [ ] allow configs to be referenced by name case insensitively
- [ ] display user-provided name for sql file
- [ ] query history

## random notes
- collections are a group of sql files, represented by a directory
- collections can be run against any connection
    two scopes:
        - user (~/.config/sqli/collections/<collection_name>/...)
        - local (cwd or ./sqli/<collection_name>/...)
- connections are stored in the user config folder