use std::fs;

use assert_cmd::Command as AssertCommand;
use predicates::prelude::*;

mod helpers;
use helpers::{contains_any_of, TestEnv};

// #[test]
// fn test_config_list_command() {
//     let env = TestEnv::new();
    
//     env.create_config(r#"
// connections:
//   - name: local
//     conn: postgresql
//     host: localhost
//     port: 5432
//     database: testdb
//     user: postgres
// "#).unwrap();

//     AssertCommand::cargo_bin("sqli")
//         .unwrap()
//         .arg("config")
//         .arg("list")
//         .current_dir(&env.temp_dir)
//         .assert()
//         .success()
//         .stdout(predicate::str::contains("local"));
// }

#[test]
fn test_missing_connection() {
    let env = TestEnv::new();
    
    AssertCommand::cargo_bin("sqli")
        .unwrap()
        .arg("query")
        .arg("--conn")
        .arg("nonexistent")
        .arg("--sql")
        .arg("SELECT 1;")
        .current_dir(&env.temp_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Connection 'nonexistent' not found"));
}

#[test]
fn test_with_collections() {
    let env = TestEnv::new();
    
    env.create_collection("testDb", &[
        ("query1.sql", "SELECT * FROM users;"),
        ("query2.sql", "SELECT * FROM orders;")
    ]).unwrap();
    
    AssertCommand::cargo_bin("sqli")
        .unwrap()
        .arg("query")
        .arg("--sql")
        .arg("sqli/testDb/query1.sql")
        .current_dir(&env.temp_dir)
        .assert()
        .failure() 
        .stderr(predicate::str::contains("Either --url or --connection must be provided"));
}

#[test]
fn test_invalid_sql() {
    let env = TestEnv::new();
    
    AssertCommand::cargo_bin("sqli")
        .unwrap()
        .arg("query")
        .arg("--url")
        .arg("postgres://user:password@localhost:5432/nonexistent")
        .arg("--sql")
        .arg("INVALID SQL")
        .current_dir(&env.temp_dir)
        .assert()
        .failure();
}

#[test]
fn test_query_with_parameters() {
    let env = TestEnv::new();
    
    fs::create_dir_all(env.temp_dir.path().join("sqli")).unwrap();
    
    env.create_config(r#"
connections:
  - name: test_conn
    conn: postgresql
    host: localhost
    port: 5432
    database: testdb
    user: postgres
"#).unwrap();

    env.create_sql_file("query_with_params.sql", "SELECT * FROM users WHERE id = $1;").unwrap();

    let result = AssertCommand::cargo_bin("sqli")
        .unwrap()
        .arg("query")
        .arg("--conn")
        .arg("test_conn")
        .arg("--sql")
        .arg("query_with_params.sql")
        .current_dir(&env.temp_dir)
        .assert()
        .failure();

    let stderr_str = String::from_utf8(result.get_output().stderr.clone()).unwrap();
    assert!(
        contains_any_of(&stderr_str, &["Connection", "connection", "not found"]),
        "Error message '{}' doesn't contain expected text",
        stderr_str
    );
}

#[test]
fn test_multiple_collections() {
    let env = TestEnv::new();
    
    env.create_collection("users", &[
        ("list.sql", "SELECT * FROM users;"),
        ("get.sql", "SELECT * FROM users WHERE id = $1;")
    ]).unwrap();
    
    env.create_collection("products", &[
        ("list.sql", "SELECT * FROM products;"),
        ("get.sql", "SELECT * FROM products WHERE id = $1;")
    ]).unwrap();
    
    AssertCommand::cargo_bin("sqli")
        .unwrap()
        .arg("query")
        .arg("--sql")
        .arg("sqli/users/list.sql")
        .current_dir(&env.temp_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Either --url or --connection must be provided"));
    
    AssertCommand::cargo_bin("sqli")
        .unwrap()
        .arg("query")
        .arg("--sql")
        .arg("sqli/products/get.sql")
        .current_dir(&env.temp_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Either --url or --connection must be provided"));
}

#[test]
fn test_query_nonexistent_file() {
    let env = TestEnv::new();
    
    let result = AssertCommand::cargo_bin("sqli")
        .unwrap()
        .arg("query")
        .arg("--url")
        .arg("postgres://invalid")
        .arg("--sql")
        .arg("nonexistent.sql")
        .current_dir(&env.temp_dir)
        .assert()
        .failure();
        
    let stderr_str = String::from_utf8(result.get_output().stderr.clone()).unwrap();
    assert!(
        contains_any_of(&stderr_str, &["No such file", "nonexistent.sql", "not found", "invalid"]),
        "Error message '{}' doesn't contain expected text",
        stderr_str
    );
}

#[test]
fn test_nested_collections() {
    let env = TestEnv::new();
    
    let nested_path = env.temp_dir.path().join("sqli").join("database").join("schemas").join("public");
    fs::create_dir_all(&nested_path).unwrap();
    
    let tables_sql = nested_path.join("tables.sql");
    fs::write(&tables_sql, "SELECT * FROM information_schema.tables;").unwrap();
    
    AssertCommand::cargo_bin("sqli")
        .unwrap()
        .arg("query")
        .arg("--sql")
        .arg("sqli/database/schemas/public/tables.sql")
        .current_dir(&env.temp_dir)
        .assert()
        .failure() 
        .stderr(predicate::str::contains("Either --url or --connection must be provided"));
}

#[test]
fn test_complex_sql_file() {
    let env = TestEnv::new();
    
    let complex_sql = r#"
-- Get users with their orders
SELECT 
    u.id AS user_id,
    u.name,
    u.email,
    o.id AS order_id,
    o.amount,
    o.created_at
FROM 
    users u
    LEFT JOIN orders o ON u.id = o.user_id
WHERE 
    u.is_active = true
    AND (o.created_at >= NOW() - INTERVAL '30 days' OR o.created_at IS NULL)
ORDER BY
    u.name ASC,
    o.created_at DESC;
"#;

    env.create_sql_file("complex_query.sql", complex_sql).unwrap();
    
    let result = AssertCommand::cargo_bin("sqli")
        .unwrap()
        .arg("query")
        .arg("--url")
        .arg("postgres://invalid")
        .arg("--sql")
        .arg("complex_query.sql")
        .current_dir(&env.temp_dir)
        .assert()
        .failure(); 
        
    let stderr_str = String::from_utf8(result.get_output().stderr.clone()).unwrap();
    assert!(
        contains_any_of(&stderr_str, &["connect", "Connect", "postgres", "PostgreSQL", "invalid"]),
        "Error message '{}' doesn't contain expected text",
        stderr_str
    );
}

#[test]
fn test_relative_paths() {
    let env = TestEnv::new();
    
    env.create_sql_file("query1.sql", "SELECT 1;").unwrap();
    
    let subdir = env.temp_dir.path().join("subdir");
    fs::create_dir_all(&subdir).unwrap();
    fs::write(subdir.join("query2.sql"), "SELECT 2;").unwrap();
    
    let result1 = AssertCommand::cargo_bin("sqli")
        .unwrap()
        .arg("query")
        .arg("--url")
        .arg("postgres://invalid")
        .arg("--sql")
        .arg("query1.sql")
        .current_dir(&env.temp_dir)
        .assert()
        .failure();
        
    let stderr1 = String::from_utf8(result1.get_output().stderr.clone()).unwrap();
    assert!(
        contains_any_of(&stderr1, &["connect", "invalid", "postgres"]),
        "Error message '{}' doesn't contain expected text",
        stderr1
    );
        
    let result2 = AssertCommand::cargo_bin("sqli")
        .unwrap()
        .arg("query")
        .arg("--url")
        .arg("postgres://invalid")
        .arg("--sql")
        .arg("subdir/query2.sql")
        .current_dir(&env.temp_dir)
        .assert()
        .failure();
        
    let stderr2 = String::from_utf8(result2.get_output().stderr.clone()).unwrap();
    assert!(
        contains_any_of(&stderr2, &["connect", "invalid", "postgres"]),
        "Error message '{}' doesn't contain expected text",
        stderr2
    );
}