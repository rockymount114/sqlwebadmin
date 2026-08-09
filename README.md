# SQL Web Admin ⚡ (Rust Edition)

A high-performance, modern database administration web application built in **Rust** using **Axum**, **Tokio**, **Tiberius** (native SQL Server TDS driver), and **SQLx**.

Replaces legacy ASP.NET `SqlAdministrator.aspx` applications while offering native Microsoft SQL Server (MSSQL) metadata inspection alongside multi-database support (PostgreSQL, MySQL, SQLite) and a modern dark mode Web UI.

---

## 🌟 Features

- **Original ASP.NET Feature Parity**:
  - **Schema Explorer**: Interactive tree view for Tables (`sys.tables`), Views (`sys.views`), and Stored Procedures (`sys.procedures`).
  - **Column & Parameter Inspection**: View column types, character lengths, and procedure parameters (`sys.columns`, `sys.parameters`, `sys.types`).
  - **Query Auto-Generation**: Selecting a table automatically generates `SELECT TOP 1000 [col1], [col2]... FROM table`.
  - **Definition Retrieval**: View definitions for Views and Stored Procedures directly from `sys.sql_modules`.
  - **CSV Export**: Stream query output as downloadable CSV files (`query_export.csv`).

- **Modern Enhancements**:
  - **Multi-Database Support**: Dynamically switch between **MSSQL**, **PostgreSQL**, **MySQL**, and **SQLite**.
  - **Panic-Free Data Handling**: Safe type conversion (`row.try_get`) for binary, GUID, date/time, and custom database types.
  - **Pure Rust Security**: Zero OpenSSL or native C-library dependencies (`rustls`). No hardcoded credentials in source code.
  - **State-of-the-Art Web UI**: Dark mode theme, CodeMirror SQL editor (syntax highlighting, `Ctrl+Enter` shortcut execution, SQL formatting, copy query, query history).

---

## 🚀 Quick Start

### 1. Configuration

Copy `config.toml.example` to `config.toml` and set your credentials:

```bash
cp config.toml.example config.toml
```

Edit `config.toml`:
```toml
default_driver = "mssql"
default_connection_string = "Server=127.0.0.1;User ID=sa;Password=your_password_here;TrustServerCertificate=True;"
port = 8080
```

Priority order for database connection strings:
1. `config.toml`
2. `Web.config` (reads `<add name="ConnectionString" connectionString="..." />`)
3. Environment variables: `CONNECTION_STRING` or `DATABASE_URL`, `DATABASE_DRIVER`, and `PORT`.
4. Web UI Connection Modal (live connection string switching).

### 2. Build & Run

```bash
# Build the application
cargo build --release

# Run the server
cargo run --release

# Run on different port
PORT=8081 cargo run --release
```

Open your browser at:
`http://localhost:8080`

### Kill current process

```bash
pkill sqlwebadmin
```

---

## 🛠️ Stack & Dependencies

- **Backend**: Rust 2021 edition
  - `axum 0.7`: Async web framework
  - `tokio`: Asynchronous runtime
  - `tiberius 0.12`: Pure-Rust TDS driver for MSSQL
  - `sqlx 0.7`: Drivers for Postgres, MySQL, SQLite
  - `csv`: High-speed CSV exporter
- **Frontend**:
  - HTML5 & Vanilla CSS3 (Custom Dark Glassmorphic Design)
  - CodeMirror 5 for SQL syntax highlighting
  - FontAwesome 6 Icons
