use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::core::math::{forward_kinematics, Quaternion, Transform};
use crate::core::skeleton::Skeleton;

pub enum ExportFormat {
    Glb,
    Fbx,
}

pub struct ExportParams<'a> {
    pub mesh: &'a serde_json::Value,
    pub skeleton: &'a serde_json::Value,
    #[allow(dead_code)]
    pub clip: Option<&'a serde_json::Value>,
    pub format: ExportFormat,
    pub file_path: String,
}

pub fn export_asset(params: &ExportParams) -> Result<(), String> {
    match params.format {
        ExportFormat::Glb => export_glb(params),
        ExportFormat::Fbx => export_fbx(params),
    }
}

fn pad4(len: usize) -> usize {
    (len + 3) & !3
}

fn write_glb_chunk(w: &mut Vec<u8>, chunk_type: u32, data: &[u8]) {
    let padded_len = pad4(data.len());
    w.extend_from_slice(&(data.len() as u32).to_le_bytes());
    w.extend_from_slice(&chunk_type.to_le_bytes());
    w.extend_from_slice(data);
    let pad = vec![0u8; padded_len - data.len()];
    w.extend_from_slice(&pad);
}

pub fn export_glb(params: &ExportParams) -> Result<(), String> {
    let vertices = params.mesh["vertices"].as_array().ok_or("No vertices")?;
    let indices = params.mesh["indices"].as_array().ok_or("No indices")?;
    let n = vertices.len();
    let m = indices.len();

    let mut pos_data = Vec::with_capacity(n * 12);
    let mut norm_data = Vec::with_capacity(n * 12);
    let mut uv_data = Vec::with_capacity(n * 8);
    let mut joint_data = Vec::with_capacity(n * 8);
    let mut weight_data = Vec::with_capacity(n * 16);

    for v in vertices {
        let p = &v["position"];
        pos_data.extend_from_slice(&f32_to_bytes(p[0].as_f64().unwrap_or(0.0) as f32));
        pos_data.extend_from_slice(&f32_to_bytes(p[1].as_f64().unwrap_or(0.0) as f32));
        pos_data.extend_from_slice(&f32_to_bytes(p[2].as_f64().unwrap_or(0.0) as f32));

        let n_ = &v["normal"];
        norm_data.extend_from_slice(&f32_to_bytes(n_[0].as_f64().unwrap_or(0.0) as f32));
        norm_data.extend_from_slice(&f32_to_bytes(n_[1].as_f64().unwrap_or(0.0) as f32));
        norm_data.extend_from_slice(&f32_to_bytes(n_[2].as_f64().unwrap_or(0.0) as f32));

        let u = &v["uv"];
        uv_data.extend_from_slice(&f32_to_bytes(u[0].as_f64().unwrap_or(0.0) as f32));
        uv_data.extend_from_slice(&f32_to_bytes(u[1].as_f64().unwrap_or(0.0) as f32));

        let bi = &v["bone_indices"];
        for j in 0..4 {
            let val = bi[j].as_u64().unwrap_or(0) as u16;
            joint_data.extend_from_slice(&val.to_le_bytes());
        }

        let bw = &v["bone_weights"];
        weight_data.extend_from_slice(&f32_to_bytes(bw[0].as_f64().unwrap_or(0.0) as f32));
        weight_data.extend_from_slice(&f32_to_bytes(bw[1].as_f64().unwrap_or(0.0) as f32));
        weight_data.extend_from_slice(&f32_to_bytes(bw[2].as_f64().unwrap_or(0.0) as f32));
        weight_data.extend_from_slice(&f32_to_bytes(bw[3].as_f64().unwrap_or(0.0) as f32));
    }

    let mut index_data = Vec::with_capacity(m * 4);
    for i in indices {
        index_data.extend_from_slice(&(i.as_u64().unwrap_or(0) as u32).to_le_bytes());
    }

    let pos_offset = 0u64;
    let norm_offset = pos_offset + pos_data.len() as u64;
    let uv_offset = norm_offset + norm_data.len() as u64;
    let joint_offset = uv_offset + uv_data.len() as u64;
    let weight_offset = joint_offset + joint_data.len() as u64;
    let index_offset = weight_offset + weight_data.len() as u64;

    let skeleton: Skeleton = serde_json::from_value(params.skeleton.clone())
        .map_err(|e| format!("Skeleton parse: {}", e))?;
    let jc = skeleton.joint_count();
    let parent_indices: Vec<i32> = skeleton.parent_indices();

    let rest_local: Vec<Transform> = skeleton.joints.iter().map(|j| {
        Transform {
            translation: (j.local_transform.translation[0], j.local_transform.translation[1], j.local_transform.translation[2]),
            rotation: Quaternion { w: j.local_transform.rotation.w, x: j.local_transform.rotation.x, y: j.local_transform.rotation.y, z: j.local_transform.rotation.z },
            scale: (j.local_transform.scale[0], j.local_transform.scale[1], j.local_transform.scale[2]),
        }
    }).collect();

    let rest_global = forward_kinematics(&rest_local, &parent_indices);
    let inv_bind: Vec<[f32; 16]> = rest_global.iter().map(|t| invert_affine(&t.to_matrix())).collect();

    let mut ibm_data = Vec::with_capacity(jc * 64);
    for mtx in &inv_bind {
        for &v in mtx {
            ibm_data.extend_from_slice(&v.to_le_bytes());
        }
    }

    let ibm_offset = index_offset + index_data.len() as u64;
    let bin_total_with_ibm = ibm_offset + ibm_data.len() as u64;

    let mut json = serde_json::json!({
        "asset": {"version": "2.0", "generator": "muse"},
        "scene": 0,
        "scenes": [{"nodes": [0]}],
    });

    let mut nodes = vec![serde_json::json!({
        "name": "root",
        "children": (1..=jc).collect::<Vec<_>>(),
    })];

    for (i, j) in skeleton.joints.iter().enumerate() {
        let t = &j.local_transform;
        let r = &t.rotation;
        let children: Vec<usize> = (0..jc).filter(|&c| parent_indices[c] == i as i32).collect();
        let mut node = serde_json::json!({
            "name": j.name,
            "translation": [t.translation[0], t.translation[1], t.translation[2]],
            "rotation": [r.x, r.y, r.z, r.w],
        });
        if !children.is_empty() {
            node["children"] = serde_json::json!(children.iter().map(|c| c + 1).collect::<Vec<_>>());
        }
        nodes.push(node);
    }

    json["nodes"] = serde_json::json!(nodes);

    json["meshes"] = serde_json::json!([{
        "primitives": [{
            "attributes": {
                "POSITION": 0,
                "NORMAL": 1,
                "TEXCOORD_0": 2,
                "JOINTS_0": 3,
                "WEIGHTS_0": 4,
            },
            "indices": 5,
        }],
        "name": "character",
    }]);

    json["accessors"] = serde_json::json!([
        {"bufferView": 0, "byteOffset": 0, "type": "VEC3", "componentType": 5126, "count": n},
        {"bufferView": 1, "byteOffset": 0, "type": "VEC3", "componentType": 5126, "count": n},
        {"bufferView": 2, "byteOffset": 0, "type": "VEC2", "componentType": 5126, "count": n},
        {"bufferView": 3, "byteOffset": 0, "type": "VEC4", "componentType": 5123, "count": n},
        {"bufferView": 4, "byteOffset": 0, "type": "VEC4", "componentType": 5126, "count": n},
        {"bufferView": 5, "byteOffset": 0, "type": "SCALAR", "componentType": 5125, "count": m},
        {"bufferView": 6, "byteOffset": 0, "type": "MAT4", "componentType": 5126, "count": jc},
    ]);

    json["bufferViews"] = serde_json::json!([
        {"buffer": 0, "byteOffset": pos_offset, "byteLength": pos_data.len()},
        {"buffer": 0, "byteOffset": norm_offset, "byteLength": norm_data.len()},
        {"buffer": 0, "byteOffset": uv_offset, "byteLength": uv_data.len()},
        {"buffer": 0, "byteOffset": joint_offset, "byteLength": joint_data.len()},
        {"buffer": 0, "byteOffset": weight_offset, "byteLength": weight_data.len()},
        {"buffer": 0, "byteOffset": index_offset, "byteLength": index_data.len()},
        {"buffer": 0, "byteOffset": ibm_offset, "byteLength": ibm_data.len()},
    ]);

    json["buffers"] = serde_json::json!([{"byteLength": bin_total_with_ibm}]);

    json["skins"] = serde_json::json!([{
        "inverseBindMatrices": 6,
        "skeleton": 1,
        "joints": (1..=jc as u64).collect::<Vec<_>>(),
    }]);

    let mut mesh_node = serde_json::json!({"mesh": 0, "skin": 0});
    if jc > 0 {
        mesh_node["children"] = serde_json::json!((1..=jc).collect::<Vec<_>>());
    }
    json["nodes"][0]["children"] = serde_json::json!([1u64]);
    nodes.insert(1, serde_json::json!({
        "name": "character",
        "mesh": 0,
        "skin": 0,
    }));
    json["nodes"] = serde_json::json!(nodes);

    let json_str = serde_json::to_string(&json).map_err(|e| format!("JSON serialize: {}", e))?;
    let json_bytes = json_str.as_bytes();

    let mut bin = Vec::new();
    bin.extend_from_slice(&pos_data);
    bin.extend_from_slice(&norm_data);
    bin.extend_from_slice(&uv_data);
    bin.extend_from_slice(&joint_data);
    bin.extend_from_slice(&weight_data);
    bin.extend_from_slice(&index_data);
    bin.extend_from_slice(&ibm_data);

    let json_chunk_len = pad4(json_bytes.len()) as u64;
    let bin_chunk_len = pad4(bin.len()) as u64;
    let total_len = 12u64 + 8 + json_chunk_len + 8 + bin_chunk_len;

    let mut glb = Vec::with_capacity(total_len as usize);
    glb.extend_from_slice(b"glTF");
    glb.extend_from_slice(&2u32.to_le_bytes());
    glb.extend_from_slice(&(total_len as u32).to_le_bytes());

    write_glb_chunk(&mut glb, 0x4E4F534A, json_bytes);
    write_glb_chunk(&mut glb, 0x004E4942, &bin);

    let path = Path::new(&params.file_path);
    let mut f = File::create(path).map_err(|e| format!("File create: {}", e))?;
    f.write_all(&glb).map_err(|e| format!("File write: {}", e))?;

    Ok(())
}

fn f32_to_bytes(v: f32) -> [u8; 4] {
    v.to_le_bytes()
}

fn invert_affine(m: &[f32; 16]) -> [f32; 16] {
    let r00 = m[0]; let r01 = m[4]; let r02 = m[8];
    let r10 = m[1]; let r11 = m[5]; let r12 = m[9];
    let r20 = m[2]; let r21 = m[6]; let r22 = m[10];
    let t0 = m[3]; let t1 = m[7]; let t2 = m[11];
    [
        r00, r01, r02, 0.0,
        r10, r11, r12, 0.0,
        r20, r21, r22, 0.0,
        -(r00 * t0 + r01 * t1 + r02 * t2),
        -(r10 * t0 + r11 * t1 + r12 * t2),
        -(r20 * t0 + r21 * t1 + r22 * t2),
        1.0,
    ]
}

pub fn export_fbx(params: &ExportParams) -> Result<(), String> {
    let vertices = params.mesh["vertices"].as_array().ok_or("No vertices")?;
    let indices = params.mesh["indices"].as_array().ok_or("No indices")?;
    let n = vertices.len();
    let m = indices.len();

    let skeleton: Skeleton = serde_json::from_value(params.skeleton.clone())
        .map_err(|e| format!("Skeleton parse: {}", e))?;
    let jc = skeleton.joint_count();
    let parent_indices: Vec<i32> = skeleton.parent_indices();

    let rest_local: Vec<Transform> = skeleton.joints.iter().map(|j| {
        Transform {
            translation: (j.local_transform.translation[0], j.local_transform.translation[1], j.local_transform.translation[2]),
            rotation: Quaternion { w: j.local_transform.rotation.w, x: j.local_transform.rotation.x, y: j.local_transform.rotation.y, z: j.local_transform.rotation.z },
            scale: (j.local_transform.scale[0], j.local_transform.scale[1], j.local_transform.scale[2]),
        }
    }).collect();
    let rest_global = forward_kinematics(&rest_local, &parent_indices);

    let mut fbx = String::new();
    fbx.push_str("; FBX 7.4.0 project file\n");
    fbx.push_str("; Generated by Muse\n");

    fbx_node_header(&mut fbx, "FBXHeaderExtension", &[]);
    fbx_prop(&mut fbx, "Creator", "Muse", true);
    fbx_prop_i32(&mut fbx, "FBXVersion", 7400);
    fbx_node_footer(&mut fbx);

    fbx_node_header(&mut fbx, "Objects", &[]);

    for i in 0..n {
        let v = &vertices[i];
        let p = &v["position"];
        fbx.push_str(&format!("\tGeometry:: {}, \"MeshVertex-{}\", \"\" {{\n", i, i));
        fbx.push_str(&format!("\t\tType: \"Mesh\"\n"));
        fbx.push_str(&format!("\t\tVertices: *{} {{\n", 3));
        fbx.push_str(&format!("\t\t\ta: {},{},{}\n", p[0], p[1], p[2]));
        fbx.push_str(&format!("\t\t}}\n"));
        fbx.push_str(&format!("\t}}\n"));
    }

    fbx.push_str(&format!(" Geometry:: {}, \"MeshShape\", \"\" {{\n", n));
    fbx.push_str("\tType: \"Mesh\"\n");

    fbx.push_str(&format!("\tVertices: *{} {{\n", n * 3));
    fbx.push_str("\t\ta: ");
    for (i, v) in vertices.iter().enumerate() {
        let p = &v["position"];
        fbx.push_str(&format!("{},{},{}", p[0], p[1], p[2]));
        if i < n - 1 { fbx.push_str(","); }
    }
    fbx.push_str("\n\t}\n");

    fbx.push_str(&format!("\tPolygonVertexIndex: *{} {{\n", m));
    fbx.push_str("\t\ta: ");
    for i in 0..m / 3 {
        let a = indices[i * 3].as_u64().unwrap_or(0) as i64;
        let b = indices[i * 3 + 1].as_u64().unwrap_or(0) as i64;
        let c = indices[i * 3 + 2].as_u64().unwrap_or(0) as i64;
        fbx.push_str(&format!("{},{},{}", a, b, -(c + 1)));
        if i < m / 3 - 1 { fbx.push_str(","); }
    }
    fbx.push_str("\n\t}\n");

    fbx.push_str(&format!("\tLayerElementNormal: 0 {{\n"));
    fbx.push_str("\t\tType: \"LayerElementNormal\"\n");
    fbx.push_str("\t\tVersion: 101\n");
    fbx.push_str("\t\tName: \"\"\n");
    fbx.push_str("\t\tMappingInformationType: \"ByVertice\"\n");
    fbx.push_str("\t\tReferenceInformationType: \"Direct\"\n");
    fbx.push_str(&format!("\t\tNormals: *{} {{\n", n * 3));
    fbx.push_str("\t\t\ta: ");
    for (i, v) in vertices.iter().enumerate() {
        let no = &v["normal"];
        fbx.push_str(&format!("{},{},{}", no[0], no[1], no[2]));
        if i < n - 1 { fbx.push_str(","); }
    }
    fbx.push_str("\n\t\t}\n\t}\n");

    fbx.push_str(&format!("\tLayerElementUV: 0 {{\n"));
    fbx.push_str("\t\tType: \"LayerElementUV\"\n");
    fbx.push_str("\t\tVersion: 101\n");
    fbx.push_str("\t\tName: \"UVMap\"\n");
    fbx.push_str("\t\tMappingInformationType: \"ByVertice\"\n");
    fbx.push_str("\t\tReferenceInformationType: \"Direct\"\n");
    fbx.push_str(&format!("\t\tUV: *{} {{\n", n * 2));
    fbx.push_str("\t\t\ta: ");
    for (i, v) in vertices.iter().enumerate() {
        let u = &v["uv"];
        fbx.push_str(&format!("{},{}", u[0], u[1]));
        if i < n - 1 { fbx.push_str(","); }
    }
    fbx.push_str("\n\t\t}\n\t}\n");

    fbx.push_str("\tLayer: 0 {\n");
    fbx.push_str("\t\tLayerElement: 0 {\n");
    fbx.push_str("\t\t\tType: \"LayerElementNormal\"\n");
    fbx.push_str("\t\t\tTypedIndex: 0\n");
    fbx.push_str("\t\t}\n");
    fbx.push_str("\t\tLayerElement: 1 {\n");
    fbx.push_str("\t\t\tType: \"LayerElementUV\"\n");
    fbx.push_str("\t\t\tTypedIndex: 0\n");
    fbx.push_str("\t\t}\n");
    fbx.push_str("\t}\n");
    fbx.push_str("}\n");

    for i in 0..jc {
        let gp = &rest_global[i];
        let q = gp.rotation;
        let t = gp.translation;
        fbx.push_str(&format!("\tModel:: {}, \"Model::{}\", \"null\" {{\n", i, skeleton.joints[i].name));
        fbx.push_str("\t\tVersion: 232\n");
        fbx.push_str("\t\tProperties70: {\n");
        fbx.push_str(&format!("\t\t\tP: \"Lcl Translation\", \"Lcl Translation\", \"\", \"A\",{},{},{}\n", t.0, t.1, t.2));
        fbx.push_str(&format!("\t\t\tP: \"Lcl Rotation\", \"Lcl Rotation\", \"\", \"A\",0,0,0\n"));
        fbx.push_str(&format!("\t\t\tP: \"Quaternion\", \"Quaternion\", \"\", \"A\",{},{},{},{}\n", q.x, q.y, q.z, q.w));
        fbx.push_str("\t\t}\n");
        fbx.push_str("\t}\n");
    }

    fbx_node_footer(&mut fbx);

    std::fs::write(&params.file_path, &fbx).map_err(|e| format!("FBX write: {}", e))?;
    Ok(())
}

fn fbx_node_header(fbx: &mut String, name: &str, props: &[&str]) {
    fbx.push_str(&format!("{}: {{", name));
    for p in props {
        fbx.push_str(&format!(" {}", p));
    }
    fbx.push_str("\n");
}

fn fbx_node_footer(fbx: &mut String) {
    fbx.push_str("}\n");
}

fn fbx_prop(fbx: &mut String, name: &str, val: &str, quoted: bool) {
    if quoted {
        fbx.push_str(&format!("\t{}: \"{}\"\n", name, val));
    } else {
        fbx.push_str(&format!("\t{}: {}\n", name, val));
    }
}

fn fbx_prop_i32(fbx: &mut String, name: &str, val: i32) {
    fbx.push_str(&format!("\t{}: {}\n", name, val));
}
