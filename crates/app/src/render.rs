//! Pure drawing geometry for a pane: edge routing, widths, coordinates.
//!
//! Ticket 03b: `route_edge` — turning a pane edge span into per-row-gap
//! waypoints — goes here in this module.

use crate::colors::lane_color;
use eframe::epaint::Stroke;
use egui::{Color32, FontId, Painter, Pos2, StrokeKind, vec2};
use groves_core::{Edge, LANE_WIDTH_PX, Pill, PillStyle, ROW_HEIGHT_PX, Row};

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

    if branch_text.chars().count() > 12 {
        branch_text = branch_text.chars().take(12).collect::<String>();
        branch_text = format!("{}…", branch_text);
    }

    if pill.plus_count > 0 {
        branch_text = format!("{0}|+{1}", branch_text, pill.plus_count);
    }

    if pill.style == PillStyle::Synced {
        branch_text = format!("⇅{}", branch_text);
    }

    let mut text_col = Color32::from_rgb(0x38, 0x23, 0x2F); // #38232f
    if pill.style == PillStyle::Outline {
        text_col = pill_col;
    }

    let galley = painter.layout_no_wrap(branch_text.to_string(), font, text_col);

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

    let text_pos = rect.center() - galley.size() / 2.0;
    painter.galley(text_pos, galley, text_col);
    rect.width()
}

pub fn draw_pill_connector(pos: Pos2, dot_x: f32, painter: &Painter, color: Color32) {
    let st = Stroke::new(0.5, color);
    painter.line_segment([pos, Pos2::new(dot_x, pos.y)], st);
}

pub fn draw_dots(row: &Row, dot_pos: Pos2, st: Stroke, painter: &Painter) {
    let color = if row.is_merge {
        lane_color(row.lane)
    } else {
        Color32::from_rgb(0x1E, 0x37, 0x3C)
    }; // #1e373c
    let radius = if row.is_merge { 2.0 } else { 3.5 };
    painter.circle(dot_pos, radius, color, st);
}

pub fn render_edges(edge: &Edge, start: Pos2, end: Pos2, painter: &Painter) {
    let col_this = lane_color(edge.parent_lane);
    let st_this = Stroke::new(2.0, col_this);

    let col_other = lane_color(edge.child_lane);
    let st_other = Stroke::new(2.0, col_other);

    let dir = if edge.parent_lane > edge.child_lane {
        -1.0
    } else {
        1.0
    };

    if edge.parent_lane == edge.child_lane {
        painter.line_segment(
            [Pos2::new(start.x, start.y), Pos2::new(end.x, end.y)],
            st_this,
        );
    } else if start.x == end.x {
        painter.line_segment(
            [Pos2::new(start.x, start.y), Pos2::new(end.x, end.y)],
            if edge.is_first_parent {
                st_other
            } else {
                st_this
            },
        );
    } else if edge.is_first_parent {
        painter.line_segment(
            [
                Pos2::new(start.x, end.y - 1.0 * ROW_HEIGHT_PX),
                Pos2::new(start.x, end.y - 0.5 * ROW_HEIGHT_PX),
            ],
            st_other,
        );
        painter.add(egui::epaint::CubicBezierShape::from_points_stroke(
            [
                Pos2::new(start.x, end.y - 0.5 * ROW_HEIGHT_PX),
                Pos2::new(start.x, end.y - 0.2 * ROW_HEIGHT_PX),
                Pos2::new(start.x - 0.2 * dir * LANE_WIDTH_PX, end.y),
                Pos2::new(start.x - 0.5 * dir * LANE_WIDTH_PX, end.y),
            ],
            false,
            Color32::TRANSPARENT,
            st_other,
        ));
        painter.line_segment(
            [
                Pos2::new(start.x - 0.5 * dir * LANE_WIDTH_PX, end.y),
                Pos2::new(end.x, end.y),
            ],
            st_other,
        );
    } else {
        painter.line_segment(
            [
                Pos2::new(start.x, start.y),
                Pos2::new(end.x + 0.5 * dir * LANE_WIDTH_PX, start.y),
            ],
            st_this,
        );
        painter.add(egui::epaint::CubicBezierShape::from_points_stroke(
            [
                Pos2::new(end.x + 0.5 * dir * LANE_WIDTH_PX, start.y),
                Pos2::new(end.x - 0.1 * dir * LANE_WIDTH_PX, start.y),
                Pos2::new(end.x, end.y - 0.4 * ROW_HEIGHT_PX),
                Pos2::new(end.x, end.y - 0.5 * ROW_HEIGHT_PX),
            ],
            false,
            Color32::TRANSPARENT,
            st_this,
        ));

        painter.line_segment(
            [
                Pos2::new(end.x, end.y - 0.5 * ROW_HEIGHT_PX),
                Pos2::new(end.x, end.y + 0.5 * ROW_HEIGHT_PX),
            ],
            st_this,
        );
    }
}
