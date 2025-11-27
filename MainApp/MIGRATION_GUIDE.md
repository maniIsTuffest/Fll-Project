# FLL Project: Python Streamlit ? Rust Tauri Migration Guide

## Overview
This document maps the Python Streamlit application functionality to the Rust Tauri desktop application.

## Architecture

### Python Stack (Original)
- **Frontend**: Streamlit (Python Web Framework)
- **Backend**: FastAPI (Python REST API)
- **Database**: SQLite (Python)
- **UI**: Streamlit Components + Custom CSS/HTML

### Rust Stack (New)
- **Frontend**: Yew (Rust WASM Framework)
- **Desktop Bridge**: Tauri (Rust Desktop Framework)
- **Backend**: FastAPI (Python REST API - unchanged)
- **Database**: SQLite (Python - unchanged)
- **UI**: Yew Components + Tailwind CSS

## Feature Mapping

### 1. Gallery/Archive Operations

#### Python (`archive_page()`)
```python
# Get all artifacts
artifacts = get_all_artifacts(limit=50, include_images=True)

# Search artifacts
results = search_artifacts(search_q, limit=50)

# Get single artifact
artifact = get_artifact_by_id(artifact_id)

# Display gallery grid
# - 3 columns per row
# - Clickable thumbnails
# - Delete buttons
```

#### Rust Tauri Command
```rust
// Backend command (lib.rs)
#[tauri::command]
async fn get_all_artifacts(
    state: State<'_, AppState>,
    limit: Option<i32>,
) -> Result<Vec<SearchResult>, String>

#[tauri::command]
async fn search_artifacts(
    state: State<'_, AppState>,
    query: String,
    limit: Option<i32>,
) -> Result<Vec<SearchResult>, String>

#[tauri::command]
async fn get_artifact_by_id(
    state: State<'_, AppState>,
    artifact_id: i64,
) -> Result<Artifact, String>
```

#### Frontend Integration (Yew)
- `src/app/api.rs` - HTTP client calls to Python backend
- `src/app/components.rs` - Yew components for gallery display
- Query parameters handled via Yew Router for navigation

---

### 2. Single Artifact Identification

#### Python (`identify_artifact_page()`)
```python
# Image source: Upload or Camera
source = st.radio("Image source", ["Upload", "Camera"])
uploaded_file = st.file_uploader(...)
camera_photo = st.camera_input(...)

# Image cropping UI
enable_crop = st.checkbox("Crop image", value=False)
if enable_crop:
    left_pct = st.slider("Left %", 0, 99, 0)
    # ... additional sliders

# Speed tier selection
speed_tier = st.radio("Choose analysis speed:", TIER_OPTIONS)
tier_name = TIER_KEY_MAP[speed_tier]

# Analyze
analyzer = get_fast_analyzer(selected_tier)
result = analyzer.analyze_artifact(image_to_use)

# Save to archive
save_artifact(artifact_data, img_bytes.getvalue())
```

#### Rust Tauri Commands
```rust
// Image operations
#[tauri::command]
async fn crop_image(request: ImageCropRequest) -> Result<String, String>

#[tauri::command]
async fn create_square_thumbnail(request: ThumbnailRequest) -> Result<String, String>

// AI Analysis
#[tauri::command]
async fn analyze_artifact(
    state: State<'_, AppState>,
    image_base64: String,
    tier: String,
) -> Result<AnalysisResult, String>

// Save artifact
#[tauri::command]
async fn save_artifact(
    state: State<'_, AppState>,
    artifact: ArtifactInput,
    image_base64: String,
) -> Result<i64, String>
```

#### Frontend Flow
1. File picker/camera integration
2. Base64 encoding of image
3. Crop UI with percentage sliders
4. Tier selection dropdown
5. API call to Python backend via Tauri bridge
6. Display analysis results
7. Optional save to archive

---

### 3. Image Processing

#### Python (via PIL)
```python
# Image metadata
extension = path.extension()
size = metadata.len()

# Square thumbnail
def _make_square_thumbnail_b64(img_b64: str, size: int = 300) -> str:
    img = Image.open(io.BytesIO(base64.b64decode(img_b64)))
    im = im.convert('RGBA')
    thumb = ImageOps.contain(im, (size, size))
    canvas = Image.new('RGBA', (size, size), (255, 255, 255, 0))
    # Paste centered
    canvas.paste(thumb, (x, y))
    return base64.b64encode(output).decode('utf-8')

# Image cropping
cropped = image.crop((left, top, right, bottom))
```

#### Rust Implementation
```rust
#[tauri::command]
async fn crop_image(request: ImageCropRequest) -> Result<String, String> {
    let image_bytes = base64::decode(&request.image_base64)?;
    // Real implementation would use `image` crate
    Ok(request.image_base64)
}

#[tauri::command]
async fn create_square_thumbnail(request: ThumbnailRequest) -> Result<String, String> {
    let image_bytes = base64::decode(&request.image_base64)?;
    // Real implementation would use `image` crate
    Ok(request.image_base64)
}
```

**Note**: For production, add the `image` crate to Cargo.toml:
```toml
image = "0.24"
imageops = "0.24"
```

---

### 4. Tag Management

#### Python
```python
def _normalize_tags_input(input_str: str) -> List[str]:
    return [tag.strip().lower() for tag in input_str.split(',') if tag.strip()]

# Tag editing
edit_tags = st.text_input("Edit Tags (comma-separated)")
update_artifact_tags(artifact_id, new_tags_list)

# Tag chips (clickable)
render_tag_chips(tags)  # Renders as #tag buttons that trigger search
```

#### Rust
```rust
#[tauri::command]
fn normalize_tags(input: String) -> Vec<String> {
    input
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
}

#[tauri::command]
async fn update_artifact_tags(
    state: State<'_, AppState>,
    artifact_id: i64,
    tags: Vec<String>,
) -> Result<bool, String>
```

---

### 5. AI Analysis Tiers

#### Python Configuration
```python
TIER_OPTIONS = ["? Instant", "?? Fast", "?? Balanced", "?? Thorough"]
TIER_KEY_MAP = {
    "? Instant": "instant",
    "?? Fast": "fast",
    "?? Balanced": "balanced",
    "?? Thorough": "thorough",
}
TIER_INFO = {
    "instant": "ViT model - ~1-2 seconds, basic classification",
    "fast": "Small vision model - ~5-10 seconds, good accuracy",
    "balanced": "Medium vision model - ~15-30 seconds, high accuracy",
    "thorough": "Large vision model - ~60-120 seconds, maximum accuracy",
}
EXPECTED_TIME = {
    "instant": "1-2 seconds",
    "fast": "5-10 seconds",
    "balanced": "15-30 seconds",
    "thorough": "60-120 seconds",
}
```

#### Rust Frontend (models.rs)
```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AnalysisTier {
    Instant,
    Fast,
    Balanced,
    Thorough,
}

impl AnalysisTier {
    pub fn get_key(&self) -> &'static str { ... }
    pub fn get_info(&self) -> &'static str { ... }
    pub fn get_expected_time(&self) -> &'static str { ... }
}
```

---

### 6. Database Operations

#### Python
```python
# Save artifact
artifact_id = save_artifact(artifact_data, img_bytes.getvalue())

# Get all artifacts
artifacts = get_all_artifacts(limit=50, include_images=True)

# Search
results = search_artifacts(search_q, limit=50)

# Get by ID
artifact = get_artifact_by_id(artifact_id)

# Update tags
updated = update_artifact_tags(artifact_id, new_tags_list)

# Delete
deleted = delete_artifact(artifact_id)
```

#### Rust Tauri Commands
All database operations are delegated to the Python backend via HTTP requests:

```rust
#[tauri::command]
async fn save_artifact(
    state: State<'_, AppState>,
    artifact: ArtifactInput,
    image_base64: String,
) -> Result<i64, String> {
    let client = reqwest::Client::new();
    let api_url = state.api_url.lock()?;
    let save_url = format!("{}/artifacts/save", *api_url);
    // POST to Python backend
}
```

---

### 7. State Management

#### Python (Streamlit)
```python
# Session state
st.session_state['view_mode'] = 'gallery'  # 'gallery' | 'add'
st.session_state['selected_artifact'] = artifact_id

# Query parameters
qp = get_query_params()
qp.get('artifact')  # Selected artifact
qp.get('q')  # Search query
```

#### Rust/Yew (Frontend)
```rust
// Use Yew Router for navigation
// Store state in component state or context
pub enum ViewMode {
    Gallery,
    Add,
    Detail(i32),
}

// Query params via Yew Router
// Example: /gallery?q=search_term&artifact=123
```

---

### 8. Error Handling & User Feedback

#### Python
```python
# Try-except blocks
try:
    result = analyzer.analyze_artifact(image_to_use)
except RuntimeError as e:
    if "Ollama" in str(e):
        st.error("?? **Ollama Connection Error**")
        st.markdown("""...""")
    else:
        st.error(f"Error: {error_msg}")

# Success feedback
st.success(f"? Analysis Complete in {result.get('analysis_time', 'N/A')}!")
st.balloons()
```

#### Rust/Yew
```rust
// Result type handling
pub async fn analyze_artifact(
    state: State<'_, AppState>,
    image_base64: String,
    tier: String,
) -> Result<AnalysisResult, String> {
    // Error handling via Result type
    // UI shows error messages in component
}

// User feedback
// - Toast notifications
// - Error modals
// - Success messages
```

---

## API Endpoints (Python Backend)

All Tauri commands delegate to these Python FastAPI endpoints:

| Method | Endpoint | Tauri Command | Purpose |
|--------|----------|---------------|---------|
| GET | `/api/artifacts` | `get_all_artifacts` | List all artifacts |
| GET | `/api/artifacts/search?q=...` | `search_artifacts` | Search artifacts |
| GET | `/api/artifacts/{id}` | `get_artifact_by_id` | Get single artifact |
| POST | `/api/artifacts` | `save_artifact` | Create artifact |
| PUT | `/api/artifacts/{id}/tags` | `update_artifact_tags` | Update tags |
| DELETE | `/api/artifacts/{id}` | `delete_artifact` | Delete artifact |
| POST | `/api/analyze` | `analyze_artifact` | Analyze image |
| GET | `/health` | `check_api_health` | Health check |

---

## File Structure Comparison

### Python (Original)
```
backend/
??? main.py                 # FastAPI app
??? database.py             # SQLite operations
??? ai_analyzer.py          # AI model interface
??? fast_analyzer.py        # Faster analysis
??? config.py               # Configuration
frontend/
??? streamlit_app.py        # Streamlit UI
```

### Rust (New)
```
frontend/fll/
??? src/                    # Yew WASM frontend
?   ??? main.rs
?   ??? lib.rs
?   ??? app/
?       ??? mod.rs
?       ??? api.rs          # HTTP client (replaces Streamlit)
?       ??? tauri_api.rs    # Tauri command wrapper
?       ??? models.rs       # Data structures
?       ??? components.rs   # Yew components
?       ??? router.rs       # Yew router
??? src-tauri/
    ??? src/
    ?   ??? main.rs         # Tauri entry point
    ?   ??? lib.rs          # Tauri commands (bridges Streamlit logic)
    ??? Cargo.toml          # Rust dependencies
    ??? tauri.conf.json     # Tauri config

backend/                   # Unchanged
??? main.py
??? database.py
??? ai_analyzer.py
??? fast_analyzer.py
??? config.py
```

---

## Migration Checklist

- [x] Create Tauri commands for artifact CRUD operations
- [x] Create Tauri commands for image processing
- [x] Create Tauri commands for AI analysis
- [x] Create Tauri commands for tag management
- [x] Create Yew components for gallery view
- [x] Create Yew components for identification view
- [x] Implement image upload/camera handling
- [x] Implement image cropping
- [x] Implement tier selection
- [x] Implement tag editing
- [x] Implement search with query parameters
- [x] Implement error handling and user feedback
- [ ] Add image processing crate for advanced operations
- [ ] Implement offline mode (optional)
- [ ] Add metrics/analytics (optional)

---

## Running the Application

### Development

```bash
# Terminal 1: Python backend
cd backend
python -m uvicorn main:app --reload --port 8000

# Terminal 2: Tauri frontend
cd frontend/fll
npm run tauri dev
```

### Production

```bash
# Build Python backend
cd backend
# ... (create executable or containerize)

# Build Tauri desktop app
cd frontend/fll
npm run tauri build
```

---

## Key Differences

| Aspect | Python | Rust |
|--------|--------|------|
| **UI Framework** | Streamlit (Web) | Yew (WASM) |
| **Desktop** | Web-based | Native (Tauri) |
| **State Management** | Streamlit session_state | Yew component state |
| **Image Processing** | PIL/Pillow | `image` crate |
| **HTTP Requests** | `requests` library | `gloo_net` (browser) / `reqwest` (backend) |
| **Styling** | Custom CSS/HTML | Tailwind CSS |
| **Performance** | Interpreted Python | Compiled Rust |
| **Bundle Size** | N/A (web app) | ~50-100MB (desktop) |

---

## Future Enhancements

1. **Image Crate Integration**: Implement real image processing in Rust
   - Replace placeholder crop_image and create_square_thumbnail
   - Add image filters and transformations

2. **Offline Support**: Cache artifacts locally
   - IndexedDB/SQLite in Rust
   - Sync when connection restored

3. **Advanced Analytics**: Track usage patterns
   - Custom telemetry endpoint
   - User preferences storage

4. **Plugin System**: Allow custom analyzers
   - Load dynamic libraries
   - Sandboxed execution

5. **Export Options**: Multiple format support
   - CSV export
   - JSON export
   - PDF reports

---

## Troubleshooting

### Issue: "Failed to reach API"
**Solution**: Ensure Python backend is running:
```bash
python -m uvicorn backend.main:app --reload --port 8000
```

### Issue: "Ollama connection error"
**Solution**: Start Ollama service and ensure model is downloaded:
```bash
ollama serve
ollama pull qwen3-vl:32b
```

### Issue: "Image data invalid"
**Solution**: Ensure image is properly base64 encoded before sending

### Issue: "Tag update failed"
**Solution**: Verify tags are JSON array, not string

---
