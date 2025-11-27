# Tauri Migration Guide

This project has been converted from a Yew web app (using Trunk) to a Tauri desktop application.

## Architecture

The project now has a dual-structure:
- **Frontend (WASM)**: `frontend/` - Yew app compiled to WebAssembly
- **Tauri Backend**: `frontend/src-tauri/` - Rust backend for the desktop app
- **Python Backend**: `backend/` - FastAPI server for artifact data

## Changes Made

### 1. **Frontend Structure**
   - `frontend/Cargo.toml` - WASM library configuration
   - `frontend/src/lib.rs` - Added WASM entry point
   - `frontend/src/bootstrap.ts` - Frontend initialization script
   - `frontend/index.html` - Updated to use Vite
   - `frontend/package.json` - Node.js dependencies
   - `frontend/vite.config.ts` - Vite bundler configuration
   - `frontend/tsconfig.json` - TypeScript configuration

### 2. **Tauri Backend**
   - `frontend/src-tauri/Cargo.toml` - Tauri application configuration
   - `frontend/src-tauri/build.rs` - Tauri build script
   - `frontend/src-tauri/src/main.rs` - Tauri application entry point
   - `frontend/tauri.conf.json` - Tauri configuration

### 3. **Updated Files**
   - `run.ps1` - Updated to use `cargo tauri dev` instead of Trunk

## Setup Instructions

### Prerequisites
- **Rust and Cargo** (1.60+)
- **Node.js and npm** (14+)
- **Python 3.8+** (for backend)
- **Windows SDK** (for Windows development)

### Installation

1. **Install Node.js dependencies:**
   ```bash
   cd frontend
   npm install
   ```

2. **Tauri CLI will be installed automatically** when you run `cargo tauri dev`

### Running the Application

#### Option 1: Using the run script (recommended)
```bash
.\run.ps1
```

This will start:
- FastAPI backend on `http://localhost:8000`
- Tauri dev server with desktop app window

#### Option 2: Manual startup

**Terminal 1 - Backend:**
```bash
python -m uvicorn backend.main:app --reload
```

**Terminal 2 - Frontend (from frontend directory):**
```bash
cargo tauri dev
```

## Development

- The Tauri dev server runs on `http://localhost:1420`
- The desktop app window opens automatically
- Hot reload is enabled for both Rust and frontend code
- DevTools are automatically opened in debug mode (press F12)

## Building for Production

```bash
cd frontend
cargo tauri build
```

This creates a production-ready executable in `frontend/src-tauri/target/release/`.

## Project Structure

```
frontend/
├── src/                    # Yew WASM app source
│   ├── app.rs
│   ├── pages/
│   ├── components/
│   ├── api/
│   ├── lib.rs
│   ├── bootstrap.ts
│   └── main.rs            # Placeholder (not used)
├── src-tauri/             # Tauri backend
│   ├── src/
│   │   └── main.rs        # Tauri entry point
│   ├── Cargo.toml
│   └── build.rs
├── index.html
├── Cargo.toml             # WASM library config
├── package.json
├── vite.config.ts
├── tsconfig.json
└── tauri.conf.json        # Tauri configuration
```

## API Communication

The frontend communicates with the backend API at `http://localhost:8000/api`. This is configured in `src/api/artifacts.rs`.

The Tauri configuration allows HTTP requests to `http://localhost:8000/*` in `tauri.conf.json`.

## Key Differences from Web Version

| Feature | Web (Trunk) | Tauri (Desktop) |
|---------|------------|-----------------|
| Runtime | Browser | Native OS |
| Dev Server | Trunk | Vite + Tauri |
| Port | 8080 | 1420 |
| Window | Browser tab | Native window |
| DevTools | Browser DevTools | Integrated DevTools (F12) |
| Distribution | Web hosting | Desktop installer |

## Troubleshooting

### "cargo tauri not found"
- Run `cargo install tauri-cli` or let Cargo handle it automatically

### Port already in use
- Change the port in `vite.config.ts` and `tauri.conf.json`

### WASM compilation errors
- Ensure you're in the `frontend/` directory
- Run `cargo clean` and try again

### Backend connection issues
- Verify FastAPI is running on `http://localhost:8000`
- Check CORS settings in `backend/main.py`

## Notes

- The application runs as a native desktop app instead of in a web browser
- All Yew components and routing remain unchanged
- The API layer (gloo-net) works seamlessly with Tauri
- The app can be distributed as a standalone executable
