//! SQL database interfaces.

use std::fmt;
use std::future::Future;

/// SQL value types.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Self::Integer(v)
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Self::Real(v)
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Self::Text(v)
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Self::Text(v.to_string())
    }
}

impl From<Vec<u8>> for Value {
    fn from(v: Vec<u8>) -> Self {
        Self::Blob(v)
    }
}

impl<T: Into<Value>> From<Option<T>> for Value {
    fn from(v: Option<T>) -> Self {
        match v {
            Some(v) => v.into(),
            None => Self::Null,
        }
    }
}

/// A row from a query result.
#[derive(Debug, Clone)]
pub struct Row {
    columns: Vec<String>,
    values: Vec<Value>,
}

impl Row {
    /// Create a new row.
    pub fn new(columns: Vec<String>, values: Vec<Value>) -> Self {
        Self { columns, values }
    }

    /// Get a value by column index.
    pub fn get(&self, index: usize) -> Option<&Value> {
        self.values.get(index)
    }

    /// Get a value by column name.
    pub fn get_by_name(&self, name: &str) -> Option<&Value> {
        self.columns
            .iter()
            .position(|c| c == name)
            .and_then(|i| self.values.get(i))
    }

    /// Get the column names.
    pub fn columns(&self) -> &[String] {
        &self.columns
    }

    /// Get all values.
    pub fn values(&self) -> &[Value] {
        &self.values
    }
}

/// Database errors.
#[derive(Debug)]
pub enum Error {
    /// Connection failed.
    ConnectionFailed,
    /// Query syntax error.
    SyntaxError(String),
    /// Constraint violation.
    ConstraintViolation(String),
    /// Type mismatch.
    TypeMismatch,
    /// Database is busy/locked.
    Busy,
    /// Other error.
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ConnectionFailed => write!(f, "connection failed"),
            Error::SyntaxError(msg) => write!(f, "syntax error: {}", msg),
            Error::ConstraintViolation(msg) => write!(f, "constraint violation: {}", msg),
            Error::TypeMismatch => write!(f, "type mismatch"),
            Error::Busy => write!(f, "database busy"),
            Error::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for Error {}

/// A database connection.
///
/// This trait defines operations on an already-opened connection.
/// The connection is obtained from a backend-specific constructor,
/// following the capability-based model (no `open(path)` in the interface).
pub trait Connection {
    /// Execute a query that returns rows.
    fn query(
        &self,
        sql: &str,
        params: &[Value],
    ) -> impl Future<Output = Result<Vec<Row>, Error>>;

    /// Execute a statement that doesn't return rows.
    fn execute(
        &self,
        sql: &str,
        params: &[Value],
    ) -> impl Future<Output = Result<u64, Error>>;

    /// Begin a transaction.
    fn begin(&self) -> impl Future<Output = Result<(), Error>>;

    /// Commit the current transaction.
    fn commit(&self) -> impl Future<Output = Result<(), Error>>;

    /// Rollback the current transaction.
    fn rollback(&self) -> impl Future<Output = Result<(), Error>>;
}
