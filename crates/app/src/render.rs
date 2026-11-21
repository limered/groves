use crate::colors::lane_color;
use eframe::epaint::{CornerRadius, Stroke};
use egui::{Color32, FontId, Painter, Pos2, StrokeKind, Vec2, vec2};
use groves_core::{Edge, Pill, PillStyle, Row};
use std::ops::Add;
use eframe::emath::Rect;

pub const ROW_HEIGHT_PX: f32 = 22.0;
pub const LANE_WIDTH_PX: f32 = 14.0;

pub fn graph_width_px(lane_count: usize) -> f32 {
    lane_count as f32 * LANE_WIDTH_PX
}

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

fn generate_branch_pill_name(pill: &Pill) -> String {
    let pill_text_parts: Vec<&str> = pill.name.split('/').collect();
    let mut branch_text = pill_text_parts.last().unwrap().to_string();
    if pill_text_parts.len() > 1 {
        let branch_type = pill_text_parts[pill_text_parts.len().saturating_sub(2)]
            .char_indices()
            .next()
            .unwrap();
        branch_text = format!("{0}.{1}", branch_type.1.to_uppercase(), branch_text);
    }
    if branch_text.chars().count() > 12 {
        branch_text = branch_text.chars().take(12).collect::<String>();
        branch_text = format!("{}…", branch_text);
    }
    if pill.style == PillStyle::Synced {
        branch_text = format!("⇅ {}", branch_text);
    }
    branch_text
}

fn draw_branch_pill(
    pill: &Pill,
    pos: Pos2,
    painter: &Painter,
    font: FontId,
    pill_col: Color32,
    text: &String,
) -> f32 {
    let padding = vec2(8.0, 2.0);

    let mut text_col = Color32::from_rgb(0x38, 0x23, 0x2F); // #38232f
    if pill.style == PillStyle::Outline {
        text_col = pill_col;
    }

    let galley = painter.layout_no_wrap(text.to_string(), font, text_col);

    let mut size = galley.size() + padding;
    size.x = size.x.max(size.y);

    let rect = egui::Rect::from_min_size(pos, size);
    let mut radius = egui::CornerRadius::same(5.0 as u8);
    if pill.plus_count > 0 {
        radius = egui::CornerRadius {
            ne: 0.0 as u8,
            se: 0.0 as u8,
            nw: 5.0 as u8,
            sw: 5.0 as u8,
        };
    }

    paint_pill(pill, painter, pill_col, rect, radius);

    let text_pos = rect.center() - galley.size() / 2.0;
    painter.galley(text_pos, galley, text_col);
    rect.width()
}
fn draw_plus_pill(
    pill: &Pill,
    pos: Pos2,
    painter: &Painter,
    font: FontId,
    pill_col: Color32,
) -> f32 {
    let padding = vec2(8.0, 2.0);

    let text = format!("+{}", pill.plus_count);
    let mut text_col = Color32::from_rgb(0x38, 0x23, 0x2F); // #38232f
    if pill.style == PillStyle::Outline {
        text_col = pill_col;
    }
    let galley = painter.layout_no_wrap(text.to_string(), font, text_col);

    let mut size = galley.size() + padding;
    size.x = size.x.max(size.y);

    let p = pos.clone() + Vec2::new(2.0, 0.0);
    let rect = egui::Rect::from_min_size(p, size);
    let radius = egui::CornerRadius {
        ne: 5.0 as u8,
        se: 5.0 as u8,
        nw: 0.0 as u8,
        sw: 0.0 as u8,
    };

    paint_pill(pill, painter, pill_col, rect, radius);

    let text_pos = rect.center() - galley.size() / 2.0;
    painter.galley(text_pos, galley, text_col);
    rect.width() + 2.0
}

fn paint_pill(pill: &Pill, painter: &Painter, pill_col: Color32, rect: Rect, radius: CornerRadius) {
    if pill.style == PillStyle::Filled || pill.style == PillStyle::Synced {
        painter.rect_filled(rect, radius, pill_col);
    } else {
        let stroke = Stroke::new(1.0, pill_col);
        let stroke_kind = StrokeKind::Middle;
        painter.rect_stroke(rect, radius, stroke, stroke_kind);
    }
}

fn draw_head_pill() -> f32 {
    0.0
}

pub fn draw_pills(
    pill: &Pill,
    pos: Pos2,
    painter: &Painter,
    font: FontId,
    pill_col: Color32,
) -> f32 {
    let mut centered_pos = pos;
    centered_pos.y -= (ROW_HEIGHT_PX / 2.0) - 2.0;

    let branch_text = generate_branch_pill_name(pill);
    let mut full_width = draw_branch_pill(
        pill,
        centered_pos,
        painter,
        font.clone(),
        pill_col,
        &branch_text,
    );

    if pill.plus_count > 0 {
        centered_pos.x += full_width;
        full_width += draw_plus_pill(pill, centered_pos, painter, font.clone(), pill_col)
    }

    full_width
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

pub fn draw_head_marker(row: &Row, pos: Pos2, st: Stroke, painter: &Painter) {
    if row.head_markers.is_empty() {
        return;
    }
    let inner_color = Color32::from_rgb(0x1E, 0x37, 0x3C);

    painter.circle(pos, 6.0, inner_color, st);
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
