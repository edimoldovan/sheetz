//! Application state shared by all UI modules.
//!
//! `SheetzApp` lives here; `app.rs` owns the update loop and command dispatch,
//! and each `ui::*` module adds its own `impl SheetzApp` block.

use std::collections::HashMap;
use std::path::PathBuf;

use crate::engine::{Engine, FindHit, Range};
use crate::keymap::Keymap;

pub const GUTTER_W: f32 = 48.0;
pub const HEADER_H: f32 = 24.0;
pub const BASE_FONT: f32 = 13.0;
pub const CELL_PAD: f32 = 4.0;
/// Width of the hit zone for dragging a header edge to resize.
pub const RESIZE_GRAB: f32 = 4.0;

/// Excel distinguishes these two modes: after typing into a cell, arrow keys
/// commit and move; after F2 (or double-click), they move the caret instead.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EditMode {
    /// Started by typing — arrows commit the edit and move the cursor.
    Enter,
    /// Started by F2 / double-click / formula bar — arrows move the caret.
    Edit,
}

pub struct EditState {
    pub row: i32,
    pub col: i32,
    pub text: String,
    pub mode: EditMode,
    pub take_focus: bool,
    pub caret_end: bool,
    pub in_formula_bar: bool,
    /// Function-name suggestions for the current text, and which is highlighted.
    pub completions: Vec<String>,
    pub completion_index: usize,
    /// Text the completion list was computed for, so we only recompute on change.
    pub completions_for: String,
}

impl EditState {
    pub fn new(row: i32, col: i32, text: String, mode: EditMode, in_formula_bar: bool) -> EditState {
        EditState {
            row,
            col,
            text,
            mode,
            take_focus: !in_formula_bar,
            caret_end: true,
            in_formula_bar,
            completions: Vec::new(),
            completion_index: 0,
            completions_for: String::new(),
        }
    }

    pub fn is_formula(&self) -> bool {
        self.text.starts_with('=')
    }
}

#[derive(Clone, PartialEq)]
pub enum Pending {
    New,
    Open,
    OpenPath(PathBuf),
    Quit,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RibbonTab {
    Home,
    Insert,
    Data,
    View,
    Sheet,
    Help,
}

/// Which modal dialog, if any, is open.
#[derive(Clone, PartialEq)]
pub enum Dialog {
    None,
    FindReplace,
    Sort,
    NameManager,
    FormatCells,
    Shortcuts,
    RenameSheet { sheet: u32, name: String },
    GotoCell,
    ConditionalFormat,
    Chart,
}

/// Draft state for the conditional-formatting dialog.
pub struct CfState {
    /// 0 = cell value rule, 1 = color scale
    pub kind: usize,
    /// 0 greater, 1 less, 2 equal, 3 between
    pub operator: usize,
    pub value: String,
    pub value2: String,
    pub fill: [u8; 3],
    pub message: String,
}

impl Default for CfState {
    fn default() -> Self {
        CfState {
            kind: 0,
            operator: 0,
            value: "0".to_string(),
            value2: String::new(),
            fill: [255, 199, 206],
            message: String::new(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ChartKind {
    Bar,
    Line,
    Pie,
}

/// A chart built from a range and drawn natively (not stored in the xlsx).
pub struct Chart {
    pub kind: ChartKind,
    pub sheet: u32,
    pub data: Range,
    pub title: String,
    /// First row of the range holds series labels.
    pub has_header: bool,
}

/// A drag in progress on a row/column header edge.
#[derive(Clone, Copy, PartialEq)]
pub enum Resizing {
    Column { index: i32, start_x: f32, start_w: f32 },
    Row { index: i32, start_y: f32, start_h: f32 },
}

/// State of the fill handle drag (the little square at the selection corner).
#[derive(Clone, Copy, PartialEq)]
pub struct FillDrag {
    pub origin: Range,
    pub to_row: i32,
    pub to_col: i32,
}

/// An active AutoFilter on a header row.
#[derive(Clone)]
pub struct Filter {
    pub header_row: i32,
    pub r0: i32,
    pub r1: i32,
    pub c0: i32,
    pub c1: i32,
    /// column index -> set of allowed display values (empty = all allowed)
    pub allowed: HashMap<i32, Vec<String>>,
    /// Which column's dropdown is currently open.
    pub open_column: Option<i32>,
}

pub struct FindState {
    pub needle: String,
    pub replacement: String,
    pub match_case: bool,
    pub whole_workbook: bool,
    pub hits: Vec<FindHit>,
    pub index: usize,
    pub message: String,
}

impl Default for FindState {
    fn default() -> Self {
        FindState {
            needle: String::new(),
            replacement: String::new(),
            match_case: false,
            whole_workbook: false,
            hits: Vec::new(),
            index: 0,
            message: String::new(),
        }
    }
}

pub struct SortState {
    pub key_column: i32,
    pub ascending: bool,
    pub has_header: bool,
}

impl Default for SortState {
    fn default() -> Self {
        SortState {
            key_column: 1,
            ascending: true,
            has_header: true,
        }
    }
}

pub struct NameState {
    pub new_name: String,
    pub new_formula: String,
    pub message: String,
}

impl Default for NameState {
    fn default() -> Self {
        NameState {
            new_name: String::new(),
            new_formula: String::new(),
            message: String::new(),
        }
    }
}

pub struct SheetzApp {
    pub engine: Engine,
    pub keymap: Keymap,
    pub sheet: u32,

    // Selection: cursor is the active cell, anchor the other corner.
    pub cursor: (i32, i32),
    pub anchor: (i32, i32),
    pub saved_cursors: HashMap<u32, ((i32, i32), (i32, i32))>,

    pub edit: Option<EditState>,
    pub scroll_to_cursor: bool,

    // Grid geometry cache: boundary offsets in px (already zoom-scaled).
    // row_off[r] is the y where row r+1 starts; row r (1-based) spans
    // row_off[r-1]..row_off[r].
    pub row_off: Vec<f32>,
    pub col_off: Vec<f32>,
    pub view_rows: i32,
    pub view_cols: i32,
    pub geom_dirty: bool,
    pub rows_per_page: i32,
    pub zoom: f32,

    // Interactions in progress.
    pub resizing: Option<Resizing>,
    pub fill_drag: Option<FillDrag>,
    /// Cells being cut, shown with a dashed outline.
    pub cut_marker: Option<(u32, Range)>,

    // Dialog / modal state.
    pub dialog: Dialog,
    pub find: FindState,
    pub sort: SortState,
    pub names: NameState,
    pub goto_text: String,
    pub filter: Option<Filter>,
    pub cf: CfState,
    /// Charts drawn in floating windows over the grid.
    pub charts: Vec<Chart>,
    pub chart_draft: ChartKind,

    // File / lifecycle.
    pub pending: Option<Pending>,
    pub allow_close: bool,
    pub status: String,
    pub last_title: String,
    pub ribbon_tab: RibbonTab,
    pub backstage: bool,
    pub recent: Vec<PathBuf>,
    pub recent_colors: Vec<[u8; 3]>,
}

impl SheetzApp {
    /// The selected block, normalized so r0 <= r1 and c0 <= c1.
    pub fn selection(&self) -> Range {
        Range {
            r0: self.cursor.0.min(self.anchor.0),
            r1: self.cursor.0.max(self.anchor.0),
            c0: self.cursor.1.min(self.anchor.1),
            c1: self.cursor.1.max(self.anchor.1),
        }
    }

    pub fn set_status(&mut self, text: impl Into<String>) {
        self.status = text.into();
    }

    /// Reports an error from a fallible engine call in the status bar.
    pub fn report(&mut self, context: &str, result: anyhow::Result<()>) {
        if let Err(e) = result {
            self.status = format!("{context}: {e}");
        }
    }

    pub fn invalidate_geometry(&mut self) {
        self.geom_dirty = true;
    }
}
