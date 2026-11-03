// Emil owns this file — ticket 01 will add the eframe app that opens a repo
// path from the CLI args and lists its commits (or shows the open error).

use std::path::Path;

use eframe::egui::{Sense, Vec2};
use egui::{Align2, Color32, FontId, Pos2, Stroke};
use groves_app::render::route_edge;
use groves_core::{LANE_WIDTH_PX, Pane, ROW_HEIGHT_PX};
use groves_repo::{Repo, RepoError, open};

const FG: Color32 = Color32::from_rgb(0xD8, 0xDC, 0xE3);

struct GrovesApp {
    content: Result<Pane, String>,
}

impl eframe::App for GrovesApp {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Groves");
            ui.separator();
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
                            let top = resp.rect.top();
                            let left = resp.rect.left();

                            let font = FontId::monospace(13.0);

                            // draw texts
                            for row_id in range.clone() {
                                let row = &pane.rows()[row_id];
                                painter.text(
                                    Pos2::new(
                                        calc_x(left, 8.0 + pane.graph_width_px(), 0),
                                        calc_y(top, range.start, row_id),
                                    ),
                                    Align2::LEFT_CENTER,
                                    &row.short,
                                    font.clone(),
                                    FG,
                                );
                                let row_text = match &row.pill {
                                    Some(p) => p.name.clone(),
                                    None => row.title.clone(),
                                };
                                painter.text(
                                    Pos2::new(
                                        calc_x(
                                            left,
                                            8.0 + pane.graph_width_px() + 5.0 * LANE_WIDTH_PX,
                                            0,
                                        ),
                                        calc_y(top, range.start, row_id),
                                    ),
                                    Align2::LEFT_CENTER,
                                    row_text,
                                    font.clone(),
                                    FG,
                                );
                            }

                            // lines
                            let range_start = row_range.start;
                            let st = Stroke::new(2.0, Color32::from_rgb(0x4F, 0xC3, 0xF7));
                            for edge in pane.edges().iter() {
                                if edge.parent_row <= range.start.saturating_sub(1)
                                    || edge.child_row >= row_range.end
                                {
                                    continue;
                                }

                                let points = route_edge(edge);
                                for point_index in 1..points.len() {
                                    let (last_row, last_lane) = points[point_index - 1];
                                    let (row, lane) = points[point_index];

                                    let start_y = calc_y(top, range_start, last_row);
                                    let end_y = calc_y(top, range_start, row);
                                    let start_x = calc_x(left, 8.0, last_lane);
                                    let end_x = calc_x(left, 8.0, lane);

                                    let dir = if edge.parent_lane > edge.child_lane {
                                        -1.0
                                    } else {
                                        1.0
                                    };

                                    if start_x == end_x {
                                        painter.line_segment(
                                            [Pos2::new(start_x, start_y), Pos2::new(end_x, end_y)],
                                            st,
                                        );
                                    } else if edge.is_first_parent {
                                        painter.line_segment(
                                            [
                                                Pos2::new(start_x, end_y - 1.0 * ROW_HEIGHT_PX),
                                                Pos2::new(start_x, end_y - 0.5 * ROW_HEIGHT_PX),
                                            ],
                                            st,
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
                                                st,
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
                                            st,
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
                                            st,
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
                                                st,
                                            ),
                                        );

                                        painter.line_segment(
                                            [
                                                Pos2::new(end_x, end_y - 0.5 * ROW_HEIGHT_PX),
                                                Pos2::new(end_x, end_y + 0.5 * ROW_HEIGHT_PX),
                                            ],
                                            st,
                                        );
                                    }
                                }
                            }

                            // draw dots
                            for row_id in range.clone() {
                                let st = Stroke::new(2.0, Color32::from_rgb(0x4F, 0xC3, 0xF7));
                                let x = calc_x(left, 8.0, pane.rows()[row_id].lane);
                                let y = calc_y(top, range_start, row_id);

                                if pane.rows()[row_id].is_merge {
                                    painter.circle(
                                        Pos2::new(x, y),
                                        2.0,
                                        Color32::from_rgb(0x4F, 0xC3, 0xF7),
                                        st,
                                    );
                                } else {
                                    painter.circle(
                                        Pos2::new(x, y),
                                        3.5,
                                        Color32::from_rgb(0xFF, 0x0, 0x0),
                                        st,
                                    );
                                }
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
