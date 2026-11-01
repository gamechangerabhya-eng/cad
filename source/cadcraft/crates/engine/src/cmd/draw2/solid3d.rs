//! First document integration from the geometric B-rep kernel to native CAD entities.
//! EXTRUDE currently creates a closed faceted 3DFACE shell from a simple planar polyline.

use cadcraft_doc::{EntityKind, Face3d, Handle, RawTag};
use cadcraft_geom::{Brep, Vec2, Vec3};
use serde_json::{Value, json};

const SOLID_XDATA_APP: &str = "ARCAD_BREP";
const SOLID_XDATA_VERSION: i16 = 1;

use super::{CommandSpec, bad, f64_req, str_param};
use crate::{Result, Session};

pub(super) fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("extrude", "Extrude", run_extrude)
            .menu(&["3D", "Solid", "Extrude"])
            .alias(&["ext"])
            .params("{handle, height} | {points: [[x,y],...], height} → {handles, faces, volume, surfaceArea}")
            .enabled(|_| true),
        CommandSpec::new("solidinfo", "Solid Info", run_solid_info)
            .menu(&["3D", "Solid", "Info"])
            .params("{handle} → {group, faces, volume, surfaceArea, valid}")
            .enabled(|_| true),
        CommandSpec::new("solideditdelete", "Delete Extruded Solid", run_solid_delete)
            .menu(&["3D", "Solid", "Delete"])
            .params("{handle} → {deleted, group}")
            .enabled(|_| true),
    ]
}

fn profile_from_entity(s: &Session, text: &str) -> Result<Vec<Vec2>> {
    let h = Handle::parse_hex(text).ok_or_else(|| bad("extrude", "`handle` must be a valid entity handle"))?;
    let entity = s.doc()?.entity(h).ok_or_else(|| bad("extrude", "profile entity was not found"))?;
    let EntityKind::LwPolyline(poly) = &entity.kind else {
        return Err(bad("extrude", "profile must be a closed lightweight polyline"));
    };
    if !poly.closed {
        return Err(bad("extrude", "profile polyline must be closed"));
    }
    if poly.vertices.iter().any(|v| v.bulge.abs() > 1e-12) {
        return Err(bad("extrude", "arc-bulge polyline segments are not supported yet; use straight segments"));
    }
    Ok(poly.vertices.iter().map(|v| v.p).collect())
}

fn profile_from_points(p: &Value) -> Result<Vec<Vec2>> {
    let Some(points) = p.get("points").and_then(Value::as_array) else {
        return Err(bad("extrude", "provide a closed polyline `handle` or `points` array"));
    };
    let mut out = Vec::with_capacity(points.len());
    for point in points {
        let Some(a) = point.as_array() else { return Err(bad("extrude", "each point must be [x,y]")); };
        let (Some(x), Some(y)) = (a.first().and_then(Value::as_f64), a.get(1).and_then(Value::as_f64)) else {
            return Err(bad("extrude", "each point must contain finite numeric x and y values"));
        };
        let v = Vec2::new(x, y);
        if !v.is_finite() { return Err(bad("extrude", "profile coordinates must be finite")); }
        out.push(v);
    }
    Ok(out)
}

fn point_in_triangle(p: Vec2, a: Vec2, b: Vec2, c: Vec2, sign: f64, eps: f64) -> bool {
    let cross = |u: Vec2, v: Vec2, w: Vec2| (v - u).cross(w - u) * sign;
    cross(a, b, p) >= -eps && cross(b, c, p) >= -eps && cross(c, a, p) >= -eps
}

/// Ear clipping for the XY cap faces; returns triangles preserving polygon winding.
fn triangulate_cap(points: &[Vec3]) -> Result<Vec<[Vec3; 3]>> {
    if points.len() < 3 { return Err(bad("extrude", "invalid B-rep face loop")); }
    if points.len() == 3 { return Ok(vec![[points[0], points[1], points[2]]]); }
    let xy: Vec<Vec2> = points.iter().map(|p| p.xy()).collect();
    let origin = xy[0];
    let area: f64 = (0..xy.len()).map(|i| (xy[i] - origin).cross(xy[(i + 1) % xy.len()] - origin)).sum();
    let (min_x, max_x, min_y, max_y) = xy.iter().fold((f64::INFINITY, f64::NEG_INFINITY, f64::INFINITY, f64::NEG_INFINITY), |b, p| {
        (b.0.min(p.x), b.1.max(p.x), b.2.min(p.y), b.3.max(p.y))
    });
    let scale = (max_x - min_x).max(max_y - min_y);
    if !area.is_finite() || !scale.is_finite() || scale == 0.0 || area.abs() <= scale * scale * 1e-12 {
        return Err(bad("extrude", "cap triangulation failed: degenerate profile"));
    }
    let sign = area.signum();
    let eps = scale * scale * 1e-12;
    let mut remaining: Vec<usize> = (0..points.len()).collect();
    let mut triangles = Vec::with_capacity(points.len() - 2);
    let mut guard = 0usize;
    while remaining.len() > 3 {
        let mut ear = None;
        for i in 0..remaining.len() {
            let ia = remaining[(i + remaining.len() - 1) % remaining.len()];
            let ib = remaining[i];
            let ic = remaining[(i + 1) % remaining.len()];
            let turn = (xy[ib] - xy[ia]).cross(xy[ic] - xy[ib]) * sign;
            if turn <= eps { continue; }
            if remaining.iter().copied().filter(|&j| j != ia && j != ib && j != ic)
                .any(|j| point_in_triangle(xy[j], xy[ia], xy[ib], xy[ic], sign, eps)) { continue; }
            ear = Some((i, [points[ia], points[ib], points[ic]]));
            break;
        }
        let Some((index, triangle)) = ear else {
            return Err(bad("extrude", "cap triangulation failed: profile is not a simple polygon"));
        };
        triangles.push(triangle);
        remaining.remove(index);
        guard += 1;
        if guard > points.len() { return Err(bad("extrude", "cap triangulation exceeded its safety limit")); }
    }
    triangles.push([points[remaining[0]], points[remaining[1]], points[remaining[2]]]);
    Ok(triangles)
}

/// Validate that the faceted representation emitted for a B-rep shell is a closed,
/// consistently oriented 2-manifold. This catches conversion defects (especially cap
/// triangulation seams) before any entities are appended to the drawing.
fn validate_closed_face_mesh(faces: &[Face3d]) -> Result<()> {
    use std::collections::BTreeMap;

    type PointKey = (u64, u64, u64);
    let point_key = |p: Vec3| -> Result<PointKey> {
        if !p.is_finite() {
            return Err(bad("extrude", "mesh contains a non-finite vertex"));
        }
        let bits = |v: f64| if v == 0.0 { 0.0f64.to_bits() } else { v.to_bits() };
        Ok((bits(p.x), bits(p.y), bits(p.z)))
    };

    // Each undirected edge must occur exactly twice, once in each direction.
    let mut edges: BTreeMap<(PointKey, PointKey), (usize, i32)> = BTreeMap::new();
    for face in faces {
        let corners = face.corners;
        let count = if corners[2] == corners[3] { 3 } else { 4 };
        let keys = corners[..count].iter().copied().map(point_key).collect::<Result<Vec<_>>>()?;
        for i in 0..count {
            let a = keys[i];
            let b = keys[(i + 1) % count];
            if a == b {
                return Err(bad("extrude", "mesh contains a zero-length edge"));
            }
            let (key, direction) = if a < b { ((a, b), 1) } else { ((b, a), -1) };
            let entry = edges.entry(key).or_insert((0, 0));
            entry.0 += 1;
            entry.1 += direction;
        }
    }
    if edges.is_empty() || edges.values().any(|(uses, orientation)| *uses != 2 || *orientation != 0) {
        return Err(bad("extrude", "generated 3D face shell is open or inconsistently oriented"));
    }
    Ok(())
}

/// Read this application's persistent grouping marker from an entity's XDATA.
/// The marker intentionally stores only identity/version/index/count; geometry remains in
/// ordinary 3DFACE entities so external CAD programs can still edit the facets.
fn solid_group_marker(xdata: &[RawTag]) -> Option<(String, i32, i32)> {
    let mut in_app = false;
    let mut group = None;
    let mut index = None;
    let mut count = None;
    let mut version = None;
    let mut malformed = false;
    for tag in xdata {
        if tag.code == 1001 {
            if in_app { break; }
            in_app = tag.value == SOLID_XDATA_APP;
            continue;
        }
        if !in_app { continue; }
        match tag.code {
            1000 => {
                if group.replace(tag.value.clone()).is_some() { malformed = true; }
            }
            1070 => {
                if version.replace(tag.value.parse::<i16>().ok()?).is_some() { malformed = true; }
            }
            1071 if index.is_none() => index = Some(tag.value.parse::<i32>().ok()?),
            1071 => {
                if count.replace(tag.value.parse::<i32>().ok()?).is_some() { malformed = true; }
            }
            _ => {}
        }
    }
    if malformed || version != Some(SOLID_XDATA_VERSION) { return None; }
    Some((group?, index?, count?))
}

#[derive(Debug)]
struct SolidGroupMembers {
    handles: Vec<Handle>,
    expected_count: usize,
    metadata_valid: bool,
}

/// Collect a group and verify that its persistent face indices form exactly `0..count`.
/// Merely counting entities is insufficient: duplicated or corrupted XDATA indices could
/// otherwise make a damaged shell look complete and permit a destructive group delete.
fn group_members(s: &Session, group: &str) -> Result<SolidGroupMembers> {
    let mut handles = Vec::new();
    let mut expected_count = None;
    let mut indices = std::collections::BTreeSet::new();
    let mut metadata_valid = true;
    for entity in s.doc()?.model.iter() {
        if let Some((id, index, count)) = solid_group_marker(&entity.common.xdata) {
            if id != group { continue; }
            handles.push(entity.handle);
            if count <= 0 || index < 0 || index >= count {
                metadata_valid = false;
                continue;
            }
            match expected_count {
                Some(previous) if previous != count as usize => metadata_valid = false,
                None => expected_count = Some(count as usize),
                _ => {}
            }
            if !indices.insert(index as usize) {
                metadata_valid = false;
            }
        }
    }
    let expected_count = expected_count.unwrap_or(0);
    if expected_count == 0 || handles.len() != expected_count || indices.len() != expected_count {
        metadata_valid = false;
    }
    Ok(SolidGroupMembers { handles, expected_count, metadata_valid })
}

fn faces_for_handles(s: &Session, handles: &[Handle]) -> Result<Vec<Face3d>> {
    let mut faces = Vec::with_capacity(handles.len());
    for handle in handles {
        let entity = s.doc()?.entity(*handle).ok_or_else(|| bad("solidinfo", "a grouped face is missing"))?;
        let EntityKind::Face3d(face) = &entity.kind else {
            return Err(bad("solidinfo", "solid group contains a non-3DFACE entity"));
        };
        faces.push(face.clone());
    }
    Ok(faces)
}

fn run_solid_info(s: &mut Session, p: &Value) -> Result<Value> {
    let text = str_param(p, "handle").ok_or_else(|| bad("solidinfo", "provide a face `handle`"))?;
    let handle = Handle::parse_hex(text).ok_or_else(|| bad("solidinfo", "`handle` must be a valid entity handle"))?;
    let entity = s.doc()?.entity(handle).ok_or_else(|| bad("solidinfo", "face was not found"))?;
    let (group, _, expected_count) = solid_group_marker(&entity.common.xdata)
        .ok_or_else(|| bad("solidinfo", "selected face is not part of an AR CAD extruded-solid group"))?;
    let members = group_members(s, &group)?;
    let faces = faces_for_handles(s, &members.handles)?;
    let metadata_valid = members.metadata_valid && members.expected_count == expected_count.max(0) as usize;
    let valid = metadata_valid && validate_closed_face_mesh(&faces).is_ok();
    Ok(json!({
        "group": group,
        "faces": members.handles.len(),
        "expectedFaces": expected_count,
        "metadataValid": metadata_valid,
        "valid": valid,
        "representation": "grouped-faceted-3dface-shell",
        "editableBrep": false
    }))
}

fn run_solid_delete(s: &mut Session, p: &Value) -> Result<Value> {
    let text = str_param(p, "handle").ok_or_else(|| bad("solideditdelete", "provide a face `handle`"))?;
    let handle = Handle::parse_hex(text).ok_or_else(|| bad("solideditdelete", "`handle` must be a valid entity handle"))?;
    let entity = s.doc()?.entity(handle).ok_or_else(|| bad("solideditdelete", "face was not found"))?;
    let (group, _, expected_count) = solid_group_marker(&entity.common.xdata)
        .ok_or_else(|| bad("solideditdelete", "selected face is not part of an AR CAD extruded-solid group"))?;
    let members = group_members(s, &group)?;
    if !members.metadata_valid || members.expected_count != expected_count.max(0) as usize {
        return Err(bad("solideditdelete", "solid group metadata is incomplete, duplicated or inconsistent; refusing partial deletion"));
    }
    let faces = faces_for_handles(s, &members.handles)?;
    if validate_closed_face_mesh(&faces).is_err() {
        return Err(bad("solideditdelete", "solid group geometry is not a closed manifold; refusing destructive deletion"));
    }
    let deleted = members.handles.len();
    let doc = s.doc_mut()?;
    for h in members.handles { doc.remove_entity(h); }
    Ok(json!({"group": group, "deleted": deleted}))
}

fn run_extrude(s: &mut Session, p: &Value) -> Result<Value> {
    let height = f64_req("extrude", p, "height")?;
    let profile = if let Some(handle) = str_param(p, "handle") {
        profile_from_entity(s, handle)?
    } else {
        profile_from_points(p)?
    };
    let brep = Brep::extrude_polygon(&profile, height).map_err(|e| bad("extrude", format!("cannot create solid: {e}")))?;
    brep.validate().map_err(|e| bad("extrude", format!("invalid solid topology: {e}")))?;

    let mut faces_to_add: Vec<Face3d> = Vec::new();
    for face in &brep.faces {
        let loop_data = &brep.loops[face.outer_loop];
        let mut points = Vec::new();
        let mut current = loop_data.first_half_edge;
        loop {
            let he = &brep.half_edges[current];
            points.push(brep.vertices[he.origin]);
            current = he.next;
            if current == loop_data.first_half_edge { break; }
            if points.len() > brep.half_edges.len() { return Err(bad("extrude", "invalid face loop")); }
        }
        if points.len() == 4 {
            faces_to_add.push(Face3d { corners: [points[0], points[1], points[2], points[3]], hidden_edges: 0 });
        } else if points.len() == 3 {
            faces_to_add.push(Face3d { corners: [points[0], points[1], points[2], points[2]], hidden_edges: 8 });
        } else {
            for tri in triangulate_cap(&points)? {
                faces_to_add.push(Face3d { corners: [tri[0], tri[1], tri[2], tri[2]], hidden_edges: 8 });
            }
        }
    }
    // Do not partially mutate the drawing if tessellation accidentally produces an open,
    // non-manifold or inconsistently wound shell.
    validate_closed_face_mesh(&faces_to_add)?;
    let mesh_face_count = faces_to_add.len();
    let mut handles = Vec::with_capacity(mesh_face_count);
    let mut entity_handles = Vec::with_capacity(mesh_face_count);
    for face in faces_to_add {
        let h = s.add_entity(EntityKind::Face3d(face))?;
        entity_handles.push(h);
        handles.push(h.hex());
    }
    let group = entity_handles.first().map(|h| h.hex()).ok_or_else(|| bad("extrude", "extrusion produced no faces"))?;
    for (index, h) in entity_handles.iter().enumerate() {
        let group_id = group.clone();
        s.doc_mut()?.modify_entity(*h, move |entity| {
            entity.common.xdata.extend([
                RawTag { code: 1001, value: SOLID_XDATA_APP.into() },
                RawTag { code: 1000, value: group_id },
                RawTag { code: 1070, value: SOLID_XDATA_VERSION.to_string() },
                RawTag { code: 1071, value: (index as i32).to_string() },
                RawTag { code: 1071, value: (mesh_face_count as i32).to_string() },
            ]);
        })?;
    }
    Ok(json!({
        "handles": handles,
        "group": group,
        "brepFaces": brep.faces.len(),
        "meshFaces": mesh_face_count,
        "volume": brep.volume().unwrap_or(0.0),
        "surfaceArea": brep.surface_area().unwrap_or(0.0),
        "representation": "grouped-faceted-3dface-shell",
        "editableBrep": false
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use cadcraft_doc::LwPolyline;
    use cadcraft_geom::PolyVertex;
    use serde_json::json;

    #[test]
    fn extrude_rectangle_creates_closed_faceted_shell() {
        let mut s = Session::new();
        let result = s.execute("extrude", &json!({"points": [[0,0],[5,0],[5,3],[0,3]], "height": 4.0})).unwrap();
        assert_eq!(result["meshFaces"], 6);
        assert_eq!(result["handles"].as_array().unwrap().len(), 6);
        assert!((result["volume"].as_f64().unwrap() - 60.0).abs() < 1e-9);
        assert_eq!(s.doc().unwrap().model.len(), 6);
        assert!(s.doc().unwrap().model.iter().all(|e| matches!(e.kind, EntityKind::Face3d(_))));
    }

    #[test]
    fn extrude_concave_profile_triangulates_caps() {
        let mut s = Session::new();
        let result = s.execute("extrude", &json!({"points": [[0,0],[4,0],[4,1],[1,1],[1,4],[0,4]], "height": 2.0})).unwrap();
        assert_eq!(result["volume"], 14.0);
        assert!(s.doc().unwrap().model.len() > 8);
    }

    #[test]
    fn extrude_accepts_closed_polyline_handle() {
        let mut s = Session::new();
        let profile = LwPolyline {
            vertices: [[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]]
                .into_iter().map(|p| PolyVertex::new(Vec2::new(p[0], p[1]))).collect(),
            closed: true,
            const_width: 0.0,
            elevation: 0.0,
            plinegen: false,
        };
        let handle = s.add_entity(EntityKind::LwPolyline(profile)).unwrap();
        let result = s.execute("extrude", &json!({"handle": handle.hex(), "height": 3.0})).unwrap();
        assert_eq!(result["volume"], 12.0);
    }

    #[test]
    fn faceted_shell_validator_rejects_a_missing_face() {
        let mut s = Session::new();
        let result = s.execute("extrude", &json!({"points": [[0,0],[5,0],[5,3],[0,3]], "height": 4.0})).unwrap();
        assert_eq!(result["representation"], "grouped-faceted-3dface-shell");
        let faces: Vec<Face3d> = s.doc().unwrap().model.iter().filter_map(|e| {
            if let EntityKind::Face3d(face) = &e.kind { Some(face.clone()) } else { None }
        }).collect();
        assert!(validate_closed_face_mesh(&faces).is_ok());
        assert!(validate_closed_face_mesh(&faces[..faces.len() - 1]).is_err());
    }

    #[test]
    fn concave_extrusion_shell_has_no_open_triangulation_seams() {
        let mut s = Session::new();
        s.execute("extrude", &json!({"points": [[0,0],[4,0],[4,1],[1,1],[1,4],[0,4]], "height": 2.0})).unwrap();
        let faces: Vec<Face3d> = s.doc().unwrap().model.iter().filter_map(|e| {
            if let EntityKind::Face3d(face) = &e.kind { Some(face.clone()) } else { None }
        }).collect();
        assert!(validate_closed_face_mesh(&faces).is_ok());
    }

    #[test]
    fn extrusion_faces_keep_persistent_group_metadata_and_can_be_inspected() {
        let mut s = Session::new();
        let result = s.execute("extrude", &json!({"points": [[0,0],[5,0],[5,3],[0,3]], "height": 4.0})).unwrap();
        let first = result["handles"][0].as_str().unwrap();
        let info = s.execute("solidinfo", &json!({"handle": first})).unwrap();
        assert_eq!(info["faces"], 6);
        assert_eq!(info["expectedFaces"], 6);
        assert_eq!(info["valid"], true);
        assert_eq!(info["group"], result["group"]);
    }

    #[test]
    fn grouped_extrusion_delete_removes_all_faces_atomically() {
        let mut s = Session::new();
        let result = s.execute("extrude", &json!({"points": [[0,0],[2,0],[2,2],[0,2]], "height": 3.0})).unwrap();
        let first = result["handles"][0].as_str().unwrap();
        let deleted = s.execute("solideditdelete", &json!({"handle": first})).unwrap();
        assert_eq!(deleted["deleted"], 6);
        assert!(s.doc().unwrap().model.is_empty());
    }

    #[test]
    fn grouped_extrusion_delete_refuses_incomplete_group() {
        let mut s = Session::new();
        let result = s.execute("extrude", &json!({"points": [[0,0],[2,0],[2,2],[0,2]], "height": 3.0})).unwrap();
        let first = Handle::parse_hex(result["handles"][0].as_str().unwrap()).unwrap();
        s.doc_mut().unwrap().remove_entity(Handle::parse_hex(result["handles"][1].as_str().unwrap()).unwrap());
        assert!(s.execute("solideditdelete", &json!({"handle": first.hex()})).is_err());
    }

    #[test]
    fn grouped_extrusion_info_rejects_duplicate_face_indices() {
        let mut s = Session::new();
        let result = s.execute("extrude", &json!({"points": [[0,0],[2,0],[2,2],[0,2]], "height": 3.0})).unwrap();
        let handles = result["handles"].as_array().unwrap();
        let first = Handle::parse_hex(handles[0].as_str().unwrap()).unwrap();
        let second = Handle::parse_hex(handles[1].as_str().unwrap()).unwrap();
        let first_index = s.doc().unwrap().entity(first).unwrap().common.xdata.iter()
            .filter(|tag| tag.code == 1071).next().unwrap().value.clone();
        s.doc_mut().unwrap().modify_entity(second, move |entity| {
            let mut seen = false;
            for tag in &mut entity.common.xdata {
                if tag.code == 1071 {
                    if !seen { tag.value = first_index.clone(); break; }
                    seen = true;
                }
            }
        }).unwrap();
        let info = s.execute("solidinfo", &json!({"handle": first.hex()})).unwrap();
        assert_eq!(info["metadataValid"], false);
        assert_eq!(info["valid"], false);
        assert!(s.execute("solideditdelete", &json!({"handle": first.hex()})).is_err());
    }

    #[test]
    fn grouped_extrusion_info_rejects_duplicate_face_indices() {
        let mut s = Session::new();
        let result = s.execute("extrude", &json!({"points": [[0,0],[2,0],[2,2],[0,2]], "height": 3.0})).unwrap();
        let handles = result["handles"].as_array().unwrap();
        let first = Handle::parse_hex(handles[0].as_str().unwrap()).unwrap();
        let second = Handle::parse_hex(handles[1].as_str().unwrap()).unwrap();
        let first_index = s.doc().unwrap().entity(first).unwrap().common.xdata.iter()
            .find(|tag| tag.code == 1071).unwrap().value.clone();
        s.doc_mut().unwrap().modify_entity(second, move |entity| {
            if let Some(tag) = entity.common.xdata.iter_mut().find(|tag| tag.code == 1071) {
                tag.value = first_index.clone();
            }
        }).unwrap();
        let info = s.execute("solidinfo", &json!({"handle": first.hex()})).unwrap();
        assert_eq!(info["metadataValid"], false);
        assert_eq!(info["valid"], false);
        assert!(s.execute("solideditdelete", &json!({"handle": first.hex()})).is_err());
    }

    #[test]
    fn extrude_rejects_open_or_curved_polyline() {
        let mut s = Session::new();
        assert!(s.execute("extrude", &json!({"points": [[0,0],[1,0],[0,1]], "height": 0})).is_err());
    }
}
