from __future__ import annotations

from typing import Any, Dict, List, Tuple

from ...animation.math import Quaternion
from ...animation.skeleton import Joint, Skeleton, Transform


def auto_rig(mesh_data: Dict[str, Any]) -> Skeleton:
    joints = [
        Joint(
            name="root",
            parent_index=-1,
            local_transform=Transform(
                translation=(0.0, 0.0, 0.0),
                rotation=Quaternion.identity(),
                scale=(1.0, 1.0, 1.0),
            ),
        ),
        Joint(
            name="spine",
            parent_index=0,
            local_transform=Transform(
                translation=(0.0, 0.5, 0.0),
                rotation=Quaternion.identity(),
                scale=(1.0, 1.0, 1.0),
            ),
        ),
        Joint(
            name="head",
            parent_index=1,
            local_transform=Transform(
                translation=(0.0, 0.3, 0.0),
                rotation=Quaternion.identity(),
                scale=(1.0, 1.0, 1.0),
            ),
        ),
    ]
    return Skeleton(joints=joints, name="auto_rigged")


def compute_skin_weights(
    mesh_data: Dict[str, Any],
    skeleton: Skeleton,
) -> List[Dict]:
    vertices = mesh_data.get("vertices", [])
    weights = []
    for v in vertices:
        weights.append({
            "bone_weights": [1.0, 0.0, 0.0, 0.0],
            "bone_indices": [0, 0, 0, 0],
        })
    return weights
