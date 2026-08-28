use std::io::BufWriter;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use ironcalc::base::expressions::types::Area;
use ironcalc::base::types::CellType;
use ironcalc::base::UserModel;
use ironcalc::export::save_xlsx_to_writer;
use ironcalc::import::load_from_xlsx;

/// Excel sheet limits.
pub const MAX_ROWS: i32 = 1_048_576;
pub const MAX_COLS: i32 = 16_384;

const LOCALE: &str = "en";
const TZ: &str = "UTC";
const LANG: &str = "en";

/// Thin wrapper around IronCalc's `UserModel` (which provides evaluation,
/// undo/redo history and edit semantics) plus file-path/dirty bookkeeping.
pub struct Engine {
    um: UserModel<'static>,
    pub path: Option<PathBuf>,
    pub dirty: bool,
}

impl Engine {
    pub fn new() -> Result<Engine> {
        let um = UserModel::new_empty("Book1", LOCALE, TZ, LANG).map_err(|e| anyhow!(e))?;
        Ok(Engine {
            um,
            path: None,
            dirty: false,
        })
    }

    pub fn open(path: &Path) -> Result<Engine> {
        let file = path.to_str().ok_or_else(|| anyhow!("path is not valid UTF-8"))?;
        let model = load_from_xlsx(file, LOCALE, TZ, LANG).map_err(|e| anyhow!("{e:?}"))?;
        Ok(Engine {
            um: UserModel::from_model(model),
            path: Some(path.to_path_buf()),
            dirty: false,
        })
    }

    pub fn save(&mut self, path: &Path) -> Result<()> {
        // ironcalc's save_to_xlsx refuses to overwrite, so write through
        // save_xlsx_to_writer with a normal (truncating) File::create.
        let file = std::fs::File::create(path)?;
        save_xlsx_to_writer(self.um.get_model(), BufWriter::new(file))
            .map_err(|e| anyhow!("{e:?}"))?;
        self.path = Some(path.to_path_buf());
        self.dirty = false;
        Ok(())
    }

    /// Value as shown in the grid (number formats applied).
    pub fn display(&self, sheet: u32, row: i32, col: i32) -> String {
        self.um
            .get_formatted_cell_value(sheet, row, col)
            .unwrap_or_default()
    }

    /// Raw cell content as typed by the user (formulas keep their `=`).
    pub fn content(&self, sheet: u32, row: i32, col: i32) -> String {
        self.um.get_cell_content(sheet, row, col).unwrap_or_default()
    }

    pub fn is_number(&self, sheet: u32, row: i32, col: i32) -> bool {
        matches!(
            self.um.get_cell_type(sheet, row, col),
            Ok(CellType::Number)
        )
    }

    pub fn is_empty(&self, sheet: u32, row: i32, col: i32) -> bool {
        self.content(sheet, row, col).is_empty()
    }

    pub fn set_input(&mut self, sheet: u32, row: i32, col: i32, value: &str) -> Result<()> {
        self.um
            .set_user_input(sheet, row, col, value)
            .map_err(|e| anyhow!(e))?;
        self.dirty = true;
        Ok(())
    }

    pub fn undo(&mut self) {
        if self.um.undo().is_ok() {
            self.dirty = true;
        }
    }

    pub fn redo(&mut self) {
        if self.um.redo().is_ok() {
            self.dirty = true;
        }
    }

    pub fn sheet_names(&self) -> Vec<String> {
        self.um.get_model().workbook.get_worksheet_names()
    }

    pub fn new_sheet(&mut self) -> Result<()> {
        self.um.new_sheet().map_err(|e| anyhow!(e))?;
        self.dirty = true;
        Ok(())
    }

    pub fn col_width(&self, sheet: u32, col: i32) -> f32 {
        self.um
            .get_column_width(sheet, col)
            .map(|w| w as f32)
            .unwrap_or(100.0)
            .max(8.0)
    }

    pub fn row_height(&self, sheet: u32, row: i32) -> f32 {
        self.um
            .get_row_height(sheet, row)
            .map(|h| h as f32)
            .unwrap_or(21.0)
            .max(10.0)
    }

    /// (last used row, last used column) of the sheet, at least (1, 1).
    pub fn used_range(&self, sheet: u32) -> (i32, i32) {
        self.um
            .get_model()
            .workbook
            .worksheet(sheet)
            .map(|ws| {
                let d = ws.dimension();
                (d.max_row.max(1), d.max_column.max(1))
            })
            .unwrap_or((1, 1))
    }

    /// Excel-style fill: extends the top row of `area` down to `to_row`,
    /// adjusting relative references.
    pub fn auto_fill_rows(&mut self, area: Area, to_row: i32) -> Result<()> {
        self.um
            .auto_fill_rows(&area, to_row)
            .map_err(|e| anyhow!(e))?;
        self.dirty = true;
        Ok(())
    }

    pub fn auto_fill_columns(&mut self, area: Area, to_col: i32) -> Result<()> {
        self.um
            .auto_fill_columns(&area, to_col)
            .map_err(|e| anyhow!(e))?;
        self.dirty = true;
        Ok(())
    }
}
