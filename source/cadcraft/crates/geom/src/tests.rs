use crate::*;

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}

#[test]
fn angles_normalize() {
    assert!(close(norm_angle(-PI / 2.0), 3.0 * PI / 2.0));
    assert!(close(ccw_sweep(0.0, 0.0), TAU));
    assert!(angle_in_sweep(0.1, 6.0, 0.5));
    assert!(!angle_in_sweep(3.0, 6.0, 0.5));
}

#[test]
fn mat_inverse_roundtrip() {
    let m = Mat3::rotate_about(Vec2::new(3.0, 4.0), 0.7).then(Mat3::scale(2.0, 2.0));
    let inv = m.inverse().unwrap();
    let p = Vec2::new(10.0, -2.0);
    let q = inv.apply(m.apply(p));
    assert!(q.near(p, 1e-9));
}

#[test]
fn mirror_matrix() {
    let m = Mat3::mirror(Vec2::ZERO, Vec2::new(1.0, 1.0));
    assert!(m.apply(Vec2::new(1.0, 0.0)).near(Vec2::new(0.0, 1.0), 1e-12));
    assert!(m.is_mirroring());
}

#[test]
fn circle_through_three_points() {
    let c = Circle::from_3_points(Vec2::new(1.0, 0.0), Vec2::new(0.0, 1.0), Vec2::new(-1.0, 0.0)).unwrap();
    assert!(c.center.near(Vec2::ZERO, 1e-9));
    assert!(close(c.radius, 1.0));
    assert!(Circle::from_3_points(Vec2::ZERO, Vec2::X, Vec2::X * 2.0).is_none());
}

#[test]
fn arc_three_points_direction() {
    let a = Arc::from_3_points(Vec2::new(1.0, 0.0), Vec2::new(0.0, 1.0), Vec2::new(-1.0, 0.0)).unwrap();
    assert!(close(a.sweep(), PI));
    assert!(a.mid_point().near(Vec2::new(0.0, 1.0), 1e-9));
    let b = Arc::from_3_points(Vec2::new(1.0, 0.0), Vec2::new(0.0, -1.0), Vec2::new(-1.0, 0.0)).unwrap();
    assert!(b.mid_point().near(Vec2::new(0.0, -1.0), 1e-9));
}

#[test]
fn arc_bounds_include_quadrants() {
    let a = Arc::new(Vec2::ZERO, 2.0, 0.0, PI);
    let b = a.bounds();
    assert!(close(b.max.y, 2.0));
    assert!(close(b.min.y, 0.0));
}

#[test]
fn bulge_semicircle() {
    let (arc, ccw) = bulge_to_arc(Vec2::new(0.0, 0.0), Vec2::new(2.0, 0.0), 1.0).unwrap();
    assert!(ccw);
    assert!(arc.center.near(Vec2::new(1.0, 0.0), 1e-9));
    assert!(close(arc.radius, 1.0));
    // CCW from (0,0) to (2,0) around (1,0) passes below.
    assert!(arc.mid_point().near(Vec2::new(1.0, -1.0), 1e-9));
    let (arc2, ccw2) = bulge_to_arc(Vec2::new(0.0, 0.0), Vec2::new(2.0, 0.0), -1.0).unwrap();
    assert!(!ccw2);
    assert!(arc2.mid_point().near(Vec2::new(1.0, 1.0), 1e-9));
}

#[test]
fn bulge_quarter_arc() {
    let b = arc_to_bulge(PI / 2.0);
    let (arc, _) = bulge_to_arc(Vec2::new(1.0, 0.0), Vec2::new(0.0, 1.0), b).unwrap();
    assert!(arc.center.near(Vec2::ZERO, 1e-9));
    assert!(close(arc.sweep(), PI / 2.0));
}

#[test]
fn arc_segment_offset_preserves_small_radii_and_rejects_unrepresentable_results() {
    let tiny = Segment::Arc {
        arc: Arc::new(Vec2::ZERO, 1.0e-12, 0.0, PI / 2.0),
        ccw: true,
    };
    match tiny.offset(5.0e-13).expect("a positive microscopic offset radius is valid") {
        Segment::Arc { arc, ccw } => {
            assert!(ccw);
            assert!((arc.radius - 5.0e-13).abs() <= 1.0e-27);
        }
        other => panic!("expected arc, got {other:?}"),
    }

    let huge = Segment::Arc {
        arc: Arc::new(Vec2::ZERO, f64::MAX, 0.0, PI / 2.0),
        ccw: false,
    };
    assert!(huge.offset(f64::MAX).is_none(), "overflowed offset radii must be rejected");
}

#[test]
fn bulge_conversion_preserves_small_angles_and_small_drawing_units() {
    let (large_radius_arc, ccw) = bulge_to_arc(Vec2::ZERO, Vec2::new(10.0, 0.0), 1.0e-13)
        .expect("a representable nonzero bulge must not be rounded to a straight segment");
    assert!(ccw);
    assert!(large_radius_arc.radius.is_finite());
    assert!((large_radius_arc.radius / 2.5e13 - 1.0).abs() < 1.0e-12);

    let (microscopic_arc, ccw) = bulge_to_arc(Vec2::ZERO, Vec2::new(1.0e-15, 0.0), 1.0e-3)
        .expect("a valid arc on a microscopic chord must remain an arc");
    assert!(ccw);
    assert!(microscopic_arc.radius.is_finite() && microscopic_arc.radius > 0.0);
}

#[test]
fn bulge_conversion_rejects_only_exactly_degenerate_or_unrepresentable_inputs() {
    assert!(bulge_to_arc(Vec2::ZERO, Vec2::X, 0.0).is_none());
    assert!(bulge_to_arc(Vec2::ZERO, Vec2::ZERO, 0.5).is_none());
    assert!(bulge_to_arc(Vec2::new(f64::NAN, 0.0), Vec2::X, 0.5).is_none());
}

#[test]
fn tiny_bulge_is_not_a_full_circle() {
    let a = Vec2::new(0.0, 0.0);
    let b = Vec2::new(10.0, 0.0);
    let p = Polyline { vertices: vec![PolyVertex::with_bulge(a, 1e-10), PolyVertex::new(b)], closed: false };
    assert!((p.len() - 10.0).abs() < 1e-6, "len={}", p.len());
    let bb = p.bounds();
    assert!(bb.max.x < 10.001 && bb.max.y < 0.001 && bb.min.y > -0.001, "bounds={bb:?}");
}

#[test]
fn polyline_area_and_length() {
    let p = Polyline::from_points(&[Vec2::new(0.0, 0.0), Vec2::new(4.0, 0.0), Vec2::new(4.0, 3.0), Vec2::new(0.0, 3.0)], true);
    assert!(close(p.area(), 12.0));
    assert!(close(p.len(), 14.0));
    assert!(p.contains(Vec2::new(1.0, 1.0)));
    assert!(!p.contains(Vec2::new(5.0, 1.0)));
}

#[test]
fn line_intersections() {
    let a = Line::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
    let b = Line::new(Vec2::new(0.0, 10.0), Vec2::new(10.0, 0.0));
    assert!(line_line(&a, &b).unwrap().near(Vec2::new(5.0, 5.0), 1e-9));
    let c = Line::new(Vec2::new(20.0, 0.0), Vec2::new(30.0, -10.0));
    assert!(line_line(&a, &c).is_none());
}

#[test]
fn line_circle_hits() {
    let l = Line::new(Vec2::new(-5.0, 0.0), Vec2::new(5.0, 0.0));
    let hits = line_circle(&l, &Circle::new(Vec2::ZERO, 2.0));
    assert_eq!(hits.len(), 2);
    let segs = intersect_segments(&Segment::Line(l), &Segment::Arc { arc: Arc::new(Vec2::ZERO, 2.0, 0.0, PI / 2.0), ccw: true });
    assert_eq!(segs.len(), 1);
    assert!(segs[0].near(Vec2::new(2.0, 0.0), 1e-9));
}

#[test]
fn circle_circle_hits() {
    let h = circle_circle(&Circle::new(Vec2::ZERO, 5.0), &Circle::new(Vec2::new(8.0, 0.0), 5.0));
    assert_eq!(h.len(), 2);
    assert!(h.iter().all(|p| close(p.x, 4.0) && close(p.y.abs(), 3.0)));
}


#[test]
fn tolerance_aware_line_circle_uses_shared_drawing_tolerance() {
    use crate::{line_circle_with_tolerance, ToleranceContext};

    let circle = Circle::new(Vec2::ZERO, 1.0);
    let tolerance = ToleranceContext::for_scale(1.0, 0.0);
    let near_tangent = Line::new(Vec2::new(-2.0, 1.0 + tolerance.linear * 0.5), Vec2::new(2.0, 1.0 + tolerance.linear * 0.5));
    let outside = Line::new(Vec2::new(-2.0, 1.0 + tolerance.linear * 4.0), Vec2::new(2.0, 1.0 + tolerance.linear * 4.0));

    assert_eq!(line_circle_with_tolerance(&near_tangent, &circle, &tolerance).len(), 1);
    assert!(line_circle_with_tolerance(&outside, &circle, &tolerance).is_empty());
}

#[test]
fn tolerance_aware_circle_circle_respects_shared_context() {
    use crate::{circle_circle_with_tolerance, ToleranceContext};

    let first = Circle::new(Vec2::ZERO, 1.0);
    let tolerance = ToleranceContext::for_scale(1.0, 0.0);
    let near_tangent = Circle::new(Vec2::new(2.0 + tolerance.linear * 0.5, 0.0), 1.0);
    let separated = Circle::new(Vec2::new(2.0 + tolerance.linear * 4.0, 0.0), 1.0);

    assert_eq!(circle_circle_with_tolerance(&first, &near_tangent, &tolerance).len(), 1);
    assert!(circle_circle_with_tolerance(&first, &separated, &tolerance).is_empty());
}

#[test]
fn small_circle_crossing_a_huge_circle_gives_two_hits() {
    // Radii 1e7 and 1 with centres 1e7 apart: the small circle crosses the large one.
    let big = Circle::new(Vec2::ZERO, 1e7);
    let small = Circle::new(Vec2::new(1e7, 0.0), 1.0);
    for hits in [circle_circle(&big, &small), circle_circle(&small, &big)] {
        assert_eq!(hits.len(), 2, "{hits:?}");
        for p in &hits {
            assert!((p.dist(small.center) - 1.0).abs() < 1e-6, "{p:?} is off the small circle");
            assert!((p.dist(big.center) - 1e7).abs() < 1e-6, "{p:?} is off the big circle");
        }
        assert!(hits[0].dist(hits[1]) > 1.0, "{hits:?}");
    }
    // Touching from outside and from inside is still one tangent point, in both orders.
    for (x, touch) in [(1e7 + 1.0, 1e7), (1e7 - 1.0, 1e7)] {
        let small = Circle::new(Vec2::new(x, 0.0), 1.0);
        for hits in [circle_circle(&big, &small), circle_circle(&small, &big)] {
            assert_eq!(hits.len(), 1, "{hits:?}");
            assert!(hits[0].dist(Vec2::new(touch, 0.0)) < 1e-6, "{hits:?}");
        }
    }
}

#[test]
fn tiny_circle_secants_and_separated_circles() {
    let first = Circle::new(Vec2::ZERO, 1e-9);
    // Both radii and center separation are below the old absolute tolerance.
    let hits = circle_circle(&first, &Circle::new(Vec2::new(5e-10, 0.0), 1e-9));
    assert_eq!(hits.len(), 2);
    let y = (1e-18_f64 - 2.5e-10_f64.powi(2)).sqrt();
    assert!(hits.iter().all(|p| (p.x - 2.5e-10).abs() < 1e-20 && (p.y.abs() - y).abs() < 1e-20), "{hits:?}");
    // A half-nanometre gap is significant relative to these circles.
    assert!(circle_circle(&first, &Circle::new(Vec2::new(2.5e-9, 0.0), 1e-9)).is_empty());
}

#[test]
fn huge_circle_secants_remain_finite() {
    let hits = circle_circle(&Circle::new(Vec2::ZERO, 1e160), &Circle::new(Vec2::new(1e160, 0.0), 1e160));
    assert_eq!(hits.len(), 2);
    assert!(
        hits.iter().all(|p| p.is_finite() && (p.x / 1e160 - 0.5).abs() < 1e-12 && (p.y.abs() / 1e160 - 3.0_f64.sqrt() / 2.0).abs() < 1e-12),
        "{hits:?}"
    );
}

#[test]
fn spline_interpolates_fit_points() {
    let fit = [Vec2::new(0.0, 0.0), Vec2::new(1.0, 2.0), Vec2::new(3.0, 1.0), Vec2::new(4.0, 4.0), Vec2::new(6.0, 0.0)];
    let s = Spline::from_fit_points(&fit);
    assert!(s.is_valid());
    let pts = s.tessellate(1e-4);
    for f in fit {
        let d = pts.iter().map(|p| p.dist(f)).fold(f64::INFINITY, f64::min);
        assert!(d < 0.05, "fit point {f:?} missed by {d}");
    }
    assert!(s.eval(0.0).near(fit[0], 1e-9));
    assert!(s.eval(1.0).near(fit[4], 1e-9));
}

#[test]
fn closed_spline_is_smooth_at_the_seam() {
    let fit = [Vec2::new(0.0, 0.0), Vec2::new(4.0, 0.0), Vec2::new(4.0, 3.0), Vec2::new(0.0, 3.0)];
    let s = Spline::from_fit_points_closed(&fit);
    assert!(s.is_valid() && s.closed && s.is_periodic());
    assert_eq!(s.fit.len(), 4, "the first fit point is not repeated");
    let (lo, hi) = s.domain();
    assert!(s.eval(lo).near(fit[0], 1e-9) && s.eval(hi).near(fit[0], 1e-9));
    let pts = s.tessellate(1e-4);
    for f in fit {
        assert!(pts.iter().any(|p| p.dist(f) < 0.05), "fit point {f:?} missed");
    }
    // Same tangent leaving and arriving at the seam (an open fit through a repeated point has a corner there).
    let h = 1e-6;
    let out = (s.eval(lo + h) - s.eval(lo)).normalized();
    let back = (s.eval(hi) - s.eval(hi - h)).normalized();
    assert!(out.near(back, 1e-4), "kink at the seam: {out:?} vs {back:?}");
    // A repeated closing point is dropped.
    let mut dup = fit.to_vec();
    dup.push(fit[0]);
    assert_eq!(Spline::from_fit_points_closed(&dup).fit.len(), 4);
}

#[test]
fn ellipse_points() {
    let e = Ellipse::full(Vec2::ZERO, Vec2::new(4.0, 0.0), 0.5);
    assert!(e.at_param(PI / 2.0).near(Vec2::new(0.0, 2.0), 1e-9));
    let b = e.bounds();
    assert!((b.width() - 8.0).abs() < 1e-3);
}

#[test]
fn segment_offset() {
    let s = Segment::Arc { arc: Arc::new(Vec2::ZERO, 5.0, 0.0, PI), ccw: true };
    let o = s.offset(1.0).unwrap();
    if let Segment::Arc { arc, .. } = o {
        assert!(close(arc.radius, 4.0))
    } else {
        panic!()
    }
    assert!(s.offset(6.0).is_none());
}

#[test]
fn arc_segments_bounded() {
    assert_eq!(arc_segments(0.0, 1.0, 0.1), 1);
    assert!(arc_segments(1e9, TAU, 1e-12) <= 4096);
    assert!(arc_segments(f64::NAN, TAU, 0.1) == 1);
}

#[test]
fn ocs_identity_for_z() {
    assert_eq!(Mat4::ocs(Vec3::Z), Mat4::IDENTITY);
    let m = Mat4::ocs(Vec3::new(0.0, 0.0, -1.0));
    let p = m.apply(Vec3::new(1.0, 0.0, 0.0));
    assert!((p.x + 1.0).abs() < 1e-9);
}

#[test]
fn common_tangents_of_two_circles() {
    // Distance from `c` to the infinite line through `p`, `q`.
    let dist = |c: Vec2, p: Vec2, q: Vec2| ((q - p).cross(c - p) / p.dist(q)).abs();
    let (a, b) = (Circle::new(Vec2::ZERO, 10.0), Circle::new(Vec2::new(30.0, 0.0), 4.0));
    let ts = a.common_tangents(&b);
    assert_eq!(ts.len(), 4, "separate circles: 2 outer + 2 inner tangents");
    for (p, q) in &ts {
        assert!(close(p.dist(a.center), 10.0) && close(q.dist(b.center), 4.0));
        assert!(close(dist(a.center, *p, *q), 10.0) && close(dist(b.center, *p, *q), 4.0));
    }
    // Overlapping circles only have the two outer tangents.
    assert_eq!(a.common_tangents(&Circle::new(Vec2::new(12.0, 0.0), 4.0)).len(), 2);
    // Concentric, nested, touching inside, non-finite: none, and no panic.
    assert!(a.common_tangents(&Circle::new(Vec2::ZERO, 3.0)).is_empty());
    assert!(a.common_tangents(&Circle::new(Vec2::new(1.0, 0.0), 3.0)).is_empty());
    assert!(a.common_tangents(&Circle::new(Vec2::new(6.0, 0.0), 4.0)).is_empty());
    assert!(a.common_tangents(&Circle::new(Vec2::new(f64::NAN, 0.0), 4.0)).is_empty());
    assert!(a.common_tangents(&Circle::new(Vec2::new(f64::INFINITY, 0.0), 4.0)).is_empty());
    // A zero-radius circle (a point): the tangents from that point.
    let z = a.common_tangents(&Circle::new(Vec2::new(30.0, 0.0), 0.0));
    assert!(!z.is_empty() && z.iter().all(|(p, q)| close(dist(a.center, *p, *q), 10.0) && q.near(Vec2::new(30.0, 0.0), 1e-9)));
}

#[test]
fn circle_three_points_preserves_large_world_coordinates() {
    // A unit circle near a survey coordinate. The absolute-square formula suffers
    // catastrophic cancellation and returns a center tens of millions of units away.
    let origin = 1.0e12;
    let c = Circle::from_3_points(Vec2::new(origin + 1.0, origin), Vec2::new(origin, origin + 1.0), Vec2::new(origin - 1.0, origin)).unwrap();
    assert!(c.center.near(Vec2::new(origin, origin), 1e-3), "{c:?}");
    assert!((c.radius - 1.0).abs() < 1e-3, "{c:?}");
}

#[test]
fn circle_three_points_accepts_small_valid_triangles() {
    let r = 1.0e-7;
    let c = Circle::from_3_points(Vec2::new(r, 0.0), Vec2::new(0.0, r), Vec2::new(-r, 0.0)).unwrap();
    assert!(c.center.near(Vec2::ZERO, 1e-12), "{c:?}");
    assert!((c.radius - r).abs() < 1e-12, "{c:?}");
    assert!(Circle::from_3_points(Vec2::ZERO, Vec2::new(r, 0.0), Vec2::new(2.0 * r, 0.0)).is_none());
    assert!(Circle::from_3_points(Vec2::ZERO, Vec2::X, Vec2::new(f64::NAN, 0.0)).is_none());
}

#[test]
fn ellipse_bounds_find_analytic_extrema() {
    // A tilted ellipse's extrema generally fall between tessellated samples.
    let center = Vec2::new(7.0, -11.0);
    let e = Ellipse::full(center, Vec2::new(3.0, 4.0), 0.5);
    let bounds = e.bounds();
    let dx = 13.0_f64.sqrt(); // hypot(major.x, minor.x)
    let dy = 18.25_f64.sqrt(); // hypot(major.y, minor.y)
    assert!((bounds.min.x - (center.x - dx)).abs() < 1e-12);
    assert!((bounds.max.x - (center.x + dx)).abs() < 1e-12);
    assert!((bounds.min.y - (center.y - dy)).abs() < 1e-12);
    assert!((bounds.max.y - (center.y + dy)).abs() < 1e-12);
}

#[test]
fn elliptical_arc_bounds_exclude_extrema_outside_the_sweep() {
    let e = Ellipse { center: Vec2::ZERO, major: Vec2::new(3.0, 4.0), ratio: 0.5, start: 0.0, end: PI / 4.0 };
    let bounds = e.bounds();
    assert!((bounds.max.x - 3.0).abs() < 1e-12);
    assert!((bounds.min.x - e.at_param(e.end).x).abs() < 1e-12);
    assert!((bounds.min.y - e.at_param(e.end).y).abs() < 1e-12);
    assert!((bounds.max.y - 18.25_f64.sqrt()).abs() < 1e-12);
}

#[test]
fn polyline_area_is_exact_for_bulge_segments() {
    // One semicircular arc from left to right plus a straight closing edge.
    let mut p = Polyline::from_points(&[Vec2::new(0.0, 0.0), Vec2::new(2.0, 0.0)], true);
    p.vertices[0].bulge = 1.0;
    assert!((p.area() - PI / 2.0).abs() < 1e-12, "{}", p.area());
    p.vertices[0].bulge = -1.0;
    assert!((p.area() + PI / 2.0).abs() < 1e-12, "{}", p.area());
}

#[test]
fn polyline_area_is_stable_far_from_the_origin() {
    let o = 1.0e12;
    let p = Polyline::from_points(&[Vec2::new(o, o), Vec2::new(o + 10.0, o), Vec2::new(o + 10.0, o + 10.0), Vec2::new(o, o + 10.0)], true);
    assert!((p.area() - 100.0).abs() < 1e-12, "{}", p.area());
    // An open polyline's measured area retains the implicit straight closing edge.
    let mut open = p;
    open.closed = false;
    assert!((open.area() - 100.0).abs() < 1e-12);
}

#[test]
fn shoelace_preserves_small_polygon_area_at_large_coordinates() {
    let o = 1.0e12;
    let corners = [Vec2::new(o, o), Vec2::new(o + 10.0, o), Vec2::new(o + 10.0, o + 10.0), Vec2::new(o, o + 10.0)];
    assert_eq!(shoelace(&corners), 100.0);
    assert_eq!(shoelace(&corners.iter().rev().copied().collect::<Vec<_>>()), -100.0);
}

#[test]
fn near_tangent_long_line_does_not_invent_a_circle_hit() {
    let c = Circle::new(Vec2::ZERO, 1.0);
    let far = 1.0e8;
    let outside = Line::new(Vec2::new(-far, 1.0 + 1e-9), Vec2::new(far, 1.0 + 1e-9));
    assert!(line_circle(&outside, &c).is_empty());
    let tangent = Line::new(Vec2::new(-far, 1.0), Vec2::new(far, 1.0));
    let hits = line_circle(&tangent, &c);
    assert_eq!(hits.len(), 1);
    assert!(hits[0].0.near(Vec2::new(0.0, 1.0), 1e-9));
}

#[test]
fn long_secant_keeps_both_intersections() {
    let c = Circle::new(Vec2::ZERO, 2.0);
    let l = Line::new(Vec2::new(-1.0e8, 0.0), Vec2::new(1.0e8, 0.0));
    let hits = line_circle(&l, &c);
    assert_eq!(hits.len(), 2);
    assert!(hits[0].0.near(Vec2::new(-2.0, 0.0), 1e-8), "{hits:?}");
    assert!(hits[1].0.near(Vec2::new(2.0, 0.0), 1e-8), "{hits:?}");
}

#[test]
fn open_polyline_centroid_ignores_last_bulge() {
    let pts = [Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0), Vec2::new(10.0, 10.0)];
    let mut pl = Polyline::from_points(&pts, false);
    let before = pl.centroid().unwrap();
    pl.vertices[2].bulge = 1.0;
    let after = pl.centroid().unwrap();
    assert!(before.near(after, 1e-9), "{before:?} -> {after:?}");
}

#[test]
fn ellipse_closest_refines_to_analytic_curve() {
    let ellipse = Ellipse::full(Vec2::new(4.0, -3.0), Vec2::new(8.0, 0.0), 0.25);
    let query = Vec2::new(1.0, 5.0);
    let closest = ellipse.closest(query);
    let parameter = ellipse.param_of(closest);
    assert!(closest.is_finite());
    assert!((ellipse.at_param(parameter).dist(closest)) < 1e-6);
    // The result must be no worse than a dense independent reference sampling.
    let reference = (0..=20_000)
        .map(|i| ellipse.at_param(std::f64::consts::TAU * i as f64 / 20_000.0).dist(query))
        .fold(f64::INFINITY, f64::min);
    assert!(closest.dist(query) <= reference + 1e-7);
}

#[test]
fn ellipse_and_spline_closest_accept_shared_operation_tolerance() {
    let ellipse = Ellipse::full(Vec2::new(1.0e8, -1.0e8), Vec2::new(100.0, 0.0), 0.4);
    let spline = Spline::from_control(
        vec![Vec2::new(1.0e8, -1.0e8), Vec2::new(1.0e8 + 30.0, -1.0e8 + 50.0),
             Vec2::new(1.0e8 + 70.0, -1.0e8 - 20.0), Vec2::new(1.0e8 + 100.0, -1.0e8)], 3);
    let query = Vec2::new(1.0e8 + 45.0, -1.0e8 + 10.0);
    let bounds = ellipse.bounds().union(&spline.bounds()).union(&Bounds2::new(query, query));
    let tolerance = ToleranceContext::from_bounds(bounds);
    let e = ellipse.closest_with_tolerance(query, &tolerance);
    let s = spline.closest_with_tolerance(query, &tolerance).unwrap_or_default();
    assert!(e.is_finite() && s.is_finite());
    assert!((ellipse.closest(query).dist(e)) < tolerance.linear.max(1.0e-6));
    assert!((spline.closest(query).unwrap_or_default().dist(s)) < tolerance.linear.max(1.0e-6));
}

#[test]
fn ellipse_arc_closest_respects_endpoints() {
    let ellipse = Ellipse { center: Vec2::ZERO, major: Vec2::new(5.0, 0.0), ratio: 0.5, start: 0.0, end: std::f64::consts::FRAC_PI_2 };
    let query = Vec2::new(8.0, -4.0);
    let closest = ellipse.closest(query);
    let start = ellipse.at_param(ellipse.start);
    let end = ellipse.at_param(ellipse.end);
    assert!(closest.dist(query) <= start.dist(query).min(end.dist(query)) + 1e-8);
}

#[test]
fn spline_closest_refines_beyond_display_sample_vertices() {
    let spline = Spline::from_control(
        vec![Vec2::new(0.0, 0.0), Vec2::new(2.0, 5.0), Vec2::new(6.0, -2.0), Vec2::new(9.0, 2.0)],
        3,
    );
    let query = Vec2::new(4.2, 1.1);
    let closest = spline.closest(query).unwrap_or_default();
    assert!(closest.is_finite());
    assert!(closest.is_finite());
    let reference = (0..=50_000)
        .map(|i| spline.eval(i as f64 / 50_000.0).dist(query))
        .fold(f64::INFINITY, f64::min);
    assert!(closest.dist(query) <= reference + 1e-6);
}

#[test]
fn spline_closest_rejects_invalid_input() {
    let spline = Spline::default();
    assert!(spline.closest(Vec2::ZERO).is_none());
    let valid = Spline::from_control(vec![Vec2::ZERO, Vec2::X], 1);
    assert!(valid.closest(Vec2::new(f64::NAN, 0.0)).is_none());
}

#[test]
fn drawing_aware_intersection_handles_scale_and_keeps_finite_segments() {
    let a = Segment::Line(Line::new(Vec2::new(1.0e7, 1.0e7), Vec2::new(1.0e7 + 10.0, 1.0e7)));
    let b = Segment::Line(Line::new(Vec2::new(1.0e7 + 5.0, 1.0e7 - 2.0), Vec2::new(1.0e7 + 5.0, 1.0e7 + 2.0)));
    let hits = intersect_segments(&a, &b);
    assert_eq!(hits.len(), 1);
    assert!(hits[0].near(Vec2::new(1.0e7 + 5.0, 1.0e7), 1e-7));

    let short_a = Segment::Line(Line::new(Vec2::ZERO, Vec2::X));
    let short_b = Segment::Line(Line::new(Vec2::new(1.0 + 1e-8, -1.0), Vec2::new(1.0 + 1e-8, 1.0)));
    assert!(intersect_segments(&short_a, &short_b).is_empty());
}

#[test]
fn exact_ellipse_line_intersections_find_both_crossings() {
    let ellipse = Ellipse::full(Vec2::ZERO, Vec2::new(5.0, 0.0), 0.5);
    let line = Line::new(Vec2::new(0.0, -10.0), Vec2::new(0.0, 10.0));
    let tolerance = ToleranceContext::from_bounds(ellipse.bounds());
    let hits = intersect_ellipse_line(&ellipse, &line, false, &tolerance);
    assert_eq!(hits.len(), 2);
    assert!(hits.iter().any(|p| p.near(Vec2::new(0.0, 2.5), 1e-8)));
    assert!(hits.iter().any(|p| p.near(Vec2::new(0.0, -2.5), 1e-8)));
}

#[test]
fn exact_ellipse_line_intersection_respects_elliptical_arc_domain() {
    let ellipse = Ellipse { center: Vec2::ZERO, major: Vec2::new(5.0, 0.0), ratio: 0.5, start: 0.0, end: std::f64::consts::FRAC_PI_2 };
    let line = Line::new(Vec2::new(-10.0, 0.0), Vec2::new(10.0, 0.0));
    let tolerance = ToleranceContext::from_bounds(ellipse.bounds());
    let hits = intersect_ellipse_line(&ellipse, &line, false, &tolerance);
    assert_eq!(hits.len(), 1);
    assert!(hits[0].near(Vec2::new(5.0, 0.0), 1e-8));
}

#[test]
fn spline_line_bernstein_isolation_finds_three_roots_in_one_bezier_span() {
    // Bernstein coefficients of (t - .2)(t - .5)(t - .8).
    let spline = Spline::from_control(vec![
        Vec2::new(0.0, -0.08),
        Vec2::new(1.0 / 3.0, 0.14),
        Vec2::new(2.0 / 3.0, -0.14),
        Vec2::new(1.0, 0.08),
    ], 3);
    let line = Line::new(Vec2::new(-1.0, 0.0), Vec2::new(2.0, 0.0));
    let tolerance = ToleranceContext::for_scale(3.0, 3.0);
    let hits = intersect_spline_line(&spline, &line, false, &tolerance);
    assert_eq!(hits.len(), 3, "expected all three isolated roots, got {hits:?}");
    for (hit, expected) in hits.iter().zip([0.2, 0.5, 0.8]) {
        assert!((hit.x - expected).abs() < 1e-7, "{hit:?} != {expected}");
        assert!(hit.y.abs() < 1e-9);
    }
}

#[test]
fn spline_line_intersection_finds_crossings_and_tangent_contact() {
    let tolerance = ToleranceContext::for_scale(4.0, 4.0);
    let crossing = Spline::from_control(vec![Vec2::new(0.0, -1.0), Vec2::new(1.0, 1.0)], 1);
    let line = Line::new(Vec2::new(-1.0, 0.0), Vec2::new(2.0, 0.0));
    let hits = intersect_spline_line(&crossing, &line, false, &tolerance);
    assert_eq!(hits.len(), 1);
    assert!(hits[0].near(Vec2::new(0.5, 0.0), 1e-7));

    let tangent = Spline::from_control(vec![Vec2::new(0.0, 1.0), Vec2::new(1.0, -1.0), Vec2::new(2.0, 1.0)], 2);
    let tangent_hits = intersect_spline_line(&tangent, &line, false, &tolerance);
    assert_eq!(tangent_hits.len(), 1);
    assert!(tangent_hits[0].near(Vec2::new(1.0, 0.0), 1e-6));
}
