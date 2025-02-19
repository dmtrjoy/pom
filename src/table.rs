use std::convert::From;

use colored::ColoredString;

use crate::quest::{Status, Tier};

/// A table cell. Supports plain or rich text.
pub struct Cell {
    content: ColoredString,
}

impl Cell {
    pub fn content(&self) -> &ColoredString {
        &self.content
    }

    pub fn width(&self) -> usize {
        self.content.chars().count()
    }
}

impl From<ColoredString> for Cell {
    fn from(value: ColoredString) -> Self {
        Self { content: value }
    }
}

impl From<i64> for Cell {
    fn from(value: i64) -> Self {
        Self {
            content: value.to_string().into(),
        }
    }
}

impl From<Status> for Cell {
    fn from(value: Status) -> Self {
        Self {
            content: value.to_string().into(),
        }
    }
}

impl From<&String> for Cell {
    fn from(value: &String) -> Self {
        Self {
            content: value.clone().into(),
        }
    }
}

impl From<String> for Cell {
    fn from(value: String) -> Self {
        Self {
            content: value.into(),
        }
    }
}

impl From<&str> for Cell {
    fn from(value: &str) -> Self {
        Self {
            content: value.to_owned().into(),
        }
    }
}

impl From<Tier> for Cell {
    fn from(value: Tier) -> Self {
        Self {
            content: value.to_colored_string(),
        }
    }
}

/// A simple table, composed of column headers and rows.
pub struct Table {
    column_widths: Vec<usize>,
    columns: Vec<Cell>,
    rows: Vec<Vec<Cell>>,
}

impl Table {
    /// Constructs a new table.
    pub fn new(columns: Vec<Cell>) -> Self {
        let mut column_widths = Vec::new();
        for column in &columns {
            column_widths.push(column.width());
        }

        Self {
            column_widths,
            columns,
            rows: Vec::new(),
        }
    }

    /// Adds a new row to the table.
    pub fn add(&mut self, row: Vec<Cell>) {
        // Update the column widths if any cells are wider than the current column widths.
        for (column_idx, cell) in row.iter().enumerate() {
            if cell.width() > self.column_widths[column_idx] {
                self.column_widths[column_idx] = cell.width()
            }
        }

        self.rows.push(row);
    }

    /// Formats and prints the table to the standard output.
    pub fn show(&self) {
        let mut table = String::new();

        // Format the column headers.
        for (column_idx, column) in self.columns.iter().enumerate() {
            let column_width = self.column_widths[column_idx];
            let column = format!("{:1$} ", column.content(), column_width);
            table.push_str(column.as_str());
        }

        // Format the rows.
        for row in &self.rows {
            table.push_str("\n");
            for (column_idx, cell) in row.iter().enumerate() {
                let column_width = self.column_widths[column_idx];
                let cell = format!("{:1$} ", cell.content(), column_width);
                table.push_str(cell.as_str());
            }
        }

        // Print the table.
        println!("{}", table);
    }
}
