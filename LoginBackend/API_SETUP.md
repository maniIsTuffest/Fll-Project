# Login Backend API Setup Guide

## Overview

This project consists of two integrated components:

1. **Rust Backend** - A Rocket web server that manages the SQLite database and provides HTTP endpoints for user authentication
2. **Ruby on Rails API** - A Rails API that can be used as a frontend to the Rust backend

## Prerequisites

### For Rust Backend
- Rust (1.56 or later) - [Install Rust](https://www.rust-lang.org/tools/install)
- SQLite3

### For Ruby on Rails API
- Ruby (3.2 or later)
- Ruby on Rails (8.1 or later)
- Bundler

## Directory Structure

```
LoginBackend/
├── main/                    # Rust backend
│   ├── src/
│   │   ├── main.rs         # Entry point, server setup, Rails subprocess management
│   │   └── db.rs           # Database operations (SQLite)
│   ├── Cargo.toml          # Rust dependencies
│   └── login.db            # SQLite database
└── api/
    └── api/                # Rails application
        ├── app/
        │   ├── controllers/
        │   │   └── api/v1/
        │   │       ├── base_controller.rb
        │   │       └── users_controller.rb
        │   └── views/
        ├── config/
        │   └── routes.rb
        ├── Gemfile
        └── Puma config (port 3000)
```

## Installation

### Step 1: Setup Rust Backend

```bash
cd LoginBackend/main
cargo build --release
```

### Step 2: Setup Rails API

```bash
cd LoginBackend/api/api
bundle install
```

### Step 3: Prepare Database

Ensure the default admin user data exists at `tempData/defaultAdmin.jsonc`:

```json
{
  "username": "admin",
  "password": "admin123",
  "rank": 1,
  "email": "admin@example.com"
}
```

## Running the System

### Option 1: Start Everything (Recommended)

The Rust backend will automatically start the Rails server as a subprocess:

```bash
cd LoginBackend/main
cargo run --release
```

This will:
1. Initialize the SQLite database
2. Start the Rust API server on `http://localhost:8000`
3. Start the Rails server on `http://localhost:3000`

### Option 2: Start Services Separately

**Terminal 1 - Rails API:**
```bash
cd LoginBackend/api/api
bundle exec rails server -p 3000
```

**Terminal 2 - Rust Backend:**
```bash
cd LoginBackend/main
cargo run
```

## API Endpoints

### Rust Backend (Port 8000)

#### Health Check
```
GET http://localhost:8000/health

Response:
{
  "status": "ok"
}
```

#### Search User (Internal Endpoint)
```
POST http://localhost:8000/search_user
Content-Type: application/json

Request:
{
  "username": "admin",
  "password": "admin123"
}

Response (Success):
{
  "success": true,
  "message": "User found",
  "data": {
    "username": "admin",
    "rank": 1,
    "email": "admin@example.com"
  }
}

Response (Failure):
{
  "success": false,
  "message": "Invalid credentials",
  "data": null
}
```

### Rails API (Port 3000)

#### User Login
```
POST http://localhost:3000/api/v1/users/login
Content-Type: application/json

Request:
{
  "username": "admin",
  "password": "admin123"
}

Response (Success):
{
  "success": true,
  "message": "Login successful",
  "data": {
    "username": "admin",
    "email": "admin@example.com",
    "rank": 1
  }
}

Response (Failure):
{
  "success": false,
  "message": "Invalid username or password",
  "errors": {}
}
```

#### Health Check
```
GET http://localhost:3000/up

Response:
{
  "status": "ok"
}
```

## Database Schema

The system uses SQLite with the following table structure:

```sql
CREATE TABLE users (
  id INTEGER PRIMARY KEY,
  username TEXT NOT NULL UNIQUE,
  password TEXT NOT NULL,
  rank INTEGER NOT NULL,
  email TEXT NOT NULL
)
```

## Architecture Flow

```
Client Request
    ↓
Rails API (Port 3000)
    ↓
Rust Backend (Port 8000)
    ↓
SQLite Database
    ↓
Response → Rails → Client
```

## Development Notes

### Security Considerations

⚠️ **WARNING**: The current implementation stores passwords in plain text. For production use:

1. Implement password hashing using `bcrypt` or `argon2`
2. Add JWT authentication tokens
3. Implement HTTPS/TLS
4. Add rate limiting
5. Implement CORS properly

### Adding New Database Operations

To add new database functions:

1. Add the function to `main/src/db.rs`
2. Export it publicly
3. Create a new Rocket endpoint in `main/src/main.rs`
4. Create a corresponding Rails controller method in `api/api/app/controllers/api/v1/`

### Debugging

Set Rust log level:
```bash
RUST_LOG=debug cargo run
```

View Rails logs:
```bash
cd LoginBackend/api/api
tail -f log/development.log
```

## Troubleshooting

### Rails Server Won't Start
```bash
cd LoginBackend/api/api
bundle install
bundle exec rails db:create
bundle exec rails server -p 3000
```

### Port Already in Use
- Rust API (8000): `netstat -ano | findstr :8000`
- Rails API (3000): `netstat -ano | findstr :3000`

Kill process on Windows:
```bash
taskkill /PID <PID> /F
```

### Database Locked
Delete `login.db` and restart (will recreate with default admin):
```bash
cd LoginBackend/main
rm login.db
cargo run
```

## Testing the API

### Using cURL

```bash
# Test Rust backend directly
curl -X POST http://localhost:8000/search_user \
  -H "Content-Type: application/json" \
  -d "{\"username\":\"admin\",\"password\":\"admin123\"}"

# Test Rails API
curl -X POST http://localhost:3000/api/v1/users/login \
  -H "Content-Type: application/json" \
  -d "{\"username\":\"admin\",\"password\":\"admin123\"}"
```

### Using Postman

1. Create a new POST request
2. Set URL: `http://localhost:3000/api/v1/users/login`
3. Set Header: `Content-Type: application/json`
4. Set Body (raw JSON):
```json
{
  "username": "admin",
  "password": "admin123"
}
```
5. Send request

## Production Deployment

For production deployment:

1. Build release binary: `cargo build --release`
2. Use a process manager (systemd, supervisord)
3. Set up proper logging and monitoring
4. Configure environment variables
5. Use a reverse proxy (nginx)
6. Implement SSL/TLS certificates
7. Set up database backups

## License

Your project license here