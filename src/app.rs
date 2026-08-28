use std::collections::HashMap;
use std::path::PathBuf;

use eframe::egui::{
    self, Align2, Context, CornerRadius, Event, FontId, Id, Key, Order, Pos2,
    Rect, ScrollArea, Sense, Stroke, TextEdit, Ui, UiBuilder, Vec2, ViewportCommand,
};
use ironcalc::base::expressions::types::Area;
use ironcalc::base::expressions::utils::number_to_column;

use crate::commands::Command;
use crate::engine::{Engine, MAX_COLS, MAX_ROWS};
use crate::keymap::Keymap;

const GUTTER_W: f32 = 48.0;
const HEADER_H: f32 = 24.0;
const CELL_FONT: f32 = 13.0;
const CELL_PAD: f32 = 4.0;

struct EditState {
    row: i32,
    col: i32,
    text: String,
    take_focus: bool,
    caret_end: bool,
    in_formula_bar: bool,
}

#[derive(Clone, PartialEq)]
enum Pending {
    New,
    Open,
    OpenPath(PathBuf),
    Quit,
}

#[derive(Clone, Copy, PartialEq)]
enum RibbonTab {
    Home,
    Sheet,
    Help,
}

pub struct SheetzApp {
    engine: Engine,
    keymap: Keymap,
    sheet: u32,
    cursor: (i32, i32), // (row, col), 1-based
    anchor: (i32, i32),
    saved_cursors: HashMap<u32, ((i32, i32), (i32, i32))>,
    edit: Option<EditState>,
    scroll_to_cursor: bool,
    // Grid geometry cache: boundary offsets in px. row_off[r] is the y where
    // row r+1 starts; row r (1-based) spans row_off[r-1]..row_off[r].
    row_off: Vec<f32>,
    col_off: Vec<f32>,
    view_rows: i32,
    view_cols: i32,
    geom_dirty: bool,
    rows_per_page: i32,
    pending: Option<Pending>,
    allow_close: bool,
    status: String,
    last_title: String,
    ribbon_tab: RibbonTab,
    backstage: bool,
    recent: Vec<PathBuf>,
}

fn recent_file() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config/sheetz/recent.txt"))
}

fn load_recent() -> Vec<PathBuf> {
    let Some(path) = recent_file() else {
        return Vec::new();
    };
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .map(PathBuf::from)
        .filter(|p| p.is_file())
        .take(10)
        .collect()
}

impl SheetzApp {
    pub fn new(path: Option<PathBuf>) -> SheetzApp {
        let (keymap, warnings) = Keymap::load();
        let mut status = warnings.join(" | ");
        let engine = match &path {
            Some(p) => Engine::open(p).unwrap_or_else(|e| {
                status = format!("Could not open {}: {e}", p.display());
                Engine::new().expect("empty workbook")
            }),
            None => Engine::new().expect("empty workbook"),
        };
        let mut app = SheetzApp {
            engine,
            keymap,
            sheet: 0,
            cursor: (1, 1),
            anchor: (1, 1),
            saved_cursors: HashMap::new(),
            edit: None,
            scroll_to_cursor: false,
            row_off: Vec::new(),
            col_off: Vec::new(),
            view_rows: 0,
            view_cols: 0,
            geom_dirty: true,
            rows_per_page: 30,
            pending: None,
            allow_close: false,
            status,
            last_title: String::new(),
            ribbon_tab: RibbonTab::Home,
            backstage: false,
            recent: load_recent(),
        };
        app.reset_view();
        app
    }

    // ----- geometry -----

    fn reset_view(&mut self) {
        let (ur, uc) = self.engine.used_range(self.sheet);
        self.view_rows = (ur + 50).clamp(200, MAX_ROWS);
        self.view_cols = (uc + 10).clamp(30, MAX_COLS);
        self.geom_dirty = true;
    }

    fn ensure_view_contains(&mut self, row: i32, col: i32) {
        if row > self.view_rows - 2 {
            self.view_rows = (row + 100).min(MAX_ROWS);
            self.geom_dirty = true;
        }
        if col > self.view_cols - 2 {
            self.view_cols = (col + 10).min(MAX_COLS);
            self.geom_dirty = true;
        }
    }

    fn ensure_geometry(&mut self) {
        if !self.geom_dirty
            && self.row_off.len() == self.view_rows as usize + 1
            && self.col_off.len() == self.view_cols as usize + 1
        {
            return;
        }
        self.row_off.clear();
        self.row_off.reserve(self.view_rows as usize + 1);
        self.row_off.push(0.0);
        let mut y = 0.0f32;
        for r in 1..=self.view_rows {
            y += self.engine.row_height(self.sheet, r);
            self.row_off.push(y);
        }
        self.col_off.clear();
        self.col_off.reserve(self.view_cols as usize + 1);
        self.col_off.push(0.0);
        let mut x = 0.0f32;
        for c in 1..=self.view_cols {
            x += self.engine.col_width(self.sheet, c);
            self.col_off.push(x);
        }
        self.geom_dirty = false;
    }

    fn cell_rect(&self, row: i32, col: i32, origin: Pos2) -> Rect {
        let r = row as usize;
        let c = col as usize;
        Rect::from_min_max(
            origin + Vec2::new(self.col_off[c - 1], self.row_off[r - 1]),
            origin + Vec2::new(self.col_off[c], self.row_off[r]),
        )
    }

    fn cell_at(&self, pos: Pos2, origin: Pos2) -> (i32, i32) {
        let p = pos - origin;
        let row = self.row_off.partition_point(|&y| y <= p.y).max(1) as i32;
        let col = self.col_off.partition_point(|&x| x <= p.x).max(1) as i32;
        (row.min(self.view_rows), col.min(self.view_cols))
    }

    fn selection(&self) -> (i32, i32, i32, i32) {
        (
            self.cursor.0.min(self.anchor.0),
            self.cursor.0.max(self.anchor.0),
            self.cursor.1.min(self.anchor.1),
            self.cursor.1.max(self.anchor.1),
        )
    }

    // ----- editing -----

    fn start_edit(&mut self, text: String, caret_end: bool) {
        self.anchor = self.cursor;
        self.edit = Some(EditState {
            row: self.cursor.0,
            col: self.cursor.1,
            text,
            take_focus: true,
            caret_end,
            in_formula_bar: false,
        });
    }

    fn commit_edit(&mut self, edit: &EditState) {
        if let Err(e) = self
            .engine
            .set_input(self.sheet, edit.row, edit.col, edit.text.trim_end())
        {
            self.status = format!("Error: {e}");
        }
    }

    // ----- commands -----

    fn move_cursor(&mut self, dr: i32, dc: i32, extend: bool) {
        self.cursor.0 = (self.cursor.0 + dr).clamp(1, MAX_ROWS);
        self.cursor.1 = (self.cursor.1 + dc).clamp(1, MAX_COLS);
        if !extend {
            self.anchor = self.cursor;
        }
        self.ensure_view_contains(self.cursor.0, self.cursor.1);
        self.scroll_to_cursor = true;
    }

    fn set_cursor(&mut self, row: i32, col: i32) {
        self.cursor = (row.clamp(1, MAX_ROWS), col.clamp(1, MAX_COLS));
        self.anchor = self.cursor;
        self.ensure_view_contains(self.cursor.0, self.cursor.1);
        self.scroll_to_cursor = true;
    }

    /// Excel Ctrl+Arrow: jump to the boundary of the current data region.
    fn jump(&mut self, dr: i32, dc: i32) {
        let (ur, uc) = self.engine.used_range(self.sheet);
        let (r, c) = self.cursor;
        let filled = |r: i32, c: i32| !self.engine.is_empty(self.sheet, r, c);
        // Scan limit in the positive direction: the used range; landing past it
        // means "edge of sheet" like Excel.
        let (target_r, target_c);
        if dr != 0 {
            let far = if dr > 0 { MAX_ROWS } else { 1 };
            let scan_end = if dr > 0 { ur } else { 1 };
            target_r = jump_axis(r, dr, far, scan_end, |i| filled(i, c));
            target_c = c;
        } else {
            let far = if dc > 0 { MAX_COLS } else { 1 };
            let scan_end = if dc > 0 { uc } else { 1 };
            target_c = jump_axis(c, dc, far, scan_end, |i| filled(r, i));
            target_r = r;
        }
        self.set_cursor(target_r, target_c);
    }

    fn clear_selection(&mut self) {
        let (r0, r1, c0, c1) = self.selection();
        for r in r0..=r1 {
            for c in c0..=c1 {
                if !self.engine.is_empty(self.sheet, r, c) {
                    let _ = self.engine.set_input(self.sheet, r, c, "");
                }
            }
        }
    }

    fn fill_down(&mut self) {
        let (r0, r1, c0, c1) = self.selection();
        let width = c1 - c0 + 1;
        let result = if r1 > r0 {
            self.engine.auto_fill_rows(
                Area {
                    sheet: self.sheet,
                    row: r0,
                    column: c0,
                    width,
                    height: 1,
                },
                r1,
            )
        } else if r0 > 1 {
            self.engine.auto_fill_rows(
                Area {
                    sheet: self.sheet,
                    row: r0 - 1,
                    column: c0,
                    width,
                    height: 1,
                },
                r0,
            )
        } else {
            Ok(())
        };
        if let Err(e) = result {
            self.status = format!("Fill down: {e}");
        }
    }

    fn fill_right(&mut self) {
        let (r0, r1, c0, c1) = self.selection();
        let height = r1 - r0 + 1;
        let result = if c1 > c0 {
            self.engine.auto_fill_columns(
                Area {
                    sheet: self.sheet,
                    row: r0,
                    column: c0,
                    width: 1,
                    height,
                },
                c1,
            )
        } else if c0 > 1 {
            self.engine.auto_fill_columns(
                Area {
                    sheet: self.sheet,
                    row: r0,
                    column: c0 - 1,
                    width: 1,
                    height,
                },
                c0,
            )
        } else {
            Ok(())
        };
        if let Err(e) = result {
            self.status = format!("Fill right: {e}");
        }
    }

    fn copy_selection(&self, ctx: &Context) {
        let (r0, r1, c0, c1) = self.selection();
        let mut out = String::new();
        for r in r0..=r1 {
            if r > r0 {
                out.push('\n');
            }
            for c in c0..=c1 {
                if c > c0 {
                    out.push('\t');
                }
                out.push_str(&self.engine.display(self.sheet, r, c));
            }
        }
        ctx.copy_text(out);
    }

    fn paste_tsv(&mut self, text: &str) {
        let (row, col) = self.cursor;
        let mut last = (row, col);
        for (i, line) in text.lines().enumerate() {
            for (j, field) in line.split('\t').enumerate() {
                let (r, c) = (row + i as i32, col + j as i32);
                if r > MAX_ROWS || c > MAX_COLS {
                    continue;
                }
                if let Err(e) = self.engine.set_input(self.sheet, r, c, field) {
                    self.status = format!("Paste: {e}");
                    return;
                }
                last = (r, c);
            }
        }
        self.anchor = self.cursor;
        self.cursor = last;
        std::mem::swap(&mut self.cursor, &mut self.anchor);
        self.ensure_view_contains(last.0, last.1);
    }

    fn switch_sheet(&mut self, sheet: u32) {
        let count = self.engine.sheet_names().len() as u32;
        let sheet = sheet.min(count.saturating_sub(1));
        if sheet == self.sheet {
            return;
        }
        self.saved_cursors
            .insert(self.sheet, (self.cursor, self.anchor));
        self.sheet = sheet;
        let (cursor, anchor) = self
            .saved_cursors
            .get(&sheet)
            .copied()
            .unwrap_or(((1, 1), (1, 1)));
        self.cursor = cursor;
        self.anchor = anchor;
        self.edit = None;
        self.reset_view();
        self.scroll_to_cursor = true;
    }

    // ----- file ops -----

    fn do_new(&mut self) {
        match Engine::new() {
            Ok(engine) => {
                self.engine = engine;
                self.sheet = 0;
                self.cursor = (1, 1);
                self.anchor = (1, 1);
                self.saved_cursors.clear();
                self.edit = None;
                self.reset_view();
                self.status.clear();
            }
            Err(e) => self.status = format!("New: {e}"),
        }
    }

    fn push_recent(&mut self, path: &std::path::Path) {
        let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        self.recent.retain(|p| *p != path);
        self.recent.insert(0, path);
        self.recent.truncate(10);
        if let Some(file) = recent_file() {
            if let Some(dir) = file.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            let text: String = self
                .recent
                .iter()
                .map(|p| format!("{}\n", p.display()))
                .collect();
            let _ = std::fs::write(file, text);
        }
    }

    fn open_path(&mut self, path: PathBuf) {
        match Engine::open(&path) {
            Ok(engine) => {
                self.engine = engine;
                self.sheet = 0;
                self.cursor = (1, 1);
                self.anchor = (1, 1);
                self.saved_cursors.clear();
                self.edit = None;
                self.reset_view();
                self.status = format!("Opened {}", path.display());
                self.push_recent(&path);
            }
            Err(e) => self.status = format!("Could not open {}: {e}", path.display()),
        }
    }

    fn do_open(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Excel workbook", &["xlsx"])
            .pick_file()
        {
            self.open_path(path);
        }
    }

    fn do_save(&mut self) -> bool {
        match self.engine.path.clone() {
            Some(path) => match self.engine.save(&path) {
                Ok(()) => {
                    self.status = format!("Saved {}", path.display());
                    self.push_recent(&path);
                    true
                }
                Err(e) => {
                    self.status = format!("Save failed: {e}");
                    false
                }
            },
            None => self.do_save_as(),
        }
    }

    fn do_save_as(&mut self) -> bool {
        let Some(mut path) = rfd::FileDialog::new()
            .add_filter("Excel workbook", &["xlsx"])
            .set_file_name("Book1.xlsx")
            .save_file()
        else {
            return false;
        };
        if path.extension().is_none() {
            path.set_extension("xlsx");
        }
        match self.engine.save(&path) {
            Ok(()) => {
                self.status = format!("Saved {}", path.display());
                self.push_recent(&path);
                true
            }
            Err(e) => {
                self.status = format!("Save failed: {e}");
                false
            }
        }
    }

    fn request(&mut self, action: Pending, ctx: &Context) {
        if self.engine.dirty {
            self.pending = Some(action);
        } else {
            self.run_pending(action, ctx);
        }
    }

    fn run_pending(&mut self, action: Pending, ctx: &Context) {
        match action {
            Pending::New => self.do_new(),
            Pending::Open => self.do_open(),
            Pending::OpenPath(path) => self.open_path(path),
            Pending::Quit => {
                self.allow_close = true;
                ctx.send_viewport_cmd(ViewportCommand::Close);
            }
        }
    }

    fn exec(&mut self, cmd: Command, ctx: &Context) {
        use Command::*;
        match cmd {
            MoveUp => self.move_cursor(-1, 0, false),
            MoveDown => self.move_cursor(1, 0, false),
            MoveLeft => self.move_cursor(0, -1, false),
            MoveRight => self.move_cursor(0, 1, false),
            ExtendUp => self.move_cursor(-1, 0, true),
            ExtendDown => self.move_cursor(1, 0, true),
            ExtendLeft => self.move_cursor(0, -1, true),
            ExtendRight => self.move_cursor(0, 1, true),
            JumpUp => self.jump(-1, 0),
            JumpDown => self.jump(1, 0),
            JumpLeft => self.jump(0, -1),
            JumpRight => self.jump(0, 1),
            PageUp => self.move_cursor(-self.rows_per_page.max(1), 0, false),
            PageDown => self.move_cursor(self.rows_per_page.max(1), 0, false),
            Home => self.set_cursor(self.cursor.0, 1),
            HomeAll => self.set_cursor(1, 1),
            EndData => {
                let (ur, uc) = self.engine.used_range(self.sheet);
                self.set_cursor(ur, uc);
            }
            EditCell => {
                let text = self.engine.content(self.sheet, self.cursor.0, self.cursor.1);
                self.start_edit(text, true);
            }
            Clear => self.clear_selection(),
            FillDown => self.fill_down(),
            FillRight => self.fill_right(),
            Undo => {
                self.engine.undo();
                self.geom_dirty = true;
            }
            Redo => {
                self.engine.redo();
                self.geom_dirty = true;
            }
            Copy => self.copy_selection(ctx),
            Cut => {
                self.copy_selection(ctx);
                self.clear_selection();
            }
            Paste => {
                match arboard::Clipboard::new().and_then(|mut cb| cb.get_text()) {
                    Ok(text) => self.paste_tsv(&text),
                    Err(e) => self.status = format!("Paste: {e}"),
                }
            }
            SelectAll => {
                let (ur, uc) = self.engine.used_range(self.sheet);
                self.anchor = (1, 1);
                self.cursor = (ur, uc);
            }
            New => self.request(Pending::New, ctx),
            Open => self.request(Pending::Open, ctx),
            Save => {
                self.do_save();
            }
            SaveAs => {
                self.do_save_as();
            }
            NextSheet => self.switch_sheet(self.sheet + 1),
            PrevSheet => self.switch_sheet(self.sheet.saturating_sub(1)),
            NewSheet => {
                if let Err(e) = self.engine.new_sheet() {
                    self.status = format!("New sheet: {e}");
                } else {
                    let last = self.engine.sheet_names().len() as u32 - 1;
                    self.switch_sheet(last);
                }
            }
            Quit => self.request(Pending::Quit, ctx),
        }
    }

    // ----- input -----

    fn collect_input(&mut self, ctx: &Context) -> Vec<Command> {
        let mut cmds = Vec::new();
        // While a text field (cell editor, formula bar, ...) has focus, global
        // shortcuts stay out of the way; Enter/Tab/Escape are handled by the
        // edit widgets themselves.
        if self.edit.is_some() || ctx.wants_keyboard_input() {
            return cmds;
        }
        ctx.input_mut(|input| {
            for (shortcut, cmd) in &self.keymap.bindings {
                if input.consume_shortcut(shortcut) {
                    cmds.push(*cmd);
                }
            }
        });
        // Typing any printable character starts editing the active cell,
        // replacing its content — Excel behavior.
        let mut typed = String::new();
        let mut clipboard = Vec::new();
        ctx.input(|input| {
            for event in &input.events {
                match event {
                    Event::Text(t) => typed.push_str(t),
                    Event::Copy => clipboard.push(Command::Copy),
                    Event::Cut => clipboard.push(Command::Cut),
                    Event::Paste(text) => {
                        // Paste arrives with its payload; handle inline.
                        clipboard.push(Command::Paste);
                        typed.clear();
                        let _ = text;
                    }
                    _ => {}
                }
            }
        });
        // Dedupe: the platform chords (Ctrl+C/X/V) can surface both as a
        // consumed shortcut and as a clipboard event in the same frame.
        for cmd in clipboard {
            if !cmds.contains(&cmd) {
                cmds.push(cmd);
            }
        }
        if !typed.is_empty() && !typed.chars().all(char::is_control) {
            self.start_edit(typed, true);
        }
        cmds
    }

    // ----- UI panels -----

    /// Excel-style ribbon button: large icon on top, label below, both
    /// horizontally centered; the whole area is clickable.
    fn ribbon_button(
        &self,
        ui: &mut Ui,
        icon: &str,
        label: &str,
        cmd: Command,
        out: &mut Vec<Command>,
    ) {
        let icon_font = FontId::proportional(20.0);
        let label_font = FontId::proportional(10.5);
        let text_w = ui
            .painter()
            .layout_no_wrap(label.to_owned(), label_font.clone(), egui::Color32::WHITE)
            .size()
            .x;
        let size = Vec2::new((text_w + 14.0).max(48.0), 48.0);
        let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
        let visuals = ui.style().interact(&resp);
        if resp.hovered() || resp.is_pointer_button_down_on() {
            ui.painter().rect_filled(rect, 4.0, visuals.weak_bg_fill);
        }
        let color = visuals.text_color();
        ui.painter().text(
            Pos2::new(rect.center().x, rect.min.y + 4.0),
            Align2::CENTER_TOP,
            icon,
            icon_font,
            color,
        );
        ui.painter().text(
            Pos2::new(rect.center().x, rect.max.y - 4.0),
            Align2::CENTER_BOTTOM,
            label,
            label_font,
            color,
        );
        let resp = match self.keymap.shortcut_for(cmd) {
            Some(sc) => resp.on_hover_text(ui.ctx().format_shortcut(&sc)),
            None => resp,
        };
        if resp.clicked() {
            out.push(cmd);
        }
    }

    /// A cluster of ribbon buttons with the group caption centered underneath.
    fn ribbon_group(
        &self,
        ui: &mut Ui,
        caption: &str,
        items: &[(&str, &str, Command)],
        out: &mut Vec<Command>,
    ) {
        ui.vertical(|ui| {
            let row = ui
                .horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 2.0;
                    for (icon, label, cmd) in items {
                        self.ribbon_button(ui, icon, label, *cmd, out);
                    }
                })
                .response
                .rect;
            let (caption_rect, _) = ui.allocate_exact_size(
                Vec2::new(row.width(), 14.0),
                Sense::hover(),
            );
            ui.painter().text(
                Pos2::new(row.center().x, caption_rect.center().y),
                Align2::CENTER_CENTER,
                caption,
                FontId::proportional(10.0),
                ui.visuals().weak_text_color(),
            );
        });
        ui.separator();
    }

    fn ribbon(&mut self, ctx: &Context) -> Vec<Command> {
        let mut cmds = Vec::new();
        egui::TopBottomPanel::top("ribbon").show(ctx, |ui| {
            // Tab strip. File opens the full-screen backstage view (like
            // Excel); the other tabs switch the ribbon content below.
            ui.horizontal(|ui| {
                if ui.selectable_label(false, "File").clicked() {
                    self.backstage = true;
                }
                for (tab, label) in [
                    (RibbonTab::Home, "Home"),
                    (RibbonTab::Sheet, "Sheet"),
                    (RibbonTab::Help, "Help"),
                ] {
                    if ui.selectable_label(self.ribbon_tab == tab, label).clicked() {
                        self.ribbon_tab = tab;
                    }
                }
            });
            ui.separator();
            ui.horizontal(|ui| match self.ribbon_tab {
                RibbonTab::Home => {
                    self.ribbon_group(
                        ui,
                        "Clipboard",
                        &[
                            ("✂", "Cut", Command::Cut),
                            ("⧉", "Copy", Command::Copy),
                            ("📋", "Paste", Command::Paste),
                        ],
                        &mut cmds,
                    );
                    self.ribbon_group(
                        ui,
                        "History",
                        &[
                            ("⟲", "Undo", Command::Undo),
                            ("⟳", "Redo", Command::Redo),
                        ],
                        &mut cmds,
                    );
                    self.ribbon_group(
                        ui,
                        "Editing",
                        &[
                            ("⌫", "Clear", Command::Clear),
                            ("⬇", "Fill Down", Command::FillDown),
                            ("➡", "Fill Right", Command::FillRight),
                            ("▦", "Select All", Command::SelectAll),
                        ],
                        &mut cmds,
                    );
                }
                RibbonTab::Sheet => {
                    self.ribbon_group(
                        ui,
                        "Sheets",
                        &[
                            ("＋", "New Sheet", Command::NewSheet),
                            ("◀", "Previous", Command::PrevSheet),
                            ("▶", "Next", Command::NextSheet),
                        ],
                        &mut cmds,
                    );
                }
                RibbonTab::Help => {
                    ui.vertical(|ui| {
                        ui.label(format!("Keymap: {}", self.keymap.source));
                        ui.label(
                            egui::RichText::new(
                                "Edit keymap.toml to customize shortcuts (Excel defaults built in).",
                            )
                            .weak()
                            .small(),
                        );
                    });
                }
            });
            ui.add_space(2.0);
        });
        cmds
    }

    fn formula_bar(&mut self, ctx: &Context) {
        egui::TopBottomPanel::top("formulabar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let name = format!(
                    "{}{}",
                    number_to_column(self.cursor.1).unwrap_or_default(),
                    self.cursor.0
                );
                ui.add_sized([70.0, 20.0], egui::Label::new(egui::RichText::new(name).monospace()));
                ui.separator();
                let editing_here = self
                    .edit
                    .as_ref()
                    .map(|e| e.in_formula_bar)
                    .unwrap_or(false);
                let mut shown = match &self.edit {
                    Some(edit) => edit.text.clone(),
                    None => self.engine.content(self.sheet, self.cursor.0, self.cursor.1),
                };
                let resp = ui.add_sized(
                    [ui.available_width(), 20.0],
                    TextEdit::singleline(&mut shown).font(FontId::monospace(CELL_FONT)),
                );
                if resp.gained_focus() && self.edit.is_none() {
                    self.anchor = self.cursor;
                    self.edit = Some(EditState {
                        row: self.cursor.0,
                        col: self.cursor.1,
                        text: shown.clone(),
                        take_focus: false,
                        caret_end: false,
                        in_formula_bar: true,
                    });
                }
                if editing_here {
                    if let Some(edit) = &mut self.edit {
                        edit.text = shown;
                    }
                    if resp.lost_focus() {
                        let edit = self.edit.take().unwrap();
                        let (esc, enter) = ui.input(|i| {
                            (i.key_pressed(Key::Escape), i.key_pressed(Key::Enter))
                        });
                        if !esc {
                            self.commit_edit(&edit);
                            if enter {
                                self.move_cursor(1, 0, false);
                            }
                        }
                    }
                } else if self.edit.is_some() && !editing_here {
                    // Mirror only; cell editor owns the text.
                }
            });
        });
    }

    fn sheet_tabs(&mut self, ctx: &Context) {
        egui::TopBottomPanel::bottom("tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let names = self.engine.sheet_names();
                let mut switch = None;
                for (i, name) in names.iter().enumerate() {
                    if ui
                        .selectable_label(i as u32 == self.sheet, name)
                        .clicked()
                    {
                        switch = Some(i as u32);
                    }
                }
                if ui.button("+").on_hover_text("New sheet").clicked() {
                    if self.engine.new_sheet().is_ok() {
                        switch = Some(names.len() as u32);
                    }
                }
                if let Some(s) = switch {
                    self.switch_sheet(s);
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(&self.status).weak());
                });
            });
        });
    }

    fn confirm_dialog(&mut self, ctx: &Context) {
        let Some(action) = self.pending.clone() else { return };
        let mut choice: Option<&str> = None;
        egui::Window::new("Unsaved changes")
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label("This workbook has unsaved changes.");
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        choice = Some("save");
                    }
                    if ui.button("Discard").clicked() {
                        choice = Some("discard");
                    }
                    if ui.button("Cancel").clicked() {
                        choice = Some("cancel");
                    }
                });
            });
        match choice {
            Some("save") => {
                self.pending = None;
                if self.do_save() {
                    self.run_pending(action, ctx);
                }
            }
            Some("discard") => {
                self.pending = None;
                self.run_pending(action, ctx);
            }
            Some("cancel") => self.pending = None,
            _ => {}
        }
    }

    // ----- grid -----

    fn grid(&mut self, ctx: &Context) {
        let bg = ctx.style().visuals.panel_fill;
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(bg))
            .show(ctx, |ui| {
                let avail = ui.available_rect_before_wrap();
                let grid_rect = Rect::from_min_max(
                    Pos2::new(avail.min.x + GUTTER_W, avail.min.y + HEADER_H),
                    avail.max,
                );
                self.rows_per_page = ((grid_rect.height() / 21.0) as i32 - 1).max(1);

                let mut grid_ui = ui.new_child(UiBuilder::new().max_rect(grid_rect));
                grid_ui.set_clip_rect(grid_rect);
                let output = ScrollArea::both()
                    .auto_shrink([false, false])
                    .show_viewport(&mut grid_ui, |ui, viewport| {
                        self.grid_contents(ui, viewport);
                    });
                let offset = output.state.offset;

                self.headers(ui, avail, grid_rect, offset);
                self.edit_overlay(ctx, grid_rect, offset);
            });
    }

    fn grid_contents(&mut self, ui: &mut Ui, viewport: Rect) {
        self.ensure_geometry();
        let total = Vec2::new(
            self.col_off[self.view_cols as usize],
            self.row_off[self.view_rows as usize],
        );
        ui.set_min_size(total);
        let origin = ui.min_rect().min;

        // Visible cell range.
        let r0 = self.row_off.partition_point(|&y| y <= viewport.min.y).max(1) as i32;
        let r1 = (self.row_off.partition_point(|&y| y < viewport.max.y) as i32)
            .clamp(r0, self.view_rows);
        let c0 = self.col_off.partition_point(|&x| x <= viewport.min.x).max(1) as i32;
        let c1 = (self.col_off.partition_point(|&x| x < viewport.max.x) as i32)
            .clamp(c0, self.view_cols);

        // Interaction covers the whole (virtual) grid.
        let response = ui.interact(
            Rect::from_min_size(origin, total),
            Id::new("sheetz-grid"),
            Sense::click_and_drag(),
        );
        let shift = ui.input(|i| i.modifiers.shift);
        if let Some(pos) = response.interact_pointer_pos() {
            let cell = self.cell_at(pos, origin);
            if response.double_clicked() {
                self.set_cursor(cell.0, cell.1);
                let text = self.engine.content(self.sheet, cell.0, cell.1);
                self.start_edit(text, true);
            } else if response.drag_started() || response.clicked() {
                self.cursor = cell;
                if !shift {
                    self.anchor = cell;
                }
            } else if response.dragged() {
                self.cursor = cell;
            }
        }

        let visuals = ui.visuals().clone();
        let painter = ui.painter().clone();
        let accent = visuals.selection.bg_fill;
        let grid_line = Stroke::new(1.0, visuals.widgets.noninteractive.bg_stroke.color);

        // Selection background.
        let (sr0, sr1, sc0, sc1) = self.selection();
        if sr1 >= r0 && sr0 <= r1 && sc1 >= c0 && sc0 <= c1 {
            let rect = Rect::from_min_max(
                self.cell_rect(sr0.max(1), sc0.max(1), origin).min,
                self.cell_rect(sr1.min(self.view_rows), sc1.min(self.view_cols), origin).max,
            );
            painter.rect_filled(rect, 0.0, accent.linear_multiply(0.18));
        }

        // Grid lines.
        let x_min = origin.x + viewport.min.x.max(0.0);
        let x_max = origin.x + viewport.max.x.min(total.x);
        let y_min = origin.y + viewport.min.y.max(0.0);
        let y_max = origin.y + viewport.max.y.min(total.y);
        for r in (r0 - 1)..=r1 {
            let y = origin.y + self.row_off[r as usize];
            painter.hline(x_min..=x_max, y, grid_line);
        }
        for c in (c0 - 1)..=c1 {
            let x = origin.x + self.col_off[c as usize];
            painter.vline(x, y_min..=y_max, grid_line);
        }

        // Cell contents.
        let font = FontId::proportional(CELL_FONT);
        let text_color = visuals.text_color();
        for r in r0..=r1 {
            for c in c0..=c1 {
                let text = self.engine.display(self.sheet, r, c);
                if text.is_empty() {
                    continue;
                }
                let rect = self.cell_rect(r, c, origin);
                let clipped = painter.with_clip_rect(rect.intersect(ui.clip_rect()));
                if self.engine.is_number(self.sheet, r, c) {
                    clipped.text(
                        Pos2::new(rect.max.x - CELL_PAD, rect.center().y),
                        Align2::RIGHT_CENTER,
                        text,
                        font.clone(),
                        text_color,
                    );
                } else {
                    clipped.text(
                        Pos2::new(rect.min.x + CELL_PAD, rect.center().y),
                        Align2::LEFT_CENTER,
                        text,
                        font.clone(),
                        text_color,
                    );
                }
            }
        }

        // Active cell border.
        let active = self.cell_rect(self.cursor.0, self.cursor.1, origin);
        painter.rect_stroke(
            active,
            CornerRadius::ZERO,
            Stroke::new(2.0, accent),
            egui::StrokeKind::Inside,
        );

        if self.scroll_to_cursor {
            ui.scroll_to_rect(active.expand(2.0), None);
            self.scroll_to_cursor = false;
        }
    }

    fn headers(&mut self, ui: &mut Ui, avail: Rect, grid_rect: Rect, offset: Vec2) {
        let visuals = ui.visuals().clone();
        let bg = visuals.faint_bg_color;
        let line = Stroke::new(1.0, visuals.widgets.noninteractive.bg_stroke.color);
        let font = FontId::proportional(12.0);
        let text_color = visuals.text_color();
        let accent = visuals.selection.bg_fill;
        let (sr0, sr1, sc0, sc1) = self.selection();

        // Column header strip.
        let col_hdr = Rect::from_min_max(
            Pos2::new(grid_rect.min.x, avail.min.y),
            Pos2::new(avail.max.x, avail.min.y + HEADER_H),
        );
        let painter = ui.painter().with_clip_rect(col_hdr);
        painter.rect_filled(col_hdr, 0.0, bg);
        let c0 = self.col_off.partition_point(|&x| x <= offset.x).max(1) as i32;
        let c1 = (self
            .col_off
            .partition_point(|&x| x < offset.x + col_hdr.width()) as i32)
            .clamp(c0, self.view_cols);
        for c in c0..=c1 {
            let x0 = grid_rect.min.x + self.col_off[c as usize - 1] - offset.x;
            let x1 = grid_rect.min.x + self.col_off[c as usize] - offset.x;
            let rect = Rect::from_min_max(
                Pos2::new(x0, col_hdr.min.y),
                Pos2::new(x1, col_hdr.max.y),
            );
            if c >= sc0 && c <= sc1 {
                painter.rect_filled(rect, 0.0, accent.linear_multiply(0.25));
            }
            painter.vline(x1, col_hdr.min.y..=col_hdr.max.y, line);
            painter.text(
                rect.center(),
                Align2::CENTER_CENTER,
                number_to_column(c).unwrap_or_default(),
                font.clone(),
                text_color,
            );
        }
        painter.hline(col_hdr.min.x..=col_hdr.max.x, col_hdr.max.y, line);

        // Row number gutter.
        let row_hdr = Rect::from_min_max(
            Pos2::new(avail.min.x, grid_rect.min.y),
            Pos2::new(avail.min.x + GUTTER_W, avail.max.y),
        );
        let painter = ui.painter().with_clip_rect(row_hdr);
        painter.rect_filled(row_hdr, 0.0, bg);
        let r0 = self.row_off.partition_point(|&y| y <= offset.y).max(1) as i32;
        let r1 = (self
            .row_off
            .partition_point(|&y| y < offset.y + row_hdr.height()) as i32)
            .clamp(r0, self.view_rows);
        for r in r0..=r1 {
            let y0 = grid_rect.min.y + self.row_off[r as usize - 1] - offset.y;
            let y1 = grid_rect.min.y + self.row_off[r as usize] - offset.y;
            let rect = Rect::from_min_max(
                Pos2::new(row_hdr.min.x, y0),
                Pos2::new(row_hdr.max.x, y1),
            );
            if r >= sr0 && r <= sr1 {
                painter.rect_filled(rect, 0.0, accent.linear_multiply(0.25));
            }
            painter.hline(row_hdr.min.x..=row_hdr.max.x, y1, line);
            painter.text(
                rect.center(),
                Align2::CENTER_CENTER,
                r.to_string(),
                font.clone(),
                text_color,
            );
        }
        painter.vline(row_hdr.max.x, row_hdr.min.y..=row_hdr.max.y, line);

        // Corner box.
        let corner = Rect::from_min_size(avail.min, Vec2::new(GUTTER_W, HEADER_H));
        let painter = ui.painter().with_clip_rect(corner);
        painter.rect_filled(corner, 0.0, bg);
        painter.hline(corner.min.x..=corner.max.x, corner.max.y, line);
        painter.vline(corner.max.x, corner.min.y..=corner.max.y, line);
    }

    fn edit_overlay(&mut self, ctx: &Context, grid_rect: Rect, offset: Vec2) {
        let Some(edit) = &self.edit else { return };
        if edit.in_formula_bar {
            return;
        }
        let (row, col) = (edit.row, edit.col);
        let r = row as usize;
        let c = col as usize;
        if r >= self.row_off.len() || c >= self.col_off.len() {
            return;
        }
        let min = Pos2::new(
            grid_rect.min.x + self.col_off[c - 1] - offset.x,
            grid_rect.min.y + self.row_off[r - 1] - offset.y,
        );
        let size = Vec2::new(
            (self.col_off[c] - self.col_off[c - 1]).max(120.0),
            self.row_off[r] - self.row_off[r - 1],
        );

        let mut committed = false;
        let mut cancelled = false;
        let mut move_after: Option<(i32, i32)> = None;

        egui::Area::new(Id::new("sheetz-cell-edit"))
            .fixed_pos(min)
            .order(Order::Foreground)
            .show(ctx, |ui| {
                let edit = self.edit.as_mut().unwrap();
                let resp = ui.add(
                    TextEdit::singleline(&mut edit.text)
                        .font(FontId::proportional(CELL_FONT))
                        .min_size(size)
                        .vertical_align(egui::Align::Center),
                );
                if edit.take_focus {
                    resp.request_focus();
                    if edit.caret_end {
                        // Place the caret at the end instead of select-all.
                        let id = resp.id;
                        if let Some(mut state) = TextEdit::load_state(ctx, id) {
                            let end = egui::text::CCursor::new(edit.text.chars().count());
                            state
                                .cursor
                                .set_char_range(Some(egui::text::CCursorRange::one(end)));
                            state.store(ctx, id);
                        }
                    }
                    edit.take_focus = false;
                }
                if resp.lost_focus() {
                    let (esc, enter, tab, shift) = ui.input(|i| {
                        (
                            i.key_pressed(Key::Escape),
                            i.key_pressed(Key::Enter),
                            i.key_pressed(Key::Tab),
                            i.modifiers.shift,
                        )
                    });
                    if esc {
                        cancelled = true;
                    } else {
                        committed = true;
                        if enter {
                            move_after = Some(if shift { (-1, 0) } else { (1, 0) });
                        } else if tab {
                            move_after = Some(if shift { (0, -1) } else { (0, 1) });
                        }
                    }
                }
            });

        if committed || cancelled {
            let edit = self.edit.take().unwrap();
            if committed {
                self.commit_edit(&edit);
            }
            if let Some((dr, dc)) = move_after {
                self.move_cursor(dr, dc, false);
            }
        }
    }

    /// Full-screen File view (Excel's "backstage"): nav sidebar on the left,
    /// workbook info and recent files on the right.
    fn backstage_ui(&mut self, ctx: &Context) {
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            self.backstage = false;
            return;
        }
        let mut action: Option<Command> = None;
        let mut open_recent: Option<PathBuf> = None;

        egui::SidePanel::left("backstage-nav")
            .resizable(false)
            .exact_width(190.0)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                if ui
                    .add(egui::Button::new(egui::RichText::new("←  Back").size(16.0)).frame(false))
                    .clicked()
                {
                    self.backstage = false;
                }
                ui.add_space(16.0);
                for (label, cmd) in [
                    ("🗋  New", Command::New),
                    ("🗁  Open…", Command::Open),
                    ("💾  Save", Command::Save),
                    ("🗐  Save As…", Command::SaveAs),
                ] {
                    let mut resp = ui.add_sized(
                        [ui.available_width(), 34.0],
                        egui::Button::new(egui::RichText::new(label).size(14.0)),
                    );
                    if let Some(sc) = self.keymap.shortcut_for(cmd) {
                        resp = resp.on_hover_text(ui.ctx().format_shortcut(&sc));
                    }
                    if resp.clicked() {
                        action = Some(cmd);
                    }
                    ui.add_space(4.0);
                }
                ui.add_space(12.0);
                ui.separator();
                ui.add_space(12.0);
                if ui
                    .add_sized(
                        [ui.available_width(), 34.0],
                        egui::Button::new(egui::RichText::new("✖  Quit").size(14.0)),
                    )
                    .clicked()
                {
                    action = Some(Command::Quit);
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(8.0);
            ui.heading("Info");
            ui.add_space(4.0);
            match &self.engine.path {
                Some(p) => {
                    ui.label(
                        egui::RichText::new(
                            p.file_name().map(|n| n.to_string_lossy()).unwrap_or_default(),
                        )
                        .strong(),
                    );
                    ui.label(egui::RichText::new(p.display().to_string()).weak());
                }
                None => {
                    ui.label(egui::RichText::new("Book1").strong());
                    ui.label(egui::RichText::new("Not saved yet").weak());
                }
            }
            ui.label(format!(
                "{} sheet(s){}",
                self.engine.sheet_names().len(),
                if self.engine.dirty {
                    " — unsaved changes"
                } else {
                    ""
                }
            ));
            ui.add_space(16.0);
            ui.heading("Recent");
            ui.add_space(4.0);
            if self.recent.is_empty() {
                ui.label(egui::RichText::new("No recent files.").weak());
            }
            for path in &self.recent {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| path.display().to_string());
                let resp = ui.link(name).on_hover_text(path.display().to_string());
                if resp.clicked() {
                    open_recent = Some(path.clone());
                }
            }
        });

        if let Some(cmd) = action {
            self.backstage = false;
            self.exec(cmd, ctx);
        }
        if let Some(path) = open_recent {
            self.backstage = false;
            if self.engine.dirty {
                self.pending = Some(Pending::OpenPath(path));
            } else {
                self.open_path(path);
            }
        }
    }

    fn update_title(&mut self, ctx: &Context) {
        let name = self
            .engine
            .path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Book1".to_string());
        let title = format!(
            "Sheetz — {}{}",
            name,
            if self.engine.dirty { " •" } else { "" }
        );
        if title != self.last_title {
            ctx.send_viewport_cmd(ViewportCommand::Title(title.clone()));
            self.last_title = title;
        }
    }
}

impl eframe::App for SheetzApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Intercept window close while there are unsaved changes.
        if ctx.input(|i| i.viewport().close_requested()) && self.engine.dirty && !self.allow_close
        {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            self.pending = Some(Pending::Quit);
        }

        if self.backstage {
            self.backstage_ui(ctx);
            self.confirm_dialog(ctx);
            self.update_title(ctx);
            return;
        }

        let mut cmds = self.collect_input(ctx);
        cmds.extend(self.ribbon(ctx));
        for cmd in cmds {
            self.exec(cmd, ctx);
        }

        self.formula_bar(ctx);
        self.sheet_tabs(ctx);
        self.grid(ctx);
        self.confirm_dialog(ctx);
        self.update_title(ctx);
    }
}

/// One axis of Excel's Ctrl+Arrow "jump to data boundary".
/// `far` is the sheet edge in that direction; `scan_end` is the last position
/// worth scanning (the used range).
fn jump_axis(start: i32, dir: i32, far: i32, scan_end: i32, filled: impl Fn(i32) -> bool) -> i32 {
    let next = start + dir;
    let in_range = |i: i32| {
        if dir > 0 {
            i <= scan_end.max(start)
        } else {
            i >= scan_end
        }
    };
    if !in_range(next) || next < 1 || next > far.max(1) {
        return far;
    }
    if filled(start) && filled(next) {
        // Run to the end of the contiguous filled block.
        let mut i = next;
        while in_range(i + dir) && filled(i + dir) {
            i += dir;
        }
        i
    } else {
        // Skip blanks to the next filled cell; land on the sheet edge if none.
        let mut i = next;
        while in_range(i) {
            if filled(i) {
                return i;
            }
            i += dir;
        }
        far
    }
}
