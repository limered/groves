// Emil owns this file — ticket 01 will add the eframe app that opens a repo
// path from the CLI args and lists its commits (or shows the open error).

use std::{collections::HashSet, path::Path};

use eframe::egui::{Sense, Vec2};
use egui::{Align2, Color32, FontId, Pos2, Stroke};
use groves_core::{Edge, LANE_WIDTH_PX, Pane, ROW_HEIGHT_PX};
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

                            let font = FontId::proportional(13.0);

                            let mut all_relevant_edges: HashSet<&Edge> = HashSet::new();
                            let from = range.start.saturating_sub(1);
                            for row_id in from..range.end.min(pane.rows().len()) {
                                let relevant_edges: Vec<&Edge> = pane
                                    .edges()
                                    .iter()
                                    .filter(|e| e.child_row.le(&row_id) && row_id.lt(&e.parent_row))
                                    .collect();
                                all_relevant_edges.extend(relevant_edges.clone());
                            }
                            for row_id in range.clone() {
                                let row = &pane.rows()[row_id];
                                painter.text(
                                    Pos2::new(
                                        left + 8.0 + pane.graph_width_px(),
                                        top + (row_id - range.start) as f32 * row_height
                                            + row_height / 2.0,
                                    ),
                                    Align2::LEFT_CENTER,
                                    &row.short,
                                    font.clone(),
                                    FG,
                                );
                            }

                            let range_start = row_range.start;
                            for edge in all_relevant_edges.iter() {
                                let start_y = top
                                    + (edge.child_row as f32 - range_start as f32) * row_height
                                    + row_height * 0.5;
                                let end_y = top
                                    + (edge.parent_row as f32 - range_start as f32) * row_height
                                    + row_height * 0.5;
                                let start_x = left + 8.0 + edge.child_lane as f32 * LANE_WIDTH_PX;
                                let end_x = left + 8.0 + edge.parent_lane as f32 * LANE_WIDTH_PX;

                                let st = Stroke::new(2.0, Color32::from_rgb(0x4F, 0xC3, 0xF7));
                                if start_x == end_x {
                                    painter.line_segment(
                                        [Pos2::new(start_x, start_y), Pos2::new(end_x, end_y)],
                                        st,
                                    );
                                } else {
                                    let pts = [
                                        Pos2::new(start_x, start_y),
                                        Pos2::new(start_x, start_y + row_height * 0.55),
                                        Pos2::new(end_x, end_y - row_height * 0.55),
                                        Pos2::new(end_x, end_y),
                                    ];
                                    painter.add(
                                        egui::epaint::CubicBezierShape::from_points_stroke(
                                            pts,
                                            false,
                                            Color32::TRANSPARENT,
                                            st,
                                        ),
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
