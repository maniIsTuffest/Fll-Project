# Project Structure

This document outlines the organization of the FLL Project, which uses a Rust + Yew frontend with a FastAPI backend.

## Directory Layout

```
Fll-Project/
├── backend/                    # Python FastAPI backend
│   └── main.py                # FastAPI application entry point
│
├── frontend/                   # Rust + Yew WebAssembly frontend
│   ├── src/
│   │   ├── lib.rs             # Library entry point (re-exports App)
│   │   ├── app.rs             # Main App component with routing
│   │   ├── api/               # API client module
│   │   │   ├── mod.rs         # API module exports
│   │   │   └── artifacts.rs   # Artifact API client functions
│   │   ├── pages/             # Page components
│   │   │   ├── mod.rs         # Page module exports
│   │   │   ├── home.rs        # Home page (artifact list)
│   │   │   ├── artifact_detail.rs  # Artifact detail page
│   │   │   ├── search.rs      # Search page
│   │   │   └── not_found.rs   # 404 page
│   │   └── components/        # Reusable UI components
│   │       ├── mod.rs         # Component module exports
│   │       └── navbar.rs      # Navigation bar component
│   ├── index.html             # Main HTML file with Tailwind CSS
│   ├── Cargo.toml             # Rust dependencies and metadata
│   └── Cargo.lock             # Locked dependency versions
│
├── database.py                # SQLAlchemy models and database operations
├── ai_analyzer.py             # AI analysis functionality (Ollama integration)
├── config.py                  # Configuration settings
├── init_db.py                 # Database initialization script
├── requirements.txt           # Python dependencies
├── run.ps1                    # PowerShell script to run both servers
├── README.md                  # Project documentation
├── PROJECT_STRUCTURE.md       # This file
├── .gitignore                 # Git ignore rules
└── artifacts.db               # SQLite database (generated at runtime)
```

## Key Files

### Backend

- **`backend/main.py`**: FastAPI application with REST API endpoints
  - `/api/health` - Health check
  - `/api/artifacts` - List all artifacts (GET) or create new (POST)
  - `/api/artifacts/{id}` - Get specific artifact
  - `/api/artifacts/search` - Search artifacts

### Frontend

- **`frontend/src/lib.rs`**: Entry point that exports the `App` component
- **`frontend/src/app.rs`**: Main application component with routing logic
- **`frontend/index.html`**: HTML template with Tailwind CSS CDN
- **`frontend/Cargo.toml`**: Rust dependencies (Yew, routing, HTTP client, etc.)

### Database & AI

- **`database.py`**: SQLAlchemy ORM models and CRUD operations
- **`ai_analyzer.py`**: Ollama integration for image analysis
- **`config.py`**: Configuration and settings

## Module Organization

### Frontend Modules

```
lib.rs
└── app.rs (main App component)
    ├── pages/
    │   ├── home.rs (displays artifact grid)
    │   ├── artifact_detail.rs (shows single artifact)
    │   ├── search.rs (search interface)
    │   └── not_found.rs (404 page)
    ├── components/
    │   └── navbar.rs (navigation bar)
    └── api/
        └── artifacts.rs (HTTP client functions)
```

### Backend Modules

```
backend/main.py
├── FastAPI app initialization
├── CORS middleware setup
├── Pydantic models (ArtifactBase, Artifact)
├── API endpoints
└── Database integration (imports from database.py)
```

## Development Workflow

### Starting the Application

1. **Backend** (Terminal 1):
   ```bash
   cd backend
   uvicorn main:app --reload
   ```
   - Runs on `http://localhost:8000`
   - API docs at `http://localhost:8000/docs`

2. **Frontend** (Terminal 2):
   ```bash
   cd frontend
   trunk serve
   ```
   - Runs on `http://localhost:8080`
   - Hot-reloading enabled

### Adding New Features

- **New API endpoint**: Add to `backend/main.py`
- **New page**: Create file in `frontend/src/pages/`, add to `mod.rs`, add route to `app.rs`
- **New component**: Create file in `frontend/src/components/`, add to `mod.rs`
- **New API client function**: Add to `frontend/src/api/artifacts.rs`

## Technology Stack

- **Frontend**: Rust + Yew (WebAssembly)
- **Backend**: Python + FastAPI
- **Database**: SQLite with SQLAlchemy ORM
- **AI**: Ollama for local image analysis
- **Styling**: Tailwind CSS
- **HTTP Client**: gloo-net (frontend), requests (backend)

## Clean Code Principles

- **Modular structure**: Each concern (pages, components, API) is separated
- **Clear naming**: File and function names clearly indicate purpose
- **Single responsibility**: Each module has one primary responsibility
- **Reusable components**: Common UI elements are in the components module
- **Centralized API**: All backend communication goes through the api module
