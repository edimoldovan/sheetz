//! Vector ribbon icons.
//!
//! Unicode symbol coverage varies wildly between systems — several glyphs the
//! ribbon wanted rendered as empty boxes. These are drawn with the painter
//! instead, so they look the same everywhere and need no font at all.
//!
//! `draw` returns false for names it doesn't know, letting the caller fall
//! back to rendering the name as text (used for "B", "I", "U", "$", "%", …).

use eframe::egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, Vec2};

/// Draws `name` centered in `rect`. Returns false if there is no such icon.
pub fn draw(painter: &Painter, rect: Rect, name: &str, color: Color32) -> bool {
    // Work in a square box so icons keep their proportions.
    let side = rect.width().min(rect.height());
    let box_rect = Rect::from_center_size(rect.center(), Vec2::splat(side));
    let p = |x: f32, y: f32| -> Pos2 {
        Pos2::new(
            box_rect.min.x + x * box_rect.width(),
            box_rect.min.y + y * box_rect.height(),
        )
    };
    let thin = Stroke::new((side * 0.075).max(1.0), color);
    let thick = Stroke::new((side * 0.11).max(1.3), color);

    let line = |a: (f32, f32), b: (f32, f32)| {
        painter.line_segment([p(a.0, a.1), p(b.0, b.1)], thin);
    };
    let rect_outline = |x0: f32, y0: f32, x1: f32, y1: f32| {
        painter.add(Shape::closed_line(
            vec![p(x0, y0), p(x1, y0), p(x1, y1), p(x0, y1)],
            thin,
        ));
    };
    let rect_filled = |x0: f32, y0: f32, x1: f32, y1: f32| {
        painter.rect_filled(Rect::from_min_max(p(x0, y0), p(x1, y1)), 0.0, color);
    };
    // Arrowhead pointing in a cardinal direction, centered on (x, y).
    let arrow_head = |x: f32, y: f32, dx: f32, dy: f32, size: f32| {
        let tip = p(x, y);
        let back = Vec2::new(-dx, -dy) * size * box_rect.width();
        let side_v = Vec2::new(-dy, dx) * size * 0.62 * box_rect.width();
        painter.add(Shape::convex_polygon(
            vec![tip, tip + back + side_v, tip + back - side_v],
            color,
            Stroke::NONE,
        ));
    };

    match name {
        // ----- clipboard -----
        "cut" => {
            line((0.28, 0.12), (0.66, 0.66));
            line((0.72, 0.12), (0.34, 0.66));
            painter.circle_stroke(p(0.30, 0.80), side * 0.11, thin);
            painter.circle_stroke(p(0.70, 0.80), side * 0.11, thin);
        }
        "copy" => {
            rect_outline(0.16, 0.10, 0.62, 0.68);
            rect_outline(0.38, 0.32, 0.84, 0.90);
        }
        "paste" => {
            rect_outline(0.20, 0.18, 0.80, 0.90);
            rect_filled(0.36, 0.08, 0.64, 0.22);
        }

        // ----- alignment -----
        "align-left" | "align-center" | "align-right" => {
            let rows = [0.22f32, 0.40, 0.58, 0.76];
            for (i, y) in rows.iter().enumerate() {
                let full = i % 2 == 0;
                let w = if full { 0.68 } else { 0.44 };
                let (x0, x1) = match name {
                    "align-left" => (0.16, 0.16 + w),
                    "align-right" => (0.84 - w, 0.84),
                    _ => (0.5 - w / 2.0, 0.5 + w / 2.0),
                };
                painter.line_segment(
                    [p(x0, *y), p(x1, *y)],
                    Stroke::new((side * 0.085).max(1.0), color),
                );
            }
        }
        "wrap" => {
            line((0.16, 0.28), (0.84, 0.28));
            line((0.16, 0.52), (0.68, 0.52));
            painter.add(Shape::line(
                vec![p(0.68, 0.52), p(0.80, 0.52), p(0.80, 0.72), p(0.40, 0.72)],
                thin,
            ));
            arrow_head(0.34, 0.72, -1.0, 0.0, 0.13);
        }

        // ----- fill / clear -----
        "fill" => {
            line((0.5, 0.12), (0.5, 0.66));
            arrow_head(0.5, 0.80, 0.0, 1.0, 0.17);
            line((0.20, 0.92), (0.80, 0.92));
        }
        "clear" => {
            // Eraser: a rounded block with a wipe stroke.
            painter.rect_stroke(
                Rect::from_min_max(p(0.16, 0.30), p(0.72, 0.70)),
                eframe::egui::CornerRadius::same(2),
                thin,
                eframe::egui::StrokeKind::Inside,
            );
            line((0.60, 0.22), (0.88, 0.50));
            line((0.76, 0.78), (0.90, 0.92));
            line((0.90, 0.78), (0.76, 0.92));
        }

        // ----- structure -----
        "insert" => {
            rect_outline(0.12, 0.12, 0.88, 0.88);
            line((0.12, 0.50), (0.88, 0.50));
            line((0.50, 0.62), (0.50, 0.86));
            line((0.38, 0.74), (0.62, 0.74));
        }
        "delete" => {
            rect_outline(0.12, 0.12, 0.88, 0.88);
            line((0.12, 0.50), (0.88, 0.50));
            line((0.36, 0.70), (0.64, 0.70));
        }
        "resize" => {
            line((0.16, 0.50), (0.84, 0.50));
            arrow_head(0.10, 0.50, -1.0, 0.0, 0.16);
            arrow_head(0.90, 0.50, 1.0, 0.0, 0.16);
            line((0.16, 0.20), (0.16, 0.80));
            line((0.84, 0.20), (0.84, 0.80));
        }

        // ----- data -----
        "sort" => {
            // Two opposed arrows, Excel's sort idiom.
            line((0.32, 0.86), (0.32, 0.24));
            arrow_head(0.32, 0.12, 0.0, -1.0, 0.16);
            line((0.68, 0.14), (0.68, 0.76));
            arrow_head(0.68, 0.88, 0.0, 1.0, 0.16);
        }
        "sort-asc" | "sort-desc" => {
            let down = name == "sort-desc";
            let widths = [0.30f32, 0.44, 0.58];
            for (i, w) in widths.iter().enumerate() {
                let y = 0.24 + i as f32 * 0.24;
                let w = if down { widths[2 - i] } else { *w };
                painter.line_segment([p(0.14, y), p(0.14 + w, y)], thin);
            }
        }
        "filter" => {
            painter.add(Shape::convex_polygon(
                vec![p(0.12, 0.18), p(0.88, 0.18), p(0.58, 0.52), p(0.42, 0.52)],
                color,
                Stroke::NONE,
            ));
            painter.add(Shape::convex_polygon(
                vec![p(0.42, 0.56), p(0.58, 0.56), p(0.55, 0.88), p(0.45, 0.80)],
                color,
                Stroke::NONE,
            ));
        }
        "find" => {
            painter.circle_stroke(p(0.44, 0.44), side * 0.26, thin);
            painter.line_segment([p(0.63, 0.63), p(0.86, 0.86)], thick);
        }
        "name" => {
            painter.add(Shape::closed_line(
                vec![p(0.10, 0.30), p(0.62, 0.30), p(0.88, 0.50), p(0.62, 0.70), p(0.10, 0.70)],
                thin,
            ));
            painter.circle_filled(p(0.26, 0.50), side * 0.06, color);
        }

        // ----- styles -----
        "cond-fmt" => {
            rect_outline(0.12, 0.12, 0.88, 0.88);
            rect_filled(0.14, 0.14, 0.86, 0.36);
            line((0.12, 0.60), (0.88, 0.60));
            line((0.50, 0.36), (0.50, 0.88));
        }
        "chart" => {
            line((0.14, 0.12), (0.14, 0.86));
            line((0.14, 0.86), (0.90, 0.86));
            rect_filled(0.26, 0.52, 0.42, 0.86);
            rect_filled(0.48, 0.30, 0.64, 0.86);
            rect_filled(0.70, 0.44, 0.86, 0.86);
        }

        // ----- view -----
        "freeze" => {
            rect_outline(0.12, 0.12, 0.88, 0.88);
            painter.line_segment([p(0.12, 0.42), p(0.88, 0.42)], thick);
            painter.line_segment([p(0.42, 0.12), p(0.42, 0.88)], thick);
        }
        "freeze-top" => {
            rect_outline(0.12, 0.12, 0.88, 0.88);
            rect_filled(0.14, 0.14, 0.86, 0.36);
            painter.line_segment([p(0.12, 0.38), p(0.88, 0.38)], thick);
        }
        "freeze-first" => {
            rect_outline(0.12, 0.12, 0.88, 0.88);
            rect_filled(0.14, 0.14, 0.36, 0.86);
            painter.line_segment([p(0.38, 0.12), p(0.38, 0.88)], thick);
        }
        "unfreeze" => {
            rect_outline(0.12, 0.12, 0.88, 0.88);
            line((0.24, 0.24), (0.76, 0.76));
            line((0.76, 0.24), (0.24, 0.76));
        }
        "gridlines" => {
            rect_outline(0.12, 0.12, 0.88, 0.88);
            line((0.12, 0.37), (0.88, 0.37));
            line((0.12, 0.63), (0.88, 0.63));
            line((0.37, 0.12), (0.37, 0.88));
            line((0.63, 0.12), (0.63, 0.88));
        }
        "zoom-in" | "zoom-out" => {
            painter.circle_stroke(p(0.44, 0.44), side * 0.26, thin);
            painter.line_segment([p(0.63, 0.63), p(0.86, 0.86)], thick);
            line((0.30, 0.44), (0.58, 0.44));
            if name == "zoom-in" {
                line((0.44, 0.30), (0.44, 0.58));
            }
        }
        "zoom-reset" => {
            painter.circle_stroke(p(0.50, 0.52), side * 0.30, thin);
            arrow_head(0.50, 0.20, 1.0, 0.0, 0.15);
        }

        // ----- sheets -----
        "sheet-new" => {
            rect_outline(0.14, 0.10, 0.70, 0.90);
            line((0.78, 0.52), (0.78, 0.86));
            line((0.62, 0.69), (0.94, 0.69));
        }
        "duplicate" => {
            rect_outline(0.14, 0.10, 0.58, 0.70);
            rect_outline(0.40, 0.30, 0.86, 0.90);
        }
        "rename" => {
            painter.add(Shape::closed_line(
                vec![p(0.18, 0.72), p(0.66, 0.14), p(0.84, 0.30), p(0.36, 0.88)],
                thin,
            ));
            line((0.18, 0.72), (0.36, 0.88));
        }
        "trash" => {
            line((0.16, 0.24), (0.84, 0.24));
            line((0.40, 0.24), (0.40, 0.14));
            line((0.60, 0.24), (0.60, 0.14));
            line((0.40, 0.14), (0.60, 0.14));
            painter.add(Shape::closed_line(
                vec![p(0.24, 0.30), p(0.76, 0.30), p(0.68, 0.90), p(0.32, 0.90)],
                thin,
            ));
        }
        "prev" | "next" => {
            let dir = if name == "next" { 1.0 } else { -1.0 };
            let x = if name == "next" { 0.72 } else { 0.28 };
            arrow_head(x, 0.50, dir, 0.0, 0.26);
            line((0.5 - dir * 0.34, 0.50), (0.5 + dir * 0.10, 0.50));
        }

        // ----- history / file -----
        "undo" | "redo" => {
            let mirror = name == "redo";
            let fx = |x: f32| if mirror { 1.0 - x } else { x };
            painter.add(Shape::line(
                vec![
                    p(fx(0.20), 0.34),
                    p(fx(0.58), 0.34),
                    p(fx(0.76), 0.52),
                    p(fx(0.62), 0.80),
                ],
                thin,
            ));
            arrow_head(fx(0.20), 0.34, if mirror { 1.0 } else { -1.0 }, 0.0, 0.17);
        }
        "save" => {
            rect_outline(0.14, 0.14, 0.86, 0.86);
            rect_filled(0.32, 0.14, 0.68, 0.40);
            rect_outline(0.28, 0.56, 0.72, 0.86);
        }
        "open" => {
            painter.add(Shape::closed_line(
                vec![p(0.10, 0.78), p(0.10, 0.26), p(0.40, 0.26), p(0.50, 0.38), p(0.84, 0.38), p(0.84, 0.78)],
                thin,
            ));
        }
        "fill-color" => {
            // Paint bucket: a tilted body with a drip.
            painter.add(Shape::closed_line(
                vec![p(0.16, 0.52), p(0.50, 0.18), p(0.84, 0.52), p(0.50, 0.86)],
                thin,
            ));
            line((0.50, 0.10), (0.50, 0.24));
            painter.circle_filled(p(0.86, 0.74), side * 0.08, color);
        }
        "new-doc" => {
            painter.add(Shape::closed_line(
                vec![p(0.22, 0.10), p(0.62, 0.10), p(0.80, 0.30), p(0.80, 0.90), p(0.22, 0.90)],
                thin,
            ));
            line((0.62, 0.10), (0.62, 0.30));
            line((0.62, 0.30), (0.80, 0.30));
        }
        _ => return false,
    }
    true
}
