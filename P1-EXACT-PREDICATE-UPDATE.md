# AR CAD geometry update — exact orientation predicate

## Implemented

`geom::orient2d` now has an integer-exact determinant path for every finite IEEE-754 binary64 input. Each coordinate is decoded as an exact signed significand and binary exponent. All six coordinates are aligned to a shared exponent; coordinate differences use 34 64-bit limbs and determinant products use 68 64-bit limbs. This is sufficient for the full finite binary64 exponent range, including subnormals and opposite-sign maximum-magnitude values.

A conservative floating-point first-stage filter returns early only when the determinant is separated from zero by the standard orient2d error bound. Overflow, underflow, cancellation and uncertain cases use the exact integer path. The exact path does not rely on FMA or compiler floating-point contraction.

Regression coverage now includes ULP-separated coordinates at maximum scale, subnormal values mixed with maximum exponents, exact collinearity and opposite-sign extreme coordinates.

## Performance note

The exact fallback uses fixed-width integer multiplication. The floating-point filter handles ordinary well-separated cases; near-degenerate and extreme-range cases pay the larger cost. Runtime profiling is still required on representative large drawings before release.

## Verification boundary

The repository's source-level acceptance suite passes, but Rust compilation and Rust unit tests have not run in this environment because `cargo` and `rustc` are unavailable. This is an implementation claim based on source changes, not a claim of compiled or runtime-verified correctness.
