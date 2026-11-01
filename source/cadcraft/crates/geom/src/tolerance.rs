//! Drawing-aware numeric tolerances for geometry operations.
//!
//! A single absolute epsilon is not suitable for drawings ranging from microns
//! to survey coordinates. `ToleranceContext` derives a linear tolerance from
//! the model extent and the floating-point resolution at the coordinate scale.
//! Consumers should pass one context through an entire operation so that
//! intersection, snapping, trimming and point-merging decisions agree.

use crate::Bounds2;

/// Tolerances derived for one drawing or one geometry operation.
///
/// `linear` is expressed in drawing units. `parameter` is dimensionless and is
/// intended for curve parameters in `[0, 1]`. `angular` is in radians.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToleranceContext {
    /// Characteristic diagonal of the drawing bounds, or 1 for a degenerate box.
    pub model_scale: f64,
    /// Largest absolute coordinate in the bounds.
    pub coordinate_magnitude: f64,
    /// Distance tolerance in drawing units.
    pub linear: f64,
    /// Parameter-space tolerance for normalized curve parameters.
    pub parameter: f64,
    /// Angular tolerance in radians.
    pub angular: f64,
}

impl ToleranceContext {
    /// Build a tolerance context from finite drawing bounds.
    ///
    /// The tolerance is the larger of a relative model-size term and a
    /// floating-point resolution term. It intentionally does not use a fixed
    /// 1e-9 drawing-unit floor, which would overwhelm very small drawings.
    pub fn from_bounds(bounds: Bounds2) -> Self {
        if bounds.is_empty() || !bounds.min.is_finite() || !bounds.max.is_finite() {
            return Self::for_scale(1.0, 0.0);
        }

        let coordinate_magnitude = bounds
            .min
            .x
            .abs()
            .max(bounds.min.y.abs())
            .max(bounds.max.x.abs())
            .max(bounds.max.y.abs());

        let width = bounds.width();
        let height = bounds.height();
        let diagonal = width.hypot(height);
        let model_scale = if diagonal.is_finite() && diagonal > 0.0 {
            diagonal
        } else if coordinate_magnitude.is_finite() && coordinate_magnitude > 0.0 {
            coordinate_magnitude
        } else {
            1.0
        };

        Self::for_scale(model_scale, coordinate_magnitude)
    }

    /// Build a context when an operation already knows its characteristic
    /// geometry scale and the magnitude of its coordinates.
    pub fn for_scale(model_scale: f64, coordinate_magnitude: f64) -> Self {
        let scale = if model_scale.is_finite() && model_scale > 0.0 { model_scale } else { 1.0 };
        let magnitude = if coordinate_magnitude.is_finite() { coordinate_magnitude.abs() } else { 0.0 };

        // 1e-12 of the model extent gives a scale-relative geometric tolerance.
        // 32 ulps at the coordinate magnitude accounts for loss of resolution
        // when a small object is located far from the world origin.
        let relative = scale * 1.0e-12;
        let floating_point = magnitude * f64::EPSILON * 32.0;
        let smallest = f64::MIN_POSITIVE * 1024.0;
        let linear = relative.max(floating_point).max(smallest);
        let parameter = (linear / scale).clamp(f64::EPSILON * 16.0, 1.0e-3);

        Self { model_scale: scale, coordinate_magnitude: magnitude, linear, parameter, angular: 1.0e-12 }
    }

    /// Whether two finite points are within this operation's linear tolerance.
    pub fn points_near(&self, a: crate::Vec2, b: crate::Vec2) -> bool {
        a.is_finite() && b.is_finite() && a.near(b, self.linear)
    }

    /// Whether a finite scalar distance is within the linear tolerance.
    pub fn distance_near(&self, distance: f64) -> bool {
        distance.is_finite() && distance.abs() <= self.linear
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Vec2;

    #[test]
    fn tolerance_scales_with_small_and_large_drawings() {
        let tiny = ToleranceContext::from_bounds(Bounds2::new(Vec2::ZERO, Vec2::new(1.0e-6, 1.0e-6)));
        let ordinary = ToleranceContext::from_bounds(Bounds2::new(Vec2::ZERO, Vec2::new(100.0, 100.0)));
        assert!(tiny.linear < ordinary.linear);
        assert!((tiny.linear / ordinary.linear - 1.0e-8).abs() < 1.0e-18);
    }

    #[test]
    fn tolerance_accounts_for_large_world_coordinates() {
        let bounds = Bounds2::new(Vec2::new(1.0e7, 1.0e7), Vec2::new(1.0e7 + 100.0, 1.0e7 + 100.0));
        let context = ToleranceContext::from_bounds(bounds);
        assert!(context.linear.is_finite());
        assert!(context.linear >= 1.0e7 * f64::EPSILON * 32.0);
        assert!(context.points_near(Vec2::new(1.0e7, 1.0e7), Vec2::new(1.0e7 + context.linear * 0.5, 1.0e7)));
    }

    #[test]
    fn degenerate_and_invalid_bounds_produce_finite_context() {
        let point = ToleranceContext::from_bounds(Bounds2::new(Vec2::ZERO, Vec2::ZERO));
        let empty = ToleranceContext::from_bounds(Bounds2::EMPTY);
        for context in [point, empty] {
            assert!(context.linear.is_finite() && context.linear > 0.0);
            assert!(context.parameter.is_finite() && context.angular.is_finite());
        }
    }

    #[test]
    fn tolerance_remains_finite_across_extreme_coordinate_scales() {
        for (origin, size) in [(0.0, 1.0e-9), (0.0, 1.0e9), (1.0e12, 1.0e3), (-1.0e12, 1.0e3)] {
            let bounds = Bounds2::new(Vec2::new(origin, origin), Vec2::new(origin + size, origin + size));
            let tolerance = ToleranceContext::from_bounds(bounds);
            assert!(tolerance.linear.is_finite() && tolerance.linear > 0.0);
            assert!(tolerance.parameter.is_finite() && tolerance.angular.is_finite());
            assert!(tolerance.linear < size.max(origin.abs()) * 1.0e-3);
        }
    }

    #[test]
    fn rejects_non_finite_distance_and_points() {
        let context = ToleranceContext::for_scale(10.0, 10.0);
        assert!(!context.distance_near(f64::NAN));
        assert!(!context.points_near(Vec2::new(f64::INFINITY, 0.0), Vec2::ZERO));
    }
}
