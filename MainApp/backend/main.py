from fastapi import FastAPI, HTTPException, UploadFile, Form, Depends
from fastapi.middleware.cors import CORSMiddleware
from fastapi.staticfiles import StaticFiles
from pydantic import BaseModel
from typing import List, Optional, Dict, Any
import uvicorn
import base64
from io import BytesIO
from PIL import Image
import logging
import sys
import os

# Add project root to path
PROJECT_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if PROJECT_DIR not in sys.path:
    sys.path.insert(0, PROJECT_DIR)

# Import existing functionality
from database import (
    init_db, save_artifact, get_all_artifacts, search_artifacts,
    get_artifact_by_id, update_artifact_tags, delete_artifact, Artifact as DBArtifact
)
from fast_analyzer import FastAnalyzer
from ai_analyzer import AIAnalyzer
import numpy as np

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Initialize FastAPI app
app = FastAPI(title="FLL Project API",
              description="API for the FLL Project",
              version="1.0.0")

# CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Initialize database
init_db()

# Models
class ArtifactBase(BaseModel):
    name: str
    description: Optional[str] = None
    tags: List[str] = []
    tier: str
    image_data: str  # base64 encoded image

class Artifact(ArtifactBase):
    id: int

@app.post("/api/artifacts")
async def create_artifact(artifact: ArtifactBase):
    """Create a new artifact"""
    try:
        # Decode base64 image
        image_data = base64.b64decode(artifact.image_data.split(',')[1] if ',' in artifact.image_data else artifact.image_data)
        
        # Create thumbnail
        image = Image.open(BytesIO(image_data))
        image.thumbnail((200, 200))
        thumbnail_buffer = BytesIO()
        image.save(thumbnail_buffer, format='PNG')
        thumbnail_data = thumbnail_buffer.getvalue()
        
        # Save artifact with both image and thumbnail
        artifact_data = {
            "name": artifact.name,
            "description": artifact.description,
            "tags": ",".join(artifact.tags) if artifact.tags else "",
            "tier": artifact.tier,
        }
        
        artifact_id = save_artifact(artifact_data, image_bytes=image_data, thumbnail_bytes=thumbnail_data)
        
        return {"id": artifact_id, "message": "Artifact created successfully"}
    except Exception as e:
        logger.error(f"Error creating artifact: {str(e)}")
        raise HTTPException(status_code=400, detail=str(e))

@app.get("/api/artifacts")
async def get_all_artifacts_endpoint():
    """Get all artifacts"""
    artifacts = get_all_artifacts()
    return [{
        "id": a["id"],
        "name": a["name"],
        "description": a.get("description"),
        "tags": a["tags"].split(",") if a.get("tags") else [],
        "tier": a.get("tier") or "standard",
        "thumbnail": f"data:image/png;base64,{a['thumbnail']}" if a.get("thumbnail") else None,
        "image_data": f"data:image/png;base64,{a['image_data']}" if a.get("image_data") else None,
        "uploaded_at": a.get("uploaded_at"),
        "analyzed_at": a.get("analyzed_at"),
        "confidence": a.get("confidence"),
    } for a in artifacts]

@app.get("/api/artifacts/search")
async def search_artifacts_endpoint(q: str = ""):
    """Search artifacts by query string"""
    results = search_artifacts(q)
    return [{
        "id": a["id"],
        "name": a["name"],
        "description": a.get("description"),
        "tags": a["tags"].split(",") if a.get("tags") else [],
        "tier": a.get("tier") or "standard",
        "thumbnail": f"data:image/png;base64,{a['thumbnail']}" if a.get("thumbnail") else None,
        "image_data": f"data:image/png;base64,{a['image_data']}" if a.get("image_data") else None,
        "uploaded_at": a.get("uploaded_at"),
        "analyzed_at": a.get("analyzed_at"),
        "confidence": a.get("confidence"),
    } for a in results]

@app.get("/api/artifacts/{artifact_id}")
async def get_artifact(artifact_id: int):
    """Get a single artifact by ID"""
    artifact = get_artifact_by_id(artifact_id)
    if not artifact:
        raise HTTPException(status_code=404, detail="Artifact not found")
    
    return {
        "id": artifact["id"],
        "name": artifact["name"],
        "description": artifact.get("description"),
        "tags": artifact["tags"].split(",") if artifact.get("tags") else [],
        "tier": artifact.get("tier") or "standard",
        "image_data": f"data:image/png;base64,{artifact['image_data']}" if artifact.get("image_data") else None,
        "thumbnail": f"data:image/png;base64,{artifact['thumbnail']}" if artifact.get("thumbnail") else None,
        "uploaded_at": artifact.get("uploaded_at"),
        "analyzed_at": artifact.get("analyzed_at"),
        "confidence": artifact.get("confidence"),
    }


class AnalyzeRequest(BaseModel):
    image_data: str
    tier: Optional[str] = "fast"

class BatchAnalyzeRequest(BaseModel):
    images: List[str]  # List of base64 encoded images
    tier: Optional[str] = "fast"

class SimilaritySearchRequest(BaseModel):
    image_data: str  # base64 encoded image
    limit: Optional[int] = 10


@app.post("/api/analyze")
async def analyze_endpoint(req: AnalyzeRequest):
    """Analyze an uploaded image and return analysis results."""
    try:
        # Decode base64 image (support data URL or raw base64)
        raw = req.image_data
        if "," in raw:
            raw = raw.split(",", 1)[1]
        image_bytes = base64.b64decode(raw)
        image = Image.open(BytesIO(image_bytes)).convert("RGB")

        # Map tier from frontend format to FastAnalyzer tier format
        tier_input = (req.tier or "fast").lower()
        tier_map = {
            "instant": "INSTANT",
            "fast": "FAST",
            "balanced": "BALANCED",
            "thorough": "QUALITY",  # Frontend uses "thorough", FastAnalyzer uses "QUALITY"
        }
        tier = tier_map.get(tier_input, "FAST")

        # Create analyzer with the specified tier
        analyzer = FastAnalyzer(tier=tier)

        # Run analysis (may raise RuntimeError from Ollama client)
        result = analyzer.analyze_artifact(image)

        # Map tier back to frontend format for response
        tier_reverse_map = {
            "INSTANT": "instant",
            "FAST": "fast",
            "BALANCED": "balanced",
            "QUALITY": "thorough",  # FastAnalyzer uses "QUALITY", frontend expects "thorough"
        }
        result_tier = result.get("tier", tier)
        response_tier = tier_reverse_map.get(result_tier, tier_input)

        # Normalize response shape similar to frontend expectations
        response = {
            "name": result.get("name", "Unknown"),
            "description": result.get("description", ""),
            "confidence": float(result.get("confidence", 0.0)),
            "method": result.get("method", "Unknown"),
            "tier": response_tier,
            "analysis_time": result.get("analysis_time", "N/A"),
        }

        # Forward embedding if present
        if "embedding" in result:
            response["embedding"] = result.get("embedding")

        return response

    except RuntimeError as e:
        # Surface runtime errors (e.g., Ollama generation failures)
        logger.error(f"Analysis runtime error: {str(e)}")
        raise HTTPException(status_code=502, detail=str(e))
    except Exception as e:
        logger.exception("Unexpected error during analysis")
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/api/analyze/batch")
async def batch_analyze_endpoint(req: BatchAnalyzeRequest):
    """Analyze multiple uploaded images and return analysis results for each."""
    try:
        results = []
        
        # Map tier from frontend format to FastAnalyzer tier format
        tier_input = (req.tier or "fast").lower()
        tier_map = {
            "instant": "INSTANT",
            "fast": "FAST",
            "balanced": "BALANCED",
            "thorough": "QUALITY",
        }
        tier = tier_map.get(tier_input, "FAST")
        analyzer = FastAnalyzer(tier=tier)
        
        for img_data in req.images:
            try:
                # Decode base64 image
                raw = img_data
                if "," in raw:
                    raw = raw.split(",", 1)[1]
                image_bytes = base64.b64decode(raw)
                image = Image.open(BytesIO(image_bytes)).convert("RGB")
                
                # Run analysis
                result = analyzer.analyze_artifact(image)
                
                # Map tier back to frontend format
                tier_reverse_map = {
                    "INSTANT": "instant",
                    "FAST": "fast",
                    "BALANCED": "balanced",
                    "QUALITY": "thorough",
                }
                result_tier = result.get("tier", tier)
                response_tier = tier_reverse_map.get(result_tier, tier_input)
                
                results.append({
                    "name": result.get("name", "Unknown"),
                    "description": result.get("description", ""),
                    "confidence": float(result.get("confidence", 0.0)),
                    "method": result.get("method", "Unknown"),
                    "tier": response_tier,
                    "analysis_time": result.get("analysis_time", "N/A"),
                })
            except Exception as e:
                logger.error(f"Error processing image in batch: {str(e)}")
                results.append({
                    "error": str(e),
                    "name": "Error",
                    "description": f"Failed to analyze: {str(e)}",
                    "confidence": 0.0,
                })
        
        return {"results": results}
        
    except Exception as e:
        logger.exception("Unexpected error during batch analysis")
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/api/similarity-search")
async def similarity_search_endpoint(req: SimilaritySearchRequest):
    """Find similar artifacts using embedding similarity search."""
    try:
        from ai_analyzer import AIAnalyzer
        
        # Decode query image
        raw = req.image_data
        if "," in raw:
            raw = raw.split(",", 1)[1]
        image_bytes = base64.b64decode(raw)
        image = Image.open(BytesIO(image_bytes)).convert("RGB")
        
        # Get embedding for query image
        analyzer = AIAnalyzer()
        query_result = analyzer.analyze_image(image, model_choice="clip")
        
        if "embedding" not in query_result:
            raise HTTPException(status_code=400, detail="Could not generate embedding for query image")
        
        query_embedding = np.array(query_result["embedding"], dtype=np.float32)
        
        # Get all artifacts with embeddings
        all_artifacts = get_all_artifacts(limit=1000, include_images=False)
        
        # Filter artifacts that have embeddings
        artifacts_with_embeddings = []
        for artifact in all_artifacts:
            # Check if artifact has embedding stored
            # Note: In the current implementation, embeddings are not stored in DB,
            # so this is a placeholder. In production, you'd store embeddings in the DB.
            # For now, we'll use the name/description similarity as a fallback
            artifacts_with_embeddings.append(artifact)
        
        # If no embeddings stored, use text-based similarity as fallback
        if not artifacts_with_embeddings:
            # Fallback to text search based on any available text fields
            search_results = search_artifacts("")
            return [{
                "id": a["id"],
                "name": a["name"],
                "description": a.get("description"),
                "similarity_score": 0.5,  # Placeholder
                "thumbnail": f"data:image/png;base64,{a['thumbnail']}" if a.get("thumbnail") else None,
            } for a in search_results[:req.limit]]
        
        # Perform similarity search using AIAnalyzer's similarity_search method
        search_result = analyzer.similarity_search(query_embedding, artifacts_with_embeddings)
        
        # Format results to match the old Streamlit version
        results = []
        
        # Add closest match if available
        if "closest_match" in search_result and "similarity_score" in search_result:
            # Find the artifact in our list
            for artifact in artifacts_with_embeddings:
                if artifact.get("name") == search_result["closest_match"]:
                    results.append({
                        "id": artifact.get("id"),
                        "name": artifact.get("name", "Unknown"),
                        "description": artifact.get("description"),
                        "similarity_score": search_result["similarity_score"],
                        "thumbnail": f"data:image/png;base64,{artifact['thumbnail']}" if artifact.get("thumbnail") else None,
                    })
                    break
        
        # Add alternative matches
        if "alternative_matches" in search_result:
            for match in search_result["alternative_matches"][:req.limit - 1]:
                artifact_data = match.get("artifact", {})
                results.append({
                    "id": artifact_data.get("id"),
                    "name": artifact_data.get("name", "Unknown"),
                    "description": artifact_data.get("description"),
                    "similarity_score": match.get("score", 0.0),
                    "thumbnail": f"data:image/png;base64,{artifact_data['thumbnail']}" if artifact_data.get("thumbnail") else None,
                })
        
        return results[:req.limit]
        
    except Exception as e:
        logger.exception("Unexpected error during similarity search")
        raise HTTPException(status_code=500, detail=str(e))


@app.patch("/api/artifacts/{artifact_id}/verification")
async def update_artifact_verification(artifact_id: int, verification_status: str):
    """Update verification status of an artifact."""
    try:
        from database import update_artifact_verification
        result = update_artifact_verification(artifact_id, verification_status)
        if not result:
            raise HTTPException(status_code=404, detail="Artifact not found")
        return result
    except Exception as e:
        logger.error(f"Error updating verification status: {str(e)}")
        raise HTTPException(status_code=400, detail=str(e))

if __name__ == "__main__":
    uvicorn.run("backend.main:app", host="0.0.0.0", port=8000, reload=True)

app = FastAPI()

RUST_API = "http://localhost:9000"   # your Rust API base URL


@app.get("/")
async def root():
    return {"python_api": "running", "connected_to": RUST_API}


# -----------------------------
# 🔐 Login (Python → Rust)
# -----------------------------
@app.post("/login")
async def login(user: dict):
    async with httpx.AsyncClient() as client:
        res = await client.post(f"{RUST_API}/login", json=user)
        return res.json()


# -----------------------------
# 🔍 Search User (Python → Rust)
# -----------------------------
@app.post("/search_user")
async def search_user(user: dict):
    async with httpx.AsyncClient() as client:
        res = await client.post(f"{RUST_API}/search_user", json=user)
        return res.json()


# -----------------------------
# 👤 User Info (Python → Rust)
# -----------------------------
@app.post("/user/info")
async def user_info(user: dict):
    async with httpx.AsyncClient() as client:
        res = await client.post(f"{RUST_API}/user/info", json=user)
        return res.json()


# -----------------------------
# 🩺 Health check (Python → Rust)
# -----------------------------
@app.get("/rust_health")
async def rust_health():
    async with httpx.AsyncClient() as client:
        res = await client.get(f"{RUST_API}/health")
        return res.json()

        from fastapi import FastAPI
import httpx

app = FastAPI()

RUST_API = "http://localhost:9000"   # your Rust API base URL


@app.get("/")
async def root():
    return {"python_api": "running", "connected_to": RUST_API}


# -----------------------------
# 🔐 Login (Python → Rust)
# -----------------------------
@app.post("/login")
async def login(user: dict):
    async with httpx.AsyncClient() as client:
        res = await client.post(f"{RUST_API}/login", json=user)
        return res.json()


# -----------------------------
# 🔍 Search User (Python → Rust)
# -----------------------------
@app.post("/search_user")
async def search_user(user: dict):
    async with httpx.AsyncClient() as client:
        res = await client.post(f"{RUST_API}/search_user", json=user)
        return res.json()


# -----------------------------
# 👤 User Info (Python → Rust)
# -----------------------------
@app.post("/user/info")
async def user_info(user: dict):
    async with httpx.AsyncClient() as client:
        res = await client.post(f"{RUST_API}/user/info", json=user)
        return res.json()


# -----------------------------
# 🩺 Health check (Python → Rust)
# -----------------------------
@app.get("/rust_health")
async def rust_health():
    async with httpx.AsyncClient() as client:
        res = await client.get(f"{RUST_API}/health")
        return res.json()
