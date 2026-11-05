// Emil owns this file — ticket 01 will add the eframe app that opens a repo
// path from the CLI args and lists its commits (or shows the open error).

use std::path::Path;

use eframe::egui::{Sense, Vec2};
use egui::{Align2, Color32, FontId, Pos2, Stroke};
use groves_app::colors::lane_color;
use groves_app::render::{draw_pill, route_edge};
use groves_core::{LANE_WIDTH_PX, Pane, ROW_HEIGHT_PX};
use groves_repo::{Repo, RepoError, open};

const FG: Color32 = Color32::from_rgb(0xC0, 0xDA, 0xDE); //#c0dade
const WIDTH_PILL_AREA: f32 = 150.0;

struct GrovesApp {
    content: Result<Pane, String>,
}

impl eframe::App for GrovesApp {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            // ui.heading("Groves");
            // ui.separator();
            match &self.content {
                Ok(pane) => {
                    let row_height = ROW_HEIGHT_PX;
                    let row_height_without_spacing = row_height - ui.spacing().item_spacing.y;
                    let sa = egui::ScrollArea::vertical().auto_shrink(false);
                    sa.show_rows(
                        ui,
                        row_height_without_spacing,
                        pane.rows().len(),
                        |ui, row_range| {
                            let full = ui.available_rect_before_wrap();

                            let range = row_range.clone();
                            let (resp, painter) = ui.allocate_painter(
                                Vec2::new(full.width(), row_range.len() as f32 * row_height),
                                Sense::click(),
                            );
                            let bg_col = Color32::from_hex("#38232f").unwrap();
                            painter.rect_filled(resp.rect, 0.0, bg_col);

                            let top = resp.rect.top();
                            let left = resp.rect.left();

                            let font = FontId::monospace(13.0);

                            // edges
                            let range_start = row_range.start;
                            for edge in pane.edges().iter() {
                                if edge.parent_row <= range.start.saturating_sub(1)
                                    || edge.child_row >= row_range.end
                                {
                                    continue;
                                }
                                let col_this = lane_color(edge.parent_lane);
                                let st_this = Stroke::new(2.0, col_this);

                                let col_other = lane_color(edge.child_lane);
                                let st_other = Stroke::new(2.0, col_other);

                                let points = route_edge(edge);
                                for point_index in 1..points.len() {
                                    let (last_row, last_lane) = points[point_index - 1];
                                    let (row, lane) = points[point_index];

                                    let start_y = calc_y(top, range_start, last_row);
                                    let end_y = calc_y(top, range_start, row);
                                    let start_x = calc_x(left + WIDTH_PILL_AREA, 8.0, last_lane);
                                    let end_x = calc_x(left + WIDTH_PILL_AREA, 8.0, lane);

                                    let dir = if edge.parent_lane > edge.child_lane {
                                        -1.0
                                    } else {
                                        1.0
                                    };

                                    if edge.parent_lane == edge.child_lane {
                                        painter.line_segment(
                                            [Pos2::new(start_x, start_y), Pos2::new(end_x, end_y)],
                                            st_this,
                                        );
                                    } else if start_x == end_x{
                                        painter.line_segment(
                                            [
                                                Pos2::new(start_x, start_y),
                                                Pos2::new(end_x, end_y),
                                            ],
                                            if edge.is_first_parent { st_other } else { st_this },
                                        );
                                    }else if edge.is_first_parent {
                                        painter.line_segment(
                                            [
                                                Pos2::new(start_x, end_y - 1.0 * ROW_HEIGHT_PX),
                                                Pos2::new(start_x, end_y - 0.5 * ROW_HEIGHT_PX),
                                            ],
                                            st_other,
                                        );
                                        painter.add(
                                            egui::epaint::CubicBezierShape::from_points_stroke(
                                                [
                                                    Pos2::new(start_x, end_y - 0.5 * ROW_HEIGHT_PX),
                                                    Pos2::new(start_x, end_y - 0.2 * ROW_HEIGHT_PX),
                                                    Pos2::new(
                                                        start_x - 0.2 * dir * LANE_WIDTH_PX,
                                                        end_y,
                                                    ),
                                                    Pos2::new(
                                                        start_x - 0.5 * dir * LANE_WIDTH_PX,
                                                        end_y,
                                                    ),
                                                ],
                                                false,
                                                Color32::TRANSPARENT,
                                                st_other,
                                            ),
                                        );
                                        painter.line_segment(
                                            [
                                                Pos2::new(
                                                    start_x - 0.5 * dir * LANE_WIDTH_PX,
                                                    end_y,
                                                ),
                                                Pos2::new(end_x, end_y),
                                            ],
                                            st_other,
                                        );
                                    } else {
                                        painter.line_segment(
                                            [
                                                Pos2::new(start_x, start_y),
                                                Pos2::new(
                                                    end_x + 0.5 * dir * LANE_WIDTH_PX,
                                                    start_y,
                                                ),
                                            ],
                                            st_this,
                                        );
                                        painter.add(
                                            egui::epaint::CubicBezierShape::from_points_stroke(
                                                [
                                                    Pos2::new(
                                                        end_x + 0.5 * dir * LANE_WIDTH_PX,
                                                        start_y,
                                                    ),
                                                    Pos2::new(
                                                        end_x - 0.1 * dir * LANE_WIDTH_PX,
                                                        start_y,
                                                    ),
                                                    Pos2::new(end_x, end_y - 0.4 * ROW_HEIGHT_PX),
                                                    Pos2::new(end_x, end_y - 0.5 * ROW_HEIGHT_PX),
                                                ],
                                                false,
                                                Color32::TRANSPARENT,
                                                st_this,
                                            ),
                                        );

                                        painter.line_segment(
                                            [
                                                Pos2::new(end_x, end_y - 0.5 * ROW_HEIGHT_PX),
                                                Pos2::new(end_x, end_y + 0.5 * ROW_HEIGHT_PX),
                                            ],
                                            st_this,
                                        );
                                    }
                                }
                            }

                            // draw dots
                            for row_id in range.clone() {
                                let row = &pane.rows()[row_id];
                                let st = Stroke::new(2.0, lane_color(row.lane)); // #5c7855
                                let x =
                                    calc_x(left + WIDTH_PILL_AREA, 8.0, pane.rows()[row_id].lane);
                                let y = calc_y(top, range_start, row_id);

                                if pane.rows()[row_id].is_merge {
                                    painter.circle(
                                        Pos2::new(x, y),
                                        2.0,
                                        lane_color(row.lane),
                                        st,
                                    );
                                } else {
                                    painter.circle(
                                        Pos2::new(x, y),
                                        3.5,
                                        Color32::from_rgb(0x1E, 0x37, 0x3C), // #1e373c
                                        st,
                                    );
                                }
                            }

                            // draw texts
                            let sub_rect = egui::Rect::from_min_size(
                                Pos2::ZERO,
                                Vec2::new(WIDTH_PILL_AREA, resp.rect.height()),
                            );
                            let pill_painter = painter.with_clip_rect(sub_rect);
                            for row_id in range.clone() {
                                let row = &pane.rows()[row_id];
                                if let Some(pill) = &row.pill {
                                    let pill_pos = Pos2::new(
                                        calc_x(left, 8.0, 0),
                                        calc_y(top, range_start, row_id),
                                    );

                                    draw_pill(
                                        pill,
                                        pill_pos,
                                        &pill_painter,
                                        font.clone(),
                                        lane_color(row.lane),
                                    );
                                }

                                painter.text(
                                    Pos2::new(
                                        calc_x(
                                            left + WIDTH_PILL_AREA,
                                            16.0 + pane.graph_width_px(),
                                            0,
                                        ),
                                        calc_y(top, range.start, row_id),
                                    ),
                                    Align2::LEFT_CENTER,
                                    &row.short,
                                    font.clone(),
                                    FG,
                                );
                                painter.text(
                                    Pos2::new(
                                        calc_x(
                                            left + WIDTH_PILL_AREA,
                                            16.0 + pane.graph_width_px() + 5.0 * LANE_WIDTH_PX,
                                            0,
                                        ),
                                        calc_y(top, range.start, row_id),
                                    ),
                                    Align2::LEFT_CENTER,
                                    row.title.clone(),
                                    font.clone(),
                                    FG,
                                );
                            }
                        },
                    );
                }
                Err(err) => {
                    ui.label(err);
                }
            }
        });
    }
}

fn calc_x(left: f32, margin: f32, lane: usize) -> f32 {
    left + margin + lane as f32 * LANE_WIDTH_PX
}

fn calc_y(top: f32, range_start: usize, row: usize) -> f32 {
    top + (row as f32 - range_start as f32) * ROW_HEIGHT_PX + ROW_HEIGHT_PX * 0.5
}

fn main() {
    let repo_path = std::env::args().nth(1).expect("usage: groves <repo path>");

    let repo_result = open(Path::new(&repo_path));
    let app = GrovesApp {
        content: create_app(repo_result),
    };
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("Groves", native_options, Box::new(|_cc| Ok(Box::new(app))))
        .expect("failed to start");
}

fn create_app(repo: Result<Repo, RepoError>) -> Result<Pane, String> {
    repo.map(|repo| Pane::load(&repo))
        .map_err(|err| err.to_string())
}
