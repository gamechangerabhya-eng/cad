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
const MEDIUM_LAYOUT_BREAKPOINT: f32 = 520.0;
const EXPANDED_LAYOUT_MIN_HEIGHT: f32 = 420.0;
const APP_BAR_BRAND_BREAKPOINT: f32 = 360.0;
const APP_BAR_SAVE_BREAKPOINT: f32 = 280.0;
const TOOL_SHEET_TWO_COLUMN_BREAKPOINT: f32 = 460.0;
const TOOL_SHEET_THREE_COLUMN_BREAKPOINT: f32 = 540.0;
const TOOL_SHEET_MAX_WIDTH: f32 = 520.0;
const EXPANDED_TOOL_SHEET_MAX_WIDTH: f32 = 620.0;
const RECENT_COMMAND_LIMIT: usize = 5;
const RECENT_COMMANDS_ID: &str = "ar_cad_android_recent_commands";
const FAVORITE_COMMANDS_ID: &str = "ar_cad_android_favorite_commands";
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
enum LayoutClass {
    #[default]
    Compact,
    Medium,
    Expanded,
}

#[derive(Clone, Copy, Debug)]
struct ResponsiveLayout {
    class: LayoutClass,
    keyboard_focused: bool,
}

impl ResponsiveLayout {
    fn new(viewport: Vec2, keyboard_focused: bool) -> Self {
        let class = if viewport.x >= WIDE_LAYOUT_BREAKPOINT && viewport.y > EXPANDED_LAYOUT_MIN_HEIGHT {
            LayoutClass::Expanded
        } else if viewport.x >= MEDIUM_LAYOUT_BREAKPOINT {
            LayoutClass::Medium
        } else {
            LayoutClass::Compact
        };
        Self { class, keyboard_focused }
    }

    fn is_expanded(self) -> bool {
        self.class == LayoutClass::Expanded
    }

    fn tool_sheet_max_width(self) -> f32 {
        if self.is_expanded() { EXPANDED_TOOL_SHEET_MAX_WIDTH } else { TOOL_SHEET_MAX_WIDTH }
    }

    fn bottom_offset(self) -> f32 {
        if self.is_expanded() || self.keyboard_focused { SPACE_MD } else { NAV_BAR_HEIGHT + SPACE_SM }
    }
}

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
    const ALL: [Self; 7] = [Self::Select, Self::Draw, Self::Modify, Self::Measure, Self::View, Self::Document, Self::More];

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
    advanced_console_open: bool,
    history_open: bool,
    guide_open: bool,
    console_input: String,
    console_error: Option<String>,
    search: String,
    recent_commands: Vec<String>,
    favorite_commands: Vec<String>,
    command_lists_loaded: bool,
    command_lists_changed: bool,
    parameter_command: Option<String>,
    parameter_json: String,
    parameter_error: Option<String>,
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
    let mut state = ctx.data(|data| data.get_temp::<WorkspaceState>(Id::unique(MOBILE_STATE_ID))).unwrap_or_default();
    if !state.command_lists_loaded {
        let (recent, favorites) = ctx.data_mut(|data| {
            (
                data.get_persisted::<Vec<String>>(Id::unique(RECENT_COMMANDS_ID)).unwrap_or_default(),
                data.get_persisted::<Vec<String>>(Id::unique(FAVORITE_COMMANDS_ID)).unwrap_or_default(),
            )
        });
        state.recent_commands = recent;
        state.favorite_commands = favorites;
        state.command_lists_loaded = true;
    }
    let content = ctx.content_rect();
    let layout = ResponsiveLayout::new(vec2(content.width(), content.height()), ctx.egui_wants_keyboard_input());
    app.ui.show_command_line = false;
    if state.tools_open
        && app.session.current_prompt().is_none()
        && ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
    {
        state.tools_open = false;
    }
    if state.parameter_command.is_some() && ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
        state.parameter_command = None;
        state.parameter_error = None;
    }
    if state.history_open && ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
        state.history_open = false;
    }
    if state.guide_open && ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
        state.guide_open = false;
    }
    if state.advanced_console_open && ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
        state.advanced_console_open = false;
        state.console_error = None;
    }

    show_app_bar(app, ui, &mut state, t);

    let selected_count = app.session.selection().len();
    if selected_count > 0 {
        show_selection_bar(app, ui, selected_count, &mut state, t);
    }

    if layout.is_expanded() {
        show_wide_tool_rail(app, ui, &mut state, t);
    } else if !layout.keyboard_focused {
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
        show_tool_sheet(app, &ctx, &mut state, t, layout);
    }
    if state.parameter_command.is_some() {
        show_parameter_sheet(app, &ctx, &mut state, t, layout);
    } else if state.history_open {
        show_history_sheet(app, &ctx, &mut state, t, layout);
    } else if state.guide_open {
        show_touch_guide(&ctx, &mut state, t, layout);
    } else if state.advanced_console_open {
        show_command_console(app, &ctx, &mut state, t, layout);
    } else if app.session.current_prompt().is_some() && !state.advanced_console_open {
        show_command_card(app, &ctx, t, layout);
    } else if !state.tools_open {
        show_view_controls(app, &ctx, t, layout);
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

    ctx.data_mut(|data| {
        if state.command_lists_changed {
            data.insert_persisted(Id::unique(RECENT_COMMANDS_ID), state.recent_commands.clone());
            data.insert_persisted(Id::unique(FAVORITE_COMMANDS_ID), state.favorite_commands.clone());
            state.command_lists_changed = false;
        }
        data.insert_temp(Id::unique(MOBILE_STATE_ID), state);
    });
    app.frame_ms = crate::now_ms() - started;
}

fn show_app_bar(app: &mut CadApp, ui: &mut egui::Ui, state: &mut WorkspaceState, t: Tokens) {
    egui::Panel::top("ar_cad_mobile_app_bar")
        .exact_size(APP_BAR_HEIGHT)
        .frame(egui::Frame::NONE.fill(t.chrome).stroke(Stroke::new(BORDER_WIDTH, t.border)))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let viewport_width = ui.available_width();
                ui.add_space(SPACE_SM);
                if viewport_width >= APP_BAR_BRAND_BREAKPOINT {
                    let compact = viewport_width < 520.0;
                    ui.label(RichText::new(if compact { "AR" } else { "AR CAD" }).strong().size(TITLE_TEXT_SIZE).color(t.text));
                    ui.separator();
                }

                if let Ok(document) = app.session.state() {
                    if document.is_dirty() {
                        ui.label(RichText::new("●").color(t.accent).size(10.0)).on_hover_text(crate::tl!("Unsaved changes"));
                    }
                    let quick_save = viewport_width >= APP_BAR_SAVE_BREAKPOINT;
                    let action_width = if quick_save { 4.0 * TOOL_HIT_TARGET.x + 3.0 * SPACE_SM } else { 3.0 * TOOL_HIT_TARGET.x + 2.0 * SPACE_SM };
                    let title_width = (ui.available_width() - action_width - SPACE_SM).clamp(0.0, 220.0);
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
                    let quick_save = viewport_width >= APP_BAR_SAVE_BREAKPOINT;
                    let action_width = if quick_save { 4.0 * TOOL_HIT_TARGET.x + 3.0 * SPACE_SM } else { 3.0 * TOOL_HIT_TARGET.x + 2.0 * SPACE_SM };
                    let label_width = (ui.available_width() - action_width - SPACE_SM).clamp(0.0, 220.0);
                    if label_width >= 72.0 {
                        ui.add_sized(
                            [label_width, TOOL_HIT_TARGET.y],
                            egui::Label::new(RichText::new(crate::tl!("No drawing")).size(BODY_TEXT_SIZE).color(t.text_dim)),
                        );
                    }
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    action_button(ui, "•••", "More tools", t, true, || {
                        state.category = Category::More;
                        state.tools_open = !state.tools_open;
                    });
                    if viewport_width >= APP_BAR_SAVE_BREAKPOINT {
                        action_button(ui, "Save", "Save drawing", t, app.session.state().is_ok(), || app.start("qsave"));
                    }
                    let can_redo = app.session.state().is_ok_and(|document| !document.redo.is_empty());
                    action_button(ui, "↷", "Redo", t, can_redo, || app.start("redo"));
                    let can_undo = app.session.state().is_ok_and(|document| !document.undo.is_empty());
                    action_button(ui, "↶", "Undo", t, can_undo, || app.start("u"));
                });
            });
        });
}

fn show_selection_bar(app: &mut CadApp, ui: &mut egui::Ui, selected: usize, state: &mut WorkspaceState, t: Tokens) {
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
                            ("Scale", "scale"),
                            ("Offset", "offset"),
                            ("Fillet", "fillet"),
                            ("Delete", "erase"),
                            ("Properties", "ui.panel.properties"),
                        ] {
                            if let Some(item) = registry
                                .as_array()
                                .and_then(|items| items.iter().find(|item| item.get("id").and_then(Value::as_str) == Some(command)))
                            {
                                if command_button(ui, app, item, state, t, 96.0, false) {
                                    if let Some(id) = item.get("id").and_then(Value::as_str) {
                                        remember_command(state, id);
                                    }
                                }
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

fn show_tool_sheet(app: &mut CadApp, ctx: &egui::Context, state: &mut WorkspaceState, t: Tokens, layout: ResponsiveLayout) {
    let screen = ctx.content_rect();
    let panel_width = screen.width().min(layout.tool_sheet_max_width());
    let max_width = (panel_width - SPACE_MD * 2.0 - BORDER_WIDTH * 2.0).max(0.0);
    let bottom_offset = layout.bottom_offset();
    let available_height = (screen.height() - APP_BAR_HEIGHT - bottom_offset - SPACE_MD * 2.0).max(0.0);
    let panel_height = (screen.height() * 0.48).min(TOOL_PANEL_MAX_HEIGHT).min(available_height);
    let max_height = (panel_height - SPACE_MD * 2.0 - BORDER_WIDTH * 2.0).max(0.0);
    let columns = tool_sheet_columns(max_width);

    egui::Area::new(Id::unique("ar_cad_mobile_tool_sheet"))
        .order(egui::Order::Foreground)
        .anchor(Align2::CENTER_BOTTOM, vec2(0.0, -bottom_offset))
        .show(ctx, |ui| {
            egui::Frame::new().fill(t.panel).stroke(Stroke::new(BORDER_WIDTH, t.border)).corner_radius(PANEL_RADIUS).inner_margin(SPACE_MD).show(
                ui,
                |ui| {
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
                    if state.category == Category::More && state.search.trim().is_empty() && !state.favorite_commands.is_empty() {
                        ui.label(RichText::new(crate::tl!("Favorites")).strong().color(t.text));
                        let favorites = state.favorite_commands.clone();
                        egui::ScrollArea::horizontal().id_salt("ar_cad_mobile_favorite_tools").max_height(TOOL_HIT_TARGET.y + SPACE_SM).show(
                            ui,
                            |ui| {
                                ui.horizontal(|ui| {
                                    for id in favorites {
                                        let Some(command) = registry.as_array().and_then(|commands| {
                                            commands.iter().find(|command| command.get("id").and_then(Value::as_str) == Some(id.as_str()))
                                        }) else {
                                            continue;
                                        };
                                        if command_button(ui, app, command, state, t, 240.0, true) {
                                            remember_command(state, &id);
                                            state.tools_open = false;
                                        }
                                    }
                                });
                            },
                        );
                    }
                    if state.category == Category::More && state.search.trim().is_empty() && !state.recent_commands.is_empty() {
                        ui.label(RichText::new(crate::tl!("Recent tools")).strong().color(t.text));
                        let recent = state.recent_commands.clone();
                        egui::ScrollArea::horizontal().id_salt("ar_cad_mobile_recent_tools").max_height(TOOL_HIT_TARGET.y + SPACE_SM).show(
                            ui,
                            |ui| {
                                ui.horizontal(|ui| {
                                    for id in recent {
                                        let Some(command) = registry.as_array().and_then(|commands| {
                                            commands.iter().find(|command| command.get("id").and_then(Value::as_str) == Some(id.as_str()))
                                        }) else {
                                            continue;
                                        };
                                        if command_button(ui, app, command, state, t, 128.0, false) {
                                            remember_command(state, &id);
                                            state.tools_open = false;
                                        }
                                    }
                                });
                            },
                        );
                    }

                    if state.category == Category::More {
                        action_button(ui, "Touch guide", "Show common touch gestures and command controls", t, true, || {
                            state.guide_open = true;
                            state.tools_open = false;
                        });
                        action_button(ui, "Command history", "Show recent command and status messages", t, true, || {
                            state.history_open = true;
                            state.tools_open = false;
                        });
                        action_button(ui, "Command console", "Open the advanced command console", t, true, || {
                            state.advanced_console_open = !state.advanced_console_open;
                            state.tools_open = false;
                        });
                    }

                    if let Some(commands) = registry.as_array() {
                        let filtered: Vec<&Value> = commands
                            .iter()
                            .filter(|command| {
                                (matches_category(command, state.category) || !state.search.trim().is_empty())
                                    && matches_search(command, &state.search)
                                    && command.get("label").and_then(Value::as_str).is_some_and(|label| !label.is_empty())
                            })
                            .collect();
                        let button_width = ((max_width - SPACE_SM * (columns - 1) as f32) / columns as f32).max(0.0);
                        egui::ScrollArea::vertical().id_salt("ar_cad_mobile_command_results").max_height(ui.available_height().max(0.0)).show_rows(
                            ui,
                            56.0,
                            filtered.len().div_ceil(columns),
                            |ui, rows| {
                                for row in rows {
                                    ui.horizontal(|ui| {
                                        for column in 0..columns {
                                            if let Some(command) = filtered.get(row * columns + column) {
                                                if command_button(ui, app, command, state, t, button_width, true) {
                                                    if let Some(id) = command.get("id").and_then(Value::as_str) {
                                                        remember_command(state, id);
                                                    }
                                                    state.tools_open = false;
                                                }
                                            }
                                        }
                                    });
                                }
                            },
                        );
                        if filtered.is_empty() {
                            ui.label(RichText::new(crate::tl!("No registered command matches.")).color(t.text_dim));
                        }
                    }
                },
            );
        });
}

fn show_history_sheet(app: &CadApp, ctx: &egui::Context, state: &mut WorkspaceState, t: Tokens, layout: ResponsiveLayout) {
    let screen = ctx.content_rect();
    let max_width = (screen.width().min(layout.tool_sheet_max_width()) - SPACE_MD * 2.0 - BORDER_WIDTH * 2.0).max(0.0);
    let bottom_offset = layout.bottom_offset();
    let available_height = (screen.height() - APP_BAR_HEIGHT - bottom_offset - SPACE_MD * 2.0).max(0.0);
    let max_height =
        ((screen.height() * 0.62).min(TOOL_PANEL_MAX_HEIGHT + 120.0).min(available_height) - SPACE_MD * 2.0 - BORDER_WIDTH * 2.0).max(0.0);
    egui::Area::new(Id::unique("ar_cad_mobile_history_sheet"))
        .order(egui::Order::Foreground)
        .anchor(Align2::CENTER_BOTTOM, vec2(0.0, -bottom_offset))
        .show(ctx, |ui| {
            egui::Frame::new().fill(t.panel).stroke(Stroke::new(BORDER_WIDTH, t.border)).corner_radius(PANEL_RADIUS).inner_margin(SPACE_MD).show(
                ui,
                |ui| {
                    ui.set_width(max_width);
                    ui.set_max_height(max_height);
                    ui.horizontal(|ui| {
                        ui.heading(RichText::new(crate::tl!("Command history")).color(t.text));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            action_button(ui, "Close", "Close command history", t, true, || state.history_open = false);
                        });
                    });
                    egui::ScrollArea::vertical().id_salt("ar_cad_mobile_history_entries").show(ui, |ui| {
                        for entry in app.session.log.iter().rev().take(50) {
                            ui.label(RichText::new(entry).size(CONTROL_TEXT_SIZE).color(t.text));
                        }
                    });
                },
            );
        });
}

fn show_touch_guide(ctx: &egui::Context, state: &mut WorkspaceState, t: Tokens, layout: ResponsiveLayout) {
    let screen = ctx.content_rect();
    let max_width = (screen.width().min(layout.tool_sheet_max_width()) - SPACE_MD * 2.0 - BORDER_WIDTH * 2.0).max(0.0);
    let bottom_offset = layout.bottom_offset();
    let available_height = (screen.height() - APP_BAR_HEIGHT - bottom_offset - SPACE_MD * 2.0).max(0.0);
    let max_height =
        ((screen.height() * 0.62).min(TOOL_PANEL_MAX_HEIGHT + 120.0).min(available_height) - SPACE_MD * 2.0 - BORDER_WIDTH * 2.0).max(0.0);
    egui::Area::new(Id::unique("ar_cad_mobile_touch_guide"))
        .order(egui::Order::Foreground)
        .anchor(Align2::CENTER_BOTTOM, vec2(0.0, -bottom_offset))
        .show(ctx, |ui| {
            egui::Frame::new().fill(t.panel).stroke(Stroke::new(BORDER_WIDTH, t.border)).corner_radius(PANEL_RADIUS).inner_margin(SPACE_MD).show(
                ui,
                |ui| {
                    ui.set_width(max_width);
                    ui.set_max_height(max_height);
                    ui.horizontal(|ui| {
                        ui.heading(RichText::new(crate::tl!("Touch guide")).color(t.text));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            action_button(ui, "Close", "Close the touch guide", t, true, || state.guide_open = false);
                        });
                    });
                    egui::ScrollArea::vertical().id_salt("ar_cad_mobile_touch_guide_content").show(ui, |ui| {
                        for (title, description) in [
                            (
                                crate::tl!("Select"),
                                crate::tl!("Choose Select, then tap entities. Use the selection actions to modify the current selection."),
                            ),
                            (
                                crate::tl!("Draw"),
                                crate::tl!("Choose Draw and a registered tool. Follow the active command card's prompt on the canvas."),
                            ),
                            (
                                crate::tl!("Exact input"),
                                crate::tl!(
                                    "Focus the active command field to enter coordinates, distances, or other values accepted by that prompt."
                                ),
                            ),
                            (
                                crate::tl!("Finish or cancel"),
                                crate::tl!("Use Confirm or Apply when offered. Cancel exits the active command without placing the next point."),
                            ),
                            (
                                crate::tl!("Pan and zoom"),
                                crate::tl!("Use two fingers to pan and pinch to zoom. One-finger input follows the active CAD command."),
                            ),
                            (
                                crate::tl!("Keyboard and pointer"),
                                crate::tl!("Existing keyboard shortcuts, mouse, and trackpad input remain available alongside touch controls."),
                            ),
                        ] {
                            ui.label(RichText::new(title).strong().color(t.text));
                            ui.label(RichText::new(description).color(t.text_dim));
                            ui.add_space(SPACE_SM);
                        }
                    });
                },
            );
        });
}

fn show_command_console(app: &mut CadApp, ctx: &egui::Context, state: &mut WorkspaceState, t: Tokens, layout: ResponsiveLayout) {
    let screen = ctx.content_rect();
    let max_width = (screen.width().min(layout.tool_sheet_max_width()) - SPACE_MD * 2.0 - BORDER_WIDTH * 2.0).max(0.0);
    let bottom_offset = layout.bottom_offset();
    let available_height = (screen.height() - APP_BAR_HEIGHT - bottom_offset - SPACE_MD * 2.0).max(0.0);
    let max_height =
        ((screen.height() * 0.62).min(TOOL_PANEL_MAX_HEIGHT + 120.0).min(available_height) - SPACE_MD * 2.0 - BORDER_WIDTH * 2.0).max(0.0);
    let mut submit = false;
    let mut close = false;

    egui::Area::new(Id::unique("ar_cad_mobile_command_console"))
        .order(egui::Order::Foreground)
        .anchor(Align2::CENTER_BOTTOM, vec2(0.0, -bottom_offset))
        .show(ctx, |ui| {
            egui::Frame::new().fill(t.panel).stroke(Stroke::new(BORDER_WIDTH, t.border)).corner_radius(PANEL_RADIUS).inner_margin(SPACE_MD).show(
                ui,
                |ui| {
                    ui.set_width(max_width);
                    ui.set_max_height(max_height);
                    ui.horizontal(|ui| {
                        ui.heading(RichText::new(crate::tl!("Command console")).color(t.text));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            action_button(ui, "Close", "Close the advanced command console", t, true, || close = true);
                        });
                    });
                    if app.session.current_prompt().is_some() {
                        ui.label(RichText::new(app.session.prompt_text()).color(t.text_dim));
                    }
                    egui::ScrollArea::vertical().id_salt("ar_cad_mobile_console_history").max_height(160.0).show(ui, |ui| {
                        for entry in app.session.log.iter().rev().take(20).rev() {
                            ui.label(RichText::new(entry).size(CONTROL_TEXT_SIZE).color(t.text_dim));
                        }
                    });
                    let input = ui.add_sized(
                        [ui.available_width(), TOOL_HIT_TARGET.y],
                        egui::TextEdit::singleline(&mut state.console_input).hint_text(crate::tl!("Command, alias, or prompt value")),
                    );
                    if input.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)) {
                        submit = true;
                    }
                    if let Some(error) = &state.console_error {
                        ui.label(RichText::new(error).color(t.warn));
                    }
                    ui.horizontal_wrapped(|ui| {
                        action_button(ui, "Run", "Submit command or prompt value", t, true, || submit = true);
                        action_button(ui, "Close", "Close the advanced command console", t, true, || close = true);
                    });
                },
            );
        });

    if close {
        state.advanced_console_open = false;
        state.console_error = None;
    } else if submit {
        let input = state.console_input.trim().to_owned();
        if !input.trim().is_empty() {
            match app.cmdline(&input) {
                Some(error) => state.console_error = Some(error),
                None => {
                    state.console_input.clear();
                    state.console_error = None;
                }
            }
        }
    }
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

    let drawings: Vec<(usize, String, bool, bool)> =
        app.session.docs.iter().enumerate().map(|(index, doc)| (index, doc.title.clone(), doc.is_dirty(), index == app.session.active)).collect();
    egui::ScrollArea::horizontal().id_salt("ar_cad_mobile_open_drawings").show(ui, |ui| {
        ui.horizontal(|ui| {
            for (index, title, dirty, active) in drawings {
                let title = format!("{}{}", title, if dirty { " *" } else { "" });
                let response = ui.add_sized([180.0, TOOL_HIT_TARGET.y], egui::Button::selectable(active && !app.ui.start_tab, title));
                crate::control::record_widget(ui, &response, "Switch drawing", "button", Some(active));
                if response.clicked() {
                    match app.run("document.switch", serde_json::json!({"index": index})) {
                        Ok(_) => app.ui.start_tab = false,
                        Err(error) => app.session.echo(error.to_string()),
                    }
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
                egui::Button::new(RichText::new(label).color(if active { t.accent } else { t.text })).fill(if active {
                    t.accent.gamma_multiply(0.18)
                } else {
                    t.control
                }),
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
    let max_width = (ctx.content_rect().width() - SPACE_MD * 2.0).max(0.0);
    egui::Area::new(Id::unique("ar_cad_mobile_status_message"))
        .order(egui::Order::Foreground)
        .anchor(Align2::LEFT_TOP, vec2(12.0, APP_BAR_HEIGHT + 8.0))
        .show(ctx, |ui| {
            ui.set_max_width(max_width);
            egui::Frame::new()
                .fill(t.cmd_bg)
                .stroke(Stroke::new(BORDER_WIDTH, t.cmd_border))
                .corner_radius(CONTROL_RADIUS)
                .inner_margin(SPACE_SM)
                .show(ui, |ui| {
                    ui.add(egui::Label::new(RichText::new(message.as_str()).color(t.text).size(CONTROL_TEXT_SIZE)).wrap());
                });
        });
}

fn show_view_controls(app: &mut CadApp, ctx: &egui::Context, t: Tokens, layout: ResponsiveLayout) {
    let bottom_offset = layout.bottom_offset();
    egui::Area::new(Id::unique("ar_cad_mobile_view_controls"))
        .order(egui::Order::Foreground)
        .anchor(Align2::RIGHT_BOTTOM, vec2(-10.0, -bottom_offset))
        .show(ctx, |ui| {
            egui::Frame::new().fill(t.chrome).stroke(Stroke::new(BORDER_WIDTH, t.border)).corner_radius(CONTROL_RADIUS).inner_margin(SPACE_XS).show(
                ui,
                |ui| {
                    ui.vertical(|ui| {
                        action_button(ui, "+", "Zoom in", t, true, || zoom(app, 0.8));
                        action_button(ui, "−", "Zoom out", t, true, || zoom(app, 1.25));
                        action_button(ui, "Fit", "Fit drawing", t, true, || app.start("zoom.extents"));
                    });
                },
            );
        });
}

fn show_command_card(app: &mut CadApp, ctx: &egui::Context, t: Tokens, layout: ResponsiveLayout) {
    let Some(prompt) = app.session.current_prompt() else {
        return;
    };
    let command_label = app.session.running.as_ref().map(|command| command.id.to_ascii_uppercase());
    let screen = ctx.content_rect();
    let panel_width = screen.width().min(TOOL_SHEET_MAX_WIDTH);
    let max_width = (panel_width - SPACE_MD * 2.0 - BORDER_WIDTH * 2.0).max(0.0);
    let bottom_offset = layout.bottom_offset();
    let available_height = (screen.height() - APP_BAR_HEIGHT - bottom_offset - SPACE_MD * 2.0).max(0.0);
    let panel_height = (screen.height() * 0.48).min(TOOL_PANEL_MAX_HEIGHT).min(available_height);
    let max_height = (panel_height - SPACE_MD * 2.0 - BORDER_WIDTH * 2.0).max(0.0);
    let mut submit = false;
    let mut confirm = false;
    let mut cancel = false;
    let mut keyword = None;

    egui::Area::new(Id::unique("ar_cad_mobile_command_card"))
        .order(egui::Order::Foreground)
        .anchor(Align2::CENTER_BOTTOM, vec2(0.0, -bottom_offset))
        .show(ctx, |ui| {
            egui::Frame::new().fill(t.panel).stroke(Stroke::new(BORDER_WIDTH, t.border)).corner_radius(PANEL_RADIUS).inner_margin(SPACE_MD).show(
                ui,
                |ui| {
                    ui.set_width(max_width);
                    ui.set_max_height(max_height);
                    let scroll_height = (ui.available_height() - TOOL_HIT_TARGET.y - SPACE_SM).max(0.0);
                    egui::ScrollArea::vertical().id_salt("ar_cad_mobile_command_content").max_height(scroll_height).show(ui, |ui| {
                        if let Some(command) = &command_label {
                            ui.label(RichText::new(command).strong().color(t.accent).size(CONTROL_TEXT_SIZE));
                        }
                        ui.label(RichText::new(app.session.prompt_text()).strong().color(t.text).size(BODY_TEXT_SIZE));
                        if prompt.accept.point {
                            ui.label(
                                RichText::new(crate::tl!("Tap the drawing or enter exact coordinates")).color(t.text_dim).size(CONTROL_TEXT_SIZE),
                            );
                        } else if prompt.accept.number {
                            ui.label(RichText::new(crate::tl!("Enter an exact number")).color(t.text_dim).size(CONTROL_TEXT_SIZE));
                        }
                        let input = ui.add_sized(
                            [ui.available_width(), TOOL_HIT_TARGET.y],
                            egui::TextEdit::singleline(&mut app.cmd.buffer).hint_text(if prompt.accept.point {
                                crate::tl!("Exact value or coordinate")
                            } else if prompt.accept.number {
                                crate::tl!("Enter a number")
                            } else {
                                crate::tl!("Enter a value")
                            }),
                        );
                        if input.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)) {
                            submit = !app.cmd.buffer.trim().is_empty();
                            confirm = app.cmd.buffer.trim().is_empty() && prompt.accept.enter;
                        }
                        if !prompt.keywords.is_empty() {
                            let keyword_width = ui.available_width().min(TOOL_HIT_TARGET.x * 2.0).max(TOOL_HIT_TARGET.x);
                            ui.horizontal_wrapped(|ui| {
                                for option in &prompt.keywords {
                                    let response = ui.add_sized([keyword_width, TOOL_HIT_TARGET.y], egui::Button::new(option));
                                    if response.clicked() {
                                        keyword = Some(option.clone());
                                    }
                                }
                            });
                        }
                    });
                    ui.horizontal_wrapped(|ui| {
                        if !app.cmd.buffer.trim().is_empty() {
                            action_button(ui, "Apply", "Submit the entered value", t, true, || submit = true);
                        }
                        if prompt.accept.enter && app.cmd.buffer.trim().is_empty() {
                            action_button(ui, "Confirm", "Accept the current prompt", t, true, || confirm = true);
                        }
                        action_button(ui, "Cancel", "Cancel the active command", t, true, || cancel = true);
                    });
                },
            );
        });

    if cancel {
        app.cmd.buffer.clear();
        app.session.cancel();
    } else if let Some(keyword) = keyword {
        app.cmdline(&keyword);
    } else if submit {
        let value = std::mem::take(&mut app.cmd.buffer);
        app.cmdline(&value);
    } else if confirm {
        app.cmdline("");
    }
}

fn tool_sheet_columns(width: f32) -> usize {
    if width >= TOOL_SHEET_THREE_COLUMN_BREAKPOINT {
        3
    } else if width >= TOOL_SHEET_TWO_COLUMN_BREAKPOINT {
        2
    } else {
        1
    }
}

fn command_button(
    ui: &mut egui::Ui,
    app: &mut CadApp,
    command: &Value,
    state: &mut WorkspaceState,
    t: Tokens,
    button_width: f32,
    allow_favorite: bool,
) -> bool {
    let Some(id) = command.get("id").and_then(Value::as_str) else {
        return false;
    };
    let label = command.get("label").and_then(Value::as_str).unwrap_or(id);
    let engine_enabled = command.get("enabled").and_then(Value::as_bool).unwrap_or(false);
    let params = command.get("params").and_then(Value::as_str).unwrap_or_default();
    let param_contract = params.split_whitespace().next().unwrap_or_default();
    let interactive = command.get("interactive").and_then(Value::as_bool).unwrap_or(false);
    let requires_arguments = requires_arguments_for_start(id, param_contract, interactive);
    let enabled = engine_enabled;
    let reason = if !engine_enabled { command.get("disabledReason").and_then(Value::as_str).unwrap_or("Command metadata unavailable") } else { "" };
    let shortcut = command.get("shortcut").and_then(Value::as_str);
    let caption = match shortcut {
        Some(key) => format!("{label}  ·  {key}"),
        None => label.to_owned(),
    };
    let command_width = if allow_favorite { (button_width - TOOL_HIT_TARGET.x - SPACE_XS).max(TOOL_HIT_TARGET.x) } else { button_width };
    let selected = ui
        .horizontal(|ui| {
            let response = ui
                .add_enabled_ui(enabled, |ui| {
                    ui.add_sized([command_width, TOOL_HIT_TARGET.y], egui::Button::new(RichText::new(caption).size(CONTROL_TEXT_SIZE).color(t.text)))
                })
                .inner;
            let response = response.on_hover_text(crate::i18n::t(label));
            crate::control::record_widget(ui, &response, crate::i18n::t(label), "button", None);
            let response = if enabled { response } else { response.on_disabled_hover_text(crate::i18n::t(reason)) };
            if allow_favorite {
                let favorite = state.favorite_commands.iter().any(|favorite| favorite == id);
                let description = if favorite { crate::tl!("Remove from favorites") } else { crate::tl!("Add to favorites") };
                let response = ui
                    .add_sized([TOOL_HIT_TARGET.x, TOOL_HIT_TARGET.y], egui::Button::new(if favorite { "★" } else { "☆" }))
                    .on_hover_text(description);
                crate::control::record_widget(ui, &response, description, "button", Some(favorite));
                if response.clicked() {
                    toggle_favorite(state, id);
                }
            }
            response.clicked()
        })
        .inner;
    if selected {
        if requires_arguments {
            state.parameter_command = Some(id.to_owned());
            state.parameter_json = "{}".to_owned();
            state.parameter_error = None;
        } else {
            app.start(id);
        }
        return true;
    }
    false
}

fn show_parameter_sheet(app: &mut CadApp, ctx: &egui::Context, state: &mut WorkspaceState, t: Tokens, layout: ResponsiveLayout) {
    let Some(command_id) = state.parameter_command.clone() else {
        return;
    };
    let registry = crate::control::all_commands(app);
    let command = registry
        .as_array()
        .and_then(|commands| commands.iter().find(|command| command.get("id").and_then(Value::as_str) == Some(command_id.as_str())));
    let label = command.and_then(|command| command.get("label").and_then(Value::as_str)).unwrap_or(&command_id);
    let parameter_help = command.and_then(|command| command.get("params").and_then(Value::as_str)).unwrap_or_default();
    let screen = ctx.content_rect();
    let max_width = (screen.width().min(layout.tool_sheet_max_width()) - SPACE_MD * 2.0 - BORDER_WIDTH * 2.0).max(0.0);
    let bottom_offset = layout.bottom_offset();
    let available_height = (screen.height() - APP_BAR_HEIGHT - bottom_offset - SPACE_MD * 2.0).max(0.0);
    let max_height =
        ((screen.height() * 0.62).min(TOOL_PANEL_MAX_HEIGHT + 120.0).min(available_height) - SPACE_MD * 2.0 - BORDER_WIDTH * 2.0).max(0.0);
    let mut execute = false;
    let mut cancel = false;

    egui::Area::new(Id::unique("ar_cad_mobile_parameter_sheet"))
        .order(egui::Order::Foreground)
        .anchor(Align2::CENTER_BOTTOM, vec2(0.0, -bottom_offset))
        .show(ctx, |ui| {
            egui::Frame::new().fill(t.panel).stroke(Stroke::new(BORDER_WIDTH, t.border)).corner_radius(PANEL_RADIUS).inner_margin(SPACE_MD).show(
                ui,
                |ui| {
                    ui.set_width(max_width);
                    ui.set_max_height(max_height);
                    ui.horizontal(|ui| {
                        ui.heading(RichText::new(label).color(t.text));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            action_button(ui, "Close", "Close parameter entry", t, true, || cancel = true);
                        });
                    });
                    ui.label(RichText::new(crate::tl!("Enter this command's parameters as a JSON object.")).color(t.text_dim));
                    if !parameter_help.is_empty() {
                        egui::ScrollArea::vertical().id_salt("ar_cad_mobile_parameter_help").max_height(88.0).show(ui, |ui| {
                            ui.label(RichText::new(parameter_help).size(CONTROL_TEXT_SIZE).color(t.text_dim));
                        });
                    }
                    let input = ui.add_sized(
                        [ui.available_width(), 112.0],
                        egui::TextEdit::multiline(&mut state.parameter_json).code_editor().desired_rows(4).hint_text("{\"key\": value}"),
                    );
                    if input.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter) && input.modifiers.command) {
                        execute = true;
                    }
                    if let Some(error) = &state.parameter_error {
                        ui.label(RichText::new(error).color(t.warn));
                    }
                    ui.horizontal_wrapped(|ui| {
                        action_button(ui, "Run", "Run the command with these parameters", t, true, || execute = true);
                        action_button(ui, "Cancel", "Cancel parameter entry", t, true, || cancel = true);
                    });
                },
            );
        });

    if cancel {
        state.parameter_command = None;
        state.parameter_error = None;
        return;
    }
    if execute {
        match parse_command_params(&state.parameter_json) {
            Ok(params) => match app.run(&command_id, params) {
                Ok(_) => {
                    remember_command(state, &command_id);
                    state.parameter_command = None;
                    state.parameter_error = None;
                }
                Err(error) => state.parameter_error = Some(error),
            },
            Err(error) => state.parameter_error = Some(error.to_string()),
        }
    }
}

fn parse_command_params(input: &str) -> Result<Value, String> {
    let params: Value = serde_json::from_str(input).map_err(|error| error.to_string())?;
    if !params.is_object() {
        return Err(crate::tl!("Parameters must be a JSON object.").to_owned());
    }
    Ok(params)
}

fn remember_command(state: &mut WorkspaceState, id: &str) {
    state.recent_commands.retain(|recent| recent != id);
    state.recent_commands.insert(0, id.to_owned());
    state.recent_commands.truncate(RECENT_COMMAND_LIMIT);
    state.command_lists_changed = true;
}

fn toggle_favorite(state: &mut WorkspaceState, id: &str) {
    if let Some(index) = state.favorite_commands.iter().position(|favorite| favorite == id) {
        state.favorite_commands.remove(index);
    } else {
        state.favorite_commands.push(id.to_owned());
    }
    state.command_lists_changed = true;
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

fn action_button(ui: &mut egui::Ui, label: &str, description: &str, t: Tokens, enabled: bool, action: impl FnOnce()) {
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
    let root = command.get("menu").and_then(Value::as_array).and_then(|path| path.first()).and_then(Value::as_str).unwrap_or_default();
    match category {
        Category::Select => {
            root.eq_ignore_ascii_case("select")
                || matches!(id.as_str(), "select" | "selectall" | "ai_deselect" | "qselect" | "ui.dialog.qselect")
                || id.starts_with("select.")
        }
        Category::Draw => {
            root.eq_ignore_ascii_case("draw")
                || matches!(
                    id.as_str(),
                    "line" | "pline" | "circle" | "rectang" | "arc" | "ellipse" | "spline" | "polygon" | "text" | "mtext" | "hatch"
                )
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
    use super::{
        Category, LayoutClass, ResponsiveLayout, WorkspaceState, matches_category, matches_search, remember_command, requires_arguments_for_start,
        tool_sheet_columns,
    };
    use serde_json::json;

    #[test]
    fn responsive_layout_uses_compact_medium_and_expanded_classes() {
        assert_eq!(ResponsiveLayout::new(egui::vec2(400.0, 800.0), false).class, LayoutClass::Compact);
        assert_eq!(ResponsiveLayout::new(egui::vec2(600.0, 400.0), false).class, LayoutClass::Medium);
        assert_eq!(ResponsiveLayout::new(egui::vec2(900.0, 600.0), false).class, LayoutClass::Expanded);
        assert_eq!(ResponsiveLayout::new(egui::vec2(900.0, 350.0), false).class, LayoutClass::Medium);
    }

    #[test]
    fn focused_keyboard_repositions_sheets_without_phone_navigation_reservation() {
        let layout = ResponsiveLayout::new(egui::vec2(400.0, 800.0), true);
        assert!(layout.keyboard_focused);
        assert_eq!(layout.bottom_offset(), super::SPACE_MD);
        assert_eq!(ResponsiveLayout::new(egui::vec2(400.0, 800.0), false).bottom_offset(), super::NAV_BAR_HEIGHT + super::SPACE_SM);
    }

    #[test]
    fn tool_sheet_uses_one_column_when_two_touch_targets_would_not_fit() {
        assert_eq!(tool_sheet_columns(420.0), 1);
        assert_eq!(tool_sheet_columns(460.0), 2);
        assert_eq!(tool_sheet_columns(540.0), 3);
    }

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

    #[test]
    fn parameter_sheet_accepts_objects_and_rejects_invalid_json_shapes() {
        assert_eq!(super::parse_command_params(r#"{"radius": 4.5}"#).unwrap(), json!({"radius": 4.5}));
        assert!(super::parse_command_params("{").is_err());
        assert!(super::parse_command_params("[1, 2]").is_err());
        assert!(super::parse_command_params("null").is_err());
    }

    #[test]
    fn recent_tools_are_unique_newest_first_and_bounded() {
        let mut state = WorkspaceState::default();
        for id in ["line", "circle", "move", "copy", "erase", "offset", "fillet"] {
            remember_command(&mut state, id);
        }
        assert_eq!(state.recent_commands, vec!["fillet".to_owned(), "offset".to_owned(), "erase".to_owned(), "copy".to_owned(), "move".to_owned()]);

        remember_command(&mut state, "copy");
        assert_eq!(state.recent_commands, vec!["copy".to_owned(), "fillet".to_owned(), "offset".to_owned(), "erase".to_owned(), "move".to_owned()]);
        assert_eq!(state.recent_commands.len(), super::RECENT_COMMAND_LIMIT);
        assert!(state.command_lists_changed);
    }

    #[test]
    fn favorite_tools_toggle_without_duplicates() {
        let mut state = WorkspaceState::default();
        super::toggle_favorite(&mut state, "fillet");
        super::toggle_favorite(&mut state, "offset");
        super::toggle_favorite(&mut state, "fillet");
        assert_eq!(state.favorite_commands, vec!["offset".to_owned()]);
        assert!(state.command_lists_changed);
    }
}
