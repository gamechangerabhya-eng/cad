//! Boundary-representation (B-rep) topology foundation for 3D CAD.
//!
//! This module deliberately starts with validated planar faces and polygonal prisms. It is not
//! yet a general ACIS/NURBS solid modeller: curved surfaces, general booleans, healing and STEP
//! exchange are separate follow-on milestones. The topology representation is explicit so those
//! operations can be added without treating a render mesh as an editable solid.

use std::collections::HashMap;

use crate::{Bounds2, Line, Segment, ToleranceContext, Vec2, Vec3, intersect_segments_with_tolerance, shoelace};

fn unit(v: Vec3) -> Vec3 {
    if !v.is_finite() { return Vec3::ZERO; }
    let scale = v.x.abs().max(v.y.abs()).max(v.z.abs());
    if scale == 0.0 { return Vec3::ZERO; }
    let scaled = v * (1.0 / scale);
    let length = scaled.len();
    if length.is_finite() && length > 0.0 { scaled * (1.0 / length) } else { Vec3::ZERO }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BrepSurface {
    Plane { origin: Vec3, normal: Vec3 },
}

#[derive(Clone, Debug, PartialEq)]
pub struct BrepEdge {
    pub vertices: [usize; 2],
    pub half_edges: [usize; 2],
}

#[derive(Clone, Debug, PartialEq)]
pub struct BrepHalfEdge {
    pub origin: usize,
    pub twin: usize,
    pub next: usize,
    pub prev: usize,
    pub edge: usize,
    pub loop_id: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BrepLoop {
    pub first_half_edge: usize,
    pub face: usize,
    pub outer: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BrepFace {
    pub surface: BrepSurface,
    pub outer_loop: usize,
    pub inner_loops: Vec<usize>,
    pub shell: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BrepShell {
    pub faces: Vec<usize>,
    pub solid: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BrepSolid {
    pub shell: usize,
}

/// A topological solid. Each edge has exactly two oppositely oriented half-edges.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Brep {
    pub vertices: Vec<Vec3>,
    pub edges: Vec<BrepEdge>,
    pub half_edges: Vec<BrepHalfEdge>,
    pub loops: Vec<BrepLoop>,
    pub faces: Vec<BrepFace>,
    pub shells: Vec<BrepShell>,
    pub solids: Vec<BrepSolid>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrepError {
    TooFewProfilePoints,
    NonFiniteProfile,
    DegenerateProfile,
    SelfIntersectingProfile,
    InvalidHeight,
    OpenOrNonManifoldEdge,
    InvalidTopology,
    NonFiniteSurface,
}

impl Brep {
    /// Validate indices, reciprocal adjacency, closed loops and shell/solid ownership.
    /// This is a topology check, not a geometric self-intersection or surface-intersection proof.
    pub fn validate(&self) -> Result<(), BrepError> {
        if self.vertices.iter().any(|p| !p.is_finite()) {
            return Err(BrepError::NonFiniteProfile);
        }
        for (edge_id, e) in self.edges.iter().enumerate() {
            if e.vertices[0] == e.vertices[1]
                || e.vertices.iter().any(|v| *v >= self.vertices.len())
                || e.half_edges.iter().any(|h| *h >= self.half_edges.len())
            {
                return Err(BrepError::InvalidTopology);
            }
            let (Some(a), Some(b)) = (self.half_edges.get(e.half_edges[0]), self.half_edges.get(e.half_edges[1])) else {
                return Err(BrepError::InvalidTopology);
            };
            if a.twin != e.half_edges[1] || b.twin != e.half_edges[0]
                || a.edge != edge_id || b.edge != edge_id
            {
                return Err(BrepError::OpenOrNonManifoldEdge);
            }
            let da = self.half_edges.get(a.next).map(|h| h.origin).ok_or(BrepError::InvalidTopology)?;
            let db = self.half_edges.get(b.next).map(|h| h.origin).ok_or(BrepError::InvalidTopology)?;
            if !((a.origin == e.vertices[0] && da == e.vertices[1] && b.origin == e.vertices[1] && db == e.vertices[0])
                || (a.origin == e.vertices[1] && da == e.vertices[0] && b.origin == e.vertices[0] && db == e.vertices[1]))
            {
                return Err(BrepError::InvalidTopology);
            }
        }
        for (i, h) in self.half_edges.iter().enumerate() {
            if h.origin >= self.vertices.len() || h.twin >= self.half_edges.len()
                || h.next >= self.half_edges.len() || h.prev >= self.half_edges.len()
                || h.edge >= self.edges.len() || h.loop_id >= self.loops.len()
            {
                return Err(BrepError::InvalidTopology);
            }
            let next = &self.half_edges[h.next];
            let prev = &self.half_edges[h.prev];
            let twin = &self.half_edges[h.twin];
            let edge = &self.edges[h.edge];
            if next.prev != i || prev.next != i || next.loop_id != h.loop_id || prev.loop_id != h.loop_id
                || twin.twin != i || twin.edge != h.edge || !edge.half_edges.contains(&i)
            {
                return Err(BrepError::InvalidTopology);
            }
        }
        let mut loop_seen = vec![false; self.half_edges.len()];
        for (i, l) in self.loops.iter().enumerate() {
            if l.first_half_edge >= self.half_edges.len() || l.face >= self.faces.len() {
                return Err(BrepError::InvalidTopology);
            }
            let mut h = l.first_half_edge;
            let mut visited = 0usize;
            loop {
                let he = self.half_edges.get(h).ok_or(BrepError::InvalidTopology)?;
                if he.loop_id != i || loop_seen[h] { return Err(BrepError::InvalidTopology); }
                loop_seen[h] = true;
                h = he.next;
                visited += 1;
                if h == l.first_half_edge { break; }
                if visited > self.half_edges.len() { return Err(BrepError::InvalidTopology); }
            }
            if visited < 3 { return Err(BrepError::InvalidTopology); }
        }
        if loop_seen.iter().any(|seen| !seen) { return Err(BrepError::InvalidTopology); }
        // Every loop must be owned exactly once by its face; otherwise an orphan loop or a
        // loop referenced by two faces can pass local adjacency checks while corrupting topology.
        let mut face_loop_seen = vec![false; self.loops.len()];
        let mut face_shell_seen = vec![false; self.faces.len()];
        for (i, f) in self.faces.iter().enumerate() {
            if f.outer_loop >= self.loops.len() || f.shell >= self.shells.len() { return Err(BrepError::InvalidTopology); }
            let outer = &self.loops[f.outer_loop];
            if outer.face != i || !outer.outer || face_loop_seen[f.outer_loop] { return Err(BrepError::InvalidTopology); }
            face_loop_seen[f.outer_loop] = true;
            for &li in &f.inner_loops {
                let inner = self.loops.get(li).ok_or(BrepError::InvalidTopology)?;
                if inner.face != i || inner.outer || face_loop_seen[li] { return Err(BrepError::InvalidTopology); }
                face_loop_seen[li] = true;
            }
            match f.surface {
                BrepSurface::Plane { origin, normal } if origin.is_finite() && normal.is_finite() && normal.len().is_finite() && normal.len() > 0.0 => {}
                _ => return Err(BrepError::NonFiniteSurface),
            }
        }
        for (i, s) in self.shells.iter().enumerate() {
            if s.solid >= self.solids.len() || s.faces.is_empty() { return Err(BrepError::InvalidTopology); }
            for &face_id in &s.faces {
                let Some(face) = self.faces.get(face_id) else { return Err(BrepError::InvalidTopology); };
                if face.shell != i || face_shell_seen[face_id] { return Err(BrepError::InvalidTopology); }
                face_shell_seen[face_id] = true;
            }
        }
        if face_shell_seen.iter().any(|seen| !seen) || face_loop_seen.iter().any(|seen| !seen) {
            return Err(BrepError::InvalidTopology);
        }
        for (i, s) in self.solids.iter().enumerate() {
            if self.shells.get(s.shell).is_none_or(|shell| shell.solid != i) { return Err(BrepError::InvalidTopology); }
        }
        Ok(())
    }

    /// Volume of the closed, outward-oriented shell, in cubic drawing units.
    /// Returns `None` when topology is invalid or the computed value is non-finite.
    pub fn volume(&self) -> Option<f64> {
        self.validate().ok()?;
        let reference = *self.vertices.first()?;
        let mut signed = 0.0;
        for face in &self.faces {
            let mut ids = Vec::new();
            let first = face.outer_loop;
            let mut h = self.loops.get(first)?.first_half_edge;
            loop {
                ids.push(self.half_edges.get(h)?.origin);
                h = self.half_edges.get(h)?.next;
                if h == self.loops.get(first)?.first_half_edge { break; }
                if ids.len() > self.half_edges.len() { return None; }
            }
            let p0 = *self.vertices.get(*ids.first()?)?;
            for i in 1..ids.len().saturating_sub(1) {
                let a = *self.vertices.get(*ids.get(i)?)?;
                let b = *self.vertices.get(*ids.get(i + 1)?)?;
                signed += (p0 - reference).dot((a - reference).cross(b - reference)) / 6.0;
            }
        }
        signed.is_finite().then_some(signed.abs())
    }

    /// Surface area of the planar faces, in square drawing units.
    /// This sums triangulated face areas; curved surface support is not implemented yet.
    pub fn surface_area(&self) -> Option<f64> {
        self.validate().ok()?;
        let mut total = 0.0;
        for face in &self.faces {
            let l = self.loops.get(face.outer_loop)?;
            let mut ids = Vec::new();
            let mut h = l.first_half_edge;
            loop {
                ids.push(self.half_edges.get(h)?.origin);
                h = self.half_edges.get(h)?.next;
                if h == l.first_half_edge { break; }
                if ids.len() > self.half_edges.len() { return None; }
            }
            let p0 = *self.vertices.get(*ids.first()?)?;
            let mut area_vector = Vec3::ZERO;
            for i in 0..ids.len() {
                let a = *self.vertices.get(*ids.get(i)?)?;
                let b = *self.vertices.get(*ids.get((i + 1) % ids.len())?)?;
                area_vector = area_vector + (a - p0).cross(b - p0);
            }
            total += area_vector.len() * 0.5;
        }
        total.is_finite().then_some(total)
    }

    /// Create a closed prism by extruding a simple planar polygon from z=0 by `height`.
    /// The profile may be clockwise or counter-clockwise; output face orientations are normalized
    /// so the shell is outward-facing for either positive or negative extrusion height.
    pub fn extrude_polygon(profile: &[Vec2], height: f64) -> Result<Self, BrepError> {
        if profile.len() < 3 { return Err(BrepError::TooFewProfilePoints); }
        if !height.is_finite() || height == 0.0 { return Err(BrepError::InvalidHeight); }
        if profile.iter().any(|p| !p.is_finite()) { return Err(BrepError::NonFiniteProfile); }
        let mut points = profile.to_vec();
        if points.first() == points.last() { points.pop(); }
        if points.len() < 3 { return Err(BrepError::TooFewProfilePoints); }
        let bounds = Bounds2::from_points(points.iter().copied());
        let tol = ToleranceContext::from_bounds(bounds);
        for i in 0..points.len() {
            let a = points[i];
            let b = points[(i + 1) % points.len()];
            if a.near(b, tol.linear) { return Err(BrepError::DegenerateProfile); }
        }
        let area = shoelace(&points);
        for i in 0..points.len() {
            let a = Segment::Line(Line::new(points[i], points[(i + 1) % points.len()]));
            for j in (i + 1)..points.len() {
                if j == i || j == (i + 1) % points.len() || i == (j + 1) % points.len() { continue; }
                let b = Segment::Line(Line::new(points[j], points[(j + 1) % points.len()]));
                if !intersect_segments_with_tolerance(&a, &b, false, &tol).is_empty() {
                    return Err(BrepError::SelfIntersectingProfile);
                }
            }
        }
        if !area.is_finite() || area.abs() <= tol.linear * tol.linear { return Err(BrepError::DegenerateProfile); }
        // CCW for +Z extrusion, CW for -Z extrusion.
        if (area > 0.0) != (height > 0.0) { points.reverse(); }
        let n = points.len();
        let mut vertices: Vec<Vec3> = points.iter().map(|p| p.to3(0.0)).collect();
        vertices.extend(points.iter().map(|p| p.to3(height)));
        let mut face_vertices: Vec<Vec<usize>> = Vec::with_capacity(n + 2);
        face_vertices.push((0..n).rev().collect());
        face_vertices.push((n..(2 * n)).collect());
        for i in 0..n {
            let j = (i + 1) % n;
            face_vertices.push(vec![i, j, j + n, i + n]);
        }
        let mut half_edges = Vec::new();
        let mut loops = Vec::new();
        let mut faces = Vec::new();
        let mut directed: HashMap<(usize, usize), usize> = HashMap::new();
        for (face_id, indices) in face_vertices.iter().enumerate() {
            let first = half_edges.len();
            let count = indices.len();
            let p0 = vertices[indices[0]];
            let mut normal = Vec3::ZERO;
            'normal: for a in 1..count {
                for b in (a + 1)..count {
                    normal = unit((vertices[indices[a]] - p0).cross(vertices[indices[b]] - p0));
                    if normal != Vec3::ZERO { break 'normal; }
                }
            }
            if normal == Vec3::ZERO { return Err(BrepError::DegenerateProfile); }
            let loop_id = loops.len();
            for k in 0..count {
                let origin = indices[k];
                let dest = indices[(k + 1) % count];
                let hi = half_edges.len();
                if directed.insert((origin, dest), hi).is_some() { return Err(BrepError::InvalidTopology); }
                half_edges.push(BrepHalfEdge { origin, twin: usize::MAX, next: first + (k + 1) % count, prev: first + (k + count - 1) % count, edge: usize::MAX, loop_id });
            }
            loops.push(BrepLoop { first_half_edge: first, face: face_id, outer: true });
            faces.push(BrepFace { surface: BrepSurface::Plane { origin: p0, normal }, outer_loop: loop_id, inner_loops: Vec::new(), shell: 0 });
        }
        let mut edges = Vec::new();
        for hi in 0..half_edges.len() {
            if half_edges[hi].edge != usize::MAX { continue; }
            let origin = half_edges[hi].origin;
            let dest = half_edges[half_edges[hi].next].origin;
            let Some(&twin) = directed.get(&(dest, origin)) else { return Err(BrepError::OpenOrNonManifoldEdge); };
            if twin == hi || half_edges[twin].edge != usize::MAX { return Err(BrepError::OpenOrNonManifoldEdge); }
            let edge_id = edges.len();
            half_edges[hi].twin = twin;
            half_edges[twin].twin = hi;
            half_edges[hi].edge = edge_id;
            half_edges[twin].edge = edge_id;
            edges.push(BrepEdge { vertices: [origin, dest], half_edges: [hi, twin] });
        }
        let face_ids: Vec<usize> = (0..faces.len()).collect();
        let shells = vec![BrepShell { faces: face_ids, solid: 0 }];
        let solids = vec![BrepSolid { shell: 0 }];
        let brep = Brep { vertices, edges, half_edges, loops, faces, shells, solids };
        brep.validate()?;
        Ok(brep)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extrudes_a_square_into_a_closed_manifold_prism() {
        let b = Brep::extrude_polygon(&[Vec2::new(0.0, 0.0), Vec2::new(4.0, 0.0), Vec2::new(4.0, 3.0), Vec2::new(0.0, 3.0)], 5.0).unwrap();
        assert_eq!((b.vertices.len(), b.edges.len(), b.faces.len(), b.solids.len()), (8, 12, 6, 1));
        assert!(b.validate().is_ok());
        assert!((b.volume().unwrap() - 60.0).abs() < 1e-10);
        assert!((b.surface_area().unwrap() - 94.0).abs() < 1e-10);
    }

    #[test]
    fn clockwise_profile_and_negative_extrusion_are_supported() {
        let b = Brep::extrude_polygon(&[Vec2::new(0.0, 0.0), Vec2::new(0.0, 3.0), Vec2::new(4.0, 3.0), Vec2::new(4.0, 0.0)], -2.0).unwrap();
        assert!(b.validate().is_ok());
        assert!(b.faces.iter().all(|f| match f.surface { BrepSurface::Plane { normal, .. } => normal.is_finite() }));
    }

    #[test]
    fn extrusion_preserves_microscopic_profiles_without_a_world_unit_epsilon() {
        let b = Brep::extrude_polygon(&[
            Vec2::new(0.0, 0.0), Vec2::new(1.0e-6, 0.0),
            Vec2::new(1.0e-6, 1.0e-6), Vec2::new(0.0, 1.0e-6),
        ], 2.0e-6).unwrap();
        assert!(b.validate().is_ok());
        assert!((b.volume().unwrap() / 2.0e-18 - 1.0).abs() < 1e-10);
    }

    #[test]
    fn concave_profile_and_large_world_translation_keep_prism_properties() {
        let x = 1.0e9;
        let y = -2.0e9;
        let b = Brep::extrude_polygon(&[
            Vec2::new(x, y), Vec2::new(x + 4.0, y), Vec2::new(x + 4.0, y + 1.0),
            Vec2::new(x + 1.0, y + 1.0), Vec2::new(x + 1.0, y + 4.0), Vec2::new(x, y + 4.0),
        ], 2.0).unwrap();
        assert!(b.validate().is_ok());
        assert!((b.volume().unwrap() - 14.0).abs() < 1e-8);
        assert!((b.surface_area().unwrap() - 46.0).abs() < 1e-8);
    }

    #[test]
    fn topology_validator_rejects_a_broken_half_edge_twin() {
        let mut b = Brep::extrude_polygon(&[
            Vec2::new(0.0, 0.0), Vec2::new(2.0, 0.0),
            Vec2::new(2.0, 2.0), Vec2::new(0.0, 2.0),
        ], 1.0).unwrap();
        b.half_edges[0].twin = 0;
        assert_eq!(b.validate(), Err(BrepError::OpenOrNonManifoldEdge));
    }

    #[test]
    fn rejects_bow_tie_and_zero_height_profiles() {
        let bow_tie = [Vec2::new(0.0, 0.0), Vec2::new(2.0, 2.0), Vec2::new(0.0, 2.0), Vec2::new(2.0, 0.0)];
        assert_eq!(Brep::extrude_polygon(&bow_tie, 1.0), Err(BrepError::SelfIntersectingProfile));
        let square = [Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), Vec2::new(1.0, 1.0), Vec2::new(0.0, 1.0)];
        assert_eq!(Brep::extrude_polygon(&square, 0.0), Err(BrepError::InvalidHeight));
    }
}
