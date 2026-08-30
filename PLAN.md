# Sheetz → Excel parity roadmap

Guiding principle: parity in the order people actually use Excel. Roughly 90%
of spreadsheet time is formatting, navigation, basic formulas, sort/filter,
and charts — that's v0.2–v0.4. Pivot tables and automation are the long tail.

Effort: **S** = a day-ish, **M** = a week-ish, **L** = multi-week.
"IC" notes what the IronCalc engine already provides vs. what we build.

## Architecture (done)

- [x] **Split the monolith** — `engine.rs` (all IronCalc access, plain-typed
      API), `state.rs` (app state), `app.rs` (update loop + command dispatch),
      `ui/{grid,ribbon,editor,dialogs,charts}.rs`. UI never imports ironcalc.
- [x] **Command registry** — ~100 named commands, every one bindable.
- [x] **Keymap** — full Excel default set in `keymap.toml`, plus an in-app
      shortcut viewer (F1).

## v0.2 — Looks like a spreadsheet (formatting & structure)

- [x] **Render cell styles** — bold/italic/underline/strike, font size, text
      and fill color, borders, horizontal + vertical alignment.
- [x] **Edit basic styles** — ribbon Home group with live toggle states.
- [x] **Number format UI** — General, currency, percent, comma, date,
      increase/decrease decimals.
- [x] **Insert/delete rows & columns** — with reference adjustment (IC).
- [x] **Autofit column width** — double-click or ribbon.
- [x] **Right-click context menu** — cut/copy/paste, insert/delete, clear,
      format.
- [x] **Sheet management** — new, rename (double-click tab), delete,
      duplicate, reorder.
- [x] **Clear** — contents / formats / all.
- [x] **Column/row resize by dragging** header edges, with resize cursors and
      double-click-to-autofit; header click/drag selects whole rows/columns.
- [x] **Merged cells** — existing merges render as one cell and select as a
      unit. *Creating* merges is blocked upstream: IronCalc exposes merged
      regions for reading only.
- [x] **Freeze panes** — frozen rows/columns painted as a pinned overlay with
      a stronger boundary line, and reported in the status bar.

## v0.3 — Edits like Excel (clipboard, fill, find)

- [x] **Formula-aware clipboard** — internal copy/paste keeps formulas, styles
      and adjusts relative references; cut marks the source; external apps
      still get TSV (IC).
- [x] **Paste values only** — Ctrl+Shift+V.
- [x] **Enter-mode vs Edit-mode** — typing then arrow commits and moves; F2
      then arrow moves the caret.
- [x] **Find & Replace** — Ctrl+F/H, match case, all-sheets, find next/prev,
      replace all.
- [x] **Undo/redo** — covers edits, styles, structure (IC history).
- [x] **Fill handle** — the drag-corner on the selection, with a preview
      outline; fills down/right through the engine's series logic.
- [x] **F4 reference cycling** — A1 → $A$1 → A$1 → $A1 while editing.
- [x] **Click-to-reference while editing a formula**, with each distinct
      reference in the formula given its own highlight color.

## v0.4 — Works with data (sort, filter, formulas UX)

- [x] **Sort** — ascending/descending on the selection or surrounding block,
      with header detection; plus a sort dialog.
- [x] **Named ranges** — name manager (add/list/delete).
- [x] **Go To** — Ctrl+G / F5 and a working Name Box.
- [x] **Selection stats** — Sum / Average / Count in the status bar.
- [x] **AutoFilter** — header dropdown buttons opening a searchable value
      checklist; non-matching rows are hidden.
- [x] **Formula autocomplete** — function-name popup while typing, Up/Down to
      choose, Tab/Enter to accept.
- [x] **Multi-key sort** — up to three levels, ascending/descending each,
      blanks last.
- [ ] **Data validation** — dropdown lists first (L)
- [ ] **Multi-range selection** — Ctrl+click disjoint ranges (M)

## v0.5 — Shows data (visual layer)

- [x] **Conditional formatting** — cell-value rules and color scales via a
      dialog; rendering comes through the engine's extended styles, so rules
      loaded from a real .xlsx paint correctly too.
- [x] **Charts (native)** — bar / line / pie drawn with the egui painter in
      floating windows over the sheet.
- [x] **Zoom** — Ctrl+scroll, ribbon buttons, status-bar readout.
- [x] **Toggle gridlines**.
- [x] **Data bars** — painted behind cell text from the engine's evaluated
      conditional formatting.
- [ ] **Charts in xlsx** — reading/writing chart XML so charts survive a round
      trip (L; native charts are view-only today)
- [ ] **Comments/notes** (M), **hyperlinks** (S)

## v0.6+ — The long tail

- [ ] **Pivot tables** — read existing first, then create/edit (L, multi-step)
- [ ] **Print / export PDF** — page setup, print areas (L)
- [ ] **Images in sheets** (M)
- [ ] **Workbook protection** (M)
- [ ] **Goal seek** (M)
- [ ] **Scripting** — Rhai, not VBA. Explicit non-goal until the above ships.

## Engine parity track (continuous, upstream)

IronCalc implements a large subset of Excel's ~500 functions. As we hit gaps
(XLOOKUP, dynamic arrays/spill, LAMBDA, …):

1. Log the missing function in an issue here,
2. Implement it **upstream in IronCalc** and pull the release — everyone wins
   and we don't fork the engine.

Same for xlsx fidelity bugs and for the missing merge-cells write API.

## App infrastructure track (continuous)

- [x] Unit tests for A1 parsing, column names, jump-to-edge, replace; xlsx
      round-trip integration test.
- [ ] Per-frame cell format cache (the grid currently queries the engine per
      visible cell per frame) (S)
- [ ] Packaging: `.desktop` file + icon, AUR PKGBUILD, release CI (S/M)
- [ ] Chorded/multi-key bindings in the keymap (S)

## Explicit non-goals

VBA/macro compatibility, Power Query, real-time collaboration, cloud sync,
add-in ecosystem. These are Microsoft-ecosystem moats, not spreadsheet
features; chasing them kills the project. KISS.
