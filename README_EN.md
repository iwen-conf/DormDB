<div align="center">

# DormDB

**Self-Service Database Provisioning Platform**

<img width="2623" height="1312" alt="DormDB" src="https://github.com/user-attachments/assets/82402620-ce77-4e39-8a6d-75f1f8343731" />

[![Rust](https://img.shields.io/badge/Rust-1.70+-f74c00?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org)
[![Actix Web](https://img.shields.io/badge/Actix_Web-4-0068d6?style=flat-square)](https://actix.rs)
[![MySQL](https://img.shields.io/badge/MySQL-8.0-4479a1?style=flat-square&logo=mysql&logoColor=white)](https://www.mysql.com)
[![License: MIT](https://img.shields.io/badge/License-MIT-22c55e?style=flat-square)](LICENSE)
[![API Docs](https://img.shields.io/badge/API-Swagger-85ea2d?style=flat-square&logo=swagger&logoColor=black)](http://localhost:3000/swagger-ui/)

中文 · [English](README_EN.md)

</div>

---

## Overview

DormDB is a database self-service provisioning platform built with Rust. Users provide their student ID, and the system automatically creates an isolated MySQL database, user account, and assigns secure permissions.

Full-stack implementation including: user application interface, admin control panel, RESTful API, and JWT authentication.

### Features

| Feature | Description |
|---------|-------------|
| **Secure Isolation** | Each user can only access their own database, cross-database access is strictly prohibited |
| **Fully Automated** | One-click provisioning — database, user, and permissions created automatically |
| **Duplicate Prevention** | Uniqueness validation based on student ID whitelist |
| **Admin Panel** | Student ID management, user management, system monitoring, batch import |
| **Transaction Safety** | Automatic rollback on failure, ensuring data consistency |
| **API Documentation** | Built-in Swagger/OpenAPI docs |

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Nuxt.js SPA   │◄──►│  Actix-Web API  │◄──►│   MySQL Server  │
│                 │    │                 │    │                 │
│  Application    │    │  JWT Auth       │    │  Dynamic DBs    │
│  Admin Panel    │    │  Permissions    │    │  User Mgmt      │
│  Student Mgmt   │    │  Transactions   │    │  Access Control │
└─────────────────┘    └────────┬────────┘    └─────────────────┘
                                │
                       ┌────────▼────────┐
                       │     SQLite      │
                       │  Records/State  │
                       └─────────────────┘
```

## Quick Start

### Requirements

- Rust 1.70+
- MySQL 5.7+ / 8.0+
- macOS / Linux / Windows

### Installation

```bash
# Clone
git clone git@github.com:iwen-conf/DormDB.git
cd DormDB

# Configure
cp .env.example .env
# Edit .env with your MySQL connection details

# Run
cargo run
```

### Environment Variables

```bash
SERVER_HOST=127.0.0.1
SERVER_PORT=3000
SQLITE_PATH=./dormdb_state.db

MYSQL_HOST=localhost
MYSQL_PORT=3306
MYSQL_USERNAME=root
MYSQL_PASSWORD=your_password    # Required
MYSQL_DATABASE=default
MYSQL_ALLOWED_HOST=localhost    # Never use % in production

ADMIN_PASSWORD=admin123         # Admin password
DEV_MODE=true                   # Development mode
RUST_LOG=info
```

### Access

| Page | URL |
|------|-----|
| Home | http://localhost:3000 |
| Admin Panel | http://localhost:3000/admin/dashboard |
| Student Management | http://localhost:3000/admin/students |
| API Docs | http://localhost:3000/swagger-ui/ |

## API

### User Endpoints

```http
POST /api/v1/apply
Content-Type: application/json

{ "identity_key": "2023010101" }
```

```json
{
  "code": 0,
  "message": "Success",
  "data": {
    "db_host": "localhost",
    "db_port": 3306,
    "db_name": "db_2023010101",
    "username": "user_2023010101",
    "password": "GeneratedSecurePassword123"
  }
}
```

```http
GET /api/v1/health              # Health check
```

### Admin Endpoints

```http
POST /api/v1/admin/login        # Login
GET  /api/v1/admin/status       # System status
GET  /api/v1/admin/stats        # Application stats
POST /api/v1/admin/repair       # Data consistency repair
GET  /api/v1/admin/students     # List student IDs
POST /api/v1/admin/students     # Add student ID
```

## Security

### User Permissions

Users are granted only the following permissions, scoped to their own database:

`SELECT` · `INSERT` · `UPDATE` · `DELETE` · `INDEX` · `LOCK TABLES`

### Prohibited Operations

- `DROP DATABASE` / `CREATE DATABASE` / `CREATE USER`
- Access to `mysql.*` system tables
- Cross-database access

## Project Structure

```
src/
├── main.rs              # Entry point
├── lib.rs               # Library exports
├── api/                 # API routes and handlers
├── auth/                # JWT authentication and middleware
├── config/              # Configuration management
├── database/            # Database operations layer
├── models/              # Data models
├── routes/              # Route config and static file serving
├── services/            # Business logic
└── utils/               # Utility functions

static/                  # Frontend static files (Nuxt.js SSG)
├── index.html           # User application page
├── admin/               # Admin pages
│   ├── login/
│   ├── dashboard/
│   └── students/
└── assets/custom.css    # Global style overrides
```

## Development

```bash
cargo test               # Run tests
cargo build --release     # Production build
RUST_LOG=debug cargo run  # Debug mode
```

## License

[MIT](LICENSE)

## Contributing

1. Fork → 2. Create branch → 3. Commit changes → 4. Open PR

---

<div align="center">

**DormDB** — Making database provisioning simple and secure

</div>
