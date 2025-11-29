import json
import logging

import requests

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


# Test the update endpoint
def test_update_artifact():
    base_url = "http://localhost:8000"

    # First, get an artifact to test with
    print("1. Getting artifacts...")
    response = requests.get(f"{base_url}/api/artifacts")
    if response.status_code == 200:
        artifacts = response.json()
        if artifacts:
            artifact_id = artifacts[0]["id"]
            print(f"   Found artifact ID: {artifact_id}")
        else:
            print("   No artifacts found, creating one...")
            # Create a test artifact
            test_artifact = {
                "name": "Test Artifact",
                "description": "Test description",
                "tags": ["test"],
                "tier": "fast",
                "image_data": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==",
            }
            create_response = requests.post(
                f"{base_url}/api/artifacts", json=test_artifact
            )
            if create_response.status_code == 200:
                artifact_id = create_response.json()["id"]
                print(f"   Created artifact ID: {artifact_id}")
            else:
                print(f"   Failed to create artifact: {create_response.text}")
                return
    else:
        print(f"   Failed to get artifacts: {response.text}")
        return

    # Test update with just name
    print("\n2. Testing update with name only...")
    update_data = {"name": "Updated Name"}
    print(f"   Sending: {json.dumps(update_data)}")

    response = requests.put(
        f"{base_url}/api/artifacts/{artifact_id}",
        json=update_data,
        headers={"Content-Type": "application/json"},
    )

    print(f"   Response status: {response.status_code}")
    print(f"   Response body: {response.text}")

    # Test update with multiple fields
    print("\n3. Testing update with multiple fields...")
    update_data = {
        "name": "Multi Update",
        "description": "Updated description",
        "tags": "tag1,tag2,tag3",
    }
    print(f"   Sending: {json.dumps(update_data)}")

    response = requests.put(
        f"{base_url}/api/artifacts/{artifact_id}",
        json=update_data,
        headers={"Content-Type": "application/json"},
    )

    print(f"   Response status: {response.status_code}")
    print(f"   Response body: {response.text}")

    # Test update with form data
    print("\n4. Testing update with form data...")
    update_data = {"form_data": {"length": 10.5, "width": 5.2, "color": "Brown"}}
    print(f"   Sending: {json.dumps(update_data)}")

    response = requests.put(
        f"{base_url}/api/artifacts/{artifact_id}",
        json=update_data,
        headers={"Content-Type": "application/json"},
    )

    print(f"   Response status: {response.status_code}")
    print(f"   Response body: {response.text}")

    # Test verification status update
    print("\n5. Testing verification status update...")
    update_data = {"verification_status": "verified"}
    print(f"   Sending: {json.dumps(update_data)}")

    response = requests.put(
        f"{base_url}/api/artifacts/{artifact_id}",
        json=update_data,
        headers={"Content-Type": "application/json"},
    )

    print(f"   Response status: {response.status_code}")
    print(f"   Response body: {response.text}")

    # Verify the updates worked
    print("\n6. Verifying updates...")
    response = requests.get(f"{base_url}/api/artifacts/{artifact_id}")
    if response.status_code == 200:
        artifact = response.json()
        print(f"   Name: {artifact.get('name')}")
        print(f"   Description: {artifact.get('description')}")
        print(f"   Tags: {artifact.get('tags')}")
        print(f"   Form data: {artifact.get('form_data')}")
        print(f"   Verification status: {artifact.get('verification_status')}")
    else:
        print(f"   Failed to get artifact: {response.text}")


if __name__ == "__main__":
    print("Testing artifact update endpoint...")
    print("Make sure the backend server is running on http://localhost:8000")
    print("=" * 50)

    try:
        test_update_artifact()
    except requests.exceptions.ConnectionError:
        print("\nERROR: Cannot connect to backend server!")
        print("Please start the backend server with:")
        print("  cd MainApp && python -m backend.main")
    except Exception as e:
        print(f"\nERROR: {e}")
