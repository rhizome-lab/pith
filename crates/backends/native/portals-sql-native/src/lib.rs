//! Native implementation of portals-sql using libsql.

use portals_sql::{Connection, Error, Row, Value};

/// A SQLite connection backed by libsql.
///
/// Create connections using [`LibsqlConnection::open`].
pub struct LibsqlConnection {
    _db: libsql::Database,
    conn: libsql::Connection,
}

impl LibsqlConnection {
    /// Open a connection to a SQLite database at the given path.
    ///
    /// Use `:memory:` for an in-memory database.
    pub async fn open(path: &str) -> Result<Self, Error> {
        let db = libsql::Builder::new_local(path)
            .build()
            .await
            .map_err(|e| Error::Other(e.to_string()))?;
        let conn = db.connect().map_err(|e| Error::Other(e.to_string()))?;
        Ok(LibsqlConnection { _db: db, conn })
    }
}

impl Connection for LibsqlConnection {
    async fn query(&self, sql: &str, params: &[Value]) -> Result<Vec<Row>, Error> {
        let params: Vec<libsql::Value> = params.iter().map(to_libsql_value).collect();
        let mut rows = self
            .conn
            .query(sql, params)
            .await
            .map_err(map_error)?;

        let mut result = Vec::new();
        let columns: Vec<String> = (0..rows.column_count())
            .map(|i| rows.column_name(i).unwrap_or("").to_string())
            .collect();

        while let Some(row) = rows.next().await.map_err(map_error)? {
            let values: Vec<Value> = (0..columns.len())
                .map(|i| from_libsql_value(row.get_value(i as i32).unwrap_or(libsql::Value::Null)))
                .collect();
            result.push(Row::new(columns.clone(), values));
        }

        Ok(result)
    }

    async fn execute(&self, sql: &str, params: &[Value]) -> Result<u64, Error> {
        let params: Vec<libsql::Value> = params.iter().map(to_libsql_value).collect();
        let rows_affected = self
            .conn
            .execute(sql, params)
            .await
            .map_err(map_error)?;
        Ok(rows_affected)
    }

    async fn begin(&self) -> Result<(), Error> {
        self.conn
            .execute("BEGIN", ())
            .await
            .map_err(map_error)?;
        Ok(())
    }

    async fn commit(&self) -> Result<(), Error> {
        self.conn
            .execute("COMMIT", ())
            .await
            .map_err(map_error)?;
        Ok(())
    }

    async fn rollback(&self) -> Result<(), Error> {
        self.conn
            .execute("ROLLBACK", ())
            .await
            .map_err(map_error)?;
        Ok(())
    }
}

fn to_libsql_value(v: &Value) -> libsql::Value {
    match v {
        Value::Null => libsql::Value::Null,
        Value::Integer(i) => libsql::Value::Integer(*i),
        Value::Real(f) => libsql::Value::Real(*f),
        Value::Text(s) => libsql::Value::Text(s.clone()),
        Value::Blob(b) => libsql::Value::Blob(b.clone()),
    }
}

fn from_libsql_value(v: libsql::Value) -> Value {
    match v {
        libsql::Value::Null => Value::Null,
        libsql::Value::Integer(i) => Value::Integer(i),
        libsql::Value::Real(f) => Value::Real(f),
        libsql::Value::Text(s) => Value::Text(s),
        libsql::Value::Blob(b) => Value::Blob(b),
    }
}

fn map_error(e: libsql::Error) -> Error {
    let msg = e.to_string();
    if msg.contains("UNIQUE") || msg.contains("constraint") {
        Error::ConstraintViolation(msg)
    } else if msg.contains("syntax") || msg.contains("parse") {
        Error::SyntaxError(msg)
    } else {
        Error::Other(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn basic_operations() {
        let conn = LibsqlConnection::open(":memory:").await.unwrap();

        // Create table
        conn.execute(
            "CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)",
            &[],
        )
        .await
        .unwrap();

        // Insert
        conn.execute(
            "INSERT INTO test (name) VALUES (?)",
            &[Value::Text("hello".to_string())],
        )
        .await
        .unwrap();

        // Query
        let rows = conn.query("SELECT * FROM test", &[]).await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].get_by_name("name"), Some(&Value::Text("hello".to_string())));
    }

    #[tokio::test]
    async fn transaction_commit() {
        let conn = LibsqlConnection::open(":memory:").await.unwrap();
        conn.execute("CREATE TABLE t (x INTEGER)", &[]).await.unwrap();

        conn.begin().await.unwrap();
        conn.execute("INSERT INTO t VALUES (1)", &[]).await.unwrap();
        conn.commit().await.unwrap();

        let rows = conn.query("SELECT * FROM t", &[]).await.unwrap();
        assert_eq!(rows.len(), 1);
    }

    #[tokio::test]
    async fn transaction_rollback() {
        let conn = LibsqlConnection::open(":memory:").await.unwrap();
        conn.execute("CREATE TABLE t (x INTEGER)", &[]).await.unwrap();

        conn.begin().await.unwrap();
        conn.execute("INSERT INTO t VALUES (1)", &[]).await.unwrap();
        conn.rollback().await.unwrap();

        let rows = conn.query("SELECT * FROM t", &[]).await.unwrap();
        assert_eq!(rows.len(), 0);
    }
}
