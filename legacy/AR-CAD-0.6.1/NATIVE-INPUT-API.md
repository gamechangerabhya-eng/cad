# Native CADCraft viewport and pointer API

AR CAD 0.6.1 adds three document operations across the Rust C ABI, JNI and Kotlin wrapper. All native JSON responses use the versioned envelope `{"schemaVersion":1,"ok":true,"result":...}` or `{"schemaVersion":1,"ok":false,"error":"..."}`. Increment the schema version only with a documented compatibility change.

## Operations

- `setViewport(widthPx, heightPx)`: sets `Session::viewport_px`; both dimensions must be finite and in the inclusive range 1–16384.
- `pointerMove(x, y, shift, nowSeconds)`: accepts world-space `f64` coordinates, updates `Session::cursor`, and calls CADCraft `Session::snap_cursor`. The returned JSON includes the snapped cursor, current command prompt state and selected handles. The query uses the active prompt's base/deferred/corner flags, session pick aperture and a four-pixel tracking tolerance. `nowSeconds` must be seconds from a monotonic clock.
- `pointerClick(x, y, shift, nowSeconds)`: with no command active, calls `Session::idle_click`; during a selection prompt, asks CADCraft's pick index for candidates and submits a native pick; during a point prompt, runs the same snap query and submits `input_snapped` or a point input. Returns command state, selection handles and entity count.

## Coordinate contract

All coordinates crossing Kotlin → JNI → C → Rust are IEEE-754 double precision. The Android view is responsible for converting screen pixels to CADCraft world coordinates using its current pan/zoom/view transform. The viewport dimensions are in physical canvas pixels. Use one monotonic clock for all pointer updates within a session.

## Threading and ownership

`CadCraftDocument` serializes every operation on its opaque handle with a per-document lock. The caller must not share a raw handle or call the C ABI concurrently outside this wrapper. Close the document once; wrapper close is idempotent.

## Current integration status

The API is implemented in source but the Compose editor does not yet call it. Native selection highlighting, gesture routing, live rendering and document persistence migration remain future work. Host wrapper tests use a fake `CadNativeApi`; they do not prove Rust runtime behavior.
