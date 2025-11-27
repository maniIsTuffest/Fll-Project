//! Archaeology Artifact Identification Application
//!
//! A desktop application for identifying historical artifacts using AI analysis.

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use chrono::Utc;
use dioxus::events::MouseData;
use dioxus::html::FileData;
use dioxus::prelude::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
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
    selected_artifact: Option<Artifact>,
    show_details_modal: bool,
}

/// Represents an identified historical artifact
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Artifact {
    id: Option<i32>,
    name: String,
    description: String,
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
    let mut current_page = use_signal(|| "database".to_string());

    use_effect(move || {
        to_owned![state];
        spawn(async move {
            if let Err(error) = load_initial_artifacts(state).await {
                log::error!("Failed to load initial artifacts: {}", error);
            }
        });
    });

    rsx! {
        style { {STYLES} }
        div { class: "app-container",
            AppHeader { current_page }
            AppMainContent { state, current_page }
        }
    }
}

// -----------------------------------------------------------------------------
// UI Components
// -----------------------------------------------------------------------------

/// Application header component
#[component]
fn AppHeader(mut current_page: Signal<String>) -> Element {
    rsx! {
        header { class: "app-header",
            div { class: "header-top",
                h1 { "🏺 Archaeology Artifact Identifier" }
            }
            nav { class: "app-nav",
                button {
                    class: if current_page() == "database" { "nav-btn active" } else { "nav-btn" },
                    onclick: move |_| current_page.set("database".to_string()),
                    "📚 Database"
                }
                button {
                    class: if current_page() == "analyze" { "nav-btn active" } else { "nav-btn" },
                    onclick: move |_| current_page.set("analyze".to_string()),
                    "🔍 Analyze"
                }
            }
        }
    }
}

#[component]
fn AppMainContent(state: Signal<AppState>, mut current_page: Signal<String>) -> Element {
    let page = current_page();
    rsx! {
        main { class: "app-main",
            LoadingIndicator { visible: state().loading }
            if page == "database" {
                ArtifactArchivePanel { state: state.clone() }
            } else if page == "analyze" {
                IdentifyArtifactPanel { state: state.clone() }
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
        }
        .into();
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
    let search_query = use_signal(|| String::new());
    let filter_era = use_signal(|| "all".to_string());
    let is_searching = use_signal(|| false);

    rsx! {
        section { class: "archive-panel",
            ArchiveHeader {}
            ArchiveControls {
                state: state.clone(),
                search_query: search_query.clone(),
                filter_era: filter_era.clone(),
                is_searching: is_searching.clone(),
            }
            ArtifactGrid {
                state: state.clone(),
                search_query: search_query.clone(),
            }
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
fn ArchiveControls(
    state: Signal<AppState>,
    search_query: Signal<String>,
    filter_era: Signal<String>,
    is_searching: Signal<bool>,
) -> Element {
    let handle_search = move |_| {
        to_owned![state, search_query, is_searching];
        spawn(async move {
            if let Err(error) = perform_search(search_query(), state, is_searching).await {
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
fn ArtifactGrid(state: Signal<AppState>, search_query: Signal<String>) -> Element {
    let artifacts = state().artifacts.clone();

    if artifacts.is_empty() {
        return rsx! {
            div { class: "empty-archive",
                p { "No artifacts identified yet. Upload an image to get started!" }
            }
        }
        .into();
    }

    rsx! {
        div { class: "artifact-grid",
            for artifact in artifacts {
                ArtifactCard {
                    artifact: artifact.clone(),
                    on_delete: move |id| handle_artifact_deletion(id, state.clone()),
                    on_view_details: move |a| {
                        let mut new_state = state();
                        new_state.selected_artifact = Some(a);
                        new_state.show_details_modal = true;
                        state.set(new_state);
                    },
                    on_tag_click: move |tag: String| {
                        search_query.set(tag);
                    },
                }
            }
            if state().show_details_modal {
                ArtifactDetailsModal {
                    artifact: state().selected_artifact.clone(),
                    on_close: move |_| {
                        let mut new_state = state();
                        new_state.show_details_modal = false;
                        state.set(new_state);
                    },
                }
            }
        }
    }
}

#[component]
fn ArtifactCard(
    artifact: Artifact,
    on_delete: EventHandler<i32>,
    on_view_details: EventHandler<Artifact>,
    on_tag_click: EventHandler<String>,
) -> Element {
    rsx! {
        div {
            class: "artifact-card",
            key: "{artifact.id:?}",
            onclick: move |_| on_view_details.call(artifact.clone()),
            ArtifactCardImage { artifact: artifact.clone() }
            ArtifactCardDetails {
                artifact: artifact.clone(),
                on_delete: on_delete,
                on_tag_click: on_tag_click,
            }
        }
    }
}

#[component]
fn ArtifactCardImage(artifact: Artifact) -> Element {
    let image_src = artifact
        .thumbnail
        .clone()
        .unwrap_or(artifact.image_data.clone());

    if image_src.is_empty() {
        return rsx! {
            div { class: "card-image-placeholder", "🏺" }
        }
        .into();
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
fn ArtifactCardDetails(
    artifact: Artifact,
    on_delete: EventHandler<i32>,
    on_tag_click: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "card-details",
            h3 { "{artifact.name}" }
            CardTags {
                tags: artifact.tags.clone(),
                on_tag_click: on_tag_click,
            }
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
fn CardTags(tags: Vec<String>, on_tag_click: EventHandler<String>) -> Element {
    if tags.is_empty() {
        return rsx! {}.into();
    }

    rsx! {
        div { class: "card-tags",
            for tag in tags {
                span {
                    class: "card-tag clickable-tag",
                    onclick: move |event: Event<MouseData>| {
                        event.stop_propagation();
                        on_tag_click.call(tag.clone());
                    },
                    "{tag}"
                }
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
                onclick: move |event: Event<MouseData>| {
                    event.stop_propagation();
                    on_delete.call(id);
                },
                "🗑️ Delete"
            }
        }
    } else {
        rsx! {}.into()
    }
}

#[component]
fn ArtifactDetailsModal(artifact: Option<Artifact>, on_close: EventHandler<()>) -> Element {
    if let Some(artifact) = artifact {
        let confidence_percent = artifact.confidence * 100.0;

        rsx! {
            div {
                class: "modal-overlay",
                onclick: move |_| on_close.call(()),
                div {
                    class: "modal-content",
                    onclick: move |event: Event<MouseData>| {
                        event.stop_propagation();
                    },
                    button {
                        class: "modal-close",
                        onclick: move |_| on_close.call(()),
                        "✕"
                    }
                    div { class: "modal-image",
                        if !artifact.image_data.is_empty() {
                            img {
                                src: "{artifact.image_data}",
                                alt: "Artifact image",
                            }
                        } else {
                            div { class: "image-placeholder", "🏺" }
                        }
                    }
                    div { class: "modal-body",
                        h2 { "{artifact.name}" }
                        div { class: "modal-section",
                            h3 { "Description" }
                            p { "{artifact.description}" }
                        }
                        div { class: "modal-section",
                            h3 { "Details" }
                            p { strong { "Confidence: " } "{confidence_percent:.1}%" }
                            p { strong { "Tier: " } "{artifact.tier}" }
                            if let Some(method) = artifact.method.clone() {
                                p { strong { "Analysis Method: " } "{method}" }
                            }
                            if let Some(time) = artifact.analysis_time.clone() {
                                p { strong { "Analysis Time: " } "{time}" }
                            }
                        }
                        if !artifact.tags.is_empty() {
                            div { class: "modal-section",
                                h3 { "Tags" }
                                div { class: "modal-tags",
                                    for tag in artifact.tags.iter() {
                                        span { class: "modal-tag", "{tag}" }
                                    }
                                }
                            }
                        }
                        if let Some(uploaded) = artifact.uploaded_at.clone() {
                            div { class: "modal-section",
                                p { strong { "Uploaded: " } "{uploaded}" }
                            }
                        }
                    }
                }
            }
        }
    } else {
        rsx! {}.into()
    }
}

// -----------------------------------------------------------------------------
// Business Logic
// -----------------------------------------------------------------------------

fn process_uploaded_file(
    mut file: FileData,
    mut state: Signal<AppState>,
    mut status_message: Signal<String>,
    mut is_processing: Signal<bool>,
    mut selected_tier: Signal<String>,
) {
    spawn(async move {
        is_processing.set(true);
        status_message.set("Reading file...".to_string());

        let bytes = match file.read_bytes().await {
            Ok(b) => b.to_vec(),
            Err(e) => {
                status_message.set(format!("❌ Failed to read file: {}", e));
                is_processing.set(false);
                return;
            }
        };

        let file_name_raw = file.name();
        let file_name = if file_name_raw.trim().is_empty() {
            "unknown".to_string()
        } else {
            file_name_raw.clone()
        };

        let result = handle_file_processing(
            file_name,
            bytes,
            state.clone(),
            status_message.clone(),
            is_processing.clone(),
            selected_tier(),
        )
        .await;

        is_processing.set(false);

        if let Err(e) = result {
            log::error!("Error processing uploaded file: {:?}", e);
            status_message.set(format!("❌ Error: {}", e));
        }
    });
}

async fn handle_file_processing(
    mut file_name: String,
    mut file_bytes: Vec<u8>,
    mut state: Signal<AppState>,
    mut status_message: Signal<String>,
    mut is_processing: Signal<bool>,
    mut tier: String,
) -> AppResult<()> {
    status_message.set("Processing image...".to_string());

    let analysis_result = analyze_artifact_with_api(file_bytes.clone(), tier.clone()).await?;

    status_message.set(format!(
        "✅ Identified: {} ({:.1}% confidence)",
        analysis_result.name,
        analysis_result.confidence * 100.0
    ));

    let artifact = create_artifact_from_analysis(file_bytes, analysis_result, tier).await?;

    let saved_artifact = save_artifact_to_api(&artifact).await?;

    let mut state_write = state.write();
    state_write.current_artifact = Some(saved_artifact.clone());
    state_write.identified = true;
    state_write.artifacts.push(saved_artifact);

    Ok(())
}

async fn create_artifact_from_analysis(
    file_bytes: Vec<u8>,
    analysis: AnalyzeResponse,
    tier: String,
) -> AppResult<Artifact> {
    let tags = extract_tags_from_analysis(&analysis);

    let base64_data = STANDARD.encode(&file_bytes);
    let data_url = format!("data:image/jpeg;base64,{}", base64_data);

    Ok(Artifact {
        id: None,
        name: analysis.name,
        description: analysis.description,
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

fn update_state_with_new_artifact(mut state: Signal<AppState>, artifact: Artifact) {
    let mut state_write = state.write();
    state_write.current_artifact = Some(artifact.clone());
    state_write.identified = true;
    state_write.artifacts.push(artifact);
}

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

fn compute_filtered_count(state: Signal<AppState>) -> usize {
    state().artifacts.len()
}

// -----------------------------------------------------------------------------
// API Client Functions
// -----------------------------------------------------------------------------

async fn load_initial_artifacts(mut state: Signal<AppState>) -> AppResult<()> {
    state.write().loading = true;

    let artifacts = load_artifacts_from_api().await?;

    state.write().artifacts = artifacts;
    state.write().loading = false;
    Ok(())
}

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

    let analysis_result: AnalyzeResponse = response
        .json()
        .await
        .map_err(|e| AppError::Serialization(e.to_string()))?;

    Ok(analysis_result)
}

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
    let created_response: serde_json::Value = response
        .json()
        .await
        .map_err(|e| AppError::Serialization(e.to_string()))?;

    if let Some(id) = created_response.get("id").and_then(|id| id.as_i64()) {
        saved_artifact.id = Some(id as i32);
    }

    Ok(saved_artifact)
}

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

    let api_artifacts: Vec<ApiArtifact> = response
        .json()
        .await
        .map_err(|e| AppError::Serialization(e.to_string()))?;

    let artifacts: Vec<Artifact> = api_artifacts
        .into_iter()
        .map(convert_api_artifact_to_domain)
        .collect();

    Ok(artifacts)
}

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

    let api_artifacts: Vec<ApiArtifact> = response
        .json()
        .await
        .map_err(|e| AppError::Serialization(e.to_string()))?;

    let artifacts: Vec<Artifact> = api_artifacts
        .into_iter()
        .map(convert_api_artifact_to_domain)
        .collect();

    Ok(artifacts)
}

async fn delete_artifact_from_api(artifact_id: i32) -> AppResult<()> {
    let client = Client::new();

    log::info!("Delete artifact with ID: {}", artifact_id);

    Ok(())
}

// -----------------------------------------------------------------------------
// Utility Functions
// -----------------------------------------------------------------------------

fn convert_api_artifact_to_domain(api_artifact: ApiArtifact) -> Artifact {
    let description = api_artifact.description.clone().unwrap_or_default();
    let name = api_artifact.name.clone();

    Artifact {
        id: Some(api_artifact.id),
        name,
        description: description.clone(),
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

fn extract_era_from_description(description: &str) -> String {
    let description_lower = description.to_lowercase();

    if description_lower.contains("ancient")
        || description_lower.contains("greek")
        || description_lower.contains("roman")
    {
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

fn extract_era_from_api_artifact(artifact: &ApiArtifact) -> String {
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

    if let Some(description) = &artifact.description {
        extract_era_from_description(description)
    } else {
        "Unknown".to_string()
    }
}

fn extract_tags_from_analysis(analysis: &AnalyzeResponse) -> Vec<String> {
    let mut tags = Vec::new();

    if analysis.confidence > 0.8 {
        tags.push("High Confidence".to_string());
    } else if analysis.confidence > 0.5 {
        tags.push("Medium Confidence".to_string());
    } else {
        tags.push("Low Confidence".to_string());
    }

    if let Some(method) = &analysis.method {
        tags.push(method.clone());
    }

    tags
}

// ============================================================================
// Styling
// ============================================================================

const STYLES: &str = r#"
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

html, body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
    background: #f8f9fa;
    color: #2d3748;
}

.app-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: #ffffff;
}

.app-header {
    background: linear-gradient(135deg, #1e40af 0%, #3b82f6 50%, #1e3a8a 100%);
    color: white;
    padding: 1.5rem 2rem;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.header-top {
    margin-bottom: 1rem;
}

.header-top h1 {
    font-size: 2rem;
    font-weight: 800;
    letter-spacing: -0.5px;
}

.app-nav {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
}

.nav-btn {
    padding: 0.5rem 1.5rem;
    background: rgba(255, 255, 255, 0.2);
    color: white;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-radius: 8px;
    font-size: 0.95rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.3s ease;
    backdrop-filter: blur(10px);
}

.nav-btn:hover {
    background: rgba(255, 255, 255, 0.3);
    border-color: rgba(255, 255, 255, 0.5);
    transform: translateY(-2px);
}

.nav-btn.active {
    background: rgba(255, 255, 255, 0.95);
    color: #1e40af;
    border-color: white;
    box-shadow: 0 4px 12px rgba(255, 255, 255, 0.3);
}

.app-main {
    flex: 1;
    overflow-y: auto;
    padding: 2rem;
    background: linear-gradient(to bottom, #f8f9fa, #ffffff);
}

.identify-panel,
.archive-panel {
    max-width: 1200px;
    margin: 0 auto;
    background: white;
    border-radius: 12px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
    padding: 2rem;
}

.panel-header {
    margin-bottom: 2rem;
    padding-bottom: 1rem;
    border-bottom: 2px solid #e5e7eb;
}

.panel-header h2 {
    font-size: 1.75rem;
    color: #1e40af;
    font-weight: 700;
}

.tier-selector {
    margin-bottom: 1.5rem;
}

.tier-selector label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 600;
    color: #374151;
}

.tier-selector select {
    width: 100%;
    max-width: 300px;
    padding: 0.75rem;
    border: 2px solid #e5e7eb;
    border-radius: 8px;
    font-size: 1rem;
    background-color: white;
    cursor: pointer;
    transition: border-color 0.3s ease;
}

.tier-selector select:hover,
.tier-selector select:focus {
    outline: none;
    border-color: #3b82f6;
    box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.file-upload-area {
    margin-bottom: 1.5rem;
}

.upload-label {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 3rem;
    border: 3px dashed #d1d5db;
    border-radius: 12px;
    background: linear-gradient(135deg, #f9fafb 0%, #f3f4f6 100%);
    cursor: pointer;
    transition: all 0.3s ease;
}

.upload-label:hover {
    border-color: #3b82f6;
    background: linear-gradient(135deg, #eff6ff 0%, #f0f9ff 100%);
}

.upload-icon {
    font-size: 3rem;
    margin-bottom: 1rem;
}

.upload-label p {
    color: #6b7280;
    font-weight: 500;
    margin: 0.25rem 0;
}

.upload-label p:first-child {
    font-size: 1.1rem;
    color: #374151;
    font-weight: 600;
}

#file-input {
    display: none;
}

.processing-status {
    margin-bottom: 1.5rem;
}

.processing-indicator {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 1rem;
    background: #dbeafe;
    border-radius: 8px;
    color: #1e40af;
    font-weight: 600;
    margin-bottom: 1rem;
}

.processing-indicator div:first-child {
    font-size: 1.5rem;
    animation: spin 2s linear infinite;
}

@keyframes spin {
    to { transform: rotate(360deg); }
}

.status-message {
    color: #6b7280;
    font-size: 0.95rem;
    margin: 0;
}

.analysis-result {
    background: linear-gradient(135deg, #f0f9ff 0%, #e0f2fe 100%);
    border: 2px solid #bfdbfe;
    border-radius: 12px;
    padding: 1.5rem;
    margin-top: 1.5rem;
}

.analysis-result h3 {
    color: #1e40af;
    margin-bottom: 1rem;
    font-size: 1.25rem;
}

.result-content {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 2rem;
}

@media (max-width: 768px) {
    .result-content {
        grid-template-columns: 1fr;
    }
}

.artifact-image {
    width: 100%;
    max-width: 300px;
    height: auto;
    border-radius: 8px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.artifact-image-placeholder {
    width: 300px;
    height: 300px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 4rem;
    background: #e5e7eb;
    border-radius: 8px;
}

.artifact-details h4 {
    font-size: 1.5rem;
    color: #1e40af;
    margin-bottom: 1rem;
}

.artifact-details p {
    margin: 0.5rem 0;
    color: #4b5563;
    line-height: 1.6;
}

.artifact-tags {
    margin-top: 1rem;
}

.tag {
    display: inline-block;
    background: #dbeafe;
    color: #1e40af;
    padding: 0.4rem 0.8rem;
    border-radius: 20px;
    font-size: 0.85rem;
    font-weight: 600;
    margin-right: 0.5rem;
    margin-bottom: 0.5rem;
}

.archive-controls {
    display: grid;
    grid-template-columns: 1fr 200px 150px;
    gap: 1rem;
    margin-bottom: 2rem;
}

@media (max-width: 768px) {
    .archive-controls {
        grid-template-columns: 1fr;
    }
}

.search-box {
    display: flex;
    gap: 0.5rem;
}

.search-box input {
    flex: 1;
    padding: 0.75rem;
    border: 2px solid #e5e7eb;
    border-radius: 8px;
    font-size: 1rem;
    transition: border-color 0.3s ease;
}

.search-box input:focus {
    outline: none;
    border-color: #3b82f6;
    box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.search-button {
    padding: 0.75rem 1.5rem;
    background: #3b82f6;
    color: white;
    border: none;
    border-radius: 8px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.3s ease;
}

.search-button:hover:not(:disabled) {
    background: #1e40af;
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(59, 130, 246, 0.3);
}

.search-button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
}

.era-filter select {
    width: 100%;
    padding: 0.75rem;
    border: 2px solid #e5e7eb;
    border-radius: 8px;
    font-size: 1rem;
    background-color: white;
    cursor: pointer;
    visibility: hidden;
    display: none;
}

.era-filter select:focus {
    outline: none;
    border-color: #3b82f6;
    box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.artifact-count {
    background: #f3f4f6;
    padding: 1rem;
    border-radius: 8px;
    text-align: center;
}

.artifact-count p {
    margin: 0.25rem 0;
    color: #6b7280;
    font-size: 0.9rem;
}

.empty-archive {
    text-align: center;
    padding: 3rem 1rem;
    background: #f9fafb;
    border: 2px dashed #d1d5db;
    border-radius: 12px;
}

.empty-archive p {
    color: #6b7280;
    font-size: 1.1rem;
    margin: 0;
}

.artifact-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 1.5rem;
}

.artifact-card {
    background: white;
    border-radius: 12px;
    overflow: hidden;
    border: 1px solid #e5e7eb;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
    transition: all 0.3s ease;
    display: flex;
    flex-direction: column;
}

.artifact-card:hover {
    transform: translateY(-4px);
    box-shadow: 0 8px 16px rgba(0, 0, 0, 0.1);
    border-color: #3b82f6;
}

.card-image {
    width: 100%;
    height: 200px;
    object-fit: cover;
    background: #f3f4f6;
}

.card-image-placeholder {
    width: 100%;
    height: 200px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 3rem;
    background: linear-gradient(135deg, #f0f9ff 0%, #e0f2fe 100%);
}

.card-details {
    padding: 1rem;
    flex: 1;
    display: flex;
    flex-direction: column;
}

.card-details h3 {
    color: #1e40af;
    font-size: 1.1rem;
    margin: 0 0 0.5rem 0;
    font-weight: 700;
}

.card-details p {
    color: #6b7280;
    font-size: 0.9rem;
    margin: 0.25rem 0;
    line-height: 1.5;
}

.card-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
    margin: 0.75rem 0;
}

.card-tag {
    display: inline-block;
    background: #e0f2fe;
    color: #0369a1;
    padding: 0.25rem 0.6rem;
    border-radius: 16px;
    font-size: 0.75rem;
    font-weight: 600;
}

.clickable-tag {
    cursor: pointer;
    transition: all 0.2s ease;
}

.clickable-tag:hover {
    background: #0ea5e9;
    color: #ffffff;
    transform: scale(1.05);
}

.delete-button {
    margin-top: auto;
    padding: 0.75rem;
    background: #fee2e2;
    color: #dc2626;
    border: none;
    border-radius: 6px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.3s ease;
}

.delete-button:hover {
    background: #fecaca;
    transform: translateY(-1px);
}

.loading-indicator {
    text-align: center;
    padding: 2rem;
    background: #dbeafe;
    border-radius: 12px;
    color: #1e40af;
    font-weight: 600;
    margin-bottom: 1rem;
}

/* Modal Styles */
.modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
}

.modal-content {
    background: white;
    border-radius: 16px;
    max-width: 800px;
    width: 90%;
    max-height: 90vh;
    overflow-y: auto;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
    position: relative;
    display: flex;
    flex-direction: column;
}

.modal-close {
    position: absolute;
    top: 1rem;
    right: 1rem;
    background: #f3f4f6;
    border: none;
    width: 36px;
    height: 36px;
    border-radius: 50%;
    font-size: 1.5rem;
    cursor: pointer;
    transition: all 0.2s ease;
    z-index: 1001;
}

.modal-close:hover {
    background: #e5e7eb;
    transform: rotate(90deg);
}

.modal-image {
    width: 100%;
    height: 300px;
    overflow: hidden;
    border-radius: 16px 16px 0 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: linear-gradient(135deg, #f0f9ff 0%, #e0f2fe 100%);
}

.modal-image img {
    width: 100%;
    height: 100%;
    object-fit: cover;
}

.image-placeholder {
    font-size: 4rem;
    color: #d1d5db;
}

.modal-body {
    padding: 2rem;
    flex: 1;
}

.modal-body h2 {
    color: #1e40af;
    font-size: 1.75rem;
    margin-bottom: 1.5rem;
    border-bottom: 3px solid #3b82f6;
    padding-bottom: 0.75rem;
}

.modal-section {
    margin-bottom: 1.5rem;
}

.modal-section h3 {
    color: #374151;
    font-size: 1.1rem;
    font-weight: 700;
    margin-bottom: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: #1e40af;
}

.modal-section p {
    color: #6b7280;
    line-height: 1.6;
    margin: 0.5rem 0;
}

.modal-section strong {
    color: #374151;
    font-weight: 700;
}

.modal-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
}

.modal-tag {
    display: inline-block;
    background: #dbeafe;
    color: #0369a1;
    padding: 0.5rem 1rem;
    border-radius: 20px;
    font-size: 0.9rem;
    font-weight: 600;
    border: 1px solid #7dd3fc;
}

@media (max-width: 600px) {
    .modal-content {
        width: 95%;
        max-height: 95vh;
    }

    .modal-body {
        padding: 1.5rem;
    }

    .modal-body h2 {
        font-size: 1.5rem;
    }
}
"#;
