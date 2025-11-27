import base64
import logging
from io import BytesIO

import requests
from PIL import Image, ImageOps

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)
# ============================================================================
# Streamlit UI + Dioxus UI Collab
# ============================================================================

import io

import streamlit as st

# API configuration
API_BASE_URL = "http://localhost:8000"


# API Client Functions
def api_request(method, endpoint, data=None, params=None):
    """Generic API request helper"""
    url = f"{API_BASE_URL}/{endpoint}"
    headers = {"Content-Type": "application/json"}

    try:
        if method.upper() == "GET":
            response = requests.get(url, params=params, headers=headers)
        elif method.upper() == "POST":
            response = requests.post(url, json=data, headers=headers)
        elif method.upper() == "PUT":
            response = requests.put(url, json=data, headers=headers)
        elif method.upper() == "DELETE":
            response = requests.delete(url, headers=headers)
        else:
            raise ValueError(f"Unsupported HTTP method: {method}")

        response.raise_for_status()
        return response.json() if response.content else {}
    except requests.exceptions.RequestException as e:
        logger.error(f"API request failed: {str(e)}")
        raise


def get_artifacts():
    """Get all artifacts"""
    return api_request("GET", "api/artifacts")


def search_artifacts(query):
    """Search artifacts by query"""
    return api_request("GET", "api/artifacts/search", params={"q": query})


def get_artifact(artifact_id):
    """Get a single artifact by ID"""
    return api_request("GET", f"api/artifacts/{artifact_id}")


def create_artifact(name, description, tags, tier, image_data):
    """Create a new artifact"""
    return api_request(
        "POST",
        "api/artifacts",
        {
            "name": name,
            "description": description,
            "tags": tags,
            "tier": tier,
            "image_data": image_data,
        },
    )


def analyze_image(image_data, tier="fast"):
    """Analyze an image"""
    return api_request("POST", "api/analyze", {"image_data": image_data, "tier": tier})


def main():
    """Main Streamlit application."""
    st.set_page_config(
        page_title="Archaeological Artifact Identifier",
        page_icon="🏺",
        layout="wide",
        initial_sidebar_state="expanded",
    )

    st.title("🏺 Artifact Gallery")

    # Query params
    def get_query_params():
        """Helper function to get query params"""
        try:
            return dict(st.query_params)
        except Exception:
            return {
                k: v[0] if isinstance(v, list) and v else v
                for k, v in st.experimental_get_query_params().items()
            }

    def set_qp(**kwargs):
        try:
            st.query_params.clear()
            for k, v in kwargs.items():
                if v is not None:
                    st.query_params[k] = v
        except Exception:
            st.experimental_set_query_params(
                **{k: v for k, v in kwargs.items() if v is not None}
            )

    # Session defaults
    if "view_mode" not in st.session_state:
        st.session_state["view_mode"] = "gallery"  # 'gallery' | 'add'

    # Sync selected artifact from query params
    qp = get_query_params()
    selected_from_qp: str = str(qp.get("artifact", ""))
    if selected_from_qp:
        try:
            st.session_state["selected_artifact"] = int(selected_from_qp)
            st.session_state["view_mode"] = "gallery"
        except (ValueError, TypeError):
            pass

    # Top toolbar
    tb1, tb2, tb3, tb4 = st.columns([0.5, 6, 0.3, 0.3])
    with tb1:
        show_back = st.session_state.get("view_mode") == "add" or (
            "selected_artifact" in st.session_state
        )
        if show_back and st.button("←", use_container_width=True):
            if "selected_artifact" in st.session_state:
                del st.session_state["selected_artifact"]
            st.session_state["view_mode"] = "gallery"
            q = qp.get("q")
            set_qp(q=q)
            # _safe_rerun()
    with tb2:
        search_val = qp.get("q", "")
        new_search = st.text_input(
            "Search",
            value=search_val,
            placeholder="Search by name, description, material, or tags",
            label_visibility="collapsed",
        )
    with tb3:
        if st.button("🔎", use_container_width=True):
            set_qp(q=new_search or None)
            # _safe_rerun()
    with tb4:
        if st.button("➕", use_container_width=True):
            if "selected_artifact" in st.session_state:
                del st.session_state["selected_artifact"]
            st.session_state["view_mode"] = "add"
    st.markdown(
        "<style>"
        'div[data-testid="stHorizontalBlock"] input {height:40px; margin-bottom:0;}'
        "div.stButton > button {height:40px; padding-top:0; padding-bottom:0; margin-bottom:0;}"
        "</style>",
        unsafe_allow_html=True,
    )

    # Route views
    if st.session_state.get("view_mode") == "add":
        identify_artifact_page()
    else:
        archive_page()


def identify_artifact_page():
    """Single artifact identification page."""
    st.header("Identify Single Artifact")

    # Choose source: Upload or Camera
    source = st.radio("Image source", ["Upload", "Camera"], horizontal=True)
    uploaded_file = None
    camera_photo = None
    if source == "Upload":
        uploaded_file = st.file_uploader(
            "Upload an artifact image",
            type=["jpg", "jpeg", "png", "webp"],
            help="Supported formats: JPG, JPEG, PNG, WEBP",
        )
    else:
        camera_photo = st.camera_input(
            "Take a picture (Identify page)", key="identify_camera"
        )

    # Resolve to a PIL image if provided
    image_input = None
    if uploaded_file is not None:
        image_input = Image.open(uploaded_file)
    elif camera_photo is not None:
        image_input = Image.open(camera_photo)

    if image_input is not None:
        # Clear previous results if a different file is uploaded
        file_key = getattr(uploaded_file, "name", None) or "camera_capture"
        if (
            "last_uploaded_file" not in st.session_state
            or st.session_state["last_uploaded_file"] != file_key
        ):
            st.session_state["last_uploaded_file"] = file_key
            if "last_analysis" in st.session_state:
                del st.session_state["last_analysis"]
            if "last_image" in st.session_state:
                del st.session_state["last_image"]

        # Display the uploaded image
        col1, col2 = st.columns([1, 1])

        # Convert image to base64
        buffered = BytesIO()
        image_input.save(buffered, format="PNG")
        img_str = base64.b64encode(buffered.getvalue()).decode()
        img_data_url = f"data:image/png;base64,{img_str}"

        with col1:
            st.subheader("Uploaded Image")
            image = image_input
            st.image(image, width="stretch")

            # Optional cropping UI
            st.markdown("**Crop Options**")
            enable_crop = st.checkbox("Crop image", value=False)
            if enable_crop:
                w, h = image.size
                colc1, colc2 = st.columns(2)
                with colc1:
                    left_pct = st.slider("Left %", 0, 99, 0)
                    top_pct = st.slider("Top %", 0, 99, 0)
                with colc2:
                    right_pct = st.slider("Right %", 1, 100, 100)
                    bottom_pct = st.slider("Bottom %", 1, 100, 100)

                # Ensure valid box
                left = int((left_pct / 100.0) * w)
                top = int((top_pct / 100.0) * h)
                right = int((right_pct / 100.0) * w)
                bottom = int((bottom_pct / 100.0) * h)
                if right <= left:
                    right = left + 1 if left < w else w
                if bottom <= top:
                    bottom = top + 1 if top < h else h

                try:
                    cropped = image.crop((left, top, right, bottom))
                except Exception:
                    cropped = image
                st.image(cropped, caption="Cropped Preview", use_container_width=True)
                image_to_use = cropped
            else:
                image_to_use = image

        with col2:
            st.subheader("Analysis Results")

            # Speed tier selection
            tier = st.selectbox(
                "Analysis Quality",
                ["fast", "balanced", "thorough"],
                index=0,
                help="Faster analysis may be less accurate",
            )

            if st.button("Analyze Artifact"):
                with st.spinner("Analyzing artifact..."):
                    try:
                        # Call the analyze endpoint
                        analysis = analyze_image(img_data_url, tier=tier)
                        st.session_state["last_analysis"] = analysis

                        # Display results
                        st.success("Analysis complete!")
                        st.json(analysis)

                        # Show save form if analysis is successful
                        with st.form("save_artifact"):
                            st.subheader("Save to Archive")
                            name = st.text_input(
                                "Artifact Name", value=analysis.get("name", "")
                            )
                            description = st.text_area(
                                "Description", value=analysis.get("description", "")
                            )
                            tags = st.text_input(
                                "Tags (comma separated)",
                                value=",".join(analysis.get("tags", [])),
                            )

                            if st.form_submit_button("Save Artifact"):
                                try:
                                    result = create_artifact(
                                        name=name,
                                        description=description,
                                        tags=[
                                            t.strip()
                                            for t in tags.split(",")
                                            if t.strip()
                                        ],
                                        tier=tier,
                                        image_data=img_data_url,
                                    )
                                    st.success(
                                        f"Artifact saved with ID: {result.get('id')}"
                                    )
                                except Exception as e:
                                    st.error(f"Failed to save artifact: {str(e)}")

                    except Exception as e:
                        st.error(f"Analysis failed: {str(e)}")
                        logger.exception("Analysis error")

            # Display results if available (persists across reruns)
            if "last_analysis" in st.session_state and "last_image" in st.session_state:
                result = st.session_state["last_analysis"]
                image = st.session_state["last_image"]

                st.success(
                    f"✅ Analysis Complete in {result.get('analysis_time', 'N/A')}!"
                )
                st.markdown(f"**Name:** {result.get('name', 'Unknown')}")
                st.markdown(f"**Description:** {result.get('description', 'N/A')}")
                st.markdown(f"**Confidence:** {result.get('confidence', 0):.2%}")
                st.markdown(f"**Method:** {result.get('method', 'N/A')}")
                st.markdown(f"**Quality Tier:** {result.get('tier', 'N/A')}")

                # Option to save to archive (now outside the analyze button block)
                tags_input = st.text_input(
                    "Tags (comma-separated)", placeholder="e.g. pottery, bronze, burial"
                )
                if st.button("Save to Archive"):
                    try:
                        img_bytes = io.BytesIO()
                        # Use cropped/selected image
                        image_to_save = st.session_state.get("last_image", image_to_use)
                        image_to_save.save(img_bytes, format="PNG")
                        tags_list = (
                            [t.strip() for t in tags_input.split(",") if t.strip()]
                            if tags_input
                            else []
                        )

                        # Convert image to base64 for API
                        img_b64 = base64.b64encode(img_bytes.getvalue()).decode()
                        img_data_url = f"data:image/png;base64,{img_b64}"

                        result_data = create_artifact(
                            name=result.get("name", "Unknown"),
                            description=result.get("description", ""),
                            tags=tags_list,
                            tier=tier,
                            image_data=img_data_url,
                        )
                        artifact_id = result_data.get("id")
                        st.success(
                            f"✅ Artifact saved to archive with ID: {artifact_id}"
                        )
                        st.balloons()
                    except Exception as e:
                        st.error(f"Error saving artifact: {str(e)}")
                        st.exception(e)


def _make_square_thumbnail_b64(img_b64: str, size: int = 300) -> str:
    """Create a square thumbnail from a base64 image string and return base64 PNG."""
    try:
        img_data = base64.b64decode(img_b64)
        with Image.open(io.BytesIO(img_data)) as im:
            im = im.convert("RGBA")
            # Fit into square with padding
            thumb = ImageOps.contain(im, (size, size))
            canvas = Image.new("RGBA", (size, size), (255, 255, 255, 0))
            x = (size - thumb.width) // 2
            y = (size - thumb.height) // 2
            canvas.paste(thumb, (x, y))
            out = io.BytesIO()
            canvas.save(out, format="PNG")
            return base64.b64encode(out.getvalue()).decode("utf-8")
    except Exception:
        return img_b64


def get_query_params() -> dict[str, str]:
    """Helper to get query parameters, always returns dict with string values"""
    try:
        params = dict(st.query_params)
    except Exception:
        params = st.experimental_get_query_params()

    # Normalize values: if it's a list, take first element; ensure strings have .strip() method
    normalized: dict[str, str] = {}
    for k, v in params.items():
        if isinstance(v, list):
            normalized[k] = v[0] if v else ""
        else:
            normalized[k] = str(v) if v is not None else ""
    return normalized


def archive_page():
    """Archive page to view saved artifacts."""
    st.header("Artifact Archive")

    # Get query parameters first
    qp = get_query_params()
    search_query = qp.get("q", "").strip()
    artifact_qp = qp.get("artifact")

    try:
        # Get all artifacts from the API
        if search_query:
            st.subheader(f"Search Results for: {search_query}")
            artifacts = search_artifacts(search_query)
        else:
            artifacts = get_artifacts()

        # Display artifacts in a grid
        if not artifacts:
            st.info("No artifacts found. Add some artifacts to get started!")
            return

        # Group artifacts into rows of 3
        for i in range(0, len(artifacts), 3):
            cols = st.columns(3)
            for j, artifact in enumerate(artifacts[i : i + 3]):
                with cols[j]:
                    image_url = artifact.get("thumbnail") or artifact.get("image_data")
                    if image_url:
                        st.image(
                            image_url,
                            width="stretch",
                            caption=artifact["name"],
                        )
                    else:
                        st.warning(f"No image for {artifact['name']}")
                    st.caption(
                        artifact.get("description", "")[:100]
                        + ("..." if len(artifact.get("description", "")) > 100 else "")
                    )
                    st.caption(f"Tags: {', '.join(artifact.get('tags', []))}")
                    if st.button("View Details", key=f"view_{artifact['id']}"):
                        st.session_state["selected_artifact"] = artifact["id"]
                        st.experimental_rerun()

    except Exception as e:
        st.error(f"Failed to load artifacts: {str(e)}")
        logger.exception("Error in archive_page")

    # Handle artifact selection from query params
    if artifact_qp and "selected_artifact" not in st.session_state:
        try:
            artifact_qp_value = (
                artifact_qp[0] if isinstance(artifact_qp, list) else artifact_qp
            )
            st.session_state["selected_artifact"] = int(artifact_qp_value)
        except (ValueError, TypeError):
            pass

    # If an artifact is selected, show only its detail page
    if "selected_artifact" in st.session_state:
        artifact_id = st.session_state["selected_artifact"]
        try:
            artifact = get_artifact(artifact_id)
            if artifact:
                st.subheader(artifact.get("name", "Artifact"))
                left, right = st.columns([1, 2])
                with left:
                    if artifact.get("image_data"):
                        # Handle base64 image data
                        img_data = artifact.get("image_data")
                        if img_data and img_data.startswith("data:image"):
                            import re

                            # Extract base64 data from data URL
                            base64_data = re.sub(
                                r"^data:image/[^;]+;base64,", "", img_data
                            )
                            img_bytes = base64.b64decode(base64_data)
                            img = Image.open(io.BytesIO(img_bytes))
                            st.image(img, width="stretch")
                with right:
                    st.markdown(
                        f"**Description:** {artifact.get('description') or 'N/A'}"
                    )
                    st.markdown(f"**Tags:** {', '.join(artifact.get('tags', []))}")
                    st.markdown(f"**Tier:** {artifact.get('tier', 'N/A')}")
                    st.markdown(f"**ID:** {artifact.get('id', 'N/A')}")
        except Exception as e:
            st.error(f"Error loading artifact details: {str(e)}")
            logger.exception("Error in artifact detail view")


def search_page():
    """Search page for finding artifacts."""
    st.header("Search Artifacts")

    query = st.text_input("Search artifacts", placeholder="Enter search terms...")

    if query:
        try:
            results = search_artifacts(query)
            if results:
                st.success(f"Found {len(results)} artifacts")
                for artifact in results:
                    with st.expander(
                        f"{artifact.get('name', 'Unknown')} (ID: {artifact.get('id')})"
                    ):
                        image_url = artifact.get("thumbnail") or artifact.get(
                            "image_data"
                        )
                        if image_url:
                            st.image(
                                image_url,
                                width=200,
                            )
                        else:
                            st.warning("No image available")
                        st.markdown(
                            f"**Description:** {artifact.get('description', 'N/A')}"
                        )
                        st.markdown(f"**Tags:** {', '.join(artifact.get('tags', []))}")
            else:
                st.info("No artifacts found matching your search.")
        except Exception as e:
            st.error(f"Error during search: {str(e)}")


if __name__ == "__main__":
    # Add a small delay to ensure backend is ready
    import time

    time.sleep(2)  # Wait 2 seconds for backend to start
    main()
