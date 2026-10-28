#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use balatro_workshop::{
    discovery,
    document::{self, Document},
    fields,
    lua::{self, Key, Value},
};
use eframe::egui::{self, Color32, RichText};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

const GOLD: Color32 = Color32::from_rgb(239, 186, 91);
const MUTED: Color32 = Color32::from_rgb(154, 165, 177);
const RED: Color32 = Color32::from_rgb(255, 139, 131);

mod edits;
mod launch;

#[cfg(test)]
mod ui_tests;

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Quick,
    Tree,
    Raw,
}
enum Pending {
    Open(PathBuf),
    Quit,
    Revert,
}

struct Workshop {
    document: Option<Document>,
    files: Vec<PathBuf>,
    tab: Tab,
    search: String,
    raw: String,
    raw_base: String,
    status: String,
    error: Option<String>,
    pending: Option<Pending>,
    allow_close: bool,
    started: Instant,
    smoke: bool,
    edits: edits::ScalarEdits,
    document_id: u64,
    launcher: launch::SteamLauncher,
    play_prompt: bool,
}

impl Workshop {
    fn configure_style(ctx: &egui::Context) {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Color32::from_rgb(23, 29, 36);
        visuals.window_fill = Color32::from_rgb(31, 38, 47);
        visuals.extreme_bg_color = Color32::from_rgb(17, 22, 28);
        visuals.selection.bg_fill = Color32::from_rgb(102, 76, 36);
        visuals.selection.stroke.color = GOLD;
        ctx.set_visuals(visuals);
        ctx.style_mut(|style| {
            style.spacing.item_spacing = egui::vec2(12.0, 10.0);
            style.spacing.button_padding = egui::vec2(14.0, 8.0);
            style
                .text_styles
                .insert(egui::TextStyle::Body, egui::FontId::proportional(16.0));
            style
                .text_styles
                .insert(egui::TextStyle::Button, egui::FontId::proportional(15.0));
        });
    }

    fn new(cc: &eframe::CreationContext<'_>, path: Option<PathBuf>, smoke: bool) -> Self {
        Self::configure_style(&cc.egui_ctx);
        let mut app = Self {
            document: None,
            files: discovery::discover(),
            tab: Tab::Quick,
            search: String::new(),
            raw: String::new(),
            raw_base: String::new(),
            status: "Ready. Your files stay on this computer.".into(),
            error: None,
            pending: None,
            allow_close: false,
            started: Instant::now(),
            smoke,
            edits: edits::ScalarEdits::default(),
            document_id: 0,
            launcher: launch::SteamLauncher::default(),
            play_prompt: false,
        };
        if let Some(path) = path {
            app.open(path);
        }
        app
    }

    fn dirty(&self) -> bool {
        self.edits.dirty()
            || self.raw != self.raw_base
            || self.document.as_ref().is_some_and(Document::dirty)
    }
    fn sync_raw(&mut self) {
        self.raw = self
            .document
            .as_ref()
            .map(|d| lua::serialize(&d.data))
            .unwrap_or_default();
        self.raw_base = self.raw.clone();
    }
    fn open(&mut self, path: PathBuf) {
        match Document::open(&path) {
            Ok(doc) => {
                self.edits.clear();
                self.document_id = self.document_id.wrapping_add(1);
                self.document = Some(doc);
                self.sync_raw();
                self.tab = Tab::Quick;
                self.search.clear();
                self.error = None;
                self.status = "Save loaded. Close Balatro before writing changes.".into();
            }
            Err(e) => self.error = Some(e),
        }
    }
    fn request(&mut self, action: Pending, ctx: &egui::Context) {
        if self.dirty() {
            self.pending = Some(action);
        } else {
            self.execute(action, ctx);
        }
    }
    fn execute(&mut self, action: Pending, ctx: &egui::Context) {
        match action {
            Pending::Open(path) => self.open(path),
            Pending::Quit => {
                self.allow_close = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            Pending::Revert => {
                self.edits.clear();
                self.document_id = self.document_id.wrapping_add(1);
                if let Some(doc) = &mut self.document {
                    doc.revert();
                }
                self.sync_raw();
                self.error = None;
                self.status = "Edits discarded. Restored the last loaded or saved state.".into();
            }
        }
    }
    fn pick_open(&mut self, ctx: &egui::Context) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Open a Balatro save or backup")
            .pick_file()
        {
            self.request(Pending::Open(path), ctx);
        }
    }
    fn commit_edits(&mut self) -> bool {
        if let Some(doc) = &mut self.document {
            if let Err(error) = self.edits.commit(&mut doc.data) {
                self.error = Some(error);
                return false;
            }
        }
        true
    }

    fn save(&mut self, copy: bool) -> bool {
        if self.raw != self.raw_base {
            self.error = Some("Apply or discard the advanced text changes before saving.".into());
            return false;
        }
        if !self.commit_edits() {
            return false;
        }
        let Some(doc) = &mut self.document else {
            return false;
        };
        if copy {
            let name = format!(
                "{}-edited.jkr",
                doc.path.file_stem().unwrap_or_default().to_string_lossy()
            );
            let dialog = rfd::FileDialog::new()
                .set_title("Save a new copy")
                .add_filter("Balatro save", &["jkr"])
                .set_file_name(name);
            let dialog = if let Some(parent) = doc.path.parent() {
                dialog.set_directory(parent)
            } else {
                dialog
            };
            if let Some(path) = dialog.save_file() {
                match doc.save_copy(&path) {
                    Ok(()) => {
                        self.error = None;
                        self.status = format!("Saved new copy: {}", doc.path.display());
                        self.sync_raw();
                        return true;
                    }
                    Err(e) => self.error = Some(e),
                }
            }
            false
        } else {
            match doc.save() {
                Ok(backup) => {
                    self.status = format!("Saved. Original backed up to {}", backup.display());
                    self.error = None;
                    self.sync_raw();
                    true
                }
                Err(e) => {
                    self.error = Some(e);
                    false
                }
            }
        }
    }

    fn request_play(&mut self, ctx: &egui::Context) {
        if self.dirty() {
            self.play_prompt = true;
        } else {
            self.start_play(ctx);
        }
    }

    fn start_play(&mut self, ctx: &egui::Context) {
        self.play_prompt = false;
        self.error = None;
        self.status = "Asking Steam to launch Balatro…".into();
        self.launcher.start(ctx);
    }

    fn save_and_play(&mut self, ctx: &egui::Context) {
        self.play_prompt = false;
        if self.save(false) {
            self.start_play(ctx);
        }
    }
    fn sidebar(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("sidebar").exact_width(245.0).resizable(false).show(ctx, |ui| {
            ui.add_space(18.0);
            ui.label(RichText::new("BW / 01").color(GOLD).size(15.0).strong());
            ui.heading("Balatro"); ui.heading("Workshop");
            ui.label(RichText::new("LOCAL SAVE EDITOR").small().color(MUTED));
            ui.add_space(22.0);
            if ui.add_enabled(!self.launcher.busy(), egui::Button::new("▶  Play Balatro").min_size(egui::vec2(210.0, 40.0)).fill(Color32::from_rgb(47, 88, 61))).on_hover_text("Launch Balatro through Steam").clicked() {
                self.request_play(ctx);
            }
            if ui.add_sized([210.0, 40.0], egui::Button::new("Open save…").fill(Color32::from_rgb(85, 65, 36))).clicked() { self.pick_open(ctx); }
            if ui.add_sized([210.0, 36.0], egui::Button::new("Browse folder…")).clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.files = discovery::scan(&path);
                    self.status = format!("Found {} save files in {}", self.files.len(), path.display());
                }
            }
            ui.add_space(12.0); ui.separator();
            ui.horizontal(|ui| {
                ui.label(RichText::new("DETECTED SAVES").small().color(MUTED));
                if ui.small_button("Rescan").clicked() { self.files = discovery::discover(); }
            });
            if self.files.is_empty() {
                ui.label(RichText::new("No saves found automatically. Open a file or browse your Balatro folder.").color(MUTED));
            }
            let mut selected = None;
            egui::ScrollArea::vertical().max_height((ui.available_height() - 100.0).max(80.0)).show(ui, |ui| {
                for (i, path) in self.files.iter().enumerate() {
                    ui.push_id(i, |ui| {
                        let name = path.file_name().unwrap_or_default().to_string_lossy();
                        let profile = path.parent().and_then(|p| p.file_name()).unwrap_or_default().to_string_lossy();
                        let active = self.document.as_ref().is_some_and(|d| &d.path == path);
                        if ui.selectable_label(active, format!("{name}\n{profile}")).on_hover_text(path.display().to_string()).clicked() { selected = Some(path.clone()); }
                    });
                }
            });
            if let Some(path) = selected { self.request(Pending::Open(path), ctx); }
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.label(RichText::new(concat!("v", env!("CARGO_PKG_VERSION"), "  ·  Rust + egui")).small().color(MUTED));
                ui.label(RichText::new("Offline. No account required.").small().color(MUTED));
            });
        });
    }

    fn editor(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if self.document.is_none() {
            ui.add_space(75.0);
            ui.label(
                RichText::new("YOUR NEXT RUN,\nON YOUR TERMS.")
                    .size(36.0)
                    .strong(),
            );
            ui.add_space(14.0);
            ui.label(
                RichText::new("Inspect and edit Balatro saves from one desktop app.")
                    .size(19.0)
                    .color(MUTED),
            );
            ui.add_space(24.0);
            for (title, body) in [
                ("01   Open your save", "Use save.jkr for a run, profile.jkr for stats, or settings.jkr for preferences."),
                ("02   Make your changes", "Quick controls for common values. A data tree and Lua table editor for everything else."),
                ("03   Save with a backup", "Every overwrite keeps the original beside your save. Save copy leaves the source untouched."),
            ] {
                egui::Frame::group(ui.style()).inner_margin(16.0).show(ui, |ui| {
                    ui.set_width(ui.available_width()); ui.label(RichText::new(title).color(GOLD).strong()); ui.label(body);
                }); ui.add_space(8.0);
            }
            ui.add_space(12.0);
            if ui.button("Choose a .jkr file…").clicked() {
                self.pick_open(ctx);
            }
            return;
        }
        let doc = self.document.as_ref().unwrap();
        let filename = doc
            .path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let kind = fields::kind(&doc.data);
        let count = doc.data.node_count();
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.heading(RichText::new(filename).size(28.0));
            ui.label(
                RichText::new(if self.dirty() {
                    "  UNSAVED CHANGES"
                } else {
                    "  SAVED"
                })
                .small()
                .color(if self.dirty() { GOLD } else { MUTED }),
            );
        });
        ui.label(
            RichText::new(doc.path.display().to_string())
                .small()
                .color(MUTED),
        );
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new(kind).color(GOLD).strong());
            ui.label(
                RichText::new(format!("{count} values  /  raw DEFLATE"))
                    .small()
                    .color(MUTED),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Save copy…").clicked() {
                    self.save(true);
                }
                if ui
                    .add_enabled(
                        self.raw == self.raw_base,
                        egui::Button::new("Save + backup").fill(Color32::from_rgb(85, 65, 36)),
                    )
                    .clicked()
                {
                    self.save(false);
                }
                if ui
                    .add_enabled(self.dirty(), egui::Button::new("Revert"))
                    .clicked()
                {
                    self.request(Pending::Revert, ctx);
                }
            });
        });
        ui.add_space(8.0);
        ui.label(RichText::new("Close Balatro before saving. Steam Cloud can replace local changes when the game syncs.").small().color(GOLD));
        ui.separator();
        let previous = self.tab;
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.tab, Tab::Quick, "Quick edit");
            ui.selectable_value(&mut self.tab, Tab::Tree, "Data tree");
            ui.selectable_value(&mut self.tab, Tab::Raw, "Advanced text");
        });
        if self.tab != previous && !self.commit_edits() {
            self.tab = previous;
            return;
        }
        if self.tab == Tab::Raw && previous != Tab::Raw && self.raw == self.raw_base {
            self.sync_raw();
        }
        if self.tab != Tab::Raw && self.raw != self.raw_base {
            ui.colored_label(GOLD, "Advanced text has unapplied changes. Apply or discard them to continue editing here.");
            return;
        }
        ui.add_space(6.0);
        match self.tab {
            Tab::Quick => {
                ui.add(
                    egui::TextEdit::singleline(&mut self.search)
                        .id_salt("quick-filter")
                        .hint_text("Filter settings…")
                        .desired_width(f32::INFINITY),
                );
                let filter = self.search.to_lowercase();
                let doc = self.document.as_mut().unwrap();
                egui::ScrollArea::vertical().auto_shrink([false, false]).id_salt("quick-scroll").show(ui, |ui| {
                    let mut group = ""; let mut shown = 0;
                    for &(section, label, path) in fields::FIELDS {
                        if !filter.is_empty() && !format!("{label} {path}").to_lowercase().contains(&filter) { continue; }
                        let Some(indices) = edits::named_path(&doc.data, path) else { continue; };
                        let Some(value) = doc.data.get_mut(path) else { continue; };
                        if matches!(value, Value::Table(_)) { continue; }
                        if section != group { ui.add_space(14.0); ui.label(RichText::new(section.to_uppercase()).small().strong().color(GOLD)); group = section; }
                        ui.push_id(path, |ui| {
                            ui.horizontal(|ui| {
                                ui.allocate_ui_with_layout(egui::vec2(280.0, 30.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                    ui.set_min_width(280.0);
                                    ui.label(label).on_hover_text(path);
                                });
                                self.edits.show(ui, value, &indices, label, self.document_id);
                            });
                        });
                        shown += 1;
                    }
                    if shown == 0 { ui.add_space(20.0); ui.label("No matching quick controls. Use Data tree or Advanced text to inspect this file."); }
                    ui.add_space(20.0);
                    ui.label(RichText::new("Only existing fields are shown. Slot counts and other run limits may need a new round to take effect.").small().color(MUTED));
                });
            }
            Tab::Tree => {
                ui.label(RichText::new("Expand tables to edit values. Numeric keys stay numeric; unknown fields are preserved.").small().color(MUTED));
                let doc = self.document.as_mut().unwrap();
                egui::ScrollArea::both()
                    .id_salt("tree-scroll")
                    .show(ui, |ui| {
                        tree(
                            ui,
                            &mut doc.data,
                            &mut self.edits,
                            self.document_id,
                            &mut Vec::new(),
                        );
                    });
            }
            Tab::Raw => {
                ui.label(RichText::new("Data-only Lua table. No Lua code runs. Edit keys or values, then Apply text before saving.").small().color(MUTED));
                ui.horizontal(|ui| {
                    if ui.button("Apply text").clicked() {
                        match lua::parse(self.raw.as_bytes()) {
                            Ok(data) => {
                                self.edits.clear();
                                self.document_id = self.document_id.wrapping_add(1);
                                self.document.as_mut().unwrap().data = data;
                                self.sync_raw();
                                self.error = None;
                                self.status = "Advanced edits applied. Save when ready.".into();
                            }
                            Err(e) => self.error = Some(e),
                        }
                    }
                    if ui.button("Discard text edits").clicked() {
                        self.sync_raw();
                    }
                    if ui.button("Export .lua…").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .set_file_name("balatro-save.lua")
                            .save_file()
                        {
                            match document::create_file(&path, self.raw.as_bytes()) {
                                Ok(()) => {
                                    self.status = format!("Exported text to {}", path.display())
                                }
                                Err(e) => self.error = Some(e),
                            }
                        }
                    }
                });
                egui::ScrollArea::both()
                    .id_salt("raw-scroll")
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.raw)
                                .id_salt("advanced-text")
                                .code_editor()
                                .desired_width(f32::INFINITY)
                                .desired_rows(30),
                        );
                    });
            }
        }
    }
}

fn tree(
    ui: &mut egui::Ui,
    value: &mut Value,
    edits: &mut edits::ScalarEdits,
    document_id: u64,
    path: &mut Vec<usize>,
) {
    if let Value::Table(entries) = value {
        for (index, (key, value)) in entries.iter_mut().enumerate() {
            path.push(index);
            let label = match key {
                Key::String(bytes) => format!("\"{}\"", String::from_utf8_lossy(bytes)),
                Key::Number(n) => format!("[{n}]"),
                Key::Bool(b) => format!("[{b}]"),
            };
            ui.push_id(index, |ui| {
                if let Value::Table(children) = value {
                    egui::CollapsingHeader::new(format!("{label}  ·  {} entries", children.len()))
                        .id_salt("table")
                        .show(ui, |ui| tree(ui, value, edits, document_id, path));
                } else {
                    ui.horizontal(|ui| {
                        ui.label(&label);
                        edits.show(ui, value, path, &label, document_id);
                    });
                }
            });
            path.pop();
        }
    }
}

impl Workshop {
    fn show(&mut self, ctx: &egui::Context) {
        if let Some(result) = self.launcher.poll(ctx) {
            match result {
                Ok(()) => self.status = "Launch request sent to Steam.".into(),
                Err(error) => self.error = Some(error),
            }
        }
        if self.smoke && self.started.elapsed() > Duration::from_secs(6) {
            self.allow_close = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if self.smoke {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
        if ctx.input(|i| i.viewport().close_requested()) && !self.allow_close && self.dirty() {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.pending = Some(Pending::Quit);
        }
        if self.pending.is_none() && !self.play_prompt {
            let copy = ctx.input_mut(|i| {
                i.consume_shortcut(&egui::KeyboardShortcut::new(
                    egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                    egui::Key::S,
                ))
            });
            if copy {
                self.save(true);
            } else if ctx.input_mut(|i| {
                i.consume_shortcut(&egui::KeyboardShortcut::new(
                    egui::Modifiers::COMMAND,
                    egui::Key::S,
                ))
            }) {
                self.save(false);
            }
            if ctx.input_mut(|i| {
                i.consume_shortcut(&egui::KeyboardShortcut::new(
                    egui::Modifiers::COMMAND,
                    egui::Key::O,
                ))
            }) {
                self.pick_open(ctx);
            }
            let dropped = ctx.input(|i| i.raw.dropped_files.first().and_then(|f| f.path.clone()));
            if let Some(path) = dropped {
                self.request(Pending::Open(path), ctx);
            }
        }
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.label(RichText::new(&self.status).small().color(MUTED));
            ui.add_space(4.0);
        });
        self.sidebar(ctx);
        egui::CentralPanel::default().show(ctx, |ui| {
            self.editor(ui, ctx);
        });
        if let Some(message) = self.error.clone() {
            egui::Window::new("Could not complete action")
                .collapsible(false)
                .resizable(true)
                .default_width(520.0)
                .show(ctx, |ui| {
                    ui.colored_label(RED, message);
                    if ui.button("Close").clicked() {
                        self.error = None;
                    }
                });
        }
        if self.pending.is_some() {
            egui::Modal::new(egui::Id::new("unsaved")).show(ctx, |ui| {
                ui.heading("Discard unsaved changes?");
                ui.label("Your edits have not been written to disk. Choose Keep editing to save them first.");
                ui.horizontal(|ui| {
                    if ui.button("Keep editing").clicked() { self.pending = None; }
                    if ui.button("Discard changes").clicked() {
                        if let Some(action) = self.pending.take() { self.execute(action, ctx); }
                    }
                });
            });
        }
        if self.play_prompt {
            egui::Modal::new(egui::Id::new("play-unsaved")).show(ctx, |ui| {
                ui.heading("Save your changes before playing?");
                ui.label("Balatro loads the save on disk. Your edits are still in the editor.");
                if self.raw != self.raw_base {
                    ui.colored_label(
                        GOLD,
                        "Apply or discard Advanced text changes before saving.",
                    );
                }
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(self.raw == self.raw_base, egui::Button::new("Save & play"))
                        .clicked()
                    {
                        self.save_and_play(ctx);
                    }
                    if ui.button("Play saved version").clicked() {
                        self.start_play(ctx);
                    }
                    if ui.button("Cancel").clicked() {
                        self.play_prompt = false;
                    }
                });
            });
        }
    }
}

impl eframe::App for Workshop {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.show(ctx);
    }
}

fn main() -> eframe::Result {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let smoke = args.iter().any(|s| s == "--smoke-test");
    let path = args
        .into_iter()
        .find(|s| s != "--smoke-test")
        .map(PathBuf::from);
    eframe::run_native(
        concat!("Balatro Workshop ", env!("CARGO_PKG_VERSION")),
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1180.0, 820.0])
                .with_min_inner_size([950.0, 640.0]),
            ..Default::default()
        },
        Box::new(move |cc| Ok(Box::new(Workshop::new(cc, path, smoke)))),
    )
}
