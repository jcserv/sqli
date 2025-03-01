use anyhow::Result;
use insta::assert_snapshot;
use ratatui::{
    backend::TestBackend,
    Terminal,
};
use std::time::Duration;
use sqli::{
    collection::CollectionScope, settings::UserSettings, sql::result::QueryResult, tui::{
        app::App,
        navigation::PaneId,
        ui::UI,
    }
};

mod helpers;
use helpers::TestEnv;

#[test]
fn test_main_ui_layout() -> Result<()> {
    let env = TestEnv::new();
    
    env.create_config(r#"
connections:
  - name: test_db
    conn: postgresql
    host: localhost
    port: 5432
    database: testdb
    user: postgres
"#)?;

    let settings = UserSettings::new(
        env.temp_dir.path().join("sqli"),
        env.temp_dir.path().join("sqli")
    );

    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::with_settings(Some(settings))?;
    let mut ui = UI::new();
    
    assert_eq!(app.query_state.available_connections.len(), 1);
    assert_eq!(app.query_state.available_connections[0], "test_db");
    
    terminal.draw(|frame| ui.render(&mut app, frame))?;
    
    assert_snapshot!(terminal.backend());
    
    Ok(())
}

#[test]
fn test_collections_pane() -> Result<()> {
    let env = TestEnv::new();
    
    env.create_collection("users", &[
        ("list.sql", "SELECT * FROM users;"),
        ("get.sql", "SELECT * FROM users WHERE id = $1;")
    ])?;
    
    env.create_collection("products", &[
        ("list.sql", "SELECT * FROM products;"),
        ("search.sql", "SELECT * FROM products WHERE name LIKE $1;")
    ])?;
    
    let settings = UserSettings::new(
        env.temp_dir.path().join("sqli"),
        env.temp_dir.path().join("sqli")
    );

    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::with_settings(Some(settings))?;
    let mut ui = UI::new();
    
    app.navigation.activate_pane(PaneId::Collections)?;
    
    terminal.draw(|frame| ui.render(&mut app, frame))?;
    
    assert_snapshot!(terminal.backend());
    
    Ok(())
}

#[test]
fn test_workspace_pane_with_content() -> Result<()> {
    let env = TestEnv::new();
    let settings = UserSettings::new(
        env.temp_dir.path().join("sqli"),
        env.temp_dir.path().join("sqli")
    );

    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::with_settings(Some(settings))?;
    let mut ui = UI::new();
    
    app.navigation.activate_pane(PaneId::Workspace)?;
    
    app.ui_state.workspace.insert_str(
        "SELECT u.id, u.name, u.email, COUNT(o.id) as order_count\n\
         FROM users u\n\
         LEFT JOIN orders o ON u.id = o.user_id\n\
         WHERE u.status = 'active'\n\
         GROUP BY u.id, u.name, u.email\n\
         ORDER BY order_count DESC\n\
         LIMIT 10;"
    );
    
    terminal.draw(|frame| ui.render(&mut app, frame))?;
    
    assert_snapshot!(terminal.backend());
    
    Ok(())
}

#[test]
fn test_results_pane_with_data() -> Result<()> {
    let env = TestEnv::new();
    let settings = UserSettings::new(
        env.temp_dir.path().join("sqli"),
        env.temp_dir.path().join("sqli")
    );

    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::with_settings(Some(settings))?;
    let mut ui = UI::new();
    
    app.navigation.activate_pane(PaneId::Results)?;
    
    app.query_state.query_result = QueryResult::new(
        vec!["id".to_string(), "name".to_string(), "email".to_string(), "order_count".to_string()],
        vec![
            vec!["1".to_string(), "John Doe".to_string(), "john@example.com".to_string(), "5".to_string()],
            vec!["2".to_string(), "Jane Smith".to_string(), "jane@example.com".to_string(), "3".to_string()],
            vec!["3".to_string(), "Bob Johnson".to_string(), "bob@example.com".to_string(), "7".to_string()],
            vec!["4".to_string(), "Alice Brown".to_string(), "alice@example.com".to_string(), "2".to_string()],
            vec!["5".to_string(), "Charlie Wilson".to_string(), "charlie@example.com".to_string(), "9".to_string()],
        ],
        Duration::from_millis(42)
    );
    
    terminal.draw(|frame| ui.render(&mut app, frame))?;
    
    assert_snapshot!(terminal.backend());
    
    Ok(())
}

#[test]
fn test_modal_dialog() -> Result<()> {
    let env = TestEnv::new();
    let settings = UserSettings::new(
        env.temp_dir.path().join("sqli"),
        env.temp_dir.path().join("sqli")
    );

    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::with_settings(Some(settings))?;
    let mut ui = UI::new();
    
    app.modal_manager.show_modal(sqli::tui::modal::ModalType::Password);
    
    terminal.draw(|frame| ui.render(&mut app, frame))?;
    
    assert_snapshot!(terminal.backend());
    
    Ok(())
}

#[test]
fn test_new_file_modal() -> Result<()> {
    let env = TestEnv::new();
    let settings = UserSettings::new(
        env.temp_dir.path().join("sqli"),
        env.temp_dir.path().join("sqli")
    );

    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::with_settings(Some(settings))?;
    let mut ui = UI::new();
    
    app.modal_manager.show_modal(sqli::tui::modal::ModalType::NewFile { 
        parent_folder: Some("users".to_string()) 
    });
    
    terminal.draw(|frame| ui.render(&mut app, frame))?;
    
    assert_snapshot!(terminal.backend());
    
    Ok(())
}

#[test]
fn test_edit_file_modal() -> Result<()> {
    let env = TestEnv::new();
    
    env.create_collection("users", &[
        ("list.sql", "SELECT * FROM users;"),
    ])?;
    
    let settings = UserSettings::new(
        env.temp_dir.path().join("sqli"),
        env.temp_dir.path().join("sqli")
    );

    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::with_settings(Some(settings))?;
    let mut ui = UI::new();
    
    app.modal_manager.show_modal(sqli::tui::modal::ModalType::EditFile { 
        name: "list.sql".to_string(),
        is_folder: false,
        current_scope: CollectionScope::Cwd,
    });
    
    terminal.draw(|frame| ui.render(&mut app, frame))?;
    
    assert_snapshot!(terminal.backend());
    
    Ok(())
}

#[test]
fn test_header_pane_with_connection() -> Result<()> {
    let env = TestEnv::new();
    
    env.create_config(r#"
connections:
  - name: local_dev
    conn: postgresql
    host: localhost
    port: 5432
    database: dev_db
    user: postgres
  - name: staging
    conn: postgresql
    host: staging-db.example.com
    port: 5432
    database: staging_db
    user: app_user
  - name: production
    conn: postgresql
    host: prod-db.example.com
    port: 5432
    database: prod_db
    user: app_user
"#)?;
    
    let settings = UserSettings::new(
        env.temp_dir.path().join("sqli"),
        env.temp_dir.path().join("sqli")
    );

    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::with_settings(Some(settings))?;
    let mut ui = UI::new();
    
    app.navigation.activate_pane(PaneId::Header)?;
    
    terminal.draw(|frame| ui.render(&mut app, frame))?;
    
    assert_snapshot!(terminal.backend());
    
    app.next_connection();
    
    terminal.draw(|frame| ui.render(&mut app, frame))?;
    
    assert_snapshot!("header_pane_second_connection", terminal.backend());
    
    Ok(())
}

#[test]
fn test_editing_mode() -> Result<()> {
    let env = TestEnv::new();
    let settings = UserSettings::new(
        env.temp_dir.path().join("sqli"),
        env.temp_dir.path().join("sqli")
    );

    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::with_settings(Some(settings))?;
    let mut ui = UI::new();
    
    app.navigation.activate_pane(PaneId::Workspace)?;
    
    app.navigation.start_editing(PaneId::Workspace)?;
    
    app.ui_state.workspace.insert_str("SELECT * FROM users WHERE active = true;");
    
    terminal.draw(|frame| ui.render(&mut app, frame))?;
    
    assert_snapshot!(terminal.backend());
    
    Ok(())
}