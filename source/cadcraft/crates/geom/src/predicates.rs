//! Scale-safe 2D predicates used for topology decisions.
//!
//! `orient2d` returns the sign of the signed area of triangle (a,b,c). It
//! normalizes coordinate differences before multiplication and uses fused
//! multiply-add residuals to recover product rounding error. This is not a
//! general arbitrary-precision arithmetic package, but it is substantially
//! more reliable than a raw `cross()` for CAD topology near collinearity.

use std::cmp::Ordering;

use crate::Vec2;

/// Exact sign of `(b - a) x (c - a)` for finite IEEE-754 binary64 inputs.
///
/// Coordinates are decoded into signed integer significands on one shared binary
/// exponent, then the determinant is evaluated with fixed-width integer arithmetic.
/// A finite f64 needs at most 2,098 bits on that shared scale; coordinate differences
/// fit in 34 limbs and their products fit in 68 limbs. This avoids overflow,
/// underflow, cancellation and dependence on FMA/compiler floating-point behaviour.
/// Non-finite inputs are deliberately reported as Equal because they have no
/// meaningful exact planar orientation.
pub fn orient2d(a: Vec2, b: Vec2, c: Vec2) -> Ordering {
    if !a.is_finite() || !b.is_finite() || !c.is_finite() { return Ordering::Equal; }
    // Shewchuk's first-stage determinant filter handles the common, well-
    // separated case cheaply. Any overflow, underflow or uncertain sign falls
    // through to the exact integer determinant below.
    let acx = a.x - c.x;
    let bcx = b.x - c.x;
    let acy = a.y - c.y;
    let bcy = b.y - c.y;
    let left = acx * bcy;
    let right = acy * bcx;
    let det = left - right;
    let products_safe = [acx, bcx, acy, bcy, left, right, det].iter().all(|v| v.is_finite())
        && ![(acx, bcy, left), (acy, bcx, right)].iter()
            .any(|(x, y, product)| *x != 0.0 && *y != 0.0 && product.abs() < f64::MIN_POSITIVE);
    if products_safe {
        let det_sum = left.abs() + right.abs();
        let error_bound = (3.0 + 16.0 * f64::EPSILON) * f64::EPSILON * det_sum;
        if det.abs() > error_bound { return det.total_cmp(&0.0); }
    }

    let values = [a.x, a.y, b.x, b.y, c.x, c.y];
    let min_exponent = values.iter().copied().filter(|v| *v != 0.0)
        .map(|v| decompose_f64(v).1).min();
    let Some(base_exponent) = min_exponent else { return Ordering::Equal; };
    let coords = values.map(|v| ExactSigned::from_f64(v, base_exponent));
    let abx = coords[2].sub(coords[0]);
    let aby = coords[3].sub(coords[1]);
    let acx = coords[4].sub(coords[0]);
    let acy = coords[5].sub(coords[1]);
    let left = ExactProduct::multiply(&abx, &acy);
    let right = ExactProduct::multiply(&aby, &acx);
    ExactProduct::subtract(left, right).sign.cmp(&0)
}

/// A signed integer magnitude with enough headroom for every finite binary64
/// coordinate after alignment to the smallest exponent in one predicate.
#[derive(Clone, Copy)]
struct ExactSigned {
    sign: i8,
    limbs: [u64; 34],
}

impl ExactSigned {
    fn zero() -> Self { Self { sign: 0, limbs: [0; 34] } }

    fn from_f64(value: f64, base_exponent: i32) -> Self {
        if value == 0.0 { return Self::zero(); }
        let (negative, exponent, mantissa) = decompose_f64(value);
        let shift = (exponent - base_exponent) as usize;
        let word = shift / 64;
        let bits = shift % 64;
        let mut limbs = [0_u64; 34];
        if word < limbs.len() {
            limbs[word] |= mantissa << bits;
            if bits != 0 && word + 1 < limbs.len() {
                limbs[word + 1] |= mantissa >> (64 - bits);
            }
        }
        let mut result = Self { sign: if negative { -1 } else { 1 }, limbs };
        if result.is_zero() { result.sign = 0; }
        result
    }

    fn is_zero(&self) -> bool { self.limbs.iter().all(|v| *v == 0) }

    fn cmp_magnitude(&self, other: &Self) -> Ordering {
        for i in (0..34).rev() {
            match self.limbs[i].cmp(&other.limbs[i]) {
                Ordering::Equal => {}, order => return order,
            }
        }
        Ordering::Equal
    }

    fn add_magnitude(a: &[u64; 34], b: &[u64; 34]) -> [u64; 34] {
        let mut out = [0_u64; 34];
        let mut carry = 0_u128;
        for i in 0..34 {
            let sum = a[i] as u128 + b[i] as u128 + carry;
            out[i] = sum as u64;
            carry = sum >> 64;
        }
        out
    }

    fn sub_magnitude(a: &[u64; 34], b: &[u64; 34]) -> [u64; 34] {
        let mut out = [0_u64; 34];
        let mut borrow = 0_u64;
        for i in 0..34 {
            let (first, b1) = a[i].overflowing_sub(b[i]);
            let (value, b2) = first.overflowing_sub(borrow);
            out[i] = value;
            borrow = (b1 || b2) as u64;
        }
        out
    }

    fn sub(self, other: Self) -> Self {
        if other.sign == 0 { return self; }
        if self.sign == 0 { return Self { sign: -other.sign, limbs: other.limbs }; }
        if self.sign != other.sign {
            let limbs = Self::add_magnitude(&self.limbs, &other.limbs);
            return Self { sign: self.sign, limbs };
        }
        match self.cmp_magnitude(&other) {
            Ordering::Equal => Self::zero(),
            Ordering::Greater => Self { sign: self.sign, limbs: Self::sub_magnitude(&self.limbs, &other.limbs) },
            Ordering::Less => Self { sign: -self.sign, limbs: Self::sub_magnitude(&other.limbs, &self.limbs) },
        }
    }
}

/// Exact product of two 34-limb signed integers.
#[derive(Clone, Copy)]
struct ExactProduct {
    sign: i8,
    limbs: [u64; 68],
}

impl ExactProduct {
    fn multiply(a: &ExactSigned, b: &ExactSigned) -> Self {
        let mut out = [0_u64; 68];
        if a.sign == 0 || b.sign == 0 { return Self { sign: 0, limbs: out }; }
        for i in 0..34 {
            let mut carry = 0_u128;
            for j in 0..34 {
                let k = i + j;
                let value = (a.limbs[i] as u128) * (b.limbs[j] as u128)
                    + out[k] as u128 + carry;
                out[k] = value as u64;
                carry = value >> 64;
            }
            out[i + 34] = carry as u64;
        }
        Self { sign: a.sign * b.sign, limbs: out }
    }

    fn cmp_magnitude(&self, other: &Self) -> Ordering {
        for i in (0..68).rev() {
            match self.limbs[i].cmp(&other.limbs[i]) {
                Ordering::Equal => {}, order => return order,
            }
        }
        Ordering::Equal
    }

    fn add_magnitude(a: &[u64; 68], b: &[u64; 68]) -> [u64; 68] {
        let mut out = [0_u64; 68];
        let mut carry = 0_u128;
        for i in 0..68 {
            let sum = a[i] as u128 + b[i] as u128 + carry;
            out[i] = sum as u64;
            carry = sum >> 64;
        }
        out
    }

    fn sub_magnitude(a: &[u64; 68], b: &[u64; 68]) -> [u64; 68] {
        let mut out = [0_u64; 68];
        let mut borrow = 0_u64;
        for i in 0..68 {
            let (first, b1) = a[i].overflowing_sub(b[i]);
            let (value, b2) = first.overflowing_sub(borrow);
            out[i] = value;
            borrow = (b1 || b2) as u64;
        }
        out
    }

    fn subtract(a: Self, b: Self) -> Self {
        if b.sign == 0 { return a; }
        if a.sign == 0 { return Self { sign: -b.sign, limbs: b.limbs }; }
        if a.sign != b.sign {
            return Self { sign: a.sign, limbs: Self::add_magnitude(&a.limbs, &b.limbs) };
        }
        match a.cmp_magnitude(&b) {
            Ordering::Equal => Self { sign: 0, limbs: [0; 68] },
            Ordering::Greater => Self { sign: a.sign, limbs: Self::sub_magnitude(&a.limbs, &b.limbs) },
            Ordering::Less => Self { sign: -a.sign, limbs: Self::sub_magnitude(&b.limbs, &a.limbs) },
        }
    }
}

/// Returns (negative, binary exponent, integer significand) such that
/// `abs(value) = significand * 2^exponent` exactly.
fn decompose_f64(value: f64) -> (bool, i32, u64) {
    let bits = value.to_bits();
    let negative = bits >> 63 != 0;
    let exponent_bits = ((bits >> 52) & 0x7ff) as i32;
    let fraction = bits & ((1_u64 << 52) - 1);
    if exponent_bits == 0 { (negative, -1074, fraction) }
    else { (negative, exponent_bits - 1023 - 52, fraction | (1_u64 << 52)) }
}

/// Whether `c` lies within `linear_tolerance` of the infinite line through
/// `a` and `b`. Unlike a raw cross-product threshold, the tolerance is in
/// drawing units and remains meaningful when the segment is very long/short.
pub fn collinear_with_tolerance(a: Vec2, b: Vec2, c: Vec2, linear_tolerance: f64) -> bool {
    if !a.is_finite() || !b.is_finite() || !c.is_finite() || !linear_tolerance.is_finite() || linear_tolerance < 0.0 {
        return false;
    }
    let direction = b - a;
    let length = direction.len();
    if !length.is_finite() || length <= linear_tolerance {
        return c.dist(a) <= linear_tolerance;
    }
    // Normalize the direction first so the cross product is a perpendicular
    // distance, not an area whose scale depends on segment length.
    let unit = direction.normalized();
    let relative = c - a;
    if !relative.is_finite() {
        return false;
    }
    let distance = unit.cross(relative).abs();
    distance.is_finite() && distance <= linear_tolerance
}

/// Whether two finite line segments have a strict proper crossing (endpoints
/// and collinear overlaps are deliberately excluded).
pub fn segments_properly_cross(a0: Vec2, a1: Vec2, b0: Vec2, b1: Vec2) -> bool {
    let ab0 = orient2d(a0, a1, b0);
    let ab1 = orient2d(a0, a1, b1);
    let ba0 = orient2d(b0, b1, a0);
    let ba1 = orient2d(b0, b1, a1);
    ab0 != Ordering::Equal && ab1 != Ordering::Equal && ba0 != Ordering::Equal && ba1 != Ordering::Equal
        && ab0 != ab1 && ba0 != ba1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orientation_handles_both_windings() {
        assert_eq!(orient2d(Vec2::ZERO, Vec2::X, Vec2::Y), Ordering::Greater);
        assert_eq!(orient2d(Vec2::ZERO, Vec2::Y, Vec2::X), Ordering::Less);
        assert_eq!(orient2d(Vec2::ZERO, Vec2::X, Vec2::new(2.0, 0.0)), Ordering::Equal);
    }

    #[test]
    fn orientation_is_translation_and_scale_aware() {
        for (origin, scale) in [(0.0, 1.0e-120), (0.0, 1.0), (1.0e12, 1.0e-3), (-1.0e12, 100.0)] {
            let a = Vec2::new(origin, origin);
            let b = Vec2::new(origin + scale, origin);
            let c = Vec2::new(origin, origin + scale);
            assert_eq!(orient2d(a, b, c), Ordering::Greater);
        }
    }

    #[test]
    fn proper_crossing_excludes_touching_and_collinear_overlap() {
        assert!(segments_properly_cross(Vec2::ZERO, Vec2::new(2.0, 2.0), Vec2::new(0.0, 2.0), Vec2::new(2.0, 0.0)));
        assert!(!segments_properly_cross(Vec2::ZERO, Vec2::X, Vec2::X, Vec2::new(1.0, 1.0)));
        assert!(!segments_properly_cross(Vec2::ZERO, Vec2::new(2.0, 0.0), Vec2::X, Vec2::new(3.0, 0.0)));
    }

    #[test]
    fn collinearity_uses_drawing_distance_not_segment_area() {
        let tol = 1.0e-6;
        assert!(collinear_with_tolerance(Vec2::ZERO, Vec2::new(1.0e9, 0.0), Vec2::new(5.0e8, 0.5e-6), tol));
        assert!(!collinear_with_tolerance(Vec2::ZERO, Vec2::new(1.0e-9, 0.0), Vec2::new(0.0, 2.0e-6), tol));
    }

    #[test]
    fn exact_expansion_resolves_nearly_collinear_large_coordinates() {
        let a = Vec2::new(1.0e12, 1.0e12);
        let b = Vec2::new(1.0e12 + 1.0, 1.0e12 + 1.0);
        let c = Vec2::new(1.0e12 + 2.0, 1.0e12 + 2.000244140625);
        assert_eq!(orient2d(a, b, c), Ordering::Greater);
    }

    #[test]
    fn orientation_handles_tiny_and_huge_power_of_two_scales() {
        for scale in [1.0e-140, 1.0e140] {
            let a = Vec2::ZERO;
            let b = Vec2::new(scale, 0.0);
            let c = Vec2::new(0.0, scale);
            assert_eq!(orient2d(a, b, c), Ordering::Greater);
        }
    }

    #[test]
    fn orientation_handles_near_binary64_extremes() {
        let hi = f64::MAX * 0.5;
        assert_eq!(
            orient2d(Vec2::new(-hi, -hi), Vec2::new(hi, -hi), Vec2::new(-hi, hi)),
            Ordering::Greater
        );

        let tiny = f64::from_bits(1);
        assert_eq!(
            orient2d(Vec2::ZERO, Vec2::new(tiny, 0.0), Vec2::new(0.0, tiny)),
            Ordering::Greater
        );
    }

    #[test]
    fn exact_integer_predicate_resolves_ulp_separation_at_max_scale() {
        let maximum = f64::MAX;
        let next = f64::from_bits(1.0_f64.to_bits() + 1);
        let next_above_one = f64::from_bits(1.0_f64.to_bits() + 1);
        assert_eq!(
            orient2d(Vec2::ZERO, Vec2::new(maximum, maximum), Vec2::new(1.0, next_above_one)),
            Ordering::Greater
        );
        assert_eq!(
            orient2d(Vec2::ZERO, Vec2::new(maximum, maximum), Vec2::new(next, next)),
            Ordering::Equal
        );
    }

    #[test]
    fn exact_integer_predicate_handles_mixed_subnormal_and_maximum_exponents() {
        let tiny = f64::from_bits(1);
        let maximum = f64::MAX;
        assert_eq!(
            orient2d(Vec2::ZERO, Vec2::new(maximum, maximum), Vec2::new(1.0, f64::from_bits(1.0_f64.to_bits() + 1))),
            Ordering::Greater
        );
        assert_eq!(
            orient2d(Vec2::ZERO, Vec2::new(tiny, tiny), Vec2::new(maximum, maximum)),
            Ordering::Equal
        );
    }

    #[test]
    fn orientation_handles_opposite_sign_extreme_coordinates() {
        let m = f64::MAX;
        assert_eq!(
            orient2d(Vec2::new(-m, 0.0), Vec2::new(m, 0.0), Vec2::new(0.0, 1.0)),
            Ordering::Greater
        );
    }

    #[test]
    fn non_finite_inputs_do_not_create_a_crossing() {
        assert!(!segments_properly_cross(Vec2::ZERO, Vec2::X, Vec2::new(f64::NAN, 0.0), Vec2::Y));
    }
}
