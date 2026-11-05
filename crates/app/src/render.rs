//! Pure drawing geometry for a pane: edge routing, widths, coordinates.
//!
//! Ticket 03b: `route_edge` — turning a pane edge span into per-row-gap
//! waypoints — goes here in this module.

use eframe::epaint::Stroke;
use egui::{Color32, FontId, Painter, Pos2, StrokeKind, vec2};
use groves_core::{Edge, Pill, PillStyle, ROW_HEIGHT_PX};

pub fn route_edge(edge: &Edge) -> Vec<(usize, usize)> {
    if edge.parent_lane == edge.child_lane {
        return vec![
            (edge.child_row, edge.child_lane),
            (edge.parent_row, edge.parent_lane),
        ];
    }
    let row_diff = edge.parent_row.saturating_sub(edge.child_row);
    let mut points: Vec<(usize, usize)> = vec![(edge.child_row, edge.child_lane)];
    if edge.is_first_parent && row_diff > 1 {
        points.push((edge.parent_row - 1, edge.child_lane));
    } else if row_diff > 1 {
        points.push((edge.child_row + 1, edge.parent_lane));
    }

    points.push((edge.parent_row, edge.parent_lane));
    points
}

pub fn draw_pill(
    pill: &Pill,
    pos: Pos2,
    painter: &Painter,
    font: FontId,
    pill_col: Color32,
) -> f32 {
    let mut centered_pos = pos;
    centered_pos.y -= ROW_HEIGHT_PX / 2.0;
    let padding = vec2(8.0, 2.0);

    let pill_text_parts: Vec<&str> = pill.name.split('/').collect();
    let mut branch_text =
        pill_text_parts[pill_text_parts.len().saturating_sub(1)..pill_text_parts.len()].join("/");
    if pill_text_parts.len() > 1 {
        let branch_type = pill_text_parts[pill_text_parts.len().saturating_sub(2)]
            .char_indices()
            .next()
            .unwrap();
        branch_text = format!("{0}/{1}", branch_type.1.to_uppercase(), branch_text);
    }

    if branch_text.len() > 12 {
        branch_text = branch_text[0..12].to_string();
    }

    if pill.plus_count > 0 {
        branch_text = format!("{0}|+{1}", branch_text, pill.plus_count);
    }

    let mut text_col = Color32::from_rgb(0x38, 0x23, 0x2F); // #38232f
    if pill.style == PillStyle::Outline {
        text_col = pill_col;
    }

    let galley = painter.layout_no_wrap(branch_text.to_string(), font.clone(), text_col);

    let mut size = galley.size() + padding * 2.0;
    size.x = size.x.max(size.y);

    let rect = egui::Rect::from_min_size(centered_pos, size);
    let radius = egui::CornerRadius::same((rect.height() / 2.0).round() as u8);

    if pill.style == PillStyle::Filled || pill.style == PillStyle::Synced {
        painter.rect_filled(rect, radius, pill_col);
    } else {
        let stroke = Stroke::new(1.0, pill_col);
        let stroke_kind = StrokeKind::Middle;
        painter.rect_stroke(rect, radius, stroke, stroke_kind);
    }

    // if pill.style == PillStyle::Synced {
    //
    // }

    let text_pos = rect.center() - galley.size() / 2.0;
    painter.galley(text_pos, galley, text_col);
    rect.width()
}

pub fn draw_pill_connector(pos: Pos2, dot_x: f32, painter: &Painter, color: Color32) {
    let st = Stroke::new(0.5, color);
    painter.line_segment([pos, Pos2::new(dot_x, pos.y)], st);
}
