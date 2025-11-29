from typing import Optional

from pydantic import BaseModel


class ArtifactUpdate(BaseModel):
    name: Optional[str] = None
    description: Optional[str] = None
    tags: Optional[str] = None
    form_data: Optional[dict] = None
    verification_status: Optional[str] = None

    class Config:
        extra = "forbid"


def test_pydantic_parsing():
    print("Testing Pydantic model parsing...")

    # Test 1: Name only
    print("\n1. Testing with name only:")
    data1 = {"name": "Updated Name"}
    model1 = ArtifactUpdate(**data1)
    print(f"   Input: {data1}")
    print(f"   Model.dict(): {model1.dict()}")
    print(f"   Model.dict(exclude_unset=True): {model1.dict(exclude_unset=True)}")
    print(f"   Model.name: {model1.name}")

    # Test 2: Multiple fields
    print("\n2. Testing with multiple fields:")
    data2 = {
        "name": "Multi Update",
        "description": "Updated description",
        "tags": "tag1,tag2,tag3",
    }
    model2 = ArtifactUpdate(**data2)
    print(f"   Input: {data2}")
    print(f"   Model.dict(): {model2.dict()}")
    print(f"   Model.dict(exclude_unset=True): {model2.dict(exclude_unset=True)}")

    # Test 3: Form data
    print("\n3. Testing with form data:")
    data3 = {"form_data": {"length": 10.5, "width": 5.2}}
    model3 = ArtifactUpdate(**data3)
    print(f"   Input: {data3}")
    print(f"   Model.dict(): {model3.dict()}")
    print(f"   Model.dict(exclude_unset=True): {model3.dict(exclude_unset=True)}")

    # Test 4: Check field presence
    print("\n4. Checking field presence:")
    data4 = {"name": "Test", "description": ""}
    model4 = ArtifactUpdate(**data4)
    update_dict = model4.dict(exclude_unset=True)
    print(f"   Input: {data4}")
    print(f"   'name' in update_dict: {'name' in update_dict}")
    print(f"   'description' in update_dict: {'description' in update_dict}")
    print(f"   'tags' in update_dict: {'tags' in update_dict}")
    print(f"   update_dict: {update_dict}")


if __name__ == "__main__":
    test_pydantic_parsing()
