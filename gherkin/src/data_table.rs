#[derive(Debug, Clone, PartialEq)]
pub struct DataTable {
    pub(crate) header: Vec<String>,
    pub(crate) rows: Vec<Vec<String>>,
}

impl DataTable {
    pub fn new(header: Vec<String>) -> Self {
        Self {
            header,
            rows: Vec::new(),
        }
    }

    pub fn new_populated(header: Vec<String>, rows: Vec<Vec<String>>) -> Option<Self> {
        if rows.iter().any(|r| r.len() != header.len()) {
            return None;
        } else {
            Some(Self { header, rows })
        }
    }

    pub fn add_row(&mut self, row: Vec<String>) -> Result<(), ()> {
        if row.len() == self.header.len() {
            self.rows.push(row);
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn header(&self) -> &Vec<String> {
        &self.header
    }

    pub fn rows(&self) -> &Vec<Vec<String>> {
        &self.rows
    }
}
