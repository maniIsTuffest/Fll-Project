# Tauri Backend Quick Reference

## Available Commands

### File Operations

```rust
// Get downloads directory (cross-platform)
pub async fn get_downloads_dir() -> Result<String, String>

// List files in directory
pub async fn list_files_in_directory(path: String) -> Result<Vec<FileInfo>, String>

// Read file as base64
pub async fn read_file_as_base64(file_path: String) -> Result<String, String>
```

### Image Operations

```rust
// Get image metadata
pub async fn get_image_metadata(file_path: String) -> Result<ImageMetadata, String>

// Crop image (base64)
pub async fn crop_image(request: ImageCropRequest) -> Result<String, String>

// Create square thumbnail
pub async fn create_square_thumbnail(request: ThumbnailRequest) -> Result<String, String>
```

### Artifact Management

```rust
// Save artifact to database
pub async fn save_artifact(
    state: State<'_, AppState>,
    artifact: ArtifactInput,
    image_base64: String,
) -> Result<i64, String>

// Get all artifacts
pub async fn get_all_artifacts(
    state: State<'_, AppState>,
    limit: Option<i32>,
) -> Result<Vec<SearchResult>, String>

// Search artifacts
pub async fn search_artifacts(
    state: State<'_, AppState>,
    query: String,
    limit: Option<i32>,
) -> Result<Vec<SearchResult>, String>

// Get single artifact by ID
pub async fn get_artifact_by_id(
    state: State<'_, AppState>,
    artifact_id: i64,
) -> Result<Artifact, String>

// Update artifact tags
pub async fn update_artifact_tags(
    state: State<'_, AppState>,
    artifact_id: i64,
    tags: Vec<String>,
) -> Result<bool, String>

// Delete artifact
pub async fn delete_artifact(
    state: State<'_, AppState>,
    artifact_id: i64,
) -> Result<bool, String>
```

### AI Analysis

```rust
// Analyze artifact image
pub async fn analyze_artifact(
    state: State<'_, AppState>,
    image_base64: String,
    tier: String,
) -> Result<AnalysisResult, String>
```

**Tier values**: `"instant"`, `"fast"`, `"balanced"`, `"thorough"`

### Tag Utilities

```rust
// Normalize comma-separated tags
fn normalize_tags(input: String) -> Vec<String>
```

### API Communication

```rust
// Check API health
pub async fn check_api_health(api_url: String) -> Result<ApiHealthResponse, String>

// Upload file to API endpoint
pub async fn upload_file_to_api(
    api_url: String,
    file_path: String,
    endpoint: String,
) -> Result<String, String>
```

### Configuration

```rust
// Get API configuration
fn get_app_config(state: State<AppState>) -> Result<String, String>

// Set API configuration
fn set_app_config(state: State<AppState>, api_url: String) -> Result<(), String>
```

### Application Info

```rust
// Get app version
fn get_app_version() -> String

// Get app name
fn get_app_name() -> String
```

### Window Management

```rust
// Open DevTools (debug only)
fn open_devtools(window: tauri::Window)
```

---

## Data Structures

### FileInfo
```rust
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub file_type: String,
}
```

### ImageMetadata
```rust
pub struct ImageMetadata {
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub file_size: u64,
    pub file_path: String,
}
```

### Artifact
```rust
pub struct Artifact {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub confidence: f32,
    pub tags: Vec<String>,
    pub image_data: Option<Vec<u8>>,
    pub image_base64: Option<String>,
    pub uploaded_at: String,
}
```

### ArtifactInput
```rust
pub struct ArtifactInput {
    pub name: String,
    pub description: String,
    pub confidence: f32,
    pub tags: Vec<String>,
}
```

### AnalysisResult
```rust
pub struct AnalysisResult {
    pub name: String,
    pub description: String,
    pub confidence: f32,
    pub method: String,      // "vit", "clip", "ollama"
    pub tier: String,        // Analysis tier used
    pub analysis_time: String,
}
```

### SearchResult
```rust
pub struct SearchResult {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub confidence: f32,
    pub tags: Vec<String>,
    pub uploaded_at: String,
    pub image_base64: Option<String>,
}
```

---

## Frontend API Usage (Yew)

### HTTP Requests (gloo_net)
```rust
use gloo_net::http::Request;

// GET request
let response = Request::get("/api/artifacts")
    .send()
    .await?;
let artifacts: Vec<Artifact> = response.json().await?;

// POST request
let response = Request::post("/api/artifacts")
    .json(&artifact_input)?
    .send()
    .await?;

// PUT request
let response = Request::put(&format!("/api/artifacts/{}/tags", id))
    .json(&serde_json::json!({"tags": tags}))?
    .send()
    .await?;

// DELETE request
let response = Request::delete(&format!("/api/artifacts/{}", id))
    .send()
    .await?;
```

### File Handling
```rust
// Get file input from user
let input: HtmlInputElement = element_ref.cast()?;
let files = input.files()?;
let file = files.get(0)?;

// Read as ArrayBuffer
let array_buffer = wasm_bindgen_futures::JsFuture::from(
    file.array_buffer()
).await?;
let uint8 = js_sys::Uint8Array::new(&array_buffer);
let bytes = uint8.to_vec();

// Encode as base64
let base64 = base64::encode(&bytes);
let data_url = format!("data:image/png;base64,{}", base64);
```

### Image Display
```rust
// Display image from base64
html! {
    <img src={image_base64} alt="artifact" />
}

// Display with data URL
html! {
    <img src={format!("data:image/png;base64,{}", base64)} alt="artifact" />
}
```

---

## Common Patterns

### Error Handling
```rust
// In Tauri command
match some_operation() {
    Ok(result) => Ok(result),
    Err(e) => Err(format!("Operation failed: {}", e)),
}

// In Yew component
match api_call.await {
    Ok(data) => {
        // Update component state
    }
    Err(msg) => {
        // Show error to user
        use_callback(move |_| {
            // Show error toast
        })
    }
}
```

### Async Operations
```rust
// Tauri async command
#[tauri::command]
async fn my_command() -> Result<String, String> {
    // async code here
    Ok("result".to_string())
}

// Yew async callback
let handle = use_callback(move |()| {
    ctx.link().send_future(async {
        match api_call().await {
            Ok(data) => Msg::DataReceived(data),
            Err(e) => Msg::Error(e),
        }
    });
}, ());
```

### State with AppState
```rust
// In Tauri command
#[tauri::command]
async fn my_command(
    state: State<'_, AppState>,
) -> Result<String, String> {
    let api_url = state.api_url.lock()
        .map_err(|e| format!("Failed to get state: {}", e))?;
    
    // Use api_url
    Ok(format!("API: {}", *api_url))
}
```

---

## Configuration

### Cargo.toml Dependencies
```toml
[dependencies]
tauri = { version = "2", features = [] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["macros", "rt", "io-util"] }
reqwest = { version = "0.11", features = ["multipart", "json"] }
base64 = "0.21"
urlencoding = "2"

# Optional for image processing
image = "0.24"
```

### Default API URL
```rust
// In lib.rs run() function
let app_state = AppState {
    api_url: Mutex::new("http://localhost:8000/api".to_string()),
    app_name: "FLL Project".to_string(),
};
```

### Change API URL at Runtime
```rust
// Frontend/Yew component
use tauri::api::command;

// Call Tauri command
match command::invoke("set_app_config", &json!({"api_url": "http://new-url:8000/api"})).await {
    Ok(_) => {
        // API URL updated
    }
    Err(e) => {
        // Error updating config
    }
}
```

---

## Debugging

### Enable Console Logging
```rust
// In Yew component
use web_sys::console;

console::log_1(&"Debug message".into());
console::warn_1(&format!("Warning: {}", msg).into());
console::error_1(&format!("Error: {}", error).into());
```

### View Tauri Logs
```bash
# Linux/macOS
RUST_LOG=debug npm run tauri dev

# Windows
$env:RUST_LOG = "debug"
npm run tauri dev
```

### Inspect Request/Response
```rust
// In Tauri command
#[tauri::command]
async fn debug_command(input: String) -> Result<String, String> {
    println!("Input: {}", input);  // stdout
    eprintln!("Debug: {}", input); // stderr
    Ok("logged".to_string())
}
```

---

## Performance Tips

### 1. Reuse HTTP Client
```rust
lazy_static::lazy_static! {
    static ref HTTP_CLIENT: reqwest::Client = {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap()
    };
}
```

### 2. Batch Operations
```rust
// ? Bad: Multiple requests
for id in ids {
    get_artifact_by_id(id).await?;
}

// ? Better: Single request with multiple IDs
get_artifacts_batch(ids).await?
```

### 3. Lazy Load Images
```rust
// Show thumbnail first, load full image on demand
html! {
    <img src={thumbnail_url} alt="artifact" />
    // Lazy load full image on click or scroll
}
```

### 4. Cache Results
```rust
// Use Yew reducer for shared state
pub enum Action {
    SetArtifacts(Vec<Artifact>),
}

pub fn artifacts_reducer(state: Vec<Artifact>, action: Action) -> Vec<Artifact> {
    match action {
        Action::SetArtifacts(artifacts) => artifacts,
    }
}
```

---

## Testing

### Unit Test Template
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_tags() {
        let result = normalize_tags("tag1, tag2, tag3".to_string());
        assert_eq!(result, vec!["tag1", "tag2", "tag3"]);
    }

    #[test]
    fn test_normalize_tags_whitespace() {
        let result = normalize_tags("  tag1  ,  tag2  ".to_string());
        assert_eq!(result, vec!["tag1", "tag2"]);
    }
}
```

### Running Tests
```bash
cargo test                    # All tests
cargo test test_name          # Specific test
cargo test -- --nocapture    # Show println! output
```

---

## Resources

- [Tauri Documentation](https://tauri.app/docs/)
- [Yew Documentation](https://yew.rs/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [FastAPI Documentation](https://fastapi.tiangolo.com/)
- [Reqwest Documentation](https://docs.rs/reqwest/)

---

## Quick Start

### 1. Install Dependencies
```bash
# Node.js and npm
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Tauri CLI
npm install -g @tauri-apps/cli
```

### 2. Clone Project
```bash
git clone https://github.com/kirprap/Fll-Project.git
cd Fll-Project
```

### 3. Install Dependencies
```bash
# Backend
cd backend
pip install -r requirements.txt

# Frontend
cd ../frontend/fll
npm install
```

### 4. Run Development
```bash
# Terminal 1: Backend
cd backend
python -m uvicorn main:app --reload

# Terminal 2: Frontend
cd frontend/fll
npm run tauri dev
```

### 5. Build for Production
```bash
cd frontend/fll
npm run tauri build
```

---

Last Updated: January 2024
