use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]

#[derive(Copy, PartialEq)]
#[derive(Default)]
pub enum Format {
    #[default]
    Table,
    Json,
    Csv,
    Raw,
}

impl Format {
    pub fn new(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "table" => Ok(Format::Table),
            "json" => Ok(Format::Json),
            "csv" => Ok(Format::Csv),
            "raw" => Ok(Format::Raw),
            _ => Err(anyhow!("Unsupported format: {}. Supported formats: table, json, csv, raw", s))
        }
    }
}


#[derive(Debug, Clone)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub execution_time: std::time::Duration,
    pub row_count: usize,
}

impl Default for QueryResult {
    fn default() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
            execution_time: std::time::Duration::from_secs(0),
            row_count: 0,
        }
    }
}

impl QueryResult {
    pub fn new(columns: Vec<String>, rows: Vec<Vec<String>>, execution_time: std::time::Duration) -> Self {
        let row_count = rows.len();
        Self {
            columns,
            rows,
            execution_time,
            row_count,
        }
    }

    pub fn empty() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
            execution_time: std::time::Duration::from_secs(0),
            row_count: 0,
        }
    }
}

#[derive(Serialize)]
struct QueryResultRow<'a> {
    #[serde(flatten)]
    data: std::collections::HashMap<&'a str, String>,
}

pub fn format_output(result: &QueryResult, format: Format) -> Result<()> {
    match format {
        Format::Table => format_table(result),
        Format::Json => format_json(result),
        Format::Csv => format_csv(result),
        Format::Raw => format_raw(result),
    }
}

fn format_table(result: &QueryResult) -> Result<()> {
    if result.columns.is_empty() {
        return Ok(());
    }

    let mut widths: Vec<usize> = result.columns.iter()
        .map(|col| col.len())
        .collect();

    for row in &result.rows {
        for (i, cell) in row.iter().enumerate() {
            if i < widths.len() {
                widths[i] = widths[i].max(cell.len());
            }
        }
    }

    print!("│ ");
    for (i, col) in result.columns.iter().enumerate() {
        print!("{:<width$} │ ", col, width=widths[i]);
    }
    println!();

    print!("├─");
    for i in widths.iter() {
        print!("{:─<item$}─┼─", "", item=i);
    }
    println!();

    for row in &result.rows {
        print!("│ ");
        for (i, cell) in row.iter().enumerate() {
            if i < widths.len() {
                print!("{:<width$} │ ", cell, width=widths[i]);
            }
        }
        println!();
    }

    Ok(())
}

fn format_json(result: &QueryResult) -> Result<()> {
    let mut rows = Vec::new();

    for row in &result.rows {
        let mut map = std::collections::HashMap::new();
        for (i, col) in result.columns.iter().enumerate() {
            if i < row.len() {
                map.insert(col.as_str(), row[i].clone());
            }
        }
        rows.push(QueryResultRow { data: map });
    }

    let json = serde_json::to_string_pretty(&rows)?;
    println!("{}", json);
    Ok(())
}

fn format_csv(result: &QueryResult) -> Result<()> {
    if result.columns.is_empty() {
        return Ok(());
    }

    let stdout = std::io::stdout();
    let mut writer = csv::Writer::from_writer(stdout.lock());

    writer.write_record(&result.columns)?;

    for row in &result.rows {
        writer.write_record(row)?;
    }

    writer.flush()?;
    Ok(())
}

fn format_raw(result: &QueryResult) -> Result<()> {
    for row in &result.rows {
        for (i, cell) in row.iter().enumerate() {
            if i > 0 {
                print!("\t");
            }
            print!("{}", cell);
        }
        println!();
    }
    Ok(())
}

