# todo

## cli
- [ ] configuring user-level collections
- [ ] query: when using a sql file with parameters, prompt for values

## known issues
- [ ] buttons don't highlight on hover
- [ ] resizing upwards is wonky

## nice to have

features:
- [ ] edit: change scope
- [ ] results pane: pagination
- [ ] view database schema in tui
- [ ] autocomplete based on tables within a database
- [ ] syntax highlighting
- [ ] sql formatting
- [ ] find/replace
- [ ] query history
- [ ] keybind/theme configuration
- [ ] server_ca/client_cert/client_key
- [ ] connection options

ui/ux improvements:
- [ ] scroll handlers
- [ ] copy button/keybind
- [ ] click to move cursor in workspace

## done!

- [X] allow configs to be referenced by name case insensitively
- [X] new file modal content is being intersected by workspace content
- [X] default workspace folder should be .sqli, instead of sqli/ 
- [X] should init with config.yaml if doesn't exist
- [X] tui crashes when the query result contains a large number of columns (repro'd on a table with 57 columns...)
