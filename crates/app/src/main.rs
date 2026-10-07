// Emil owns this file — ticket 01 will add the eframe app that opens a repo
// path from the CLI args and lists its commits (or shows the open error).

use std::path::Path;

use groves_core::Pane;
use groves_repo::{Repo, RepoError, open};

struct GrovesApp {
    content: Result<Pane, String>,
}

impl eframe::App for GrovesApp {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Groves");
            match &self.content {
                Ok(pane) => {
                    let row_height = ui.text_style_height(&egui::TextStyle::Monospace);
                    egui::ScrollArea::vertical().show_rows(
                        ui,
                        row_height,
                        pane.rows().len(),
                        |ui, row_range| {
                            for row_id in row_range {
                                let row = &pane.rows()[row_id];
                                ui.monospace(format!("{} {}", row.short, row.title));
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
