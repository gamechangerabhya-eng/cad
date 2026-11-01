use crate::{Arc, Bounds2, Circle, Ellipse, Line, Segment, Spline, ToleranceContext, Vec2, angle_in_sweep};

/// Intersection of two infinite lines through (a1,a2) and (b1,b2), with parameters on each.
/// The compatibility wrapper derives an angular tolerance from the operands.
pub fn line_line_infinite(a1: Vec2, a2: Vec2, b1: Vec2, b2: Vec2) -> Option<(Vec2, f64, f64)> {
    let bounds = Bounds2::from_points([a1, a2, b1, b2]);
    let tolerance = ToleranceContext::from_bounds(bounds);
    line_line_infinite_with_tolerance(a1, a2, b1, b2, &tolerance)
}

/// Infinite-line intersection using the shared operation tolerance.
pub fn line_line_infinite_with_tolerance(
    a1: Vec2, a2: Vec2, b1: Vec2, b2: Vec2, tolerance: &ToleranceContext,
) -> Option<(Vec2, f64, f64)> {
    if !a1.is_finite() || !a2.is_finite() || !b1.is_finite() || !b2.is_finite() {
        return None;
    }
    let r = a2 - a1;
    let s = b2 - b1;
    let r_len = r.len();
    let s_len = s.len();
    if !(r_len.is_finite() && s_len.is_finite() && r_len > tolerance.linear && s_len > tolerance.linear) {
        return None;
    }
    let den = r.cross(s);
    if !den.is_finite() || den.abs() <= tolerance.angular * r_len * s_len {
        return None;
    }
    let delta = b1 - a1;
    let t = delta.cross(s) / den;
    let u = delta.cross(r) / den;
    let point = a1 + r * t;
    (t.is_finite() && u.is_finite() && point.is_finite()).then_some((point, t, u))
}

/// Intersection of two finite segments.
pub fn line_line(a: &Line, b: &Line) -> Option<Vec2> {
    let bounds = Bounds2::from_points([a.a, a.b, b.a, b.b]);
    let tolerance = ToleranceContext::from_bounds(bounds);
    line_line_with_tolerance(a, b, &tolerance)
}

/// Finite line intersection using a drawing-aware distance tolerance converted
/// into parameter-space tolerances for each segment.
pub fn line_line_with_tolerance(a: &Line, b: &Line, tolerance: &ToleranceContext) -> Option<Vec2> {
    let (p, t, u) = line_line_infinite_with_tolerance(a.a, a.b, b.a, b.b, tolerance)?;
    let ta = (tolerance.linear / a.len().max(tolerance.linear)).max(tolerance.parameter);
    let tb = (tolerance.linear / b.len().max(tolerance.linear)).max(tolerance.parameter);
    ((-ta..=1.0 + ta).contains(&t) && (-tb..=1.0 + tb).contains(&u) && p.is_finite()).then_some(p)
}

/// Intersections of the infinite line through `l` with a circle. Returns points with line params.
/// The compatibility wrapper derives a tolerance from both operands.
pub fn line_circle(l: &Line, c: &Circle) -> Vec<(Vec2, f64)> {
    let bounds = Bounds2::from_points([l.a, l.b]).union(&c.bounds());
    let tolerance = ToleranceContext::from_bounds(bounds);
    line_circle_with_tolerance(l, c, &tolerance)
}

/// Line/circle intersections using the caller's drawing-wide tolerance context.
pub fn line_circle_with_tolerance(l: &Line, c: &Circle, tolerance: &ToleranceContext) -> Vec<(Vec2, f64)> {
    let d = l.b - l.a;
    let f = l.a - c.center;
    let length = d.len();
    if !(length.is_finite() && length > tolerance.linear && c.radius.is_finite()
        && c.radius >= 0.0 && l.a.is_finite() && c.center.is_finite() && f.is_finite()) {
        return Vec::new();
    }

    // Project onto the infinite line and classify by radial distance. This avoids
    // subtracting two large quadratic terms and keeps tangency in drawing units.
    let unit = d / length;
    let along = -f.dot(unit);
    let t = along / length;
    let nearest = f + unit * along;
    let nearest_distance = nearest.len();
    if !t.is_finite() || !nearest_distance.is_finite() {
        return Vec::new();
    }
    let radial_gap = c.radius - nearest_distance;
    if radial_gap < -tolerance.linear {
        return Vec::new();
    }
    let foot = c.center + nearest;
    if !foot.is_finite() {
        return Vec::new();
    }
    if radial_gap.abs() <= tolerance.linear {
        return vec![(foot, t)];
    }

    // Compute the half chord in a normalized frame to avoid overflow in r²-d².
    let scale = c.radius.max(nearest_distance);
    let r = c.radius / scale;
    let n = nearest_distance / scale;
    let half_chord = scale * ((r - n) * (r + n)).max(0.0).sqrt();
    let offset = half_chord / length;
    if !offset.is_finite() {
        return Vec::new();
    }
    let step = unit * half_chord;
    let hits = vec![(foot - step, t - offset), (foot + step, t + offset)];
    hits.into_iter().filter(|(point, parameter)| point.is_finite() && parameter.is_finite()).collect()
}

/// Intersections of two full circles. The compatibility wrapper derives a
/// tolerance from the combined circle bounds.
pub fn circle_circle(c1: &Circle, c2: &Circle) -> Vec<Vec2> {
    let tolerance = ToleranceContext::from_bounds(c1.bounds().union(&c2.bounds()));
    circle_circle_with_tolerance(c1, c2, &tolerance)
}

/// Circle/circle intersections using a shared drawing-wide tolerance context.
pub fn circle_circle_with_tolerance(c1: &Circle, c2: &Circle, tolerance: &ToleranceContext) -> Vec<Vec2> {
    let delta = c2.center - c1.center;
    let d = delta.len();
    let (r1, r2) = (c1.radius, c2.radius);
    if !(d.is_finite() && r1.is_finite() && r2.is_finite()) || d == 0.0 || r1 < 0.0 || r2 < 0.0 {
        return Vec::new();
    }

    // Work at the scale of the two circles: fixed absolute tolerances classify
    // small secants as tangencies, and raw radius squares overflow on large drawings.
    // Measure the chord from the smaller circle: from the larger one the half-chord is a
    // difference of nearly equal numbers, so a tiny circle crossing a huge one is lost.
    let swapped = r2 < r1;
    let (from, dir) = if swapped { (c2.center, -(delta / d)) } else { (c1.center, delta / d) };
    let scale = d.max(r1).max(r2);
    let (ds, r1s, r2s) = (d / scale, r1 / scale, r2 / scale);
    let distance_slack = (tolerance.linear / scale).max(8.0 * f64::EPSILON);
    if ds > r1s + r2s + distance_slack || ds < (r1s - r2s).abs() - distance_slack {
        return Vec::new();
    }
    let (rs, ro) = if swapped { (r2s, r1s) } else { (r1s, r2s) };
    let a = ((rs - ro) * (rs + ro) + ds * ds) / (2.0 * ds);
    let h2 = (rs - a) * (rs + a);
    // Tangency is judged against the smaller circle, which sets the size of the chord, with
    // room for the rounding in `a` (a few ulps of the unit scale over ds, times 2a in h2).
    let normalized_linear = tolerance.linear / scale;
    let tol = (normalized_linear * (2.0 * rs + normalized_linear) / ds)
        .max(8.0 * f64::EPSILON * rs / ds);
    if h2 < -tol {
        return Vec::new();
    }
    let p = from + dir * (a * scale);
    if !p.is_finite() {
        return Vec::new();
    }
    if h2 <= tol {
        return vec![p];
    }
    let step = dir.perp() * (h2.sqrt() * scale);
    let (p1, p2) = (p + step, p - step);
    if !p1.is_finite() || !p2.is_finite() {
        return Vec::new();
    }
    // Keep the result order independent of which circle was used as the base.
    if swapped { vec![p2, p1] } else { vec![p1, p2] }
}


/// Intersections of an ellipse/elliptical arc and a line. The equation is
/// solved in a normalized frame to avoid overflow in cross products and line
/// parameter calculations. The returned points are residual-checked against the
/// operation tolerance; this is a floating-point error-controlled solver, not a
/// formal interval-arithmetic certificate.
pub fn intersect_ellipse_line(e: &Ellipse, line: &Line, extend: bool, tolerance: &ToleranceContext) -> Vec<Vec2> {
    let direction = line.b - line.a;
    let direction_len = direction.len();
    if !direction.is_finite() || !direction_len.is_finite() || direction_len <= tolerance.linear
        || !e.center.is_finite() || !e.major.is_finite() || !e.ratio.is_finite() || e.ratio <= 0.0
    { return Vec::new(); }
    let minor = e.minor();
    if !minor.is_finite() { return Vec::new(); }
    let coordinate_scale = e.center.x.abs().max(e.center.y.abs())
        .max(line.a.x.abs()).max(line.a.y.abs())
        .max(e.major.x.abs()).max(e.major.y.abs())
        .max(minor.x.abs()).max(minor.y.abs()).max(direction_len);
    if !coordinate_scale.is_finite() || coordinate_scale == 0.0 { return Vec::new(); }
    let unit = direction / direction_len;
    let major_n = e.major / coordinate_scale;
    let minor_n = minor / coordinate_scale;
    let offset_n = e.center / coordinate_scale - line.a / coordinate_scale;
    let (a, b, c) = (unit.cross(major_n), unit.cross(minor_n), unit.cross(offset_n));
    let radius = a.hypot(b);
    if !radius.is_finite() || radius <= f64::MIN_POSITIVE { return Vec::new(); }
    let target = -c / radius;
    // Bound the classification slack by the requested geometric tolerance plus
    // accumulated rounding in the normalized coefficient construction.
    let geometric_slack = tolerance.linear / coordinate_scale;
    let roundoff_slack = 32.0 * f64::EPSILON * (a.abs() + b.abs() + c.abs() + 1.0);
    let slack = (geometric_slack + roundoff_slack) / radius;
    if !target.is_finite() || target.abs() > 1.0 + slack { return Vec::new(); }
    let offset = target.clamp(-1.0, 1.0).acos();
    let phase = b.atan2(a);
    let mut out = Vec::with_capacity(2);
    for parameter in [phase + offset, phase - offset] {
        if !e.is_full() && !angle_in_sweep(parameter, e.start, e.end) { continue; }
        let point = e.at_param(parameter);
        if !point.is_finite() { continue; }
        let relative_n = point / coordinate_scale - line.a / coordinate_scale;
        let line_parameter = relative_n.dot(unit) / (direction_len / coordinate_scale);
        let parameter_tolerance = (tolerance.linear / direction_len).max(tolerance.parameter);
        let residual = unit.cross(relative_n).abs() * coordinate_scale;
        let residual_bound = tolerance.linear + coordinate_scale * roundoff_slack;
        if (extend || (-parameter_tolerance..=1.0 + parameter_tolerance).contains(&line_parameter))
            && residual.is_finite() && residual <= residual_bound
            && !out.iter().any(|existing: &Vec2| existing.dist(point) <= tolerance.linear)
        { out.push(point); }
    }
    out
}

/// Count sign changes in a Bernstein coefficient sequence, ignoring exact zeros.
///
/// This is the Descartes sign-variation count used for root-interval pruning.
/// Do not introduce an epsilon cutoff here: a small nonzero coefficient can
/// determine whether a polynomial has one or more roots in the interval.
fn bernstein_sign_variations(values: &[f64]) -> usize {
    let mut previous = 0_i8;
    let mut count = 0;
    for value in values {
        if !value.is_finite() { return 0; }
        let sign = if *value > 0.0 { 1 } else if *value < 0.0 { -1 } else { 0 };
        if sign != 0 {
            if previous != 0 && sign != previous { count += 1; }
            previous = sign;
        }
    }
    count
}

/// Intersections of a rational or non-rational B-spline with a line.
/// Positive-weight splines are decomposed into homogeneous Bézier pieces and
/// isolated in Bernstein form. Sign variation prunes intervals that cannot contain
/// roots; de Casteljau subdivision retains even-multiplicity/tangent roots. The
/// returned points are numerically refined and residual-checked. This is not a
/// formally interval-arithmetic-certified enclosure: floating-point coefficient
/// construction and subdivision still require outward-rounded interval arithmetic
/// before the result can be advertised as a mathematical certificate.
pub fn intersect_spline_line(spline: &Spline, line: &Line, extend: bool, tolerance: &ToleranceContext) -> Vec<Vec2> {
    if !spline.is_valid() || !line.a.is_finite() || !line.b.is_finite() {
        return Vec::new();
    }
    let direction = line.b - line.a;
    let length = direction.len();
    if !length.is_finite() || length <= tolerance.linear {
        return Vec::new();
    }
    let (domain_lo, domain_hi) = spline.domain();
    let parameter_tolerance = (tolerance.linear / length).max(tolerance.parameter)
        * (domain_hi - domain_lo).abs();
    let Some(pieces) = spline.homogeneous_bezier_pieces() else {
        return Vec::new();
    };

    fn split_bernstein(values: &[f64]) -> (Vec<f64>, Vec<f64>) {
        let n = values.len();
        let mut work = values.to_vec();
        let mut left = Vec::with_capacity(n);
        let mut right = Vec::with_capacity(n);
        if let Some(first) = work.first().copied() { left.push(first); }
        if let Some(last) = work.last().copied() { right.push(last); }
        for level in 1..n {
            for i in 0..(n - level) {
                work[i] = work[i] * 0.5 + work[i + 1] * 0.5;
            }
            left.push(work[0]);
            right.push(work[n - level - 1]);
        }
        right.reverse();
        (left, right)
    }

    fn isolate(values: &[f64], lo: f64, hi: f64, target: f64, depth: usize, out: &mut Vec<(f64, f64)>) {
        if values.iter().all(|v| !v.is_finite()) { return; }
        let scale = values.iter().fold(0.0_f64, |m, v| m.max(v.abs()));
        if scale == 0.0 || !scale.is_finite() { return; } // curve coincident with the line or invalid
        let normalized: Vec<f64> = values.iter().map(|v| *v / scale).collect();
        let count = bernstein_sign_variations(&normalized);
        // Only exact zero coefficients are endpoint roots. Treating a merely
        // small coefficient as zero can both invent and discard endpoint roots.
        let endpoint_zero = normalized.first().is_some_and(|v| *v == 0.0)
            || normalized.last().is_some_and(|v| *v == 0.0);
        if count == 0 && !endpoint_zero { return; }
        if depth == 0 || hi - lo <= target {
            out.push((lo, hi));
            return;
        }
        let (left, right) = split_bernstein(&normalized);
        let mid = lo + (hi - lo) * 0.5;
        isolate(&left, lo, mid, target, depth - 1, out);
        isolate(&right, mid, hi, target, depth - 1, out);
    }

    let unit = direction / length;
    let mut intervals = Vec::new();
    for (lo, hi, controls) in pieces {
        // The line equation in homogeneous coordinates is
        // cross(unit_direction, weighted_point - line_origin * weight).
        let mut coefficients = Vec::with_capacity(controls.len());
        for (weighted_point, weight) in controls {
            let relative = weighted_point - line.a * weight;
            let coefficient = unit.cross(relative);
            if !coefficient.is_finite() { return Vec::new(); }
            coefficients.push(coefficient);
        }
        isolate(&coefficients, lo, hi, parameter_tolerance.max(f64::EPSILON * (hi - lo)), 64, &mut intervals);
    }
    intervals.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut merged: Vec<(f64, f64)> = Vec::new();
    for interval in intervals {
        if let Some(last) = merged.last_mut() {
            if interval.0 <= last.1 + parameter_tolerance {
                last.1 = last.1.max(interval.1);
                continue;
            }
        }
        merged.push(interval);
    }

    let mut out = Vec::new();
    for (lo, hi) in merged {
        let mut t = lo + (hi - lo) * 0.5;
        // Safeguarded Newton refinement of the signed line distance.
        for _ in 0..12 {
            let point = spline.eval(t);
            let residual = unit.cross(point - line.a);
            let slope = unit.cross(spline.derivative(t));
            if !residual.is_finite() || !slope.is_finite() || slope.abs() <= f64::MIN_POSITIVE { break; }
            let next = t - residual / slope;
            if !next.is_finite() || next < lo || next > hi { break; }
            if (next - t).abs() <= f64::EPSILON * t.abs().max(1.0) { t = next; break; }
            t = next;
        }
        let point = spline.eval(t);
        let residual = unit.cross(point - line.a).abs();
        let line_parameter = (point - line.a).dot(direction) / direction.len2();
        let line_parameter_tolerance = (tolerance.linear / length).max(tolerance.parameter);
        if !point.is_finite() || !residual.is_finite() || residual > tolerance.linear
            || (!extend && !(-line_parameter_tolerance..=1.0 + line_parameter_tolerance).contains(&line_parameter))
        { continue; }
        if !out.iter().any(|existing: &Vec2| existing.dist(point) <= tolerance.linear) { out.push(point); }
    }
    out
}

/// Intersections between two segments (finite).
pub fn intersect_segments(s1: &Segment, s2: &Segment) -> Vec<Vec2> {
    intersect_segments_ext(s1, s2, false)
}

/// Intersections; with `extend` true, lines are infinite and arcs are full circles
/// (used by TRIM/EXTEND edge mode and apparent intersections).
pub fn intersect_segments_ext(s1: &Segment, s2: &Segment, extend: bool) -> Vec<Vec2> {
    let bounds = s1.bounds().union(&s2.bounds());
    let tolerance = ToleranceContext::from_bounds(bounds);
    intersect_segments_with_tolerance(s1, s2, extend, &tolerance)
}

/// Intersections with an explicit tolerance context. Pass the same context to
/// all pairwise tests in one trim/snap operation so near-coincident decisions
/// remain consistent across the drawing.
pub fn intersect_segments_with_tolerance(
    s1: &Segment,
    s2: &Segment,
    extend: bool,
    tolerance: &ToleranceContext,
) -> Vec<Vec2> {
    match (s1, s2) {
        (Segment::Line(a), Segment::Line(b)) => {
            if extend {
                line_line_infinite_with_tolerance(a.a, a.b, b.a, b.b, tolerance).map(|x| vec![x.0]).unwrap_or_default()
            } else {
                line_line_with_tolerance(a, b, tolerance).into_iter().collect()
            }
        }
        (Segment::Line(l), Segment::Arc { arc, .. }) | (Segment::Arc { arc, .. }, Segment::Line(l)) => {
            let ta = (tolerance.linear / l.len().max(tolerance.linear)).max(tolerance.parameter);
            line_circle_with_tolerance(l, &Circle::new(arc.center, arc.radius), tolerance)
                .into_iter()
                .filter(|(p, t)| extend || ((-ta..=1.0 + ta).contains(t) && arc.closest(*p).dist(*p) <= tolerance.linear.max(tolerance.angular * arc.radius)))
                .map(|(p, _)| p)
                .collect()
        }
        (Segment::Arc { arc: a, .. }, Segment::Arc { arc: b, .. }) => {
            circle_circle_with_tolerance(&Circle::new(a.center, a.radius), &Circle::new(b.center, b.radius), tolerance)
                .into_iter()
                .filter(|p| extend || (a.closest(*p).dist(*p) <= tolerance.linear.max(tolerance.angular * a.radius)
                    && b.closest(*p).dist(*p) <= tolerance.linear.max(tolerance.angular * b.radius)))
                .collect()
        }
    }
}

pub use intersect_segments_ext as intersect_ext;


#[cfg(test)]
mod bernstein_variation_tests {
    use super::bernstein_sign_variations;

    #[test]
    fn small_nonzero_coefficients_are_not_erased_by_an_epsilon_cutoff() {
        // The middle coefficient is far below machine epsilon relative to the
        // outer coefficients, but its negative sign is mathematically decisive.
        assert_eq!(bernstein_sign_variations(&[1.0, -1.0e-30, 1.0]), 2);
    }

    #[test]
    fn exact_zero_coefficients_are_ignored_when_counting_variations() {
        assert_eq!(bernstein_sign_variations(&[1.0, 0.0, -1.0]), 1);
        assert_eq!(bernstein_sign_variations(&[1.0, 0.0, 0.0, 1.0]), 0);
    }

    #[test]
    fn non_finite_bernstein_coefficients_are_rejected() {
        assert_eq!(bernstein_sign_variations(&[1.0, f64::NAN, -1.0]), 0);
    }
}
