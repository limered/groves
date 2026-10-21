// Emil owns this file — ticket 01 will add the eframe app that opens a repo
// path from the CLI args and lists its commits (or shows the open error).

use std::path::Path;

use eframe::egui::{Sense, Vec2};
use egui::{Color32, Pos2, Stroke};
use groves_core::{Edge, LANE_WIDTH_PX, Pane, ROW_HEIGHT_PX};
use groves_repo::{Repo, RepoError, open};
use itertools::Itertools;

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
                    let row_height = ui.text_style_height(&egui::TextStyle::Monospace);
                    let sa = egui::ScrollArea::vertical().auto_shrink(false);
                    sa.show_rows(ui, row_height, pane.rows().len(), |ui, row_range| {
                        ui.horizontal(|ui| {
                            let (resp, painter) = ui.allocate_painter(
                                Vec2::new(
                                    pane.graph_width_px(),
                                    row_range.len() as f32 * ROW_HEIGHT_PX,
                                ),
                                Sense::click(),
                            );
                            let top = resp.rect.top();
                            let left = resp.rect.left();

                            let mut all_relevant_edges: Vec<&Edge> = Vec::new();
                            ui.vertical(|ui| {
                                for row_id in row_range {
                                    let relevant_edges: Vec<&Edge> = pane
                                        .edges()
                                        .iter()
                                        .filter(|e| {
                                            e.child_row.le(&row_id) && row_id.lt(&e.parent_row)
                                        })
                                        .collect();
                                    all_relevant_edges.append(&mut relevant_edges.clone());

                                    let row = &pane.rows()[row_id];
                                    ui.monospace(format!("{} {}", row.short, row.title));
                                }
                            });

                            for edge in all_relevant_edges.iter().unique() {
                                let start_y = edge.child_row as f32 * row_height;
                                let end_y = edge.parent_row as f32 * row_height;
                                let start_x = left + 8.0 + edge.child_lane as f32 * LANE_WIDTH_PX;
                                let end_x = left + 8.0 + edge.parent_lane as f32 * LANE_WIDTH_PX;

                                let st = Stroke::new(2.0, Color32::from_rgb(0x4F, 0xC3, 0xF7));
                                painter.line_segment(
                                    [Pos2::new(start_x, start_y), Pos2::new(end_x, end_y)],
                                    st,
                                );
                            }
                        });
                    });
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
