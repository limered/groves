use eframe::egui::{Sense, Vec2};
use egui::{Align, Align2, CentralPanel, FontId, Key, Pos2, Rect, Stroke, Ui};
use groves_app::colors::{BACKGROUND_COLOR, FONT_COLOR, lane_color};
use groves_app::head_jump::next_head_jump;
use groves_app::render::{
    LANE_WIDTH_PX, PaneMarkerContext, ROW_HEIGHT_PX, draw_dots, draw_head_marker,
    draw_pill_connector, draw_pills, graph_width_px, render_edges, route_edge,
};
use groves_core::Pane;
use groves_core::commit_graph::GraphOptions;
use groves_repo::{Repo, RepoError, open};
use std::path::Path;
use std::time::SystemTime;

const WIDTH_PILL_AREA: f32 = 200.0;

struct GrovesApp {
    content: Result<Pane, String>,
    head_jump_cursor: Option<usize>,
    pane_marker_context: PaneMarkerContext,
}

impl eframe::App for GrovesApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ui, |ui| {
            match &self.content {
                Ok(pane) => {
                    // prepare jump to head
                    let head_jump_targets = pane.head_jump_rows();
                    let mut is_jump_pending = false;
                    if ui.input(|i| i.key_pressed(Key::H)) {
                        if let Some(next) =
                            next_head_jump(self.head_jump_cursor, &head_jump_targets)
                        {
                            self.head_jump_cursor = Some(next.0);
                        }
                        is_jump_pending = true;
                    }

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

                            // jump to head
                            if is_jump_pending
                                && let Some(jump_target_index) = self.head_jump_cursor
                            {
                                let target_row = head_jump_targets[jump_target_index];
                                let y_center = calc_y(resp.rect.top(), row_range.start, target_row);
                                let jump_rect = Rect::from_center_size(
                                    Pos2::new(resp.rect.center().x, y_center),
                                    Vec2::new(resp.rect.width(), ROW_HEIGHT_PX),
                                );
                                ui.scroll_to_rect(jump_rect, Some(Align::Center))
                            }

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

                            // draw pills
                            let sub_rect = Rect::from_min_size(
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

                                    let pill_width = draw_pills(
                                        row,
                                        pill,
                                        pill_pos,
                                        &pill_painter,
                                        &self.pane_marker_context,
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
                                let dot_center = Pos2::new(dot_x, y);
                                draw_dots(row, &dot_center, st, &painter);
                                draw_head_marker(row, &dot_center, st, &painter);
                                if !row.head_markers.is_empty() {
                                    let rect =
                                        Rect::from_center_size(dot_center, Vec2::new(12.0, 12.0));
                                    let response = ui
                                        .interact(
                                            rect,
                                            ui.id().with(format!("head_marker-{}", row.id.clone())),
                                            Sense::click(),
                                        )
                                        .on_hover_cursor(egui::CursorIcon::PointingHand);
                                    if response.clicked() {
                                        self.pane_marker_context.toggle_pinned_head(row.id.clone())
                                    }
                                    if response.hovered() {
                                        self.pane_marker_context.hover_marker(row.id.clone());
                                    } else {
                                        self.pane_marker_context.unhover_marker();
                                    }
                                }

                                let hash_rect = painter.text(
                                    Pos2::new(
                                        calc_x(
                                            left + WIDTH_PILL_AREA,
                                            16.0 + graph_width_px(pane.lane_count()),
                                            0,
                                        ),
                                        calc_y(top, range.start, row_id),
                                    ),
                                    Align2::LEFT_CENTER,
                                    &row.short,
                                    font.clone(),
                                    FONT_COLOR,
                                );

                                let hash_response = ui
                                    .interact(
                                        hash_rect,
                                        ui.id().with(row.id.clone()),
                                        Sense::click(),
                                    )
                                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                                if hash_response.clicked() {
                                    ui.ctx().copy_text(row.id.to_owned());
                                }

                                painter.text(
                                    Pos2::new(
                                        calc_x(
                                            left + WIDTH_PILL_AREA,
                                            16.0 + graph_width_px(pane.lane_count())
                                                + 5.0 * LANE_WIDTH_PX,
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

    let options = GraphOptions {
        now: stamp_now(),
        stale_after_days: 14,
    };

    let app = GrovesApp {
        content: create_app(repo, &options),
        head_jump_cursor: None,
        pane_marker_context: PaneMarkerContext::new(),
    };
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("Groves", native_options, Box::new(|_cc| Ok(Box::new(app))))
        .expect("failed to start");
}

fn create_app(repo: Result<Repo, RepoError>, options: &GraphOptions) -> Result<Pane, String> {
    repo.map(|repo| Pane::load(&repo, options))
        .map_err(|err| err.to_string())
}

fn stamp_now() -> i64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
