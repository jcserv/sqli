# todo

## tui
- [ ] collections
- [ ] command palette and keyboard navigation
- [ ] autocomplete based on tables within a database
- [ ] syntax highlighting
- [ ] sql formatting
- [ ] find/replace

## cli
- [ ] collections

## bugs

## nice to have
- [ ] allow configs to be referenced by name case insensitively

## refactors
- [ ] file manager


## random notes
- collections are a group of sql files, represented by a directory
- collections can be run against any connection
    two scopes:
        - user (~/.config/sqli/collections/<collection_name>/...)
        - local (cwd or ./sqli/<collection_name>/...)
- connections are stored in the user config folder