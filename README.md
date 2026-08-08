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
  - **Web.config Parser**: Auto-detects and loads connection strings from existing ASP.NET `Web.config` files.

- **Modern Enhancements**:
  - **Multi-Database Support**: Dynamically switch between **MSSQL**, **PostgreSQL**, **MySQL**, and **SQLite**.
  - **Panic-Free Data Handling**: Safe type conversion (`row.try_get`) for binary, GUID, date/time, and custom database types.
  - **Pure Rust Security**: Zero OpenSSL or native C-library dependencies (`rustls`).
  - **State-of-the-Art Web UI**: Dark mode theme, CodeMirror SQL editor (syntax highlighting, `Ctrl+Enter` shortcut execution, SQL formatting, copy query, query history).

---

## 🚀 Quick Start

### 1. Build & Run

```bash
# Build the application
cargo build --release

# Run the server
cargo run --release
```

Open your browser at:
`http://localhost:8080`

### 2. Configuration

Priority order for database connection strings:
1. `Web.config` (reads `<add name="ConnectionString" connectionString="..." />`)
2. Environment variables: `CONNECTION_STRING` or `DATABASE_URL`, `DATABASE_DRIVER` (`mssql`, `postgres`, `mysql`, `sqlite`), and `PORT`.
3. Web UI Connection Modal (live connection string switching).

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
