# Sheetz

A fast, native, Excel-compatible spreadsheet for Linux. 100% Rust — no web
runtime, no JavaScript, one binary.

## Stack

- **[IronCalc](https://github.com/ironcalc/IronCalc)** — spreadsheet engine:
  xlsx read/write, Excel formula language, dependency-graph recalculation,
  styles, conditional formatting, undo/redo history.
- **[egui](https://github.com/emilk/egui)** (via eframe) — GPU-rendered
  immediate-mode GUI. The grid paints only visible cells, so large sheets
  scroll smoothly.
- **rfd** — native file dialogs. **arboard** — system clipboard.

## Build & run

```bash
cargo run --release -- demo.xlsx
```

Generate a demo workbook (styles, currency/percent formats, conditional
formatting, a formula showcase sheet):

```bash
cargo run --example make_demo demo.xlsx
```

Install for your desktop (binary, icon, `.desktop` entry, keymap):

```bash
./packaging/install.sh
```

## Architecture

One concern per module, so the pieces stay independently workable:

| File | Responsibility |
| --- | --- |
| [engine.rs](src/engine.rs) | All IronCalc access, exposed as plain types — the UI never imports ironcalc |
| [state.rs](src/state.rs) | `SheetzApp` state |
| [app.rs](src/app.rs) | Update loop, input handling, command dispatch |
| [commands.rs](src/commands.rs) | The command registry every binding and button targets |
| [keymap.rs](src/keymap.rs) | `keymap.toml` → key chords |
| [ui/](src/ui/) | `grid`, `ribbon`, `editor`, `dialogs`, `charts` |

## Keyboard shortcuts

Excel defaults, fully customizable. Bindings live in [keymap.toml](keymap.toml)
and map chords to command ids:

```toml
[bindings]
"Ctrl+D" = "fill_down"
"Ctrl+Shift+L" = "toggle_filter"
"F2" = "edit_cell"
```

Lookup order (first hit wins): `$SHEETZ_KEYMAP`, `./keymap.toml`,
`~/.config/sheetz/keymap.toml`, then the copy compiled into the binary. Press
**F1** in the app to see every binding currently loaded. The full command list
is `Command::from_id` in [commands.rs](src/commands.rs).

## What works

**Navigation** — arrows, Tab/Enter, Ctrl+Arrow jump-to-edge, Shift to extend,
Ctrl+Shift+Arrow to extend to the edge, Page Up/Down, Ctrl+Home/End, Go To
(Ctrl+G) and a working Name Box.

**Editing** — Excel's two edit modes (typing then arrow commits and moves; F2
then arrow moves the caret), fill down/right with reference adjustment,
formula-aware copy/paste that rewrites relative references, paste-values-only,
undo/redo, Find & Replace across one sheet or the workbook.

**Formatting** — bold/italic/underline/strike, font size and color, fill color,
borders, horizontal and vertical alignment, wrap, and the number formats
(currency, percent, comma, date, custom codes, increase/decrease decimals).
All of it renders from — and saves back into — the .xlsx.

**Structure** — insert/delete rows and columns with reference fixing, column
and row resize, autofit, freeze panes, hide rows, sheet add/rename/duplicate/
delete/reorder/tab-color.

**Data** — sort (single and multi-key), AutoFilter, named ranges, conditional
formatting (cell rules and color scales, including rules loaded from real
workbooks), and Sum/Average/Count of the selection in the status bar.

**Charts** — bar, line and pie, drawn natively in floating windows.

See [PLAN.md](PLAN.md) for the parity roadmap and what is still missing.

## Known limitations

- **Charts are view-only** — they are not written into the .xlsx (chart XML
  round-tripping is on the roadmap).
- **Merging cells** is not possible: IronCalc exposes merged regions for
  reading but has no write API yet. Existing merges from a file are rendered.
- **Sorting moves formulas verbatim** — references inside sorted cells are not
  rewritten.
- **No pivot tables, print/PDF, images, or macros** (see PLAN.md; VBA is an
  explicit non-goal).
- IronCalc implements a large subset of Excel's functions, not all of them.
  Missing ones are best fixed upstream rather than worked around here.

## Tests

```bash
cargo test
```

Covers A1 parsing, column naming, jump-to-edge semantics, and engine-level
integration tests for formulas, styles, structure edits, sorting, find and
replace, the clipboard, conditional formatting and xlsx round-tripping.
