# Sheetz

A fast, native, Excel-compatible spreadsheet for Linux. 100% Rust — no web
runtime, no JavaScript, one binary.

## Stack

- **[IronCalc](https://github.com/ironcalc/IronCalc)** — spreadsheet engine:
  xlsx read/write, Excel formula language, dependency-graph recalculation,
  undo/redo history.
- **[egui](https://github.com/emilk/egui)** (via eframe) — GPU-rendered
  immediate-mode GUI. The grid paints only visible cells, so large sheets
  scroll smoothly.
- **rfd** — native file dialogs. **arboard** — clipboard.

## Build & run

```bash
cargo run --release [file.xlsx]
```

Generate a demo workbook:

```bash
cargo run --example make_demo demo.xlsx
```

## Keyboard shortcuts

Excel defaults, fully customizable. Bindings live in [keymap.toml](keymap.toml)
mapping chords to command ids:

```toml
[bindings]
"Ctrl+D" = "fill_down"
"F2" = "edit_cell"
```

Lookup order (first hit wins): `$SHEETZ_KEYMAP`, `./keymap.toml`,
`~/.config/sheetz/keymap.toml`, then the built-in copy. Edit and restart.
Command ids are the `from_id` names in [src/commands.rs](src/commands.rs).

Implemented Excel behaviors: arrow/Tab/Enter navigation, Ctrl+Arrow data-edge
jumps, Shift extends selection, F2/typing edits (typing replaces), Delete
clears, Ctrl+D/Ctrl+R fill with relative-reference adjustment, Ctrl+Z/Y
undo/redo, TSV copy/paste, Ctrl+PageUp/Down sheet switching, ribbon UI,
formula bar, unsaved-changes guard.

## Known v0.1 limitations

- Cell styling (colors, borders, fonts) is not yet rendered or editable;
  IronCalc preserves what the file already has on save.
- Arrow keys always move the caret while editing (Excel distinguishes
  Enter-mode from Edit-mode).
- Copy places display values on the clipboard, not formulas.
- No column/row resize by dragging, frozen panes, merged-cell rendering,
  charts, or pivot tables yet.
- IronCalc is v0.x: most common functions work; exotic ones may not.
