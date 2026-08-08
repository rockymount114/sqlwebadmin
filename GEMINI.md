# GEMINI.md - SQL Web Admin (Rust Port)

## Project Overview
`sqlwebadmin` is a high-performance web administration application for SQL Server (MSSQL), PostgreSQL, MySQL, and SQLite databases written in **Rust** with an **Axum** async HTTP backend and a modern glassmorphic web front-end.

This project replaces the legacy ASP.NET `SqlAdministrator.aspx` single-file application while preserving 100% feature parity for MSSQL schema browsing, stored procedure inspection, query execution, and CSV exports.

## Tech Stack & Architecture
- **Language**: Rust (2021 edition)
- **Web Framework**: Axum 0.7 + Tokio
- **Database Drivers**:
  - `tiberius`: Pure-Rust TDS driver for Microsoft SQL Server
  - `sqlx`: Dynamic async drivers for PostgreSQL, MySQL, and SQLite
- **Security / TLS**: `rustls` (Zero native C-library or OpenSSL build dependencies)
- **Serialization**: `serde` & `serde_json`
- **Frontend**: HTML5, Vanilla CSS3 (Dark Mode Glassmorphic Design System), CodeMirror 5 SQL Editor, FontAwesome 6

## Directory Structure
```
sqlwebadmin/
├── Cargo.toml          # Cargo dependencies and binary metadata
├── Web.config          # Legacy ASP.NET config file (parsed for default connection string)
├── README.md           # User documentation and quick start guide
├── GEMINI.md           # Project architecture and developer instructions
├── src/
│   ├── main.rs         # Application entry point, Axum routes, CORS & static file server
│   ├── config.rs       # Configuration loader (Web.config & environment variables)
│   ├── models.rs       # Data models and API DTOs
│   ├── db/
│   │   ├── mod.rs      # Dynamic multi-database dispatcher
│   │   ├── mssql.rs    # MSSQL native schema inspection & panic-free query execution
│   │   ├── postgres.rs # PostgreSQL driver handler
│   │   ├── mysql.rs    # MySQL driver handler
│   │   └── sqlite.rs   # SQLite driver handler
│   └── handlers/
│       └── mod.rs      # Axum HTTP route handlers (/api/schema, /api/query, /api/connect)
└── static/
    ├── index.html      # Single page app interface
    ├── styles.css      # Dark mode glassmorphic CSS design system
    └── app.js          # CodeMirror integration, dynamic schema tree, & error handling
```

## Useful Commands

### Build & Run
```bash
# Check code without building
cargo check

# Build debug binary
cargo build

# Run application
cargo run

# Build release binary
cargo build --release
```

### Environment Variables
- `CONNECTION_STRING` or `DATABASE_URL`: Default connection string (overrides `Web.config`)
- `DATABASE_DRIVER`: Default database driver (`mssql`, `postgres`, `mysql`, `sqlite`)
- `PORT`: HTTP listener port (default: `8080`)

## Guidelines for Developers
- Keep column value extraction safe: always use `row.try_get::<T, _>(idx)` rather than `row.get` to prevent panics when handling unusual or binary database column types.
- Ensure all API endpoints return valid JSON responses with appropriate HTTP status codes so the frontend `safeFetchJson` helper never encounters invalid JSON tokens.
