//! Narrow C ABI for the CADCraft-backed AR CAD document engine.
//!
//! All pointers crossing this boundary are owned by the caller or returned as opaque handles.
//! Returned strings/buffers must be released with the matching `arcad_*_free` function.
#![allow(unsafe_code)]

use cadcraft_engine::{Input, Session};
use cadcraft_engine::snap::tracking::CursorQuery;
use cadcraft_doc::Handle;
use cadcraft_engine::geom::Vec2;
use cadcraft_io;
use serde_json::{Value, json};
use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr;
use std::slice;

const MAX_INPUT_BYTES: usize = 64 * 1024 * 1024;
const MAX_FILENAME_BYTES: usize = 4096;

thread_local! {
    static LAST_ERROR: RefCell<CString> = RefCell::new(CString::default());
}

fn set_last_error(message: impl AsRef<str>) {
    let clean = message.as_ref().replace('\0', "?");
    let value = CString::new(clean).unwrap_or_default();
    LAST_ERROR.with(|slot| *slot.borrow_mut() = value);
}

fn clear_last_error() {
    LAST_ERROR.with(|slot| *slot.borrow_mut() = CString::default());
}

fn null_on_error<T>(result: Result<T, String>) -> Option<T> {
    match result {
        Ok(value) => { clear_last_error(); Some(value) }
        Err(message) => { set_last_error(message); None }
    }
}

pub struct ArcadHandle {
    session: Session,
}

pub struct ArcadBuffer {
    bytes: Vec<u8>,
}

const ABI_SCHEMA_VERSION: u32 = 1;

fn error_json(message: impl Into<String>) -> CString {
    let value = json!({"schemaVersion": ABI_SCHEMA_VERSION, "ok": false, "error": message.into()}).to_string();
    CString::new(value).unwrap_or_else(|_| CString::new("{\"schemaVersion\":1,\"ok\":false,\"error\":\"invalid error text\"}").unwrap_or_default())
}

fn result_json(value: Value) -> CString {
    let payload = json!({"schemaVersion": ABI_SCHEMA_VERSION, "ok": true, "result": value}).to_string();
    CString::new(payload).unwrap_or_default()
}

unsafe fn c_string(value: *const c_char, max: usize) -> Result<String, String> {
    if value.is_null() {
        return Err("null string pointer".into());
    }
    // SAFETY: caller promises a valid NUL-terminated string; bounded length rejects oversized input.
    let raw = unsafe { CStr::from_ptr(value) };
    if raw.to_bytes().len() > max {
        return Err("string exceeds length limit".into());
    }
    raw.to_str().map(str::to_owned).map_err(|_| "string is not valid UTF-8".into())
}

fn guarded<T>(f: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(result) => result,
        Err(_) => Err("native CAD engine trapped an internal panic".into()),
    }
}

fn command_state(session: &Session) -> Value {
    let prompt = session.current_prompt();
    let command_active = prompt.is_some();
    json!({
        "prompt": prompt,
        "promptText": session.prompt_text(),
        "commandActive": command_active
    })
}

fn parse_interactive_input(value: Value) -> Result<Input, String> {
    let kind = value.get("kind").and_then(Value::as_str).ok_or("input.kind is required")?;
    let payload = value.get("value");
    match kind {
        "point" => {
            let point = payload.and_then(Value::as_object).ok_or("point input value must be an object with x and y")?;
            let x = point.get("x").and_then(Value::as_f64).ok_or("point.x must be a number")?;
            let y = point.get("y").and_then(Value::as_f64).ok_or("point.y must be a number")?;
            if !x.is_finite() || !y.is_finite() || x.abs() > 1.0e12 || y.abs() > 1.0e12 {
                return Err("point coordinates must be finite and within ±1e12".into());
            }
            Ok(Input::Point(Vec2::new(x, y)))
        }
        "keyword" | "text" => {
            let text = payload.and_then(Value::as_str).ok_or("text/keyword input value must be a string")?;
            if text.len() > 4096 { return Err("text/keyword input exceeds 4096 bytes".into()); }
            if kind == "keyword" { Ok(Input::Keyword(text.to_owned())) } else { Ok(Input::Text(text.to_owned())) }
        }
        "pick" => {
            let handles = payload.and_then(Value::as_array).ok_or("pick input value must be an array of numeric handles")?;
            if handles.len() > 100_000 { return Err("pick input exceeds 100000 handles".into()); }
            let parsed = handles.iter().map(|item| item.as_u64().map(Handle).ok_or_else(|| "pick handles must be unsigned integers".to_owned())).collect::<Result<Vec<_>, _>>()?;
            Ok(Input::Pick(parsed))
        }
        "enter" => Ok(Input::Enter),
        "cancel" => Ok(Input::Cancel),
        _ => Err(format!("unsupported interactive input kind `{kind}`")),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn arcad_core_version() -> *const c_char {
    static VERSION: &[u8] = b"AR CAD native bridge 0.1.0 / CADCraft 0.15.1\0";
    VERSION.as_ptr().cast()
}

#[unsafe(no_mangle)]
pub extern "C" fn arcad_document_new(metric: i32) -> *mut ArcadHandle {
    null_on_error(guarded(|| {
        let mut session = Session::empty();
        session.new_drawing(metric != 0);
        Ok(Box::into_raw(Box::new(ArcadHandle { session })))
    })).unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn arcad_document_open(
    bytes: *const u8,
    length: usize,
    filename: *const c_char,
) -> *mut ArcadHandle {
    null_on_error(guarded(|| {
        if bytes.is_null() || length == 0 || length > MAX_INPUT_BYTES {
            return Err("input buffer is null, empty, or exceeds 64 MB".into());
        }
        // SAFETY: caller supplies a readable byte buffer of `length` bytes for this call.
        let input = unsafe { slice::from_raw_parts(bytes, length) };
        // SAFETY: filename is validated as a bounded NUL-terminated UTF-8 string.
        let name = unsafe { c_string(filename, MAX_FILENAME_BYTES)? };
        let drawing = cadcraft_io::read(input, name.as_str()).map_err(|e| e.to_string())?;
        let mut session = Session::empty();
        session.open_drawing(drawing, &name, None);
        Ok(Box::into_raw(Box::new(ArcadHandle { session })))
    })).unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn arcad_document_free(handle: *mut ArcadHandle) {
    if !handle.is_null() {
        // SAFETY: handle must be a live pointer returned by arcad_document_new/open exactly once.
        // Destructors are contained so no Rust panic can cross the C ABI boundary.
        if catch_unwind(AssertUnwindSafe(|| drop(unsafe { Box::from_raw(handle) }))).is_err() {
            set_last_error("panic while releasing native CAD document");
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn arcad_document_entity_count(handle: *const ArcadHandle) -> usize {
    match catch_unwind(AssertUnwindSafe(|| {
        if handle.is_null() { return Err("document handle is null".to_owned()); }
        // SAFETY: handle must point to a live ArcadHandle and remain immutable for this call.
        let handle = unsafe { &*handle };
        handle.session.doc().map(|drawing| drawing.model.len()).map_err(|e| e.to_string())
    })) {
        Ok(Ok(count)) => { clear_last_error(); count }
        Ok(Err(error)) => { set_last_error(error); 0 }
        Err(_) => { set_last_error("panic while reading entity count"); 0 }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn arcad_document_read_notes(handle: *const ArcadHandle) -> *mut c_char {
    let response = guarded(|| {
        if handle.is_null() { return Err("document handle is null".into()); }
        // SAFETY: caller must keep the document alive and prevent concurrent mutation.
        let handle = unsafe { &*handle };
        let drawing = handle.session.doc().map_err(|e| e.to_string())?;
        Ok(result_json(json!({"notes": &drawing.read_notes})))
    }).unwrap_or_else(error_json);
    response.into_raw()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn arcad_command_execute(
    handle: *mut ArcadHandle,
    command: *const c_char,
    params_json: *const c_char,
) -> *mut c_char {
    let response = guarded(|| {
        if handle.is_null() { return Err("document handle is null".into()); }
        // SAFETY: pointers are validated and expected to refer to live inputs for this call.
        let command = unsafe { c_string(command, 256)? };
        let params = if params_json.is_null() { Value::Null } else {
            // SAFETY: bounded NUL-terminated JSON input supplied by caller.
            let text = unsafe { c_string(params_json, 1024 * 1024)? };
            serde_json::from_str::<Value>(text).map_err(|e| format!("invalid command JSON: {e}"))?
        };
        // SAFETY: caller must ensure exclusive access to a document during mutations.
        let handle = unsafe { &mut *handle };
        let result = handle.session.execute(&command, &params).map_err(|e| e.to_string())?;
        Ok(result_json(result))
    }).unwrap_or_else(error_json);
    response.into_raw()
}

/// Start a prompt-driven CADCraft command and return the current prompt/state as JSON.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn arcad_command_start(handle: *mut ArcadHandle, command: *const c_char) -> *mut c_char {
    let response = guarded(|| {
        if handle.is_null() { return Err("document handle is null".into()); }
        let command = unsafe { c_string(command, 256)? };
        if command.trim().is_empty() { return Err("command name cannot be empty".into()); }
        let handle = unsafe { &mut *handle };
        handle.session.start(&command).map_err(|e| e.to_string())?;
        Ok(result_json(command_state(&handle.session)))
    }).unwrap_or_else(error_json);
    response.into_raw()
}

/// Feed a bounded JSON input to the running CADCraft command and return its next prompt/state.
/// Supported kinds: point, keyword, text, pick, enter and cancel.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn arcad_command_input(handle: *mut ArcadHandle, input_json: *const c_char) -> *mut c_char {
    let response = guarded(|| {
        if handle.is_null() { return Err("document handle is null".into()); }
        let raw = unsafe { c_string(input_json, 1024 * 1024)? };
        let value = serde_json::from_str::<Value>(&raw).map_err(|e| format!("invalid interactive input JSON: {e}"))?;
        let input = parse_interactive_input(value)?;
        let handle = unsafe { &mut *handle };
        handle.session.input(input).map_err(|e| e.to_string())?;
        Ok(result_json(command_state(&handle.session)))
    }).unwrap_or_else(error_json);
    response.into_raw()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn arcad_document_undo(handle: *mut ArcadHandle) -> *mut c_char {
    let response = guarded(|| {
        if handle.is_null() { return Err("document handle is null".into()); }
        // SAFETY: caller must ensure exclusive access to a document during mutations.
        let handle = unsafe { &mut *handle };
        let label = handle.session.undo().map_err(|e| e.to_string())?;
        Ok(result_json(json!({"label": label})))
    }).unwrap_or_else(error_json);
    response.into_raw()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn arcad_document_redo(handle: *mut ArcadHandle) -> *mut c_char {
    let response = guarded(|| {
        if handle.is_null() { return Err("document handle is null".into()); }
        // SAFETY: caller must ensure exclusive access to a document during mutations.
        let handle = unsafe { &mut *handle };
        let label = handle.session.redo().map_err(|e| e.to_string())?;
        Ok(result_json(json!({"label": label})))
    }).unwrap_or_else(error_json);
    response.into_raw()
}

/// Update the native viewport dimensions used by CADCraft for pick apertures and view commands.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn arcad_document_set_viewport(
    handle: *mut ArcadHandle,
    width_px: f64,
    height_px: f64,
) -> *mut c_char {
    let response = guarded(|| {
        if handle.is_null() { return Err("document handle is null".into()); }
        if !width_px.is_finite() || !height_px.is_finite()
            || !(1.0..=16384.0).contains(&width_px)
            || !(1.0..=16384.0).contains(&height_px)
        {
            return Err("viewport dimensions must be finite and between 1 and 16384 pixels".into());
        }
        let handle = unsafe { &mut *handle };
        handle.session.viewport_px = (width_px, height_px);
        Ok(result_json(json!({"viewportPx": [width_px, height_px]})))
    });
    match response { Ok(value) => value, Err(error) => error_json(error) }.into_raw()
}

/// Feed a world-coordinate cursor move through CADCraft's native OSNAP/grid/ortho/tracking path.
/// `now_seconds` must come from one monotonic clock for the lifetime of the interaction session.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn arcad_document_pointer_move(
    handle: *mut ArcadHandle,
    x: f64,
    y: f64,
    shift: i32,
    now_seconds: f64,
) -> *mut c_char {
    let response = guarded(|| {
        if handle.is_null() { return Err("document handle is null".into()); }
        if !x.is_finite() || !y.is_finite() || x.abs() > 1.0e12 || y.abs() > 1.0e12
            || !now_seconds.is_finite() || now_seconds < 0.0
        {
            return Err("cursor coordinates or monotonic timestamp are invalid".into());
        }
        let handle = unsafe { &mut *handle };
        let raw = Vec2::new(x, y);
        handle.session.cursor = raw;
        let prompt = handle.session.current_prompt();
        let settings = handle.session.drafting_settings();
        let query = CursorQuery {
            base: prompt.as_ref().and_then(|p| p.base),
            deferred: prompt.as_ref().is_some_and(|p| p.deferred),
            corner: prompt.as_ref().is_some_and(|p| p.corner),
            aperture: handle.session.pixel_size() * settings.pickbox.max(1.0) * 1.5,
            tol: handle.session.pixel_size() * 4.0,
            now: now_seconds,
            shift: shift != 0,
        };
        let snapped = handle.session.snap_cursor(raw, &query);
        Ok(result_json(json!({
            "cursor": snapped,
            "command": command_state(&handle.session),
            "selection": handle.session.selection().iter().map(|h| h.0).collect::<Vec<_>>()
        })))
    });
    match response { Ok(value) => value, Err(error) => error_json(error) }.into_raw()
}

/// Feed a tap into the native session. Selection prompts use CADCraft's own pick index;
/// point prompts use the exact same snap calculation as pointer_move.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn arcad_document_pointer_click(
    handle: *mut ArcadHandle,
    x: f64,
    y: f64,
    shift: i32,
    now_seconds: f64,
) -> *mut c_char {
    let response = guarded(|| {
        if handle.is_null() { return Err("document handle is null".into()); }
        if !x.is_finite() || !y.is_finite() || x.abs() > 1.0e12 || y.abs() > 1.0e12
            || !now_seconds.is_finite() || now_seconds < 0.0
        {
            return Err("cursor coordinates or monotonic timestamp are invalid".into());
        }
        let handle = unsafe { &mut *handle };
        let raw = Vec2::new(x, y);
        handle.session.cursor = raw;
        if let Some(prompt) = handle.session.current_prompt() {
            if prompt.accept.select {
                let candidates = handle.session.pick_candidates(raw);
                if candidates.is_empty() {
                    handle.session.input(Input::Point(raw)).map_err(|e| e.to_string())?;
                } else {
                    handle.session.input(Input::Pick(candidates)).map_err(|e| e.to_string())?;
                }
            } else {
                let settings = handle.session.drafting_settings();
                let query = CursorQuery {
                    base: prompt.base,
                    deferred: prompt.deferred,
                    corner: prompt.corner,
                    aperture: handle.session.pixel_size() * settings.pickbox.max(1.0) * 1.5,
                    tol: handle.session.pixel_size() * 4.0,
                    now: now_seconds,
                    shift: shift != 0,
                };
                let snapped = handle.session.snap_cursor(raw, &query);
                if let Some(hit) = snapped.snap {
                    handle.session.input_snapped(hit).map_err(|e| e.to_string())?;
                } else {
                    handle.session.input(Input::Point(snapped.point)).map_err(|e| e.to_string())?;
                }
            }
        } else {
            handle.session.idle_click(raw, shift != 0).map_err(|e| e.to_string())?;
        }
        Ok(result_json(json!({
            "command": command_state(&handle.session),
            "selection": handle.session.selection().iter().map(|h| h.0).collect::<Vec<_>>(),
            "entityCount": handle.session.doc().map_err(|e| e.to_string())?.model.len()
        })))
    });
    match response { Ok(value) => value, Err(error) => error_json(error) }.into_raw()
}

/// Render model space to a bounded PNG preview using CADCraft's own display-list/raster pipeline.
/// Dimensions are deliberately capped to keep mobile memory use predictable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn arcad_document_render_png(
    handle: *const ArcadHandle,
    width: u32,
    height: u32,
) -> *mut ArcadBuffer {
    null_on_error(guarded(|| {
        if handle.is_null() { return Err("document handle is null".into()); }
        if !(16..=4096).contains(&width) || !(16..=4096).contains(&height) {
            return Err("preview dimensions must each be between 16 and 4096 pixels".into());
        }
        let pixels = u64::from(width) * u64::from(height);
        if pixels > 16_777_216 {
            return Err("preview exceeds the 16-megapixel limit".into());
        }
        // SAFETY: caller keeps the document alive and prevents concurrent mutation for this call.
        let handle = unsafe { &*handle };
        let drawing = handle.session.doc().map_err(|e| e.to_string())?;
        let bytes = cadcraft_io::png(drawing, &cadcraft_doc::Space::Model, width, height)
            .map_err(|e| e.to_string())?;
        if bytes.is_empty() { return Err("CADCraft returned an empty PNG preview".into()); }
        Ok(Box::into_raw(Box::new(ArcadBuffer { bytes })))
    })).unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn arcad_document_save(
    handle: *const ArcadHandle,
    filename: *const c_char,
) -> *mut ArcadBuffer {
    null_on_error(guarded(|| {
        if handle.is_null() { return Err("document handle is null".into()); }
        // SAFETY: filename is validated as a bounded NUL-terminated UTF-8 string.
        let name = unsafe { c_string(filename, MAX_FILENAME_BYTES)? };
        // SAFETY: caller must prevent concurrent mutation while saving.
        let handle = unsafe { &*handle };
        let drawing = handle.session.doc().map_err(|e| e.to_string())?;
        let bytes = cadcraft_io::write(drawing, name.as_str()).map_err(|e| e.to_string())?;
        Ok(Box::into_raw(Box::new(ArcadBuffer { bytes })))
    })).unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn arcad_buffer_data(buffer: *const ArcadBuffer) -> *const u8 {
    match catch_unwind(AssertUnwindSafe(|| {
        if buffer.is_null() { return Err("buffer handle is null".to_owned()); }
        // SAFETY: buffer must be a live pointer returned by arcad_document_save.
        Ok(unsafe { (&*buffer).bytes.as_ptr() })
    })) {
        Ok(Ok(data)) => { clear_last_error(); data }
        Ok(Err(error)) => { set_last_error(error); ptr::null() }
        Err(_) => { set_last_error("panic while reading native buffer"); ptr::null() }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn arcad_buffer_len(buffer: *const ArcadBuffer) -> usize {
    match catch_unwind(AssertUnwindSafe(|| {
        if buffer.is_null() { return Err("buffer handle is null".to_owned()); }
        // SAFETY: buffer must be a live pointer returned by arcad_document_save.
        Ok(unsafe { (&*buffer).bytes.len() })
    })) {
        Ok(Ok(length)) => { clear_last_error(); length }
        Ok(Err(error)) => { set_last_error(error); 0 }
        Err(_) => { set_last_error("panic while reading native buffer length"); 0 }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn arcad_buffer_free(buffer: *mut ArcadBuffer) {
    if !buffer.is_null() {
        // SAFETY: buffer must be a live pointer returned by arcad_document_save exactly once.
        if catch_unwind(AssertUnwindSafe(|| drop(unsafe { Box::from_raw(buffer) }))).is_err() {
            set_last_error("panic while releasing native CAD buffer");
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn arcad_last_error() -> *mut c_char {
    match catch_unwind(AssertUnwindSafe(|| LAST_ERROR.with(|slot| slot.borrow().clone().into_raw()))) {
        Ok(value) => value,
        Err(_) => ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn arcad_string_free(value: *mut c_char) {
    if !value.is_null() {
        // SAFETY: value must be a CString returned by this library exactly once.
        if catch_unwind(AssertUnwindSafe(|| drop(unsafe { CString::from_raw(value) }))).is_err() {
            set_last_error("panic while releasing native string");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_metric_document_and_reports_version() {
        let handle = arcad_document_new(1);
        assert!(!handle.is_null());
        // SAFETY: handle was just returned by arcad_document_new.
        assert_eq!(unsafe { arcad_document_entity_count(handle) }, 0);
        // SAFETY: handle remains live and is not mutated during the read.
        let notes = unsafe { arcad_document_read_notes(handle) };
        assert!(!notes.is_null());
        // SAFETY: returned JSON is an owned CString from this library.
        let notes_json = unsafe { CStr::from_ptr(notes) }.to_string_lossy();
        assert!(notes_json.contains("notes"));
        assert!(notes_json.contains("\"schemaVersion\":1"));
        // SAFETY: returned JSON is released exactly once.
        unsafe { arcad_string_free(notes) };
        // SAFETY: handle is released exactly once.
        unsafe { arcad_document_free(handle) };
        let version = arcad_core_version();
        // SAFETY: version is a static NUL-terminated string.
        assert!(unsafe { CStr::from_ptr(version) }.to_string_lossy().contains("CADCraft 0.15.1"));
    }

    #[test]
    fn empty_dxf_save_open_round_trip() {
        let handle = arcad_document_new(1);
        assert!(!handle.is_null());
        let filename = CString::new("roundtrip.dxf").expect("static filename");
        // SAFETY: handle and filename are valid for the duration of this call.
        let buffer = unsafe { arcad_document_save(handle, filename.as_ptr()) };
        assert!(!buffer.is_null());
        // SAFETY: buffer is live and owned by this test.
        let len = unsafe { arcad_buffer_len(buffer) };
        // SAFETY: returned pointer addresses `len` readable bytes while buffer remains live.
        let bytes = unsafe { slice::from_raw_parts(arcad_buffer_data(buffer), len) }.to_vec();
        // SAFETY: buffer is freed exactly once after copying its bytes.
        unsafe { arcad_buffer_free(buffer) };
        // SAFETY: byte vector and filename remain alive during the call.
        let reopened = unsafe { arcad_document_open(bytes.as_ptr(), bytes.len(), filename.as_ptr()) };
        assert!(!reopened.is_null());
        // SAFETY: both handles are live and are released exactly once.
        assert_eq!(unsafe { arcad_document_entity_count(reopened) }, 0);
        unsafe { arcad_document_free(reopened) };
        unsafe { arcad_document_free(handle) };
    }

    #[test]
    fn command_errors_are_structured_json() {
        let handle = arcad_document_new(1);
        let command = CString::new("not-a-real-command").expect("static command");
        let params = CString::new("{}").expect("static JSON");
        // SAFETY: handle and C strings are valid for this call.
        let response = unsafe { arcad_command_execute(handle, command.as_ptr(), params.as_ptr()) };
        assert!(!response.is_null());
        // SAFETY: response is a CString allocated by this library.
        let text = unsafe { CStr::from_ptr(response) }.to_string_lossy().into_owned();
        assert!(text.contains("\"ok\":false"));
        assert!(text.contains("\"schemaVersion\":1"));
        // SAFETY: response and handle are each released exactly once.
        unsafe { arcad_string_free(response) };
        unsafe { arcad_document_free(handle) };
    }
}

#[cfg(test)]
mod interactive_input_tests {
    use super::{Input, parse_interactive_input};
    use cadcraft_doc::Handle;
    use cadcraft_engine::geom::Vec2;
    use serde_json::json;

    #[test]
    fn accepts_supported_prompt_input_kinds() {
        assert!(matches!(parse_interactive_input(json!({"kind":"point","value":{"x":12.5,"y":-3.0}})), Ok(Input::Point(p)) if p == Vec2::new(12.5, -3.0)));
        assert!(matches!(parse_interactive_input(json!({"kind":"keyword","value":"Close"})), Ok(Input::Keyword(value)) if value == "Close"));
        assert!(matches!(parse_interactive_input(json!({"kind":"text","value":"25.4"})), Ok(Input::Text(value)) if value == "25.4"));
        assert!(matches!(parse_interactive_input(json!({"kind":"pick","value":[1,42]})), Ok(Input::Pick(values)) if values == vec![Handle(1), Handle(42)]));
        assert!(matches!(parse_interactive_input(json!({"kind":"enter"})), Ok(Input::Enter)));
        assert!(matches!(parse_interactive_input(json!({"kind":"cancel"})), Ok(Input::Cancel)));
    }

    #[test]
    fn rejects_invalid_or_oversized_prompt_inputs() {
        assert!(parse_interactive_input(json!({"kind":"point","value":{"x":1e20,"y":0}})).is_err());
        assert!(parse_interactive_input(json!({"kind":"point","value":{"x":"1","y":0}})).is_err());
        assert!(parse_interactive_input(json!({"kind":"keyword","value":"x".repeat(4097)})).is_err());
        assert!(parse_interactive_input(json!({"kind":"pick","value":[-1]})).is_err());
        assert!(parse_interactive_input(json!({"kind":"unknown"})).is_err());
    }
}
