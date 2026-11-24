use crate::colors::{BACKGROUND_COLOR, lane_color};
use eframe::emath::Rect;
use eframe::epaint::{CornerRadius, Stroke};
use egui::{Color32, FontId, Painter, Pos2, StrokeKind, Vec2, vec2};
use groves_core::{Edge, Pill, PillStyle, Row};
use std::collections::HashSet;
use std::string::ToString;

pub const ROW_HEIGHT_PX: f32 = 22.0;
pub const LANE_WIDTH_PX: f32 = 14.0;
const SYNCED_ICON: &str = "🔘";
const FOLDER_ICON: &str = "📁";
const CHAR_BUDGET: usize = 20;
const MARKER_SIZE: usize = 9;
const PLUS_MARKER_SIZE: usize = 3;

const PILL_FONT:FontId = FontId::monospace(14.0);

pub struct PaneMarkerContext {
    pinned_head_rows: HashSet<String>,
    hovered_head_row: Option<String>,
}

impl PaneMarkerContext {
    pub fn new() -> Self {
        Self {
            pinned_head_rows: HashSet::new(),
            hovered_head_row: None,
        }
    }
    pub fn head_marker_to_show(&self) -> HashSet<String> {
        let mut res = self.pinned_head_rows.clone();
        if let Some(hovered) = self.hovered_head_row.as_ref() {
            res.insert(hovered.clone());
        };
        res
    }

    pub fn toggle_pinned_head(&mut self, marker_id: String) {
        if !self.pinned_head_rows.contains(&marker_id) {
            self.pinned_head_rows.insert(marker_id);
        } else {
            self.pinned_head_rows.remove(&marker_id);
        }
    }

    pub fn hover_marker(&mut self, marker_id: String) {
        self.hovered_head_row = Some(marker_id);
    }

    pub fn unhover_marker(&mut self) {
        self.hovered_head_row = None;
    }
}

impl Default for PaneMarkerContext {
    fn default() -> Self {
        Self::new()
    }
}

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

    if pill.style == PillStyle::Synced {
        branch_text = format!("{}{}", SYNCED_ICON, branch_text);
    }
    branch_text
}

fn draw_branch_pill(
    pill: &Pill,
    pos: Pos2,
    painter: &Painter,
    pill_col: Color32,
    text: &String,
) -> f32 {
    let mut text_col = BACKGROUND_COLOR;
    if pill.style == PillStyle::Outline {
        text_col = pill_col;
    }

    let galley = painter.layout_no_wrap(text.to_string(), PILL_FONT, text_col);

    let padding = vec2(8.0, 2.0);
    let mut size = galley.size() + padding;
    size.x = size.x.max(size.y);

    let rect = Rect::from_min_size(pos, size);
    let mut radius = CornerRadius::same(5.0 as u8);
    if pill.plus_count > 0 {
        radius = CornerRadius {
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
    pill_col: Color32,
) -> f32 {
    let text = format!("+{}", pill.plus_count);
    let mut text_col = BACKGROUND_COLOR;
    if pill.style == PillStyle::Outline {
        text_col = pill_col;
    }
    let galley = painter.layout_no_wrap(text.to_string(), PILL_FONT, text_col);

    let padding = vec2(8.0, 2.0);
    let mut size = galley.size() + padding;
    size.x = size.x.max(size.y);

    let p = pos + Vec2::new(2.0, 0.0);
    let rect = Rect::from_min_size(p, size);
    let radius = CornerRadius {
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

fn draw_head_pill(
    row: &Row,
    pos: Pos2,
    painter: &Painter,
    pill_col: Color32,
    marker_budget: usize,
) -> f32 {
    if let Some(main_marker) = row.head_markers.first() {
        let mut text = format!("{}{}", FOLDER_ICON, main_marker.name);

        text = shorten_text_to(text, marker_budget.saturating_sub(2));

        let galley = painter.layout_no_wrap(text.to_string(), PILL_FONT, BACKGROUND_COLOR);

        let mut size = galley.size() + vec2(8.0, 2.0);
        size.x = size.x.max(size.y);

        let p = pos + Vec2::new(2.0, 0.0);
        let rect = Rect::from_min_size(p, size);
        let radius = CornerRadius::same(5.0 as u8);

        painter.rect_filled(rect, radius, pill_col.gamma_multiply(0.6));

        let text_pos = rect.center() - galley.size() / 2.0;
        painter.galley(text_pos, galley, BACKGROUND_COLOR);
        return rect.width() + 2.0;
    }
    0.0
}

fn shorten_text_to(text: String, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.clone();
    }
    let t = text.chars().take(max_chars - 1).collect::<String>();
    format!("{}…", t)
}

pub fn draw_pills(
    row: &Row,
    pill: &Pill,
    pos: Pos2,
    painter: &Painter,
    pill_col: Color32,
    pane: &PaneMarkerContext,
) -> f32 {
    let has_head_marker = !row.head_markers.is_empty()
        && (row.head_markers.iter().any(|hm| hm.is_main)
            || pane.head_marker_to_show().contains(&row.id));
    let has_plus_marker = pill.plus_count > 0;
    let branch_text = generate_branch_pill_name(pill);
    let (branch_char_budget, marker_budget) = calculate_char_budget(
        has_head_marker,
        has_plus_marker,
        branch_text.chars().count(),
    );
    let final_branch_length = shorten_text_to(branch_text, branch_char_budget);

    let centered_pos = center_position_vertically(pos);
    let mut full_width = draw_branch_pill(
        pill,
        centered_pos,
        painter,
        pill_col,
        &final_branch_length,
    );

    if has_plus_marker {
        let plus_pos = Pos2::new(centered_pos.x + full_width, centered_pos.y);
        full_width += draw_plus_pill(pill, plus_pos, painter, pill_col)
    }
    if has_head_marker {
        let head_marker_pos = Pos2::new(centered_pos.x + full_width, centered_pos.y);
        full_width += draw_head_pill(
            row,
            head_marker_pos,
            painter,
            pill_col,
            marker_budget,
        );
    }
    full_width
}

fn center_position_vertically(pos: Pos2) -> Pos2 {
    let mut centered_pos = pos;
    centered_pos.y -= (ROW_HEIGHT_PX / 2.0) - 2.0;
    centered_pos
}

pub fn calculate_char_budget(
    has_head_marker: bool,
    has_plus_marker: bool,
    pill_name_length: usize,
) -> (usize, usize) {
    let mut branch_budget: usize = CHAR_BUDGET;
    let mut marker_budget: usize;
    if has_plus_marker {
        branch_budget -= PLUS_MARKER_SIZE;
    }
    if has_head_marker && branch_budget - MARKER_SIZE > pill_name_length {
        marker_budget = CHAR_BUDGET - pill_name_length;
        if has_plus_marker {
            marker_budget -= PLUS_MARKER_SIZE;
        }
        return (pill_name_length, marker_budget);
    }

    if has_head_marker {
        branch_budget -= MARKER_SIZE;
    }

    (branch_budget, MARKER_SIZE)
}

pub fn draw_pill_connector(pos: Pos2, dot_x: f32, painter: &Painter, color: Color32) {
    let st = Stroke::new(0.5, color);
    painter.line_segment([pos, Pos2::new(dot_x, pos.y)], st);
}

pub fn draw_dots(row: &Row, dot_pos: &Pos2, st: Stroke, painter: &Painter) {
    let color = if row.is_merge {
        lane_color(row.lane)
    } else {
        Color32::from_rgb(0x1E, 0x37, 0x3C)
    }; // #1e373c
    let radius = if row.is_merge { 2.0 } else { 3.5 };
    painter.circle(*dot_pos, radius, color, st);
}

pub fn draw_head_marker(row: &Row, pos: &Pos2, st: Stroke, painter: &Painter) {
    if row.head_markers.is_empty() {
        return;
    }
    let inner_color = Color32::from_rgb(0x1E, 0x37, 0x3C);

    painter.circle(*pos, 6.0, inner_color, st);
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
