//! Pure drawing geometry for a pane: edge routing, widths, coordinates.
//!
//! Ticket 03b: `route_edge` — turning a pane edge span into per-row-gap
//! waypoints — goes here in this module.

use egui::{Color32, FontId, Painter, Pos2, vec2};
use groves_core::{Edge, Pill, ROW_HEIGHT_PX};

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

pub fn draw_pill(pill: &Pill, pos: Pos2, painter: &Painter, font: FontId, col: Color32) {
    let mut centered_pos = pos;
    centered_pos.y -= ROW_HEIGHT_PX / 2.0;
    let padding = vec2(8.0, 2.0);
    let bg = Color32::from_rgb(50, 90, 150);

    let start = pill.name.char_indices()
        .rev()
        .nth(10)
        .map_or(0, |(i, _)| i);

    let pill_text = &pill.name[start..];
    let galley = painter.layout_no_wrap(pill_text.to_string(), font.clone(), col);

    let mut size = galley.size() + padding * 2.0;
    size.x = size.x.max(size.y);

    let rect = egui::Rect::from_min_size(centered_pos, size);
    let radius = egui::CornerRadius::same((rect.height() / 2.0).round() as u8);

    painter.rect_filled(rect, radius, bg);

    let text_pos = rect.center() - galley.size() / 2.0;
    painter.galley(text_pos, galley, col);
}
