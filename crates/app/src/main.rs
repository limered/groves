// Emil owns this file — ticket 01 will add the eframe app that opens a repo
// path from the CLI args and lists its commits (or shows the open error).

use eframe::egui::{Sense, Vec2};
use egui::{Align2, FontId, Pos2, Stroke};
use groves_app::colors::{BACKGROUND_COLOR, FONT_COLOR, lane_color};
use groves_app::render::{draw_dots, draw_head_marker, draw_pill, draw_pill_connector, render_edges, route_edge};
use groves_core::{LANE_WIDTH_PX, Pane, ROW_HEIGHT_PX};
use groves_repo::{Repo, RepoError, open};
use std::path::Path;

const WIDTH_PILL_AREA: f32 = 200.0;

struct GrovesApp {
    content: Result<Pane, String>,
}

impl eframe::App for GrovesApp {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
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
                            painter.rect_filled(resp.rect, 0.0, BACKGROUND_COLOR);

                            let top = resp.rect.top();
                            let left = resp.rect.left();

                            let font = FontId::monospace(13.0);
                            let pill_font = FontId::monospace(14.0);

                            // edges
                            let range_start = row_range.start;
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
                                    let start_x = calc_x(left + WIDTH_PILL_AREA, 8.0, last_lane);
                                    let end_x = calc_x(left + WIDTH_PILL_AREA, 8.0, lane);

                                    render_edges(
                                        edge,
                                        Pos2::new(start_x, start_y),
                                        Pos2::new(end_x, end_y),
                                        &painter,
                                    );
                                }
                            }

                            // draw texts
                            let sub_rect = egui::Rect::from_min_size(
                                resp.rect.min,
                                Vec2::new(WIDTH_PILL_AREA, resp.rect.height()),
                            );
                            let pill_painter = painter.with_clip_rect(sub_rect);
                            for row_id in range.clone() {
                                let row = &pane.rows()[row_id];
                                let dot_x = calc_x(left + WIDTH_PILL_AREA, 8.0, row.lane);
                                if let Some(pill) = &row.pill {
                                    let pill_pos = Pos2::new(
                                        calc_x(left, 8.0, 0),
                                        calc_y(top, range_start, row_id),
                                    );

                                    let pill_width = draw_pill(
                                        pill,
                                        pill_pos,
                                        &pill_painter,
                                        pill_font.clone(),
                                        lane_color(row.lane),
                                    );
                                    draw_pill_connector(
                                        Pos2::new(pill_pos.x + pill_width, pill_pos.y),
                                        dot_x - 2.0,
                                        &painter,
                                        lane_color(row.lane),
                                    );
                                }

                                // draw dots
                                let st = Stroke::new(2.0, lane_color(row.lane)); // #5c7855
                                let y = calc_y(top, range_start, row_id);
                                draw_dots(row, Pos2::new(dot_x, y), st, &painter);
                                draw_head_marker(row, Pos2::new(dot_x, y), st, &painter);

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
                                    FONT_COLOR,
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
                                    FONT_COLOR,
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
    let repo = if std::env::args().len() > 1 {
        let path_string = std::env::args().nth(1).expect("usage: groves <repo path>");
        open(Path::new(&path_string))
    } else {
        let path = std::env::current_dir().expect("can't access current directory");
        open(&path)
    };

    let app = GrovesApp {
        content: create_app(repo),
    };
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("Groves", native_options, Box::new(|_cc| Ok(Box::new(app))))
        .expect("failed to start");
}

fn create_app(repo: Result<Repo, RepoError>) -> Result<Pane, String> {
    repo.map(|repo| Pane::load(&repo))
        .map_err(|err| err.to_string())
}
