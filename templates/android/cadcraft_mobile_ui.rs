//! Builder-owned Android workspace for CADCraft.
//!
//! The drawing, prompts, command metadata, panels and dialogs remain owned by CADCraft's existing
//! UI and engine. This module replaces only the Android window chrome and adds touch-sized command
//! entry points over the existing command registry.

use egui::{Align, Align2, Color32, Id, Layout, RichText, Stroke, Vec2, vec2};
use serde_json::Value;

use crate::{CadApp, theme::Tokens};

const APP_BAR_HEIGHT: f32 = 56.0;
const NAV_BAR_HEIGHT: f32 = 64.0;
const TOOL_HIT_TARGET: Vec2 = vec2(48.0, 48.0);
const TOOL_PANEL_MAX_HEIGHT: f32 = 360.0;
const WIDE_LAYOUT_BREAKPOINT: f32 = 760.0;
const SPACE_XS: f32 = 4.0;
const SPACE_SM: f32 = 8.0;
const SPACE_MD: f32 = 12.0;
const BORDER_WIDTH: f32 = 1.0;
const PANEL_RADIUS: f32 = 10.0;
const CONTROL_RADIUS: f32 = 8.0;
const TITLE_TEXT_SIZE: f32 = 16.0;
const BODY_TEXT_SIZE: f32 = 14.0;
const CONTROL_TEXT_SIZE: f32 = 13.0;
const CATEGORY_TEXT_SIZE: f32 = 12.0;
const MOBILE_STATE_ID: &str = "ar_cad_android_workspace";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Category {
    Select,
    Draw,
    Modify,
    Measure,
    View,
    Document,
    #[default]
    More,
}

impl Category {
    const ALL: [Self; 7] = [
        Self::Select,
        Self::Draw,
        Self::Modify,
        Self::Measure,
        Self::View,
        Self::Document,
        Self::More,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Select => crate::tl!("Select"),
            Self::Draw => crate::tl!("Draw"),
            Self::Modify => crate::tl!("Modify"),
            Self::Measure => crate::tl!("Measure"),
            Self::View => crate::tl!("View"),
            Self::Document => crate::tl!("Document"),
            Self::More => crate::tl!("More"),
        }
    }

    fn short_label(self) -> &'static str {
        match self {
            Self::Select => crate::tl!("Select"),
            Self::Draw => crate::tl!("Draw"),
            Self::Modify => crate::tl!("Edit"),
            Self::Measure => crate::tl!("Measure"),
            Self::View => crate::tl!("View"),
            Self::Document => crate::tl!("File"),
            Self::More => crate::tl!("More"),
        }
    }
}

#[derive(Clone, Default)]
struct WorkspaceState {
    category: Category,
    tools_open: bool,
    search: String,
}

pub fn draw(app: &mut CadApp, ui: &mut egui::Ui) {
    let started = crate::now_ms();
    let ctx = ui.ctx().clone();
    crate::control::begin_frame(&ctx);

    // The existing command, panel and dialog implementations remain available, but desktop chrome
    // is not part of the Android workspace.
    app.ui.show_toolbar = false;
    app.ui.show_status_bar = false;
    app.ui.show_file_tabs = false;
    app.ui.in_window_menu = false;

    let t = Tokens::get();
    let mut state = ctx.data(|data| data.get_temp::<WorkspaceState>(Id::new(MOBILE_STATE_ID))).unwrap_or_default();
    let content = ctx.content_rect();
    let wide = content.width() >= WIDE_LAYOUT_BREAKPOINT && content.height() > 420.0;

    show_app_bar(app, ui, &mut state, t);

    let selected_count = app.session.selection().len();
    if selected_count > 0 {
        show_selection_bar(app, ui, selected_count, t);
    }

    if wide {
        show_wide_tool_rail(app, ui, &mut state, t);
    } else {
        show_phone_navigation(ui, &mut state, t);
    }

    egui::CentralPanel::default().frame(egui::Frame::NONE.fill(t.canvas)).show(ui, |ui| {
        let has_doc = !app.session.docs.is_empty() && !app.ui.start_tab;
        if has_doc {
            crate::docking::show(app, ui);
        } else {
            crate::chrome::start_page(app, ui);
        }
    });

    if state.tools_open {
        show_tool_sheet(app, &ctx, &mut state, t, wide);
    }
    if !state.tools_open {
        show_view_controls(app, &ctx, t, wide);
    }
    show_status_message(app, &ctx, t);

    app.opening.show(&ctx);
    crate::mtext_editor::show(app, &ctx);
    crate::dialogs::show(app, &ctx);
    crate::palettes::patterns::prompt_picker(app, &ctx);
    crate::closing::prompt(app, &ctx);
    crate::save_loss::prompt(app, &ctx);
    crate::autosave::show(app, &ctx);
    crate::autosave::gpu_notice(app, &ctx);

    ctx.data_mut(|data| data.insert_temp(Id::new(MOBILE_STATE_ID), state));
    app.frame_ms = crate::now_ms() - started;
}

fn show_app_bar(app: &mut CadApp, ui: &mut egui::Ui, state: &mut WorkspaceState, t: Tokens) {
    egui::Panel::top("ar_cad_mobile_app_bar")
        .exact_size(APP_BAR_HEIGHT)
        .frame(egui::Frame::NONE.fill(t.chrome).stroke(Stroke::new(BORDER_WIDTH, t.border)))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(SPACE_SM);
                let compact = ui.available_width() < 520.0;
                ui.label(RichText::new(if compact { "AR" } else { "AR CAD" }).strong().size(TITLE_TEXT_SIZE).color(t.text));
                ui.separator();

                if let Ok(document) = app.session.state() {
                    if document.is_dirty() {
                        ui.label(RichText::new("●").color(t.accent).size(10.0))
                            .on_hover_text(crate::tl!("Unsaved changes"));
                    }
                    let title_width = (ui.available_width() - 208.0).clamp(64.0, 220.0);
                    let title = ui.add_sized(
                        [title_width, TOOL_HIT_TARGET.y],
                        egui::Button::new(RichText::new(&document.title).size(BODY_TEXT_SIZE).color(t.text)).fill(t.chrome),
                    );
                    crate::control::record_widget(ui, &title, crate::tl!("Open drawings"), "button", None);
                    if title.clicked() {
                        state.category = Category::Document;
                        state.tools_open = true;
                        state.search.clear();
                    }
                } else {
                    ui.label(RichText::new(crate::tl!("No drawing")).size(BODY_TEXT_SIZE).color(t.text_dim));
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    action_button(ui, "•••", "More tools", t, true, || {
                        state.category = Category::More;
                        state.tools_open = !state.tools_open;
                    });
                    action_button(ui, "Save", "Save drawing", t, app.session.state().is_ok(), || app.start("qsave"));
                    let can_redo = app.session.state().is_ok_and(|document| !document.redo.is_empty());
                    action_button(ui, "↷", "Redo", t, can_redo, || app.start("redo"));
                    let can_undo = app.session.state().is_ok_and(|document| !document.undo.is_empty());
                    action_button(ui, "↶", "Undo", t, can_undo, || app.start("u"));
                });
            });
        });
}

fn show_selection_bar(app: &mut CadApp, ui: &mut egui::Ui, selected: usize, t: Tokens) {
    egui::Panel::top("ar_cad_mobile_selection_bar")
        .exact_size(56.0)
        .frame(egui::Frame::NONE.fill(t.panel).stroke(Stroke::new(BORDER_WIDTH, t.border)))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(SPACE_SM);
                ui.label(RichText::new(crate::tf!("{n} selected", n = selected)).strong().color(t.text));
                egui::ScrollArea::horizontal().id_salt("ar_cad_mobile_selection_actions").show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let registry = crate::control::all_commands(app);
                        for (label, command) in [
                            ("Move", "move"),
                            ("Copy", "copy"),
                            ("Rotate", "rotate"),
                            ("Delete", "erase"),
                            ("Properties", "ui.panel.properties"),
                        ] {
                            if let Some(item) = registry
                                .as_array()
                                .and_then(|items| items.iter().find(|item| item.get("id").and_then(Value::as_str) == Some(command)))
                            {
                                let _ = command_button(ui, app, item, t, 96.0);
                            } else {
                                ui.add_enabled(false, egui::Button::new(crate::i18n::t(label)).min_size(TOOL_HIT_TARGET))
                                    .on_disabled_hover_text(crate::tl!("This action is not registered."));
                            }
                        }
                    });
                });
            });
        });
}

fn show_phone_navigation(ui: &mut egui::Ui, state: &mut WorkspaceState, t: Tokens) {
    egui::Panel::bottom("ar_cad_mobile_navigation")
        .exact_size(NAV_BAR_HEIGHT)
        .frame(egui::Frame::NONE.fill(t.chrome).stroke(Stroke::new(BORDER_WIDTH, t.border)))
        .show(ui, |ui| {
            egui::ScrollArea::horizontal().id_salt("ar_cad_mobile_navigation_scroll").show(ui, |ui| {
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    for category in Category::ALL {
                        category_button(ui, state, category, t, true);
                    }
                });
            });
        });
}

fn show_wide_tool_rail(app: &mut CadApp, ui: &mut egui::Ui, state: &mut WorkspaceState, t: Tokens) {
    egui::Panel::left("ar_cad_mobile_tool_rail")
        .exact_size(76.0)
        .resizable(false)
        .frame(egui::Frame::NONE.fill(t.chrome).stroke(Stroke::new(BORDER_WIDTH, t.border)))
        .show(ui, |ui| {
            egui::ScrollArea::vertical().id_salt("ar_cad_mobile_categories").show(ui, |ui| {
                for category in Category::ALL {
                    category_button(ui, state, category, t, false);
                }
                if !app.session.selection().is_empty() {
                    ui.separator();
                    action_button(ui, "Props", "Open properties", t, true, || app.start("ui.panel.properties"));
                }
            });
        });
}

fn category_button(ui: &mut egui::Ui, state: &mut WorkspaceState, category: Category, t: Tokens, horizontal: bool) {
    let selected = state.category == category && state.tools_open;
    let label = if horizontal { category.short_label() } else { category.label() };
    let button = egui::Button::new(RichText::new(label).size(CATEGORY_TEXT_SIZE).color(if selected { Color32::WHITE } else { t.text }))
        .min_size(if horizontal { vec2(48.0, 48.0) } else { vec2(68.0, 48.0) })
        .fill(if selected { t.accent } else { t.control });
    let response = ui.add(button);
    crate::control::record_widget(ui, &response, category.label(), "button", None);
    if response.clicked() {
        if state.category == category {
            state.tools_open = !state.tools_open;
        } else {
            state.category = category;
            state.tools_open = true;
        }
        state.search.clear();
    }
}

fn show_tool_sheet(app: &mut CadApp, ctx: &egui::Context, state: &mut WorkspaceState, t: Tokens, wide: bool) {
    let screen = ctx.content_rect();
    let max_width = (screen.width() - 24.0).max(220.0).min(if wide { 620.0 } else { 520.0 });
    let max_height = (screen.height() * 0.48).min(TOOL_PANEL_MAX_HEIGHT).max(140.0);
    let bottom_offset = if wide { 12.0 } else { NAV_BAR_HEIGHT + 8.0 };

    egui::Area::new(Id::new("ar_cad_mobile_tool_sheet"))
        .order(egui::Order::Foreground)
        .anchor(Align2::CENTER_BOTTOM, vec2(0.0, -bottom_offset))
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(t.panel)
                .stroke(Stroke::new(BORDER_WIDTH, t.border))
                .corner_radius(PANEL_RADIUS)
                .inner_margin(SPACE_MD)
                .show(ui, |ui| {
                    ui.set_width(max_width);
                    ui.set_max_height(max_height);
                    ui.horizontal(|ui| {
                        ui.heading(RichText::new(state.category.label()).color(t.text));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            action_button(ui, "Close", "Close tool panel", t, true, || state.tools_open = false);
                        });
                    });

                    let search = ui.add_sized(
                        [ui.available_width(), TOOL_HIT_TARGET.y],
                        egui::TextEdit::singleline(&mut state.search).hint_text(crate::tl!("Search every CAD command")),
                    );
                    search.on_hover_text(crate::tl!("Search engine commands, UI commands, aliases and menu names"));

                    if state.category == Category::View {
                        show_drafting_toggles(app, ui, t);
                    }
                    if state.category == Category::Document {
                        show_open_drawings(app, ui, state, t);
                    }

                    let registry = crate::control::all_commands(app);
                    if let Some(commands) = registry.as_array() {
                        let filtered: Vec<&Value> = commands
                            .iter()
                            .filter(|command| {
                                (matches_category(command, state.category) || !state.search.trim().is_empty())
                                    && matches_search(command, &state.search)
                                    && command.get("label").and_then(Value::as_str).is_some_and(|label| !label.is_empty())
                            })
                            .collect();
                        let columns = if max_width >= 540.0 { 3 } else { 2 };
                        let button_width = ((max_width - 52.0) / columns as f32).clamp(128.0, 244.0);
                        egui::ScrollArea::vertical()
                            .id_salt("ar_cad_mobile_command_results")
                            .max_height((max_height - 128.0).max(96.0))
                            .show_rows(ui, 56.0, filtered.len().div_ceil(columns), |ui, rows| {
                                for row in rows {
                                    ui.horizontal(|ui| {
                                        for column in 0..columns {
                                            if let Some(command) = filtered.get(row * columns + column) {
                                                if command_button(ui, app, command, t, button_width) {
                                                    state.tools_open = false;
                                                }
                                            }
                                        }
                                    });
                                }
                            });
                        if filtered.is_empty() {
                            ui.label(RichText::new(crate::tl!("No registered command matches.")).color(t.text_dim));
                        }
                    }
                });
        });
}

fn show_open_drawings(app: &mut CadApp, ui: &mut egui::Ui, state: &mut WorkspaceState, t: Tokens) {
    ui.label(RichText::new(crate::tl!("Open drawings")).strong().color(t.text));
    ui.horizontal_wrapped(|ui| {
        action_button(ui, "New", "New drawing", t, true, || {
            app.start("new");
            app.ui.start_tab = false;
            state.tools_open = false;
        });
        action_button(ui, "Start", "Show start page", t, true, || {
            app.ui.start_tab = true;
            state.tools_open = false;
        });
        if app.session.state().is_ok() {
            action_button(ui, "Close active", "Close active drawing", t, true, || {
                app.request_close(None);
                state.tools_open = false;
            });
        }
    });

    let drawings: Vec<(usize, String, bool, bool)> = app
        .session
        .docs
        .iter()
        .enumerate()
        .map(|(index, doc)| (index, doc.title.clone(), doc.is_dirty(), index == app.session.active))
        .collect();
    egui::ScrollArea::horizontal().id_salt("ar_cad_mobile_open_drawings").show(ui, |ui| {
        ui.horizontal(|ui| {
            for (index, title, dirty, active) in drawings {
                let title = format!("{}{}", title, if dirty { " *" } else { "" });
                let response = ui.add_sized(
                    [180.0, TOOL_HIT_TARGET.y],
                    egui::Button::selectable(active && !app.ui.start_tab, title),
                );
                crate::control::record_widget(ui, &response, "Switch drawing", "button", Some(active));
                if response.clicked() {
                    let _ = app.run("document.switch", serde_json::json!({"index": index}));
                    app.ui.start_tab = false;
                }
                action_button(ui, "×", "Close drawing", t, true, || app.request_close(Some(index)));
            }
        });
    });

    let recent_files = app.ui.recent_files.clone();
    if !recent_files.is_empty() {
        ui.label(RichText::new(crate::tl!("Recent files")).strong().color(t.text));
        egui::ScrollArea::vertical().id_salt("ar_cad_mobile_recent_files").max_height(112.0).show(ui, |ui| {
            for (index, path) in recent_files.iter().enumerate() {
                let label = std::path::Path::new(path).file_name().and_then(|name| name.to_str()).unwrap_or(path);
                let command = format!("ui.recent.{index}");
                action_button(ui, label, path, t, true, || {
                    app.start(&command);
                    state.tools_open = false;
                });
            }
        });
        action_button(ui, "Clear recent", "Clear recent files", t, true, || app.start("ui.recent.clear"));
    }
}

fn show_drafting_toggles(app: &mut CadApp, ui: &mut egui::Ui, t: Tokens) {
    let settings = app.session.settings.clone();
    let toggles = [
        (crate::tl!("Grid"), settings.gridmode, "grid"),
        (crate::tl!("Snap"), settings.snapmode, "snap"),
        (crate::tl!("Ortho"), settings.orthomode, "ortho"),
        (crate::tl!("Polar"), settings.polarmode, "polar"),
        (crate::tl!("OSnap"), settings.osmode & cadcraft_engine::snap::mode::OFF == 0 && settings.osmode != 0, "osnap"),
        (crate::tl!("Input"), settings.dynmode, "dynmode"),
    ];
    ui.horizontal_wrapped(|ui| {
        for (label, active, command) in toggles {
            let response = ui.add_sized(
                [76.0, TOOL_HIT_TARGET.y],
                egui::Button::new(RichText::new(label).color(if active { t.accent } else { t.text }))
                    .fill(if active { t.accent.gamma_multiply(0.18) } else { t.control }),
            );
            crate::control::record_widget(ui, &response, label, "button", Some(active));
            if response.clicked() {
                crate::menus::run_transparent(app, command);
            }
        }
    });
}

fn show_status_message(app: &CadApp, ctx: &egui::Context, t: Tokens) {
    let Some((message, at)) = app.status.as_ref() else {
        return;
    };
    let elapsed = crate::now_ms() - at;
    if elapsed >= 5000.0 {
        return;
    }
    if let Ok(remaining) = std::time::Duration::try_from_secs_f64((5000.0 - elapsed) / 1000.0) {
        ctx.request_repaint_after(remaining);
    }
    egui::Area::new(Id::new("ar_cad_mobile_status_message"))
        .order(egui::Order::Foreground)
        .anchor(Align2::LEFT_TOP, vec2(12.0, APP_BAR_HEIGHT + 8.0))
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(t.cmd_bg)
                .stroke(Stroke::new(BORDER_WIDTH, t.cmd_border))
                .corner_radius(CONTROL_RADIUS)
                .inner_margin(SPACE_SM)
                .show(ui, |ui| {
                    ui.label(RichText::new(message.as_str()).color(t.text).size(CONTROL_TEXT_SIZE));
                });
        });
}

fn show_view_controls(app: &mut CadApp, ctx: &egui::Context, t: Tokens, wide: bool) {
    let bottom_offset = if wide { 10.0 } else { NAV_BAR_HEIGHT + 10.0 };
    egui::Area::new(Id::new("ar_cad_mobile_view_controls"))
        .order(egui::Order::Foreground)
        .anchor(Align2::RIGHT_BOTTOM, vec2(-10.0, -bottom_offset))
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(t.chrome)
                .stroke(Stroke::new(BORDER_WIDTH, t.border))
                .corner_radius(CONTROL_RADIUS)
                .inner_margin(SPACE_XS)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        action_button(ui, "+", "Zoom in", t, true, || zoom(app, 0.8));
                        action_button(ui, "−", "Zoom out", t, true, || zoom(app, 1.25));
                        action_button(ui, "Fit", "Fit drawing", t, true, || app.start("zoom.extents"));
                    });
                });
        });
}

fn command_button(ui: &mut egui::Ui, app: &mut CadApp, command: &Value, t: Tokens, button_width: f32) -> bool {
    let Some(id) = command.get("id").and_then(Value::as_str) else {
        return false;
    };
    let label = command.get("label").and_then(Value::as_str).unwrap_or(id);
    let engine_enabled = command.get("enabled").and_then(Value::as_bool).unwrap_or(false);
    let params = command.get("params").and_then(Value::as_str).unwrap_or_default();
    let param_contract = params.split_whitespace().next().unwrap_or_default();
    let interactive = command.get("interactive").and_then(Value::as_bool).unwrap_or(false);
    let requires_arguments = requires_arguments_for_start(id, param_contract, interactive);
    let enabled = engine_enabled && (!requires_arguments || interactive);
    let reason = if !engine_enabled {
        command.get("disabledReason").and_then(Value::as_str).unwrap_or("Command metadata unavailable")
    } else if requires_arguments && !interactive {
        crate::tl!("Requires parameters; use the command line's JSON form.")
    } else {
        ""
    };
    let shortcut = command.get("shortcut").and_then(Value::as_str);
    let caption = match shortcut {
        Some(key) => format!("{label}  ·  {key}"),
        None => label.to_owned(),
    };
    let response = ui.add_enabled(
        enabled,
        egui::Button::new(RichText::new(caption).size(CONTROL_TEXT_SIZE).color(t.text)).min_size(vec2(button_width, TOOL_HIT_TARGET.y)),
    );
    let response = response.on_hover_text(crate::i18n::t(label));
    crate::control::record_widget(ui, &response, crate::i18n::t(label), "button", None);
    let response = if enabled { response } else { response.on_disabled_hover_text(crate::i18n::t(reason)) };
    if response.clicked() {
        app.start(id);
        return true;
    }
    false
}

fn requires_arguments_for_start(id: &str, param_contract: &str, interactive: bool) -> bool {
    let requires_arguments = !param_contract.is_empty() && param_contract != "{}" && !param_contract.contains('?');
    requires_arguments
        && !interactive
        && !matches!(
            id,
            "open"
                | "saveas"
                | "recover"
                | "import"
                | "imp"
                | "export"
                | "exp"
                | "publish"
                | "preview"
                | "pre"
                | "plot"
                | "print"
                | "exportpdf"
                | "appload"
                | "ap"
        )
}

fn action_button(
    ui: &mut egui::Ui,
    label: &str,
    description: &str,
    t: Tokens,
    enabled: bool,
    action: impl FnOnce(),
) {
    let response = ui.add_enabled(
        enabled,
        egui::Button::new(RichText::new(crate::i18n::t(label)).size(CONTROL_TEXT_SIZE).color(t.text)).min_size(TOOL_HIT_TARGET),
    );
    let response = response.on_hover_text(crate::i18n::t(description));
    crate::control::record_widget(ui, &response, crate::i18n::t(description), "button", None);
    if response.clicked() {
        action();
    }
}

fn zoom(app: &mut CadApp, factor: f64) {
    match app.session.state().map(|state| state.view().center) {
        Ok(center) => {
            if let Err(error) = app.session.zoom_about(factor, center) {
                app.session.echo(error.to_string());
            }
        }
        Err(error) => app.session.echo(error.to_string()),
    }
}

fn matches_category(command: &Value, category: Category) -> bool {
    if category == Category::More {
        return true;
    }
    let id = command.get("id").and_then(Value::as_str).unwrap_or_default().to_ascii_lowercase();
    let root = command
        .get("menu")
        .and_then(Value::as_array)
        .and_then(|path| path.first())
        .and_then(Value::as_str)
        .unwrap_or_default();
    match category {
        Category::Select => {
            root.eq_ignore_ascii_case("select")
                || matches!(id.as_str(), "select" | "selectall" | "ai_deselect" | "qselect" | "ui.dialog.qselect")
                || id.starts_with("select.")
        }
        Category::Draw => {
            root.eq_ignore_ascii_case("draw")
                || matches!(id.as_str(), "line" | "pline" | "circle" | "rectang" | "arc" | "ellipse" | "spline" | "polygon" | "text" | "mtext" | "hatch")
        }
        Category::Modify => {
            root.eq_ignore_ascii_case("modify")
                || (root.eq_ignore_ascii_case("edit") && !matches!(id.as_str(), "selectall" | "ai_deselect" | "qselect" | "ui.dialog.qselect"))
        }
        Category::Measure => {
            root.eq_ignore_ascii_case("dimension")
                || root.eq_ignore_ascii_case("inquiry")
                || id.starts_with("dim")
                || matches!(id.as_str(), "dist" | "area" | "list")
        }
        Category::View => root.eq_ignore_ascii_case("view") || id.starts_with("zoom.") || id == "pan",
        Category::Document => {
            root.eq_ignore_ascii_case("file")
                || matches!(id.as_str(), "new" | "open" | "qsave" | "saveas" | "ui.open" | "ui.saveas" | "import" | "export")
        }
        Category::More => true,
    }
}

fn matches_search(command: &Value, query: &str) -> bool {
    let query = query.trim().to_ascii_lowercase();
    if query.is_empty() {
        return true;
    }
    ["id", "label", "shortcut"]
        .into_iter()
        .filter_map(|field| command.get(field).and_then(Value::as_str))
            .any(|value| value.to_ascii_lowercase().contains(&query))
        || command
                .get("aliases")
                .and_then(Value::as_array)
                .is_some_and(|aliases| aliases.iter().filter_map(Value::as_str).any(|alias| alias.to_ascii_lowercase().contains(&query)))
            || command
            .get("menu")
            .and_then(Value::as_array)
            .is_some_and(|path| path.iter().filter_map(Value::as_str).any(|value| value.to_ascii_lowercase().contains(&query)))
}

#[cfg(test)]
mod tests {
    use super::{Category, matches_category, matches_search, requires_arguments_for_start};
    use serde_json::json;

    #[test]
    fn categories_route_registered_commands_without_replacing_the_registry() {
        let draw = json!({"id":"circle","label":"Circle","menu":["Draw","Circle"]});
        let modify = json!({"id":"offset","label":"Offset","menu":["Modify","Offset"]});
        let select = json!({"id":"selectall","label":"Select All","menu":["Edit","Select All"]});
        assert!(matches_category(&draw, Category::Draw));
        assert!(matches_category(&modify, Category::Modify));
        assert!(matches_category(&select, Category::Select));
        assert!(matches_category(&draw, Category::More));
    }

    #[test]
    fn search_matches_command_ids_shortcuts_and_menu_paths() {
        let command = json!({
            "id":"dimlinear",
            "label":"Linear",
            "shortcut":"DLI",
            "menu":["Dimension","Linear"]
        });
        assert!(matches_search(&command, "dimline"));
        assert!(matches_search(&command, "dli"));
        assert!(matches_search(&command, "dimension"));
        assert!(!matches_search(&command, "fillet"));
    }

    #[test]
    fn file_commands_keep_their_existing_picker_flows() {
        for id in ["open", "saveas", "import", "export", "publish", "preview", "plot", "recover"] {
            assert!(!requires_arguments_for_start(id, "{path}", false), "{id} should use its existing UI flow");
        }
        assert!(requires_arguments_for_start("document.switch", "{index}", false));
        assert!(!requires_arguments_for_start("line", "{point}", true));
    }
}
