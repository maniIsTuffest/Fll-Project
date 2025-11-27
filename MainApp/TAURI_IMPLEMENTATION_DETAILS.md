// ============================================================================
// Tauri Backend Implementation Details
// ============================================================================
// 
// This file documents the implementation of Tauri commands that bridge
// the Yew frontend and Python FastAPI backend.
//
// Key Points:
// 1. All database operations delegate to Python backend via HTTP
// 2. Image processing is simplified (use `image` crate for production)
// 3. AI analysis routes to Python backend (handles Ollama/ViT models)
// 4. State management stores API URL for flexible configuration

// ============================================================================
// Command Flow Examples
// ============================================================================

/*
EXAMPLE 1: Get All Artifacts
==========================

Frontend (Yew):
  1. User navigates to gallery
  2. Component calls: ApiService::get_all_artifacts(Some(50))
  3. Component renders list of artifacts

Backend Flow:
  Frontend (JavaScript/WASM) 
    ? gloo_net HTTP GET
  Python FastAPI (port 8000)
    ? Database query
  SQLite
    ? Returns artifact list
  Python FastAPI
    ? Returns JSON
  Frontend (JavaScript/WASM)
    ? Parse and render


EXAMPLE 2: Analyze Image (with Tauri bridge)
=============================================

If using Tauri Desktop App:

Frontend (Yew WASM)
  1. User uploads image
  2. Encodes to base64
  3. Calls: tauri.invoke('analyze_artifact', { 
       image_base64: "data:...", 
       tier: "fast" 
     })

Tauri Backend (Rust):
  4. Receives command in lib.rs analyze_artifact()
  5. Extracts image_base64 and tier from request
  6. Creates HTTP POST to Python backend:
     POST /api/analyze {
       "image_data": "data:...",
       "tier": "fast"
     }

Python FastAPI:
  7. Receives analysis request
  8. Decodes base64 image
  9. Selects model based on tier:
     - "instant" ? ViT model
     - "fast" ? Small CLIP model
     - "balanced" ? Medium vision model
     - "thorough" ? Large Ollama model
  10. Runs inference
  11. Returns AnalysisResult JSON:
      {
        "name": "Greek Pottery",
        "description": "Terracotta vessel...",
        "confidence": 0.87,
        "method": "clip",
        "tier": "fast",
        "analysis_time": "8.2 seconds"
      }

Tauri Backend (Rust):
  12. Receives response
  13. Deserializes into AnalysisResult struct
  14. Returns to frontend

Frontend (Yew WASM):
  15. Receives AnalysisResult
  16. Displays results to user
  17. User can save to archive


EXAMPLE 3: Save Artifact
=======================

Frontend (Yew):
  1. User fills form:
     - Name: "Greek Pottery"
     - Description: "..."
     - Tags: "pottery, bronze, burial"
     - Image: base64 encoded
  2. Normalizes tags
  3. Calls: ApiService::create_artifact(CreateArtifactRequest { ... })
  
HTTP Client (gloo_net):
  4. POST to Python backend:
     POST /api/artifacts {
       "name": "Greek Pottery",
       "description": "...",
       "tags": ["pottery", "bronze", "burial"],
       "tier": "fast",
       "image_data": "data:image/png;base64,..."
     }

Python FastAPI:
  5. Receives create_artifact request
  6. Decodes base64 image
  7. Creates thumbnail from image (PIL)
  8. Calls database.save_artifact() which:
     - Inserts artifact record
     - Stores image blob
     - Stores thumbnail blob
     - Returns artifact_id
  9. Returns JSON: { "id": 123, "message": "..." }

Frontend (Yew):
  10. Receives artifact_id: 123
  11. Shows success message
  12. Optionally navigates to artifact detail view


EXAMPLE 4: Search Artifacts
===========================

Frontend (Yew):
  1. User enters search query: "pottery"
  2. Clicks search button
  3. Calls: ApiService::search_artifacts("pottery", Some(50))

HTTP Client:
  4. GET to Python backend:
     GET /api/artifacts/search?q=pottery&limit=50

Python FastAPI:
  5. Parses query parameter
  6. Calls database.search_artifacts("pottery")
  7. Database does SQL LIKE query across:
     - name
     - description
     - material
     - cultural_context
     - tags
  8. Returns matching artifacts (up to 50)

Frontend:
  9. Receives results list
  10. Displays in gallery grid
  11. User can click to view details or remove


EXAMPLE 5: Update Tags
======================

Frontend (Yew):
  1. User views artifact detail
  2. Edits tags: "pottery, bronze, burial"
  3. Normalizes: ["pottery", "bronze", "burial"]
  4. Calls: ApiService::update_artifact_tags(123, ["pottery", "bronze", "burial"])

HTTP Client:
  5. PUT to Python backend:
     PUT /api/artifacts/123/tags {
       "tags": ["pottery", "bronze", "burial"]
     }

Python FastAPI:
  6. Receives update request
  7. Calls database.update_artifact_tags(123, tags)
  8. Updates database record
  9. Returns success/error

Frontend:
  10. Shows success notification
  11. Updates displayed tags


EXAMPLE 6: Delete Artifact
==========================

Frontend (Yew):
  1. User clicks delete button on artifact
  2. Shows confirmation dialog
  3. Calls: ApiService::delete_artifact(123)

HTTP Client:
  4. DELETE to Python backend:
     DELETE /api/artifacts/123

Python FastAPI:
  5. Receives delete request
  6. Calls database.delete_artifact(123)
  7. Deletes artifact record and associated image/thumbnail
  8. Returns success

Frontend:
  9. Removes artifact from gallery
  10. Shows success message
*/

// ============================================================================
// Data Structure Mappings
// ============================================================================

/*
Python Database Record ? Rust struct ? Frontend Model

DATABASE (SQLite):
  artifacts table:
  - id (INTEGER PRIMARY KEY)
  - name (TEXT)
  - description (TEXT)
  - tier (TEXT)
  - tags (TEXT) - comma separated
  - image_data (BLOB)
  - thumbnail (BLOB)
  - confidence (FLOAT)
  - uploaded_at (TEXT - ISO8601)
  - analyzed_at (TEXT - ISO8601)

PYTHON Response (FastAPI):
  {
    "id": 123,
    "name": "Greek Pottery",
    "description": "Terracotta vessel from 5th century BC",
    "tier": "fast",
    "tags": ["pottery", "greek", "terracotta"],
    "confidence": 0.87,
    "image_data": "data:image/png;base64,...",
    "thumbnail": "data:image/png;base64,...",
    "uploaded_at": "2024-01-15T10:30:00Z",
    "analyzed_at": "2024-01-15T10:32:00Z"
  }

RUST Struct (lib.rs):
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

RUST SearchResult (for gallery):
  pub struct SearchResult {
      pub id: i64,
      pub name: String,
      pub description: String,
      pub confidence: f32,
      pub tags: Vec<String>,
      pub uploaded_at: String,
      pub image_base64: Option<String>,
  }

RUST AnalysisResult (AI output):
  pub struct AnalysisResult {
      pub name: String,
      pub description: String,
      pub confidence: f32,
      pub method: String,        // "vit", "clip", "ollama"
      pub tier: String,          // "instant", "fast", "balanced", "thorough"
      pub analysis_time: String, // "8.2 seconds"
  }

FRONTEND Artifact (models.rs):
  pub struct Artifact {
      pub id: Option<i32>,
      pub name: String,
      pub description: String,
      pub tags: Vec<String>,
      pub tier: String,
      pub image_data: Option<String>,      // base64
      pub thumbnail: Option<String>,       // base64
      pub uploaded_at: Option<DateTime<Utc>>,
      pub analyzed_at: Option<DateTime<Utc>>,
  }

Conversion chain:
  Python JSON
    ? (reqwest deserialize)
  Rust struct (Artifact/SearchResult)
    ? (impl From<...>)
  Frontend Artifact
    ? (Yew component display)
  HTML/CSS rendering
*/

// ============================================================================
// Error Handling Strategy
// ============================================================================

/*
Error Propagation:

Tauri Command:
  pub async fn get_artifact_by_id(
      state: State<'_, AppState>,
      artifact_id: i64,
  ) -> Result<Artifact, String> {
      //                           ? Error is serialized as String
      // If Ok: returns Artifact as JSON
      // If Err: returns error string to frontend
  }

Frontend (Yew):
  match ApiService::get_artifact(id).await {
      Ok(artifact) => {
          // Display artifact
      }
      Err(msg) => {
          // Show error toast: "Failed to fetch artifact: ..."
      }
  }

Common Errors:
  - "Path is not a file"
  - "Failed to read directory"
  - "Failed to decode image"
  - "Failed to reach API"
  - "Failed to parse response"
  - "Analysis timeout - model is taking too long"
  - "Ollama Connection Error" (from Python backend)

Ollama Errors:
  - Check if Ollama is running: `ollama serve`
  - Check if model is downloaded: `ollama list`
  - Pull model: `ollama pull qwen3-vl:32b`
  - Default timeout: 300 seconds (5 minutes)
  - If longer, increase timeout in analyze_artifact()
*/

// ============================================================================
// Configuration & State Management
// ============================================================================

/*
AppState (Tauri):
  pub struct AppState {
      pub api_url: Mutex<String>,  // "http://localhost:8000/api"
      pub app_name: String,         // "FLL Project"
  }

Initialization in run():
  let app_state = AppState {
      api_url: Mutex::new("http://localhost:8000/api".to_string()),
      app_name: "FLL Project".to_string(),
  };
  
  tauri::Builder::default()
      .manage(app_state)
      ...

Usage in commands:
  #[tauri::command]
  async fn get_all_artifacts(
      state: State<'_, AppState>,
      limit: Option<i32>,
  ) -> Result<Vec<SearchResult>, String> {
      let api_url = state.api_url.lock()
          .map_err(|e| format!("Failed to access API URL: {}", e))?;
      
      let url = format!("{}/artifacts?limit={}", *api_url, limit.unwrap_or(50));
      // ... make HTTP request
  }

Configuration at runtime:
  #[tauri::command]
  fn set_app_config(state: State<AppState>, api_url: String) -> Result<(), String> {
      let mut config = state.api_url.lock()?;
      *config = api_url;
      Ok(())
  }

This allows:
  1. Development: localhost:8000
  2. Production: remote API server
  3. Testing: mock API server
*/

// ============================================================================
// Performance Considerations
// ============================================================================

/*
HTTP Connection Pooling:
  // ? Bad: Creates new client per request
  let client = reqwest::Client::new();
  
  // ? Better: Reuse client
  // Wrap in Arc<Mutex<>> or lazy_static
  lazy_static::lazy_static! {
      static ref HTTP_CLIENT: reqwest::Client = reqwest::Client::new();
  }
  
  // Then use: HTTP_CLIENT.get(...).send().await

Timeouts:
  Analysis: 300 seconds (large Ollama models)
  API Health: 5 seconds
  File operations: No timeout (local I/O)
  
Image Processing:
  // Current: Placeholder
  // Future: Use `image` crate for real processing
  
  #[tauri::command]
  async fn crop_image(request: ImageCropRequest) -> Result<String, String> {
      let image_bytes = base64::decode(&request.image_base64)?;
      
      // TODO: Implement with `image` crate
      let image = image::load_from_memory(&image_bytes)?;
      let width = image.width() as f32;
      let height = image.height() as f32;
      
      let left = (request.left_pct / 100.0) * width as f32;
      let cropped = image.crop_imm(left as u32, ...);
      
      // Encode back to base64
      let mut buf = Vec::new();
      cropped.write_to(&mut Cursor::new(&mut buf), ImageFormat::PNG)?;
      Ok(base64::encode(&buf))
  }

Memory:
  - Base64 encoding increases size by ~33%
  - Keep large images compressed
  - Stream large files instead of loading fully
  - Use thumbnails in gallery (300x300)
*/

// ============================================================================
// Testing Strategy
// ============================================================================

/*
Unit Tests (Rust):
  #[cfg(test)]
  mod tests {
      use super::*;
      
      #[test]
      fn test_normalize_tags() {
          let result = normalize_tags("pottery, bronze, burial".to_string());
          assert_eq!(result, vec!["pottery", "bronze", "burial"]);
      }
      
      #[test]
      fn test_normalize_tags_empty() {
          let result = normalize_tags("".to_string());
          assert!(result.is_empty());
      }
  }

Integration Tests (Frontend):
  // Mock API server
  // Test artifact CRUD operations
  // Test search functionality
  // Test error handling

Manual Testing:
  1. Start Python backend
  2. Start Tauri dev mode
  3. Test each feature:
     - View gallery
     - Upload image
     - Analyze artifact
     - Save to archive
     - Search artifacts
     - Edit tags
     - Delete artifact

End-to-End Testing:
  - Cypress or Playwright for UI automation
  - Test user workflows
  - Test error scenarios
*/
