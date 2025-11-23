//! Archaeology Artifact Identification Application
//!
//! A desktop application for identifying historical artifacts using AI analysis.

use std::time::Duration;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use chrono::Utc;
use dioxus::prelude::*;
use dioxus::html::FileData;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// ----------------------------------------------------------------------------- 
// Error Types
// ----------------------------------------------------------------------------- 

/// Application-level errors
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("API error: {0}")]
    Api(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("File processing error: {0}")]
    FileProcessing(String),
}

/// Result type alias for application operations
pub type AppResult<T> = Result<T, AppError>;

// ----------------------------------------------------------------------------- 
// Data Structures
// ----------------------------------------------------------------------------- 

/// Main application state
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AppState {
    artifacts: Vec<Artifact>,
    current_artifact: Option<Artifact>,
    identified: bool,
    loading: bool,
}

/// Represents an identified historical artifact
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Artifact {
    id: Option<i32>,
    name: String,
    description: String,
    era: String,
    tags: Vec<String>,
    tier: String,
    image_data: String,
    thumbnail: Option<String>,
    uploaded_at: Option<String>,
    analyzed_at: Option<String>,
    confidence: f32,
    method: Option<String>,
    analysis_time: Option<String>,
}

/// Request payload for artifact analysis
#[derive(Serialize)]
struct AnalyzeRequest {
    image_data: String,
    tier: String,
}

/// Request payload for artifact creation
#[derive(Serialize)]
struct CreateArtifactRequest {
    name: String,
    description: String,
    tags: Vec<String>,
    tier: String,
    image_data: String,
}

/// Response from analysis API
#[derive(Deserialize)]
struct AnalyzeResponse {
    name: String,
    description: String,
    era: String,
    confidence: f32,
    method: Option<String>,
    tier: String,
    analysis_time: Option<String>,
}

/// Artifact representation from API
#[derive(Deserialize)]
struct ApiArtifact {
    id: i32,
    name: String,
    description: Option<String>,
    tags: Vec<String>,
    tier: String,
    thumbnail: Option<String>,
    image_data: Option<String>,
    uploaded_at: Option<String>,
    analyzed_at: Option<String>,
    confidence: Option<f32>,
}

// ----------------------------------------------------------------------------- 
// Constants
// ----------------------------------------------------------------------------- 

/// API base URL
const API_BASE_URL: &str = "http://localhost:8000/api";

/// Default analysis tier
const DEFAULT_ANALYSIS_TIER: &str = "fast";

/// Maximum file size for upload (200MB)
const MAX_FILE_SIZE_BYTES: usize = 200 * 1024 * 1024;

// ----------------------------------------------------------------------------- 
// Main Application
// ----------------------------------------------------------------------------- 

fn main() {
    launch(App);
}

/// Root application component
#[component]
fn App() -> Element {
    let state = use_signal(|| AppState::default());

    use_effect(move || {
        to_owned![state];
        spawn(async move {
            if let Err(error) = load_initial_artifacts(state).await {
                log::error!("Failed to load initial artifacts: {}", error);
            }
        });
    });

    rsx! {
        div { class: "app-container",
            AppHeader {}
            AppMainContent { state: state.clone() }
        }
    }
}

// ----------------------------------------------------------------------------- 
// UI Components
// ----------------------------------------------------------------------------- 

/// Application header component
#[component]
fn AppHeader() -> Element {
    rsx! {
        header { class: "app-header",
            h1 { "🏺 Archaeology Artifact Identifier" }
            p { "Upload images to identify historical artifacts using AI analysis" }
        }
    }
}

#[component]
fn AppMainContent(state: Signal<AppState>) -> Element {
    rsx! {
        main { class: "app-main",
            LoadingIndicator { visible: state().loading }
            div { class: "content-grid",
                IdentifyArtifactPanel { state: state.clone() }
                ArtifactArchivePanel { state: state.clone() }
            }
        }
    }
}

#[component]
fn LoadingIndicator(visible: bool) -> Element {
    if !visible {
        return rsx! {}.into();
    }

    rsx! {
        div { class: "loading-indicator",
            p { "Loading artifacts..." }
        }
    }
}

#[component]
fn IdentifyArtifactPanel(state: Signal<AppState>) -> Element {
    let status_message = use_signal(|| "Upload an image to identify artifacts".to_string());
    let is_processing = use_signal(|| false);
    let selected_tier = use_signal(|| DEFAULT_ANALYSIS_TIER.to_string());

    rsx! {
        section { class: "identify-panel",
            IdentifyArtifactHeader {}
            TierSelector { selected_tier: selected_tier.clone() }
            FileUploadArea {
                state: state.clone(),
                status_message: status_message.clone(),
                is_processing: is_processing.clone(),
                selected_tier: selected_tier.clone(),
            }
            ProcessingStatus {
                is_processing: is_processing.clone(),
                status_message: status_message.clone(),
            }
            AnalysisResult { state: state.clone() }
        }
    }
}

#[component]
fn IdentifyArtifactHeader() -> Element {
    rsx! {
        div { class: "panel-header",
            h2 { "🔍 Identify Artifact" }
        }
    }
}

#[component]
fn TierSelector(selected_tier: Signal<String>) -> Element {
    rsx! {
        div { class: "tier-selector",
            label {
                "Analysis Tier: ",
                select {
                    // render actual current value from the signal
                    value: "{selected_tier()}",
                    onchange: move |event| selected_tier.set(event.value().clone()),
                    option { value: "instant", "⚡ Instant" }
                    option { value: "fast", "🚀 Fast" }
                    option { value: "balanced", "⚖️ Balanced" }
                    option { value: "thorough", "🔍 Thorough" }
                }
            }
        }
    }
}

#[component]
fn FileUploadArea(
    state: Signal<AppState>,
    status_message: Signal<String>,
    is_processing: Signal<bool>,
    selected_tier: Signal<String>,
) -> Element {
    let handle_file_select = move |event: Event<FormData>| {
        let files = event.files();
        // files.get(0) returns FileData; clone it to move into task
        if let Some(file) = files.get(0).cloned() {
            process_uploaded_file(
                file,
                state.clone(),
                status_message.clone(),
                is_processing.clone(),
                selected_tier.clone(),
            );
        }
    };

    rsx! {
        div { class: "file-upload-area",
            input {
                r#type: "file",
                accept: "image/*",
                onchange: handle_file_select,
                id: "file-input",
                disabled: "{is_processing()}"
            }
            label {
                r#for: "file-input",
                class: "upload-label",
                div { class: "upload-icon", "📁" }
                p { "Click to upload or drag & drop" }
                p { "Supports JPG, PNG, WebP (max 10MB)" }
            }
        }
    }
}

#[component]
fn ProcessingStatus(is_processing: Signal<bool>, status_message: Signal<String>) -> Element {
    rsx! {
        div { class: "processing-status",
            if is_processing() {
                div { class: "processing-indicator",
                    div { "⏳" }
                    p { "Analyzing artifact..." }
                }
            }
            p { class: "status-message", "{status_message()}" }
        }
    }
}

#[component]
fn AnalysisResult(state: Signal<AppState>) -> Element {
    let state_read = state.read();
    if state_read.current_artifact.is_none() {
        return rsx! {}.into();
    }

    let artifact = state_read.current_artifact.as_ref().unwrap();
    let confidence_percent = artifact.confidence * 100.0;

    rsx! {
        div { class: "analysis-result",
            h3 { "🎯 Identification Result" }
            div { class: "result-content",
                ArtifactImage { artifact: artifact.clone() }
                ArtifactDetails {
                    artifact: artifact.clone(),
                    confidence_percent,
                }
            }
        }
    }
}

#[component]
fn ArtifactImage(artifact: Artifact) -> Element {
    if artifact.image_data.is_empty() {
        return rsx! {
            div { class: "artifact-image-placeholder", "🏺" }
        }.into();
    }

    rsx! {
        img {
            class: "artifact-image",
            src: "{artifact.image_data}",
            width: "200",
            height: "200",
            alt: "Artifact image",
        }
    }
}

#[component]
fn ArtifactDetails(artifact: Artifact, confidence_percent: f32) -> Element {
    rsx! {
        div { class: "artifact-details",
            h4 { "{artifact.name}" }
            p { "📅 Era: {artifact.era}" }
            p { "📝 {artifact.description}" }
            p { "🎯 Confidence: {confidence_percent:.1}%" }
            p { "⚡ Tier: {artifact.tier}" }
            OptionalDetail {
                value: artifact.method.clone(),
                label: "🔧 Method:",
            }
            OptionalDetail {
                value: artifact.analysis_time.clone(),
                label: "⏱️ Analysis Time:",
            }
            ArtifactTags { tags: artifact.tags.clone() }
        }
    }
}

#[component]
fn OptionalDetail(value: Option<String>, label: &'static str) -> Element {
    if let Some(value) = value {
        rsx! {
            p { "{label} {value}" }
        }
    } else {
        rsx! {}.into()
    }
}

#[component]
fn ArtifactTags(tags: Vec<String>) -> Element {
    if tags.is_empty() {
        return rsx! {}.into();
    }

    rsx! {
        div { class: "artifact-tags",
            "🏷️ Tags: ",
            for tag in tags {
                span { class: "tag", "{tag}" }
            }
        }
    }
}

#[component]
fn ArtifactArchivePanel(state: Signal<AppState>) -> Element {
    rsx! {
        section { class: "archive-panel",
            ArchiveHeader {}
            ArchiveControls { state: state.clone() }
            ArtifactGrid { state: state.clone() }
        }
    }
}

#[component]
fn ArchiveHeader() -> Element {
    rsx! {
        div { class: "panel-header",
            h2 { "📚 Artifact Archive" }
        }
    }
}

#[component]
fn ArchiveControls(state: Signal<AppState>) -> Element {
    let search_query = use_signal(|| String::new());
    let filter_era = use_signal(|| "all".to_string());
    let is_searching = use_signal(|| false);

    let handle_search = move |_| {
        to_owned![state, search_query, is_searching];
        spawn(async move {
            if let Err(error) = perform_search(
                search_query(),
                state,
                is_searching,
            ).await {
                log::error!("Search failed: {}", error);
            }
        });
    };

    rsx! {
        div { class: "archive-controls",
            SearchBox {
                query: search_query.clone(),
                on_search: handle_search,
                is_searching: is_searching.clone(),
            }
            EraFilter { current_filter: filter_era.clone() }
            ArtifactCount { state: state.clone() }
        }
    }
}

#[component]
fn SearchBox(
    query: Signal<String>,
    on_search: EventHandler<()>,
    is_searching: Signal<bool>,
) -> Element {
    rsx! {
        div { class: "search-box",
            input {
                r#type: "text",
                placeholder: "Search artifacts...",
                value: "{query()}",
                oninput: move |event| query.set(event.value().clone()),
                onkeypress: move |event| {
                    if event.key() == Key::Enter {
                        on_search.call(());
                    }
                },
            }
            button {
                onclick: move |_| on_search.call(()),
                disabled: "{is_searching()}",
                class: "search-button",
                if is_searching() {
                    "Searching..."
                } else {
                    "🔍 Search"
                }
            }
        }
    }
}

#[component]
fn EraFilter(current_filter: Signal<String>) -> Element {
    rsx! {
        div { class: "era-filter",
            select {
                onchange: move |event| current_filter.set(event.value().clone()),
                option { value: "all", "All Eras" }
                option { value: "ancient", "Ancient" }
                option { value: "medieval", "Medieval" }
                option { value: "renaissance", "Renaissance" }
                option { value: "modern", "Modern" }
            }
        }
    }
}

#[component]
fn ArtifactCount(state: Signal<AppState>) -> Element {
    let total_count = state().artifacts.len();
    let filtered_count = compute_filtered_count(state);

    rsx! {
        div { class: "artifact-count",
            p { "Total artifacts: {total_count}" }
            p { "Showing: {filtered_count}" }
        }
    }
}

#[component]
fn ArtifactGrid(state: Signal<AppState>) -> Element {
    let artifacts = state().artifacts.clone();

    if artifacts.is_empty() {
        return rsx! {
            div { class: "empty-archive",
                p { "No artifacts identified yet. Upload an image to get started!" }
            }
        }.into();
    }

    rsx! {
        div { class: "artifact-grid",
            for artifact in artifacts {
                ArtifactCard {
                    artifact: artifact.clone(),
                    on_delete: move |id| handle_artifact_deletion(id, state.clone()),
                }
            }
        }
    }
}

#[component]
fn ArtifactCard(artifact: Artifact, on_delete: EventHandler<i32>) -> Element {
    rsx! {
        div {
            class: "artifact-card",
            key: "{artifact.id:?}",
            ArtifactCardImage { artifact: artifact.clone() }
            ArtifactCardDetails {
                artifact: artifact.clone(),
                on_delete: on_delete,
            }
        }
    }
}

#[component]
fn ArtifactCardImage(artifact: Artifact) -> Element {
    let image_src = artifact.thumbnail
        .clone()
        .unwrap_or(artifact.image_data.clone());

    if image_src.is_empty() {
        return rsx! {
            div { class: "card-image-placeholder", "🏺" }
        }.into();
    }

    rsx! {
        img {
            class: "card-image",
            src: "{image_src}",
            width: "150",
            height: "150",
            alt: "Artifact thumbnail",
        }
    }
}

#[component]
fn ArtifactCardDetails(artifact: Artifact, on_delete: EventHandler<i32>) -> Element {
    let confidence_percent = artifact.confidence * 100.0;

    rsx! {
        div { class: "card-details",
            h3 { "{artifact.name}" }
            p { "Era: {artifact.era}" }
            p { "{artifact.description}" }
            p { "Confidence: {confidence_percent:.1}%" }
            p { "Tier: {artifact.tier}" }
            UploadTime { uploaded_at: artifact.uploaded_at.clone() }
            CardTags { tags: artifact.tags.clone() }
            DeleteButton {
                artifact_id: artifact.id,
                on_delete: on_delete,
            }
        }
    }
}

#[component]
fn UploadTime(uploaded_at: Option<String>) -> Element {
    if let Some(time) = uploaded_at {
        rsx! {
            p { "Uploaded: {time}" }
        }
    } else {
        rsx! {}.into()
    }
}

#[component]
fn CardTags(tags: Vec<String>) -> Element {
    if tags.is_empty() {
        return rsx! {}.into();
    }

    rsx! {
        div { class: "card-tags",
            for tag in tags {
                span { class: "card-tag", "{tag}" }
            }
        }
    }
}

#[component]
fn DeleteButton(artifact_id: Option<i32>, on_delete: EventHandler<i32>) -> Element {
    if let Some(id) = artifact_id {
        rsx! {
            button {
                class: "delete-button",
                onclick: move |_| on_delete.call(id),
                "🗑️ Delete"
            }
        }
    } else {
        rsx! {}.into()
    }
}

// ----------------------------------------------------------------------------- 
// Business Logic
// ----------------------------------------------------------------------------- 

/// Process an uploaded file for analysis
/// Process an uploaded file for analysis
fn process_uploaded_file(
    mut file: FileData,
    mut state: Signal<AppState>,
    mut status_message: Signal<String>,
    mut is_processing: Signal<bool>,
    mut selected_tier: Signal<String>,
) {
    spawn(async move {
        // Immediately set processing to true
        is_processing.set(true);
        status_message.set("Reading file...".to_string());

        // Read the file bytes
        let bytes = match file.read_bytes().await {
            Ok(b) => b.to_vec(),
            Err(e) => {
                status_message.set(format!("❌ Failed to read file: {}", e));
                is_processing.set(false);
                return;
            }
        };

        // Determine a safe file name
        let file_name_raw = file.name();
        let file_name = if file_name_raw.trim().is_empty() {
            "unknown".to_string()
        } else {
            file_name_raw.clone()
        };

        // Run the main processing pipeline
        let result = handle_file_processing(
            file_name,
            bytes,
            state.clone(),
            status_message.clone(),
            is_processing.clone(),
            selected_tier(),
        )
        .await;

        // ALWAYS reset is_processing at the end
        is_processing.set(false);

        // Show error message if something went wrong
        if let Err(e) = result {
            log::error!("Error processing uploaded file: {:?}", e);
            status_message.set(format!("❌ Error: {}", e));
        }
    });
}

/// Handle file processing pipeline
async fn handle_file_processing(
    mut file_name: String,
    mut file_bytes: Vec<u8>,
    mut state: Signal<AppState>,
    mut status_message: Signal<String>,
    mut is_processing: Signal<bool>,
    mut tier: String,
) -> AppResult<()> {
    // Start processing
    status_message.set("Processing image...".to_string());

    // Call the backend API
    let analysis_result = analyze_artifact_with_api(file_bytes.clone(), tier.clone()).await?;

    // Show the identification result early
    status_message.set(format!(
        "✅ Identified: {} ({:.1}% confidence)",
        analysis_result.name,
        analysis_result.confidence * 100.0
    ));

    // Create artifact object from analysis
    let artifact = create_artifact_from_analysis(file_bytes, analysis_result, tier).await?;

    // Save artifact to backend API
    let saved_artifact = save_artifact_to_api(&artifact).await?;

    // Update UI state with new artifact
    let mut state_write = state.write();
    state_write.current_artifact = Some(saved_artifact.clone());
    state_write.identified = true;
    state_write.artifacts.push(saved_artifact);

    Ok(())
}

/// Create artifact from analysis results
async fn create_artifact_from_analysis(
    file_bytes: Vec<u8>,
    analysis: AnalyzeResponse,
    tier: String,
) -> AppResult<Artifact> {
    // Extract tags BEFORE moving analysis fields
    let tags = extract_tags_from_analysis(&analysis);

    let base64_data = STANDARD.encode(&file_bytes);
    let data_url = format!("data:image/jpeg;base64,{}", base64_data);

    Ok(Artifact {
        id: None,
        name: analysis.name,
        description: analysis.description,
        era: analysis.era,
        tags,
        tier,
        image_data: data_url,
        thumbnail: None,
        uploaded_at: Some(Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()),
        analyzed_at: Some(Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()),
        confidence: analysis.confidence,
        method: analysis.method,
        analysis_time: analysis.analysis_time,
    })
}

/// Update application state with new artifact
fn update_state_with_new_artifact(mut state: Signal<AppState>, artifact: Artifact) {
    let mut state_write = state.write();
    state_write.current_artifact = Some(artifact.clone());
    state_write.identified = true;
    state_write.artifacts.push(artifact);
}

/// Handle artifact deletion
fn handle_artifact_deletion(artifact_id: i32, mut state: Signal<AppState>) {
    spawn(async move {
        if let Err(error) = delete_artifact_from_api(artifact_id).await {
            log::error!("Failed to delete artifact {}: {}", artifact_id, error);
        } else {
            let mut state_write = state.write();
            state_write.artifacts.retain(|a| a.id != Some(artifact_id));
        }
    });
}

/// Perform search operation
async fn perform_search(
    query: String,
    mut state: Signal<AppState>,
    mut is_searching: Signal<bool>,
) -> AppResult<()> {
    is_searching.set(true);

    let artifacts = if query.is_empty() {
        load_artifacts_from_api().await?
    } else {
        search_artifacts_in_api(&query).await?
    };

    state.write().artifacts = artifacts;
    is_searching.set(false);
    Ok(())
}

/// Compute filtered artifact count
fn compute_filtered_count(state: Signal<AppState>) -> usize {
    // In a real implementation, this would apply current filters
    state().artifacts.len()
}

// ----------------------------------------------------------------------------- 
// API Client Functions
// ----------------------------------------------------------------------------- 

/// Load initial artifacts on app startup
async fn load_initial_artifacts(mut state: Signal<AppState>) -> AppResult<()> {
    state.write().loading = true;

    let artifacts = load_artifacts_from_api().await?;

    state.write().artifacts = artifacts;
    state.write().loading = false;
    Ok(())
}

/// Analyze artifact using the API
async fn analyze_artifact_with_api(
    file_bytes: Vec<u8>,
    tier: String,
) -> AppResult<AnalyzeResponse> {
    let client = Client::new();

    let base64_data = STANDARD.encode(&file_bytes);
    let data_url = format!("data:image/jpeg;base64,{}", base64_data);

    let request = AnalyzeRequest {
        image_data: data_url,
        tier,
    };

    let response = client
        .post(&format!("{}/analyze", API_BASE_URL))
        .json(&request)
        .timeout(Duration::from_secs(60))
        .send()
        .await
        .map_err(|e| AppError::Network(e.to_string()))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(AppError::Api(error_text));
    }

    let analysis_result: AnalyzeResponse = response.json()
        .await
        .map_err(|e| AppError::Serialization(e.to_string()))?;

    Ok(analysis_result)
}

/// Save artifact to API
async fn save_artifact_to_api(artifact: &Artifact) -> AppResult<Artifact> {
    let client = Client::new();

    let request = CreateArtifactRequest {
        name: artifact.name.clone(),
        description: artifact.description.clone(),
        tags: artifact.tags.clone(),
        tier: artifact.tier.clone(),
        image_data: artifact.image_data.clone(),
    };

    let response = client
        .post(&format!("{}/artifacts", API_BASE_URL))
        .json(&request)
        .send()
        .await
        .map_err(|e| AppError::Network(e.to_string()))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(AppError::Api(error_text));
    }

    let mut saved_artifact = artifact.clone();
    let created_response: serde_json::Value = response.json()
        .await
        .map_err(|e| AppError::Serialization(e.to_string()))?;

    if let Some(id) = created_response.get("id").and_then(|id| id.as_i64()) {
        saved_artifact.id = Some(id as i32);
    }

    Ok(saved_artifact)
}

/// Load all artifacts from API
async fn load_artifacts_from_api() -> AppResult<Vec<Artifact>> {
    let client = Client::new();

    let response = client
        .get(&format!("{}/artifacts", API_BASE_URL))
        .send()
        .await
        .map_err(|e| AppError::Network(e.to_string()))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(AppError::Api(error_text));
    }

    let api_artifacts: Vec<ApiArtifact> = response.json()
        .await
        .map_err(|e| AppError::Serialization(e.to_string()))?;

    let artifacts: Vec<Artifact> = api_artifacts.into_iter()
        .map(convert_api_artifact_to_domain)
        .collect();

    Ok(artifacts)
}

/// Search artifacts in API
async fn search_artifacts_in_api(query: &str) -> AppResult<Vec<Artifact>> {
    let client = Client::new();

    let response = client
        .get(&format!("{}/artifacts/search", API_BASE_URL))
        .query(&[("q", query)])
        .send()
        .await
        .map_err(|e| AppError::Network(e.to_string()))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(AppError::Api(error_text));
    }

    let api_artifacts: Vec<ApiArtifact> = response.json()
        .await
        .map_err(|e| AppError::Serialization(e.to_string()))?;

    let artifacts: Vec<Artifact> = api_artifacts.into_iter()
        .map(convert_api_artifact_to_domain)
        .collect();

    Ok(artifacts)
}

/// Delete artifact from API
async fn delete_artifact_from_api(artifact_id: i32) -> AppResult<()> {
    let client = Client::new();

    // Note: API endpoint not yet implemented
    log::info!("Delete artifact with ID: {}", artifact_id);

    // Uncomment when DELETE endpoint is available:
    /*
    let response = client
        .delete(&format!("{}/artifacts/{}", API_BASE_URL, artifact_id))
        .send()
        .await
        .map_err(|e| AppError::Network(e.to_string()))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(AppError::Api(error_text));
    }
    */

    Ok(())
}

// ----------------------------------------------------------------------------- 
// Utility Functions
// ----------------------------------------------------------------------------- 

/// Convert API artifact to domain model
fn convert_api_artifact_to_domain(api_artifact: ApiArtifact) -> Artifact {
    let description = api_artifact.description.clone().unwrap_or_default();
    let name = api_artifact.name.clone();

    Artifact {
        id: Some(api_artifact.id),
        name,
        description: description.clone(),
        era: extract_era_from_api_artifact(&api_artifact),
        tags: api_artifact.tags,
        tier: api_artifact.tier,
        image_data: api_artifact.image_data.unwrap_or_default(),
        thumbnail: api_artifact.thumbnail,
        uploaded_at: api_artifact.uploaded_at,
        analyzed_at: api_artifact.analyzed_at,
        confidence: api_artifact.confidence.unwrap_or(0.0),
        method: None,
        analysis_time: None,
    }
}

/// Extract era from artifact description (fallback)
fn extract_era_from_description(description: &str) -> String {
    let description_lower = description.to_lowercase();

    if description_lower.contains("ancient") || description_lower.contains("greek") || description_lower.contains("roman") {
        "Ancient".to_string()
    } else if description_lower.contains("medieval") {
        "Medieval".to_string()
    } else if description_lower.contains("renaissance") {
        "Renaissance".to_string()
    } else if description_lower.contains("modern") {
        "Modern".to_string()
    } else {
        "Unknown".to_string()
    }
}

/// Extract era from API artifact
fn extract_era_from_api_artifact(artifact: &ApiArtifact) -> String {
    // Try to extract from tags first
    for tag in &artifact.tags {
        let tag_lower = tag.to_lowercase();
        if tag_lower.contains("ancient") {
            return "Ancient".to_string();
        }
        if tag_lower.contains("medieval") {
            return "Medieval".to_string();
        }
        if tag_lower.contains("renaissance") {
            return "Renaissance".to_string();
        }
        if tag_lower.contains("modern") {
            return "Modern".to_string();
        }
    }

    // Fall back to description
    if let Some(description) = &artifact.description {
        extract_era_from_description(description)
    } else {
        "Unknown".to_string()
    }
}

/// Extract tags from analysis results
fn extract_tags_from_analysis(analysis: &AnalyzeResponse) -> Vec<String> {
    let mut tags = Vec::new();

    // Add era-based tag
    let era = &analysis.era;
    if era != "Unknown" {
        tags.push(era.clone());
    }

    // Add confidence-based tag
    if analysis.confidence > 0.8 {
        tags.push("High Confidence".to_string());
    } else if analysis.confidence > 0.5 {
        tags.push("Medium Confidence".to_string());
    } else {
        tags.push("Low Confidence".to_string());
    }

    // Add method tag if available
    if let Some(method) = &analysis.method {
        tags.push(method.clone());
    }

    tags
}
