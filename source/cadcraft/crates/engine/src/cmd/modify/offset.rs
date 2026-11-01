use super::{
    Accept, Arc, EngineError, Entity, EntityKind, Handle, Input, Interactive, Line, PolyVertex, Polyline, Prompt, Result, Segment, Session, Step,
    Value, Vec2, arc, bad, bool_or, ccw_sweep, circle, f64_req, intersect_segments_with_tolerance, json, line, lwpoly, number, point_req, str_param, targets, v3,
};

/// Whether OFFSET can work on this kind of object at all.
fn offsettable(k: &EntityKind) -> bool {
    matches!(
        k,
        EntityKind::Line(_)
            | EntityKind::Circle(_)
            | EntityKind::Arc(_)
            | EntityKind::Ellipse(_)
            | EntityKind::LwPolyline(_)
            | EntityKind::Spline(_)
            | EntityKind::XLine(_)
            | EntityKind::Ray(_)
    )
}

/// Offset an entity by `dist` towards `side`. Returns the new geometry.
pub(crate) fn offset_kind(k: &EntityKind, dist: f64, side: Vec2) -> Option<EntityKind> {
    offset_kind_with_gap_type(k, dist, side, 0)
}

/// OFFSET with the drawing's OFFSETGAPTYPE policy (0 extend, 1 fillet, 2 chamfer).
pub(crate) fn offset_kind_with_gap_type(k: &EntityKind, dist: f64, side: Vec2, gap_type: i64) -> Option<EntityKind> {
    // Reject invalid numeric input before it reaches geometry constructors. The
    // tolerance below is derived per entity so micron drawings are not rejected
    // by a world-unit epsilon.
    if !(0..=2).contains(&gap_type) || !dist.is_finite() || dist <= 0.0 || !side.is_finite() {
        return None;
    }
    match k {
        EntityKind::Line(l) => {
            let ln = Line::new(l.a.xy(), l.b.xy());
            let sgn = if ln.side(side) >= 0.0 { 1.0 } else { -1.0 };
            let o = ln.offset(dist * sgn);
            Some(line(o.a, o.b))
        }
        EntityKind::Circle(c) => {
            if !(c.radius.is_finite() && c.radius > 0.0 && c.center.xy().is_finite()) {
                return None;
            }
            let tolerance = cadcraft_geom::ToleranceContext::for_scale(c.radius, c.center.xy().len());
            let inside = side.dist(c.center.xy()) < c.radius;
            let r = if inside { c.radius - dist } else { c.radius + dist };
            (r.is_finite() && r > tolerance.linear).then(|| circle(c.center.xy(), r))
        }
        EntityKind::Arc(a) => {
            if !(a.radius.is_finite() && a.radius > 0.0 && a.center.xy().is_finite()) {
                return None;
            }
            let tolerance = cadcraft_geom::ToleranceContext::for_scale(a.radius, a.center.xy().len());
            let inside = side.dist(a.center.xy()) < a.radius;
            let r = if inside { a.radius - dist } else { a.radius + dist };
            (r.is_finite() && r > tolerance.linear).then(|| arc(&Arc::new(a.center.xy(), r, a.start, a.end)))
        }
        EntityKind::Ellipse(e) => {
            // Approximate with a polyline offset of the tessellation.
            let ge = cadcraft_geom::Ellipse { center: e.center.xy(), major: e.major.xy(), ratio: e.ratio, start: e.start, end: e.end };
            // Inside = on the centre side of the full ellipse, even for an open arc.
            let inside = {
                let a = e.major.xy().len();
                let b = a * e.ratio.abs();
                let u = e.major.xy().normalized();
                let d = side - e.center.xy();
                let (x, y) = (d.dot(u) * b, d.dot(u.perp()) * a);
                x * x + y * y < (a * b) * (a * b)
            };
            // offset_points shifts along the left normal, which points into a counter-clockwise ellipse.
            let mut pts = Vec::new();
            ge.tessellate(e.major.xy().len() * 1e-3, &mut pts);
            let off = offset_points(&pts, if inside { dist } else { -dist }, ge.is_full());
            Some(lwpoly(off.into_iter().map(PolyVertex::new).collect(), ge.is_full()))
        }
        EntityKind::LwPolyline(p) => {
            let pl = Polyline { vertices: p.vertices.clone(), closed: p.closed };
            // Side: compare with the nearest segment.
            let segs = pl.segments();
            let nearest = segs.iter().min_by(|a, b| a.closest(side).dist(side).total_cmp(&b.closest(side).dist(side)))?;
            let c = nearest.closest(side);
            let t = nearest.tangent(0.5);
            let left = t.cross(side - c) > 0.0;
            let d = if left { dist } else { -dist };
            offset_polyline_with_gap_type(&pl, d, gap_type).map(|vs| {
                let mut k = lwpoly(vs, p.closed);
                if let EntityKind::LwPolyline(n) = &mut k {
                    n.const_width = p.const_width;
                }
                k
            })
        }
        EntityKind::Spline(sp) => {
            let bounds = sp.bounds();
            let scale = if bounds.is_empty() { 1.0 } else { bounds.width().hypot(bounds.height()).max(f64::MIN_POSITIVE) };
            let pts = sp.tessellate((scale * 1e-4).max(f64::MIN_POSITIVE));
            let n = pts.len();
            let mid = pts.get(n / 2).copied()?;
            let next = pts.get((n / 2 + 1).min(n.saturating_sub(1))).copied()?;
            let left = (next - mid).cross(side - mid) > 0.0;
            let off = offset_points(&pts, if left { dist } else { -dist }, sp.closed);
            // A closed spline stays closed (periodic), smooth across its start point.
            Some(EntityKind::Spline(cadcraft_geom::Spline::from_fit(&decimate(&off, 40), sp.closed)))
        }
        EntityKind::XLine(r) | EntityKind::Ray(r) => {
            let ln = Line::new(r.base.xy(), r.base.xy() + r.dir.xy());
            let sgn = if ln.side(side) >= 0.0 { 1.0 } else { -1.0 };
            let o = ln.offset(dist * sgn);
            let rl = cadcraft_doc::RayLine { base: v3(o.a), dir: r.dir };
            Some(if matches!(k, EntityKind::XLine(_)) { EntityKind::XLine(rl) } else { EntityKind::Ray(rl) })
        }
        _ => None,
    }
}

fn decimate(pts: &[Vec2], max: usize) -> Vec<Vec2> {
    if pts.len() <= max {
        return pts.to_vec();
    }
    let step = pts.len() as f64 / max as f64;
    let mut v: Vec<Vec2> = (0..max).filter_map(|i| pts.get((i as f64 * step) as usize).copied()).collect();
    if let Some(l) = pts.last() {
        v.push(*l);
    }
    v
}

fn offset_points(pts: &[Vec2], d: f64, closed: bool) -> Vec<Vec2> {
    let n = pts.len();
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let p = pts.get(i).copied().unwrap_or_default();
        let prev = if i > 0 {
            pts.get(i - 1)
        } else if closed {
            pts.get(n.saturating_sub(2))
        } else {
            None
        };
        let next = if i + 1 < n {
            pts.get(i + 1)
        } else if closed {
            pts.get(1)
        } else {
            None
        };
        let t = match (prev, next) {
            (Some(a), Some(b)) => (*b - *a).normalized(),
            (None, Some(b)) => (*b - p).normalized(),
            (Some(a), None) => (p - *a).normalized(),
            _ => Vec2::X,
        };
        out.push(p + t.perp() * d);
    }
    out
}

/// Offset a polyline: offset each segment, then join neighbours at their intersections.
pub(crate) fn offset_polyline(pl: &Polyline, d: f64) -> Option<Vec<PolyVertex>> {
    offset_polyline_with_gap_type(pl, d, 0)
}

pub(crate) fn offset_polyline_with_gap_type(pl: &Polyline, d: f64, gap_type: i64) -> Option<Vec<PolyVertex>> {
    let mut loops = offset_polyline_with_gap_type_multi(pl, d, gap_type)?;
    (loops.len() == 1).then(|| loops.pop()).flatten()
}

/// Multi-entity OFFSET geometry path. Multiple filled loops are returned separately;
/// unsupported curved self-intersections still fail closed rather than flattening arcs.
pub(crate) fn offset_polyline_with_gap_type_multi(pl: &Polyline, d: f64, gap_type: i64) -> Option<Vec<Vec<PolyVertex>>> {
    if !(0..=2).contains(&gap_type) || !d.is_finite() || d == 0.0
        || pl.vertices.iter().any(|v| !v.p.is_finite() || !v.bulge.is_finite())
    {
        return None;
    }
    let segs: Vec<Segment> = pl.segments().iter().filter_map(|s| s.offset(d)).collect();
    if segs.is_empty() {
        return None;
    }
    let n = segs.len();
    let tolerance = cadcraft_geom::ToleranceContext::from_bounds(pl.bounds());
    // Junction i is between segment i-1 and segment i (modulo n for a
    // closed polyline). Keep missing joins explicit so OFFSETGAPTYPE can
    // choose a round or chamfer connector instead of silently stretching.
    let mut joins: Vec<Option<Vec2>> = vec![None; n];
    for i in 0..n {
        if i == 0 && !pl.closed {
            continue;
        }
        let prev = segs.get((i + n - 1) % n)?;
        let current = segs.get(i)?;
        joins[i] = join_point(prev, current, gap_type == 0, &tolerance);
        if joins[i].is_none() && gap_type == 0 {
            joins[i] = Some(current.start());
        }
    }

    let mut verts: Vec<PolyVertex> = Vec::new();
    for i in 0..n {
        let segment = segs.get(i)?;
        let start = if i == 0 && !pl.closed { segment.start() } else { joins[i].unwrap_or(segment.start()) };
        let next_i = (i + 1) % n;
        let has_next = i + 1 < n || pl.closed;
        let joined_end = if has_next { joins[next_i] } else { None };
        let segment_end = joined_end.unwrap_or(segment.end());
        let bulge = match segment {
            Segment::Line(_) => 0.0,
            Segment::Arc { arc, ccw } => {
                let sweep = if start.near(segment_end, tolerance.linear) {
                    arc.sweep()
                } else if *ccw {
                    ccw_sweep(arc.center.angle_to(start), arc.center.angle_to(segment_end))
                } else {
                    ccw_sweep(arc.center.angle_to(segment_end), arc.center.angle_to(start))
                };
                cadcraft_geom::arc_to_bulge(if *ccw { sweep } else { -sweep })
            }
        };
        verts.push(PolyVertex { p: start, bulge, start_width: 0.0, end_width: 0.0 });

        // A missing join is a real gap between the finite offset segments.
        // 0 preserves legacy extension behavior; 1 inserts a circular fillet
        // where the endpoints lie on the expected offset radius; 2 chamfers.
        if has_next && joined_end.is_none() && gap_type != 0 {
            let next = segs.get(next_i)?;
            let next_start = next.start();
            let connector_bulge = if gap_type == 1 {
                let source_vertex = pl.vertices.get(next_i)?.p;
                // The signed source turn and offset side determine which sweep
                // connects the two offset endpoints around the source vertex.
                // Choosing the shortest unsigned sweep can create an arc on the
                // wrong side for clockwise paths.
                let previous_vertex = if next_i > 0 {
                    pl.vertices.get(next_i - 1).map(|v| v.p)
                } else if pl.closed {
                    pl.vertices.last().map(|v| v.p)
                } else {
                    None
                };
                let following_vertex = if next_i + 1 < pl.vertices.len() {
                    pl.vertices.get(next_i + 1).map(|v| v.p)
                } else if pl.closed {
                    pl.vertices.first().map(|v| v.p)
                } else {
                    None
                };
                let sweep_direction = previous_vertex.zip(following_vertex)
                    .map(|(before, after)| -(source_vertex - before).cross(after - source_vertex) * d)
                    .unwrap_or(0.0);
                // OFFSETGAPTYPE=1 explicitly requests a circular connector.
                // Never silently replace an invalid/unconstructable fillet with a
                // straight chamfer: fail the operation so the user can correct it.
                round_join_bulge(
                    source_vertex,
                    segment_end,
                    next_start,
                    d.abs(),
                    sweep_direction,
                    tolerance.linear,
                    tolerance.angular,
                )?
            } else {
                0.0
            };
            verts.push(PolyVertex { p: segment_end, bulge: connector_bulge, start_width: 0.0, end_width: 0.0 });
        }
    }
    if !pl.closed {
        verts.push(PolyVertex::new(segs.last()?.end()));
    }

    // A mathematically valid offset may collapse or self-intersect when the requested
    // distance is larger than a local feature. Do not emit corrupt topology: detect
    // non-adjacent crossings and zero-length edges and reject the operation safely.
    if verts.iter().any(|vertex| !vertex.p.is_finite() || !vertex.bulge.is_finite()) {
        return None;
    }
    let result = Polyline { vertices: verts.clone(), closed: pl.closed };
    let result_segments = result.segments();
    if result_segments.iter().any(|segment| segment.len() <= tolerance.linear) {
        return None;
    }
    let mut self_intersects = false;
    for i in 0..result_segments.len() {
        for j in (i + 1)..result_segments.len() {
            let adjacent = j == i + 1 || (pl.closed && i == 0 && j + 1 == result_segments.len());
            if adjacent { continue; }
            let proper_line_crossing = match (&result_segments[i], &result_segments[j]) {
                (Segment::Line(a), Segment::Line(b)) => cadcraft_geom::segments_properly_cross(a.a, a.b, b.a, b.b),
                _ => false,
            };
            if proper_line_crossing
                || !intersect_segments_with_tolerance(&result_segments[i], &result_segments[j], false, &tolerance).is_empty()
            {
                self_intersects = true;
                break;
            }
        }
        if self_intersects { break; }
    }
    if self_intersects {
        // The current command emits one entity. Repair line-only closed contours through
        // planar face extraction; preserve arc geometry by rejecting unsupported cases.
        if !pl.closed || result_segments.iter().any(|segment| !matches!(segment, Segment::Line(_))) {
            return None;
        }
        let points: Vec<Vec2> = verts.iter().map(|vertex| vertex.p).collect();
        let repaired = cadcraft_geom::region::repair_winding_loops(&points, tolerance.linear)?;
        return Some(repaired.into_iter().filter(|loop_points| loop_points.len() >= 3)
            .map(|loop_points| loop_points.into_iter().map(PolyVertex::new).collect()).collect());
    }
    Some(vec![verts])
}

fn round_join_bulge(
    center: Vec2,
    start: Vec2,
    end: Vec2,
    radius: f64,
    sweep_direction: f64,
    tolerance: f64,
    angular_tolerance: f64,
) -> Option<f64> {
    let a = start - center;
    let b = end - center;
    let ra = a.len();
    let rb = b.len();
    let radial_tolerance = tolerance.max(radius * 1.0e-8);
    if !(center.is_finite()
        && start.is_finite()
        && end.is_finite()
        && radius.is_finite()
        && radius > tolerance
        && sweep_direction.is_finite()
        && sweep_direction != 0.0
        && (ra - radius).abs() <= radial_tolerance
        && (rb - radius).abs() <= radial_tolerance)
    {
        return None;
    }

    // atan2 gives a signed principal angle. Select the orientation implied by
    // the source corner and signed offset, rather than silently using whichever
    // principal sweep happens to be returned.
    let mut sweep = a.cross(b).atan2(a.dot(b));
    if !sweep.is_finite() || !angular_tolerance.is_finite() || angular_tolerance < 0.0 {
        return None;
    }
    if sweep_direction > 0.0 && sweep < 0.0 {
        sweep += std::f64::consts::TAU;
    } else if sweep_direction < 0.0 && sweep > 0.0 {
        sweep -= std::f64::consts::TAU;
    }
    if sweep.abs() <= angular_tolerance
        || sweep.abs() >= std::f64::consts::PI + angular_tolerance
    {
        return None;
    }
    let bulge = (sweep * 0.25).tan();
    bulge.is_finite().then_some(bulge)
}

fn join_point(a: &Segment, b: &Segment, extend: bool, tolerance: &cadcraft_geom::ToleranceContext) -> Option<Vec2> {
    if a.end().near(b.start(), tolerance.linear) {
        return Some(b.start());
    }
    let hits = intersect_segments_with_tolerance(a, b, extend, tolerance);
    hits.into_iter().min_by(|x, y| x.dist(a.end()).total_cmp(&y.dist(a.end())))
}

/// Split a directed line/arc at known points while retaining exact primitive geometry.
/// Parameters are measured in the segment's travel direction, so clockwise arcs are
/// handled without reversing their output order. Points outside the finite segment are
/// ignored; near-duplicate parameters are merged using the drawing tolerance.
fn split_segment_at_points(segment: &Segment, points: &[Vec2], tolerance: &cadcraft_geom::ToleranceContext) -> Option<Vec<Segment>> {
    if !segment.start().is_finite() || !segment.end().is_finite() || !segment.len().is_finite() {
        return None;
    }
    let mut cuts = vec![0.0, 1.0];
    for point in points.iter().copied().filter(|point| point.is_finite()) {
        let parameter = match segment {
            Segment::Line(line) => {
                let direction = line.b - line.a;
                let length_squared = direction.dot(direction);
                if !(length_squared.is_finite() && length_squared > 0.0) { return None; }
                let t = (point - line.a).dot(direction) / length_squared;
                let projected = line.at(t);
                if projected.dist(point) <= tolerance.linear { Some(t) } else { None }
            }
            Segment::Arc { arc, ccw } => {
                let radial_error = (point.dist(arc.center) - arc.radius).abs();
                if radial_error > tolerance.linear.max(arc.radius * tolerance.angular) { None }
                else {
                    let angle = arc.center.angle_to(point);
                    let sweep = if *ccw { ccw_sweep(arc.start, angle) } else { ccw_sweep(angle, arc.end) };
                    Some(sweep / arc.sweep())
                }
            }
        };
        if let Some(t) = parameter.filter(|t| t.is_finite() && *t > tolerance.parameter && *t < 1.0 - tolerance.parameter) {
            cuts.push(t);
        }
    }
    cuts.sort_by(f64::total_cmp);
    cuts.dedup_by(|a, b| (*a - *b).abs() <= tolerance.parameter);
    let mut pieces = Vec::with_capacity(cuts.len().saturating_sub(1));
    for pair in cuts.windows(2) {
        let (Some(&a), Some(&b)) = (pair.first(), pair.get(1)) else { continue };
        if b - a <= tolerance.parameter { continue; }
        let piece = match segment {
            Segment::Line(_) => Segment::Line(Line::new(segment.at(a), segment.at(b))),
            Segment::Arc { arc, ccw: true } => Segment::Arc {
                arc: Arc::new(arc.center, arc.radius, arc.center.angle_to(segment.at(a)), arc.center.angle_to(segment.at(b))),
                ccw: true,
            },
            Segment::Arc { arc, ccw: false } => Segment::Arc {
                arc: Arc::new(arc.center, arc.radius, arc.center.angle_to(segment.at(b)), arc.center.angle_to(segment.at(a))),
                ccw: false,
            },
        };
        if piece.len() > tolerance.linear { pieces.push(piece); }
    }
    (!pieces.is_empty()).then_some(pieces)
}

pub(super) fn run_offset(s: &mut Session, p: &Value) -> Result<Value> {
    let h = targets(s, p)?.first().copied().ok_or_else(|| bad("offset", "`handle` is required"))?;
    let dist = f64_req("offset", p, "distance")?;
    if dist == 0.0 {
        return Err(bad("offset", "Requires a positive distance."));
    }
    let side = point_req("offset", p, "side")?;
    let e = s.doc()?.entity(h).map(|e| (**e).clone()).ok_or_else(|| bad("offset", "no such object"))?;
    let gap_type = s.doc()?.header.i64("OFFSETGAPTYPE", 0).clamp(0, 2);
    // Closed straight-segment polylines can split into more than one filled result.
    // Emit every retained loop as its own entity instead of silently losing faces.
    if let EntityKind::LwPolyline(poly) = &e.kind {
        let pl = Polyline { vertices: poly.vertices.clone(), closed: poly.closed };
        let nearest = pl.segments().into_iter()
            .min_by(|a, b| a.closest(side).dist(side).total_cmp(&b.closest(side).dist(side)))
            .ok_or_else(|| bad("offset", "cannot offset an empty polyline"))?;
        let closest = nearest.closest(side);
        let left = nearest.tangent(0.5).cross(side - closest) > 0.0;
        let signed_distance = if left { dist.abs() } else { -dist.abs() };
        let loops = offset_polyline_with_gap_type_multi(&pl, signed_distance, gap_type)
            .ok_or_else(|| bad("offset", "cannot offset that polyline without invalid topology"))?;
        let current_layer = match str_param(p, "layer").map(str::to_ascii_lowercase).as_deref() {
            None | Some("source") => false,
            Some("current") => true,
            Some(_) => return Err(bad("offset", "`layer` is \"source\" or \"current\"")),
        };
        let erase = bool_or(p, "erase", false);
        let mut handles = Vec::with_capacity(loops.len());
        for vertices in loops {
            let mut result = lwpoly(vertices, poly.closed);
            if let EntityKind::LwPolyline(ref mut result_poly) = result {
                result_poly.const_width = poly.const_width;
            }
            handles.push(add_offset(s, &e, result, current_layer, false)?);
        }
        if erase {
            let locked = s.doc()?.layer(&e.common.layer).is_some_and(|layer| layer.locked);
            if !locked { s.doc_mut()?.remove_entity(e.handle); }
        }
        if handles.len() == 1 {
            return Ok(json!({ "handle": handles[0].hex() }));
        }
        return Ok(json!({ "handles": handles.iter().map(|handle| handle.hex()).collect::<Vec<_>>(), "count": handles.len() }));
    }
    let k = offset_kind_with_gap_type(&e.kind, dist.abs(), side, gap_type).ok_or_else(|| bad("offset", "cannot offset that object"))?;
    let current_layer = match str_param(p, "layer").map(str::to_ascii_lowercase).as_deref() {
        None | Some("source") => false,
        Some("current") => true,
        Some(_) => return Err(bad("offset", "`layer` is \"source\" or \"current\"")),
    };
    let nh = add_offset(s, &e, k, current_layer, bool_or(p, "erase", false))?;
    Ok(json!({ "handle": nh.hex() }))
}

/// Add the offset copy `k` of `src` (on the current layer instead of the source's when
/// `current_layer`), then erase the source when `erase` and its layer is not locked.
fn add_offset(s: &mut Session, src: &Entity, k: EntityKind, current_layer: bool, erase: bool) -> Result<Handle> {
    let space = s.space();
    let d = s.doc_mut()?;
    let mut common = src.common.clone();
    if current_layer {
        common.layer = d.header.str("CLAYER", "0");
    }
    let nh = d.new_handle();
    if let Some(st) = d.space_mut(&space) {
        st.push(Entity { handle: nh, common, kind: k });
    }
    let locked = d.layer(&src.common.layer).is_some_and(|l| l.locked);
    if erase && !locked {
        d.remove_entity(src.handle);
    }
    Ok(nh)
}

/// An OFFSET option prompt opened from "Specify offset distance".
#[derive(Clone, Copy, PartialEq)]
enum OffsetAsk {
    Erase,
    Layer,
}

pub(super) struct OffsetM {
    dist: Option<f64>,
    distance_first: Option<Vec2>,
    through: bool,
    picked: Option<Handle>,
    ask: Option<OffsetAsk>,
    /// Multiple mode: each side point offsets the newest copy again, until Enter.
    multiple: bool,
    /// Undo: the drawing before each offset, with the object picked then.
    undo: Vec<(std::sync::Arc<cadcraft_doc::Drawing>, Option<Handle>)>,
}

impl OffsetM {
    pub(super) fn new(s: &Session) -> Self {
        let _ = s;
        OffsetM { dist: None, distance_first: None, through: false, picked: None, ask: None, multiple: false, undo: Vec::new() }
    }

    fn set_distance(&mut self, s: &mut Session, distance: f64) -> Result<()> {
        if !(distance.is_finite() && distance > 0.0) {
            return Err(EngineError::Other("Requires a positive distance.".into()));
        }
        s.doc_mut()?.header.set_f64("OFFSETDIST", distance);
        self.dist = Some(distance);
        self.distance_first = None;
        Ok(())
    }
}

/// OFFSET's remembered options: erase the source (OFFSETERASE) and put copies on the current
/// layer instead of the source's (kept in the drawing header beside it).
fn offset_options(s: &Session) -> (bool, bool) {
    s.doc().map(|d| (d.header.i64("OFFSETERASE", 0) != 0, d.header.i64("OFFSETLAYER", 0) != 0)).unwrap_or((false, false))
}

fn offset_gap_type(s: &Session) -> i64 {
    s.doc().map(|d| d.header.i64("OFFSETGAPTYPE", 0).clamp(0, 2)).unwrap_or(0)
}

impl Interactive for OffsetM {
    fn name(&self) -> &'static str {
        "OFFSET"
    }
    /// Starts OFFSET, keeping a single preselected offsettable object (issue #1610) so the prompt
    /// sequence goes distance then side instead of asking for the object again; any other
    /// selection is cleared and the object loop asks for it. With PICKFIRST off the selection is
    /// likewise ignored and cleared.
    fn begin(&mut self, s: &mut Session) -> Result<Step> {
        // Exactly one offsettable object selected before the command is taken at once, so only the
        // side is asked for; the object loop stays for the ones after it (issue #1610). A multiple
        // selection is left for the select prompt, as AutoCAD does.
        let pre = s.selection();
        if s.settings.pickfirst
            && let [h] = pre[..]
            && s.doc()?.entity(h).is_some_and(|e| offsettable(&e.kind))
        {
            self.picked = Some(h);
            s.set_selection(vec![h]);
        } else {
            s.set_selection(Vec::new());
        }
        let (erase, current) = offset_options(s);
        s.echo(format!(
            "Current settings: Erase source={}  Layer={}  OFFSETGAPTYPE={}",
            if erase { "Yes" } else { "No" },
            if current { "Current" } else { "Source" },
            offset_gap_type(s)
        ));
        Ok(Step::Continue)
    }
    fn prompt(&self, s: &Session) -> Prompt {
        let (erase, current) = offset_options(s);
        match self.ask {
            Some(OffsetAsk::Erase) => {
                return Prompt::new("Erase source object after offsetting?", Accept::TEXT).kw(&["Yes", "No"]).default(if erase {
                    "Yes"
                } else {
                    "No"
                });
            }
            Some(OffsetAsk::Layer) => {
                return Prompt::new("Enter layer option for offset objects", Accept::TEXT).kw(&["Current", "Source"]).default(if current {
                    "Current"
                } else {
                    "Source"
                });
            }
            None => {}
        }
        let last = s.doc().map(|d| d.header.f64("OFFSETDIST", -1.0)).unwrap_or(-1.0);
        if self.dist.is_none()
            && !self.through
            && let Some(base) = self.distance_first
        {
            return Prompt::new("Specify second point", Accept::POINT).base(base);
        }
        match (self.dist.is_some() || self.through, self.picked) {
            (false, _) => Prompt::new("Specify offset distance", Accept::POINT_OR_NUMBER).kw(&["Through", "Erase", "Layer"]).default(if last < 0.0 {
                "Through".to_string()
            } else {
                format!("{last:.4}")
            }),
            (true, None) => Prompt::new("Select object to offset", Accept::POINT).kw(&["Exit", "Undo"]).default("Exit"),
            (true, Some(_)) if self.multiple => {
                let msg = if self.through { "Specify through point" } else { "Specify point on side to offset" };
                Prompt::new(msg, Accept::POINT).kw(&["Exit", "Undo"]).default("next object")
            }
            (true, Some(_)) if self.through => Prompt::new("Specify through point", Accept::POINT).kw(&["Exit", "Multiple", "Undo"]).default("Exit"),
            (true, Some(_)) => Prompt::new("Specify point on side to offset", Accept::POINT).kw(&["Exit", "Multiple", "Undo"]).default("Exit"),
        }
    }
    fn input(&mut self, s: &mut Session, i: Input) -> Result<Step> {
        if let Some(ask) = self.ask {
            let word = match &i {
                Input::Keyword(k) | Input::Text(k) => k.trim().to_ascii_lowercase(),
                Input::Enter => String::new(),
                _ => return Ok(Step::Continue),
            };
            let (key, yes, no) = match ask {
                OffsetAsk::Erase => ("OFFSETERASE", "y", "n"),
                OffsetAsk::Layer => ("OFFSETLAYER", "c", "s"),
            };
            let v = if word.is_empty() {
                None
            } else if word.starts_with(yes) {
                Some(1)
            } else if word.starts_with(no) {
                Some(0)
            } else {
                return Err(EngineError::Other("Invalid option keyword.".into()));
            };
            if let Some(v) = v {
                s.doc_mut()?.header.set_i64(key, v);
            }
            self.ask = None;
            return Ok(Step::Continue);
        }
        if self.dist.is_none() && !self.through {
            match i {
                Input::Text(t) => {
                    let d = number(&t).ok_or_else(|| EngineError::Other("Requires a positive distance.".into()))?;
                    self.set_distance(s, d)?;
                }
                Input::Point(p) => {
                    if !p.is_finite() {
                        return Err(EngineError::Other("Invalid point.".into()));
                    }
                    match self.distance_first {
                        Some(base) => self.set_distance(s, base.dist(p))?,
                        None => self.distance_first = Some(p),
                    }
                }
                Input::Keyword(k) if k == "Through" => {
                    self.through = true;
                    self.distance_first = None;
                }
                Input::Keyword(k) if k == "Erase" => self.ask = Some(OffsetAsk::Erase),
                Input::Keyword(k) if k == "Layer" => self.ask = Some(OffsetAsk::Layer),
                Input::Enter => {
                    self.distance_first = None;
                    let last = s.doc()?.header.f64("OFFSETDIST", -1.0);
                    if last > 0.0 {
                        self.dist = Some(last);
                    } else {
                        self.through = true;
                    }
                }
                _ => {}
            }
            return Ok(Step::Continue);
        }
        match (self.picked, i) {
            (_, Input::Keyword(k)) if k == "Exit" => {
                s.set_selection(Vec::new());
                Ok(Step::Done)
            }
            (_, Input::Keyword(k)) if k == "Undo" => {
                match self.undo.pop() {
                    Some((doc, picked)) => {
                        // Settings changed meanwhile stay.
                        let header = s.doc()?.header.clone();
                        s.state_mut()?.doc = doc;
                        s.doc_mut()?.header = header;
                        self.picked = if self.multiple { picked } else { None };
                        if self.picked.is_none() {
                            self.multiple = false;
                        }
                        s.set_selection(self.picked.into_iter().collect());
                    }
                    None => s.echo("Nothing to undo."),
                }
                Ok(Step::Continue)
            }
            (Some(_), Input::Keyword(k)) if k == "Multiple" => {
                self.multiple = true;
                Ok(Step::Continue)
            }
            (Some(_), Input::Enter) if self.multiple => {
                // <next object>: back to selecting.
                self.multiple = false;
                self.picked = None;
                s.set_selection(Vec::new());
                Ok(Step::Continue)
            }
            (_, Input::Enter) => {
                s.set_selection(Vec::new());
                Ok(Step::Done)
            }
            (None, Input::Point(p)) => {
                let ap = s.pixel_size() * s.settings.pickbox.max(1.0) * 1.5;
                let space = s.space();
                match s.pick_object(&space, p, ap)? {
                    Some(h) => {
                        self.picked = Some(h);
                        s.set_selection(vec![h]);
                    }
                    None => s.echo("*Invalid selection*"),
                }
                Ok(Step::Continue)
            }
            (Some(h), Input::Point(p)) => {
                let e = s.doc()?.entity(h).map(|e| (**e).clone());
                let mut made = None;
                if let Some(e) = e {
                    let dist = match self.dist {
                        Some(d) => d,
                        None => {
                            let polys = crate::select::hit_polylines(s.doc()?, &e, 1e-4);
                            polys
                                .iter()
                                .flat_map(|pl| pl.windows(2).filter_map(|w| Some(Line::new(*w.first()?, *w.get(1)?).dist(p))))
                                .fold(f64::INFINITY, f64::min)
                        }
                    };
                    match offset_kind_with_gap_type(&e.kind, dist, p, offset_gap_type(s)) {
                        Some(k) => {
                            let before = s.state()?.doc.clone();
                            let (erase, current) = offset_options(s);
                            made = Some(add_offset(s, &e, k, current, erase)?);
                            self.undo.push((before, Some(h)));
                        }
                        None => s.echo("Cannot offset that object."),
                    }
                }
                // Multiple mode carries on from the new copy.
                self.picked = if self.multiple { made.or(self.picked) } else { None };
                s.set_selection(self.picked.into_iter().collect());
                Ok(Step::Continue)
            }
            _ => Ok(Step::Continue),
        }
    }
    fn preview(&self, s: &Session, c: Vec2) -> Vec<EntityKind> {
        if let Some(base) = self.distance_first
            && c.is_finite()
        {
            return vec![line(base, c)];
        }
        let (Some(h), Ok(d)) = (self.picked, s.doc()) else { return Vec::new() };
        let Some(e) = d.entity(h) else { return Vec::new() };
        let dist = match self.dist {
            Some(x) => x,
            None => crate::select::entity_distance(d, e, c, 1e-3),
        };
        offset_kind_with_gap_type(&e.kind, dist, c, offset_gap_type(s)).into_iter().collect()
    }
}

#[cfg(test)]
mod p1_scale_safety_tests {
    use super::*;

    #[test]
    fn circle_offset_preserves_valid_microscopic_geometry() {
        let source = circle(Vec2::ZERO, 1.0e-15);
        let result = offset_kind_with_gap_type(&source, 2.0e-16, Vec2::new(2.0e-15, 0.0), 0)
            .expect("a valid micron-scale circle offset must not hit a world-unit epsilon");
        match result {
            EntityKind::Circle(c) => assert!((c.radius - 1.2e-15).abs() <= 1.0e-28),
            other => panic!("expected circle, got {other:?}"),
        }
    }

    #[test]
    fn round_join_uses_source_turn_direction_for_counter_clockwise_corner() {
        let bulge = round_join_bulge(
            Vec2::ZERO,
            Vec2::Y,
            -Vec2::X,
            1.0,
            1.0,
            1.0e-12,
            1.0e-12,
        ).expect("counter-clockwise outside corner should produce a round join");
        assert!((bulge - (std::f64::consts::FRAC_PI_8).tan()).abs() < 1.0e-12);
    }

    #[test]
    fn round_join_uses_source_turn_direction_for_clockwise_corner() {
        let bulge = round_join_bulge(
            Vec2::ZERO,
            Vec2::Y,
            Vec2::X,
            1.0,
            -1.0,
            1.0e-12,
            1.0e-12,
        ).expect("clockwise outside corner should produce a clockwise round join");
        assert!((bulge + (std::f64::consts::FRAC_PI_8).tan()).abs() < 1.0e-12);
    }

    #[test]
    fn round_join_rejects_wrong_radius_and_ambiguous_direction() {
        assert!(round_join_bulge(
            Vec2::ZERO, Vec2::Y, Vec2::new(-2.0, 0.0), 1.0, 1.0, 1.0e-12, 1.0e-12
        ).is_none());
        assert!(round_join_bulge(
            Vec2::ZERO, Vec2::Y, -Vec2::X, 1.0, 0.0, 1.0e-12, 1.0e-12
        ).is_none());
    }

    #[test]
    fn open_polyline_round_gap_type_emits_a_circular_join() {
        let polyline = Polyline {
            vertices: vec![
                PolyVertex::new(Vec2::new(0.0, 0.0)),
                PolyVertex::new(Vec2::new(10.0, 0.0)),
                PolyVertex::new(Vec2::new(10.0, 10.0)),
            ],
            closed: false,
        };
        // Right-side offset of a CCW corner creates an outer quarter-circle join.
        let loops = offset_polyline_with_gap_type_multi(&polyline, -2.0, 1)
            .expect("a regular right-angle open path should support a round join");
        assert_eq!(loops.len(), 1);
        let vertices = loops.first().expect("one offset path");
        assert!(vertices.iter().any(|vertex| (vertex.bulge - (std::f64::consts::FRAC_PI_8).tan()).abs() < 1.0e-10));
    }

    #[test]
    fn round_gap_type_rejects_a_corner_without_a_defined_sweep() {
        let reversing_path = Polyline {
            vertices: vec![
                PolyVertex::new(Vec2::new(0.0, 0.0)),
                PolyVertex::new(Vec2::new(10.0, 0.0)),
                PolyVertex::new(Vec2::new(0.0, 0.0)),
            ],
            closed: false,
        };
        assert!(offset_polyline_with_gap_type_multi(&reversing_path, 2.0, 1).is_none());
    }

    #[test]
    fn offset_rejects_non_finite_or_non_positive_distance() {
        let source = circle(Vec2::ZERO, 10.0);
        assert!(offset_kind_with_gap_type(&source, f64::NAN, Vec2::X, 0).is_none());
        assert!(offset_kind_with_gap_type(&source, f64::INFINITY, Vec2::X, 0).is_none());
        assert!(offset_kind_with_gap_type(&source, 0.0, Vec2::X, 0).is_none());
        assert!(offset_kind_with_gap_type(&source, 1.0, Vec2::new(f64::NAN, 0.0), 0).is_none());
    }

    #[test]
    fn segment_split_preserves_line_geometry_and_order() {
        let tolerance = cadcraft_geom::ToleranceContext::for_scale(10.0, 10.0);
        let source = Segment::Line(Line::new(Vec2::ZERO, Vec2::new(10.0, 0.0)));
        let pieces = split_segment_at_points(&source, &[Vec2::new(7.5, 0.0), Vec2::new(2.5, 0.0)], &tolerance).unwrap();
        assert_eq!(pieces.len(), 3);
        assert!(pieces[0].start().near(Vec2::ZERO, tolerance.linear));
        assert!(pieces[0].end().near(Vec2::new(2.5, 0.0), tolerance.linear));
        assert!(pieces[2].end().near(Vec2::new(10.0, 0.0), tolerance.linear));
    }

    #[test]
    fn segment_split_preserves_clockwise_arc_direction_and_radius() {
        let tolerance = cadcraft_geom::ToleranceContext::for_scale(10.0, 10.0);
        let arc = Arc::new(Vec2::ZERO, 5.0, 0.0, std::f64::consts::FRAC_PI_2);
        let source = Segment::Arc { arc, ccw: false };
        let cut = source.at(0.4);
        let pieces = split_segment_at_points(&source, &[cut], &tolerance).unwrap();
        assert_eq!(pieces.len(), 2);
        assert!(pieces[0].start().near(source.start(), tolerance.linear));
        assert!(pieces[0].end().near(cut, tolerance.linear));
        assert!(pieces[1].end().near(source.end(), tolerance.linear));
        for piece in pieces {
            match piece {
                Segment::Arc { arc, ccw: false } => assert!((arc.radius - 5.0).abs() < tolerance.linear),
                other => panic!("expected clockwise arc fragment, got {other:?}"),
            }
        }
    }
}
