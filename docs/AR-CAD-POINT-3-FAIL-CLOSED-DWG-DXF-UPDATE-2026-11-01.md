# AR CAD Point 3 — DWG/DXF integrity hardening

Date: 2026-11-01

## Scope

Point 3 is the file-integrity requirement: a drawing must not silently lose unsupported entities, foreign metadata, or records skipped during import. DXF is the preservation format for content successfully read by CADCraft. The current DWG implementation is a third-party DXF bridge (`acadrust`), not a native lossless DWG codec.

## Implemented in this increment

### 1. Fail-closed DWG export at the I/O boundary

`source/cadcraft/crates/io/src/lib.rs` now performs a preflight before every DWG conversion. This is below the UI, so it applies to QSAVE, Save As, scripts, MCP/control-channel saves, `document.bytes`, and layout templates that route through the same I/O hook.

The preflight refuses DWG output when it detects:

- Unknown/unmodelled entities, except ACIS modeler entities with a matching data-store record in a DWG version that supports the data store.
- Entity reactors and colour-book names.
- Foreign entity XDATA and unmodelled entity properties.
- Table/block XDATA, reactors, colour-book names, and unmodelled properties.
- Retained custom-class objects.
- DWG reader notes or DXF parser-skipped records from an incomplete source read.

The refusal names the blocker categories and directs the caller to DXF where successfully read content can be preserved. It does not silently generate a lossy DWG.

### 2. Incomplete reads cannot be re-exported as complete drawings

DXF export now also refuses a drawing whose source parser reported skipped records or whose DWG reader recorded skipped object notes. The warning distinguishes an incomplete source read: skipped records cannot be reconstructed by changing the output format. The source must be reopened from a valid file.

### 3. UI cannot bypass the protection

The Save Anyway option has been removed from the interactive loss warning. The user can choose Save As or Cancel; the actual I/O boundary independently enforces the refusal for non-UI callers. A partial source read is called out separately in the dialog.

### 4. Regression coverage

Added I/O tests for:

- Refusing unmodelled entity output to DWG.
- Refusing table/block XDATA, entity reactors, colour-book names, foreign XDATA/properties and custom-class objects.
- Refusing DWG output after a partial/failed source read.
- Continuing to permit DWG output for a fully modelled clean drawing.
- Refusing DXF export when the parser recorded skipped source records.

The existing DXF round-trip suite already exercises unknown entity payloads and references across ASCII/binary DXF versions. Existing ACIS data-store and supported DWG round-trip tests remain in the suite; unsupported DWG cases now expect explicit refusal rather than silent loss.

## Validation performed here

- `./tools/test-ar-cad.sh`: **197/197 source-level assertions passed** (including **30/30** critical-gap assertions after the latest check addition), shell syntax passed, Python compileall passed.
- Rust tests were not executed: `cargo`, `rustc` and `rustfmt` are unavailable in this environment.
- No real DWG corpus was available in this workspace. AutoCAD/ODA open-save-reopen comparisons were not run.
- No APK was built; APK work remains deferred as requested.

## Point 3 status

**Fail-closed data-loss protection: implemented in source.**

**Full DWG/DXF interoperability certification: still pending.** This cannot honestly be marked complete until Rust-enabled CI executes the test suite and a licensed real-file corpus is round-tripped through independent AutoCAD/ODA implementations. The current bridge is not converted into a native lossless DWG codec by this change. The implementation now prefers an explicit refusal over a silent destructive export for the known unsupported cases.

## Remaining acceptance gates

1. Run the complete Rust workspace tests in a Rust-enabled CI runner.
2. Assemble a licensed corpus covering DWG 2000/2004/2007/2010/2013/2018, ASCII and binary DXF, proxy/custom classes, XDATA, reactors, colour-book names, ACIS solids, blocks, layouts, xrefs, dimensions, hatches, tables, large drawings and damaged/truncated files.
3. Compare entities, handles/references, metadata, layouts, units, styles and ACIS payloads before and after independent AutoCAD/ODA open-save-reopen cycles.
4. For any remaining unsupported object family, add an explicit blocker category and regression test before enabling DWG export for that family.
5. Only then change the release status to fully certified.
