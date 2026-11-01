//! The current DWG bridge cannot preserve every unsupported entity and metadata category. The
//! UI holds a lossy QSAVE and offers Save As or Cancel; the I/O layer independently refuses the
//! lossy DWG write, so scripts, MCP and other non-UI callers cannot bypass the protection.
//!
//! Programmatic saves (`engine.execute`, the control channel, MCP, scripts) never ask: the I/O
//! layer returns a direct error with the blocker inventory instead of producing lossy bytes.

use std::collections::BTreeMap;

use egui::{RichText, vec2};
use serde_json::{Value, json};

use cadcraft_engine::cmd::file::loss_on_save;

use crate::CadApp;
use crate::theme::Tokens;

/// How many object types the warning lists; a file can name as many as it likes.
const MAX_LISTED: usize = 8;
/// How many characters of a type name the warning shows; a file chooses these too.
const MAX_NAME_CHARS: usize = 40;

/// The user's answer to the non-destructive save warning. Lossy overwrite is not an option.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Choice {
    SaveAs,
    Cancel,
}

/// A loss that requires a non-destructive Save As choice: this drawing, file and content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LossSnapshot {
    uid: u64,
    path: String,
    lost: BTreeMap<String, usize>,
}

/// A save held back until the user answers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pending {
    loss: LossSnapshot,
}

impl Pending {
    /// The file the save would replace.
    pub fn path(&self) -> &str {
        &self.loss.path
    }

    /// Whether opening the source itself skipped records, so neither DWG nor DXF can be
    /// represented as a complete save of the original.
    pub fn source_incomplete(&self) -> bool {
        self.loss.lost.get("DWG_READ_NOTES").copied().unwrap_or_default() > 0
            || self.loss.lost.get("SKIPPED_DXF_RECORDS").copied().unwrap_or_default() > 0
    }

    /// The objects the file wouldn't keep, as "2 MLINE, 1 3DSOLID".
    pub fn summary(&self) -> String {
        let lost = &self.loss.lost;
        let name = |t: &str| t.chars().take(MAX_NAME_CHARS).collect::<String>();
        let mut parts: Vec<String> = lost.iter().take(MAX_LISTED).map(|(t, n)| format!("{n} {}", name(t))).collect();
        if lost.len() > MAX_LISTED {
            parts.push(crate::tf!("{count} more types", count = lost.len() - MAX_LISTED));
        }
        parts.join(", ")
    }
}

impl CadApp {
    /// Hold a user's QSAVE when the DWG bridge cannot guarantee preservation, and ask for a
    /// non-destructive Save As or cancellation. Returns whether it was held.
    pub fn hold_lossy_save(&mut self) -> bool {
        let Ok(st) = self.session.state() else { return false };
        let (uid, Some(path)) = (st.uid, st.path.clone()) else { return false };
        let lost = loss_on_save(&self.session, &path);
        if lost.is_empty() {
            return false;
        }
        let loss = LossSnapshot { uid, path, lost };
        self.save_loss = Some(Pending { loss });
        true
    }

    /// Answer the open warning (no-op without one).
    pub fn answer_save_loss(&mut self, choice: Choice) {
        let Some(Pending { loss }) = self.save_loss.take() else { return };
        // The drawing can go away, or stop being the active one, while the warning is up.
        let i = self.session.docs.iter().position(|d| d.uid == loss.uid);
        match (choice, i) {
            (Choice::Cancel, _) | (_, None) => self.session.echo(crate::tf!("Save cancelled: {path} was not changed.", path = loss.path)),
            (Choice::SaveAs, Some(i)) => {
                self.activate_doc(i);
                let _ = self.run("ui.saveas", Value::Null);
            }
        }
    }

    fn activate_doc(&mut self, i: usize) {
        if self.session.active != i {
            let _ = self.run("document.switch", json!({ "index": i }));
        }
    }
}

/// The warning, while a save waits for an answer. Cancel is what Escape does.
pub fn prompt(app: &mut CadApp, ctx: &egui::Context) {
    let Some(pending) = app.save_loss.clone() else { return };
    let t = Tokens::get();
    let path = std::path::Path::new(pending.path());
    let file = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| pending.path().to_string());
    let mut choice = None;
    crate::modal::instant(ctx, egui::Id::unique("cc_save_loss_prompt")).show(ctx, |ui| {
        ui.set_width(420.0);
        ui.label(RichText::new(crate::tf!("DWG export is blocked for {name}", name = file)).strong());
        ui.add_space(4.0);
        ui.label(crate::tf!("Exporting would risk losing these objects or metadata: {objects}.", objects = pending.summary()));
        if pending.source_incomplete() {
            ui.label(RichText::new(crate::tl!("The source file was only partially read. Reopen a valid source before saving a complete drawing; skipped records cannot be reconstructed.")).color(t.text_dim));
        } else {
            ui.label(RichText::new(crate::tl!("Choose Save As and use DXF to preserve the content that was successfully read. A lossy DWG overwrite is disabled.")).color(t.text_dim));
        }
        ui.add_space(10.0);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let size = vec2(96.0, 22.0);
            let save_as = ui.add(egui::Button::new(RichText::new(crate::tl!("Save As...")).color(t.text)).fill(t.accent).min_size(size));
            if save_as.clicked() {
                choice = Some(Choice::SaveAs);
            }
            if ui.add(egui::Button::new(crate::tl!("Cancel")).min_size(size)).clicked() {
                choice = Some(Choice::Cancel);
            }
        });
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            choice = Some(Choice::Cancel);
        }
    });
    if let Some(c) = choice {
        app.answer_save_loss(c);
        ctx.request_repaint();
    }
}
