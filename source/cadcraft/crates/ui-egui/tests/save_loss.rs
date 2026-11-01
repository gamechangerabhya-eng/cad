//! Saving over a file that would lose content asks first, and only when a user saves. These run
//! in their own test binary because the engine's I/O hooks are set once per process, and the
//! library's unit tests register a writer that always fails.

use std::path::{Path, PathBuf};

use cadcraft_engine::Session;
use cadcraft_engine::cmd::file::{IoHooks, set_io};
use cadcraft_engine::doc::{Common, Drawing, EntityKind, Handle, RawTag, Space, Unknown};
use cadcraft_engine::doc::foreign::{Foreign, ForeignExtra, ForeignObject};
use serde_json::json;

use cadcraft_ui_egui::save_loss::{Choice, Pending};
use cadcraft_ui_egui::{CadApp, Services, menus};

const ORIGINAL: &[u8] = b"original";

fn unknown(kind: &str) -> EntityKind {
    EntityKind::Unknown(Unknown { dxf_type: kind.into(), tags: vec![RawTag { code: 8, value: "0".into() }] })
}

/// A drawing with objects of `types`, which DWG can't keep, opened from a DWG file on disk.
fn opened(tag: &str, types: &[&str]) -> (CadApp, PathBuf) {
    set_io(IoHooks { read: |_, _| Err("no reader in this test".into()), write: |_, _| Ok(b"0\nEOF\n".to_vec()), plot: None });
    let dir = std::env::temp_dir().join(format!("cadcraft-loss-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("Plan.dwg");
    std::fs::write(&path, ORIGINAL).unwrap();
    let mut d = Drawing::new_metric();
    for t in types {
        d.add(&Space::Model, Common::default(), unknown(t)).unwrap();
    }
    let mut s = Session::empty();
    s.open_drawing(d, "Plan.dwg", Some(path.to_string_lossy().into_owned()));
    (CadApp::new(s, Services::default()), path)
}

fn cleanup(path: &Path) {
    if let Some(d) = path.parent() {
        let _ = std::fs::remove_dir_all(d);
    }
}

#[test]
fn a_users_save_asks_before_the_file_changes_and_cancel_leaves_it() {
    let (mut app, path) = opened("cancel", &["3DSOLID", "MLINE", "MLINE"]);
    menus::activate(&mut app, "qsave");
    let pending = app.save_loss.clone().expect("the save waits for an answer");
    assert_eq!(pending.summary(), "1 3DSOLID, 2 MLINE");
    assert!(app.shows_modal());
    assert_eq!(std::fs::read(&path).unwrap(), ORIGINAL, "nothing is written before the answer");

    app.answer_save_loss(Choice::Cancel);
    assert!(app.save_loss.is_none());
    assert_eq!(std::fs::read(&path).unwrap(), ORIGINAL);
    cleanup(&path);
}

#[test]
fn lossy_overwrite_has_no_save_anyway_choice() {
    let (mut app, path) = opened("no-lossy-overwrite", &["MLINE"]);
    menus::activate(&mut app, "qsave");
    assert!(app.save_loss.is_some());
    app.answer_save_loss(Choice::Cancel);
    assert_eq!(std::fs::read(&path).unwrap(), ORIGINAL, "the original DWG remains untouched");
    cleanup(&path);
}

#[test]
fn a_save_that_loses_nothing_does_not_ask() {
    let (mut app, path) = opened("clean", &[]);
    menus::activate(&mut app, "qsave");
    assert!(app.save_loss.is_none());
    assert_ne!(std::fs::read(&path).unwrap(), ORIGINAL);
    cleanup(&path);
}

/// Programmatic calls never open dialogs: the loss is in the result instead.
#[test]
fn programmatic_saves_never_ask() {
    let (mut app, path) = opened("json", &["MLINE"]);
    let r = app.run("qsave", json!({})).unwrap();
    assert!(app.save_loss.is_none());
    assert_eq!(r["notSaved"], json!({ "MLINE": 1 }));
    cleanup(&path);
}

/// Type names come from the file: however many, however long, the warning stays readable.
#[test]
fn a_hostile_list_of_types_is_capped() {
    let long = "X".repeat(5000);
    let many: Vec<String> = (0..50).map(|i| format!("{long}{i}")).collect();
    let refs: Vec<&str> = many.iter().map(String::as_str).collect();
    let (mut app, path) = opened("hostile", &refs);
    menus::activate(&mut app, "qsave");
    let s = app.save_loss.as_ref().unwrap().summary();
    assert!(s.chars().count() < 500, "{}", s.len());
    assert!(s.ends_with("42 more types"), "{s}");
    cleanup(&path);
}


/// Foreign metadata alone must trigger the same pre-save guard as unsupported entities.
#[test]
fn metadata_only_dwg_loss_is_blocked_until_the_user_answers() {
    let (mut app, path) = opened("metadata-cancel", &[]);
    let tag = |code, value: &str| RawTag { code, value: value.to_string() };
    let mut foreign = Foreign::default();
    foreign.entities.insert(Handle(0x10), ForeignExtra {
        reactors: vec![Handle(0x20)],
        color_book: Some(("VendorBook".into(), "Accent".into())),
        ..ForeignExtra::default()
    });
    foreign.records.insert(("LAYER".into(), "WALLS".into()), ForeignExtra {
        xdata: vec![tag(1001, "THIRD_PARTY_APP")],
        ..ForeignExtra::default()
    });
    foreign.classes.push(vec![tag(1, "VENDOR_OBJECT"), tag(2, "VendorObject")]);
    foreign.objects.push(ForeignObject { handle: Handle(0x30), kind: "VENDOR_OBJECT".into(), tags: vec![] });
    *app.session.doc_mut().unwrap().foreign = std::sync::Arc::new(foreign);

    menus::activate(&mut app, "qsave");
    let pending = app.save_loss.clone().expect("metadata loss must hold the save");
    assert!(pending.summary().contains("ENTITY_REACTORS"));
    assert!(pending.summary().contains("ENTITY_COLOR_BOOK_NAMES"));
    assert!(pending.summary().contains("TABLE_OR_BLOCK_XDATA"));
    assert!(pending.summary().contains("CUSTOM_CLASS_OBJECT:VENDOR_OBJECT"));
    assert!(app.shows_modal());
    assert_eq!(std::fs::read(&path).unwrap(), ORIGINAL, "metadata is not discarded before consent");

    app.answer_save_loss(Choice::Cancel);
    assert_eq!(std::fs::read(&path).unwrap(), ORIGINAL);
    cleanup(&path);
}

#[test]
fn incomplete_source_read_is_identified_in_the_save_warning() {
    let (mut app, path) = opened("partial-source", &[]);
    app.session.doc_mut().unwrap().read_notes.push("one source object was skipped".into());
    menus::activate(&mut app, "qsave");
    assert!(app.save_loss.as_ref().is_some_and(Pending::source_incomplete));
    app.answer_save_loss(Choice::Cancel);
    assert_eq!(std::fs::read(&path).unwrap(), ORIGINAL);
    cleanup(&path);
}
