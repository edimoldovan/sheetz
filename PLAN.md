# Sheetz → Excel parity roadmap

Guiding principle: parity in the order people actually use Excel. Roughly 90%
of spreadsheet time is formatting, navigation, basic formulas, sort/filter,
and charts — that's v0.2–v0.4. Pivot tables and automation are the long tail.

Effort: **S** = a day-ish, **M** = a week-ish, **L** = multi-week.
"IC" notes what the IronCalc engine already provides vs. what we build.

## v0.2 — Looks like a spreadsheet (formatting & structure)

The single biggest visible gap: styles exist in the file but aren't rendered.

- [ ] **Render cell styles** — bold/italic/underline, font size, text/fill
      color, borders, alignment. IC: styles are parsed and stored; we read
      `get_cell_style` and paint. (M)
- [ ] **Edit basic styles** — ribbon Home additions: bold/italic/underline,
      fill color, text color, borders, align left/center/right. IC: has
      `update_range_style`. (M)
- [ ] **Number format UI** — General, currency, percent, comma, date,
      decimals +/-. IC: formats already applied on display; we just set
      `num_fmt`. (S)
- [ ] **Column/row resize** — drag header edges; double-click to auto-fit.
      IC: `set_column_width`/`set_row_height` exist. (S/M)
- [ ] **Insert/delete rows & columns** — with reference adjustment. IC:
      `insert_rows`/`delete_rows`/`insert_columns`/`delete_columns`. (S)
- [ ] **Right-click context menu** — cut/copy/paste, insert/delete,
      clear, format. (S)
- [ ] **Merged cells** — render existing merges correctly; add
      merge/unmerge. IC: stores merges. (M)
- [ ] **Freeze panes** — render frozen rows/cols; toggle from ribbon View
      group. IC: `get_frozen_rows_count` etc. exist; grid needs split
      rendering. (M)
- [ ] **Sheet management** — rename (double-click tab), delete, reorder,
      tab colors. IC: all supported. (S)

## v0.3 — Edits like Excel (clipboard, fill, find)

- [ ] **Formula-aware clipboard** — internal copy/paste keeps formulas and
      adjusts relative references; cut-paste moves them. IC:
      `copy_to_clipboard`/`paste_from_clipboard` do exactly this. External
      apps still get TSV. (M)
- [ ] **Fill handle** — the little drag-corner on the selection; smart
      series (1,2,3…, Mon,Tue…, drag formulas). IC: `auto_fill_*` already
      used for Ctrl+D/R. (M)
- [ ] **Find & Replace** — Ctrl+F/Ctrl+H dialog, within sheet/workbook. (M)
- [ ] **Enter-mode vs Edit-mode** — Excel's two editing modes: after typing,
      arrows commit and move; after F2, arrows move the caret. (S)
- [ ] **F4 cycling** — toggle $A$1 / A$1 / $A1 / A1 while editing. (S)
- [ ] **Click-to-reference** — while editing a formula, clicking cells
      inserts their reference; referenced ranges get colored outlines. (M)
- [ ] **Undo/redo polish** — cover style edits, sheet ops, paste. IC:
      history covers most; verify and fill gaps. (S)

## v0.4 — Works with data (sort, filter, formulas UX)

- [ ] **Sort** — by column, ascending/descending, multi-key dialog,
      preserve-rows semantics. (M)
- [ ] **AutoFilter** — header dropdowns with value checklists like Excel;
      Ctrl+Shift+L toggle. IC: has table/filter model support to verify. (L)
- [ ] **Formula autocomplete** — function name suggestions + argument
      tooltip while typing. IC: exposes completion info API. (M)
- [ ] **Named ranges** — define/manage, usable in formulas. IC: supported
      in the parser. (M)
- [ ] **Data validation** — dropdown lists first (most used), then rules. (L)
- [ ] **Multi-range selection** — Ctrl+click disjoint ranges; header
      click/drag selects whole rows/cols. (M)

## v0.5 — Shows data (visual layer)

- [ ] **Conditional formatting** — color scales, data bars, cell rules.
      IC: `evaluate_conditional_formatting` exists — render its output. (M)
- [ ] **Charts** — bar/line/pie/scatter rendered natively (egui paints
      them fine); read/write xlsx chart XML. This is the hardest
      file-format work in the plan. (L)
- [ ] **Comments/notes** — read/write, hover popups. (M)
- [ ] **Hyperlinks** — render, click-to-open, insert. (S)
- [ ] **Zoom** — Ctrl+scroll, status-bar slider. (S)

## v0.6+ — The long tail

- [ ] **Pivot tables** — read existing (render cached results), then
      create/edit. Massive; do read-only first. (L, multi-milestone)
- [ ] **Print / export PDF** — page setup, print areas, headers/footers. (L)
- [ ] **Images in sheets** — render embedded pictures; insert. (M)
- [ ] **Workbook protection** — sheet/cell locking, password open. (M)
- [ ] **Goal seek** — simple solver. (M)
- [ ] **Scripting** — NOT VBA. If automation is ever wanted: embed a Rust
      scripting language (Rhai) with a spreadsheet API. Explicit non-goal
      until everything above ships. (L)

## Engine parity track (continuous, upstream)

IronCalc implements ~200 of Excel's ~500 functions — the common ones. As we
hit gaps (XLOOKUP, dynamic arrays/spill, LAMBDA, etc.):

1. Log the missing function in an issue here,
2. Implement it **upstream in IronCalc** and pull the release — everyone
   wins and we don't fork the engine.

Same for xlsx fidelity bugs (styles lost on save, exotic format codes).

## App infrastructure track (continuous)

- [ ] Per-frame value cache (don't call the engine per visible cell per
      frame once styling lands) (S)
- [ ] Headless UI tests for commands/keymap; golden-file xlsx tests (M)
- [ ] Packaging: `.desktop` file + icon, AUR PKGBUILD, release CI (S/M)
- [ ] Keymap: chorded/multi-key bindings, in-app keymap viewer (S)

## Explicit non-goals

VBA/macro compatibility, Power Query, real-time collaboration, cloud sync,
add-in ecosystem. These are Microsoft-ecosystem moats, not spreadsheet
features; chasing them kills the project. KISS.
