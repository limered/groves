use eframe::egui::{self, Align2, Color32, CornerRadius, FontId, Key, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};
use graphcore::{Graph, PALETTE, RefKind, rel_time};
use std::time::Instant;

const ROW_H: f32 = 22.0;
const LANE_W: f32 = 14.0;
const BG: Color32 = Color32::from_rgb(0x1B, 0x1E, 0x24);
const BG_ALT: Color32 = Color32::from_rgb(0x1F, 0x22, 0x29);
const SEL: Color32 = Color32::from_rgb(0x2D, 0x3A, 0x55);
const FG: Color32 = Color32::from_rgb(0xD8, 0xDC, 0xE3);
const DIM: Color32 = Color32::from_rgb(0x80, 0x87, 0x93);

fn pal(c: u16) -> Color32 {
    let (r, g, b) = PALETTE[c as usize % PALETTE.len()];
    Color32::from_rgb(r, g, b)
}

struct App {
    g: Graph,
    sel: usize,
    show_tags: bool,
    scroll_to: Option<usize>,
    frame_ms: f32,
    now: i64,
}

impl eframe::App for App {
    fn ui(&mut self, root: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = root.ctx().clone();
        let t0 = Instant::now();
        let n = self.g.commits.len();
        ctx.input(|i| {
            let mut mv = |d: isize| {
                self.sel = (self.sel as isize + d).clamp(0, n as isize - 1) as usize;
                self.scroll_to = Some(self.sel);
            };
            if i.key_pressed(Key::J) || i.key_pressed(Key::ArrowDown) { mv(1) }
            if i.key_pressed(Key::K) || i.key_pressed(Key::ArrowUp) { mv(-1) }
            if i.key_pressed(Key::PageDown) { mv(40) }
            if i.key_pressed(Key::PageUp) { mv(-40) }
            if i.key_pressed(Key::End) { mv(n as isize) }
            if i.key_pressed(Key::H) || i.key_pressed(Key::Home) {
                self.sel = self.g.head_row;
                self.scroll_to = Some(self.sel);
            }
            if i.key_pressed(Key::T) { self.show_tags = !self.show_tags }
        });

        egui::Panel::bottom("status").show(root, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!(
                    "egui · {} commits · {} lanes · load {:.0} ms · layout {:.0} ms · frame {:.2} ms · tags {} (t) · j/k h PgUp/PgDn",
                    n, self.g.max_lanes, self.g.load_ms, self.g.layout_ms, self.frame_ms,
                    if self.show_tags { "on" } else { "off" }
                )).color(DIM).small());
            });
        });

        egui::Panel::right("detail").default_size(340.0).show(root, |ui| {
            let c = &self.g.commits[self.sel];
            ui.add_space(6.0);
            ui.label(egui::RichText::new(&c.subject).color(FG).size(15.0).strong());
            ui.add_space(4.0);
            ui.label(egui::RichText::new(format!("{}  ·  {} ago", c.author, rel_time(c.time, self.now))).color(DIM));
            ui.label(egui::RichText::new(&c.sha).monospace().color(DIM).small());
            ui.label(egui::RichText::new(format!("parents: {}", c.parents.iter().map(|p| &self.g.commits[*p as usize].sha[..8]).collect::<Vec<_>>().join(" "))).monospace().color(DIM).small());
            ui.separator();
            ui.label(egui::RichText::new("Changed files (stub)").color(DIM));
            for f in ["src/Backend/Api/Controllers/TenantController.cs", "src/Frontend/app/tenant/settings.tsx", "Documentation/feature.md"] {
                ui.label(egui::RichText::new(format!("M  {f}")).monospace().color(FG).small());
            }
        });

        egui::CentralPanel::default().frame(egui::Frame::NONE.fill(BG)).show(root, |ui| {
            let mut sa = egui::ScrollArea::vertical().auto_shrink(false);
            if let Some(r) = self.scroll_to.take() {
                let h = ui.available_height();
                sa = sa.vertical_scroll_offset((r as f32 * ROW_H - h / 2.0).max(0.0));
            }
            sa.show_rows(ui, ROW_H, n, |ui, range| {
                let full = ui.available_rect_before_wrap();
                let (resp, painter) = ui.allocate_painter(Vec2::new(full.width(), range.len() as f32 * ROW_H), Sense::click());
                let top = resp.rect.top();
                let left = resp.rect.left();
                let graph_w = (self.g.max_lanes.min(12) as f32) * LANE_W + 10.0;
                let cx = |l: u16| left + 8.0 + l as f32 * LANE_W;
                let cy = |i: usize| top + (i - range.start) as f32 * ROW_H + ROW_H / 2.0;
                let clip = resp.rect;

                if resp.clicked() {
                    if let Some(p) = resp.interact_pointer_pos() {
                        self.sel = range.start + ((p.y - top) / ROW_H) as usize;
                    }
                }

                // row backgrounds
                for i in range.clone() {
                    let r = Rect::from_min_size(Pos2::new(left, cy(i) - ROW_H / 2.0), Vec2::new(clip.width(), ROW_H));
                    let fill = if i == self.sel { SEL } else if i % 2 == 0 { BG } else { BG_ALT };
                    painter.rect_filled(r, 0.0, fill);
                }
                // edges: one gap before range start too so lines enter from top
                let from = range.start.saturating_sub(1);
                for i in from..range.end.min(n) {
                    for e in self.g.gap_edges(i) {
                        let (x0, y0) = (cx(e.src), cy_s(top, range.start, i));
                        let (x1, y1) = (cx(e.dst), cy_s(top, range.start, i + 1));
                        let st = Stroke::new(2.0, pal(e.color));
                        if x0 == x1 {
                            painter.line_segment([Pos2::new(x0, y0), Pos2::new(x1, y1)], st);
                        } else {
                            let pts = [Pos2::new(x0, y0), Pos2::new(x0, y0 + ROW_H * 0.55), Pos2::new(x1, y1 - ROW_H * 0.55), Pos2::new(x1, y1)];
                            painter.add(egui::epaint::CubicBezierShape::from_points_stroke(pts, false, Color32::TRANSPARENT, st));
                        }
                    }
                }
                // dots, pills, text
                let font = FontId::proportional(13.0);
                let small = FontId::proportional(11.5);
                for i in range.clone() {
                    let c = &self.g.commits[i];
                    let col = pal(self.g.color[i]);
                    let p = Pos2::new(cx(self.g.lane[i]), cy(i));
                    if c.parents.len() > 1 {
                        painter.circle_filled(p, 3.5, col);
                    } else {
                        painter.circle_filled(p, 5.0, BG);
                        painter.circle_stroke(p, 4.5, Stroke::new(2.0, col));
                    }
                    let mut x = left + graph_w;
                    for r in &c.refs {
                        if r.kind == RefKind::Tag && !self.show_tags { continue }
                        let txt = if r.kind == RefKind::Synced { format!("{} ⇅", r.name) } else { r.name.clone() };
                        let gal = painter.layout_no_wrap(txt, small.clone(), Color32::BLACK);
                        let w = gal.size().x + 12.0;
                        let rr = Rect::from_min_size(Pos2::new(x, cy(i) - 8.0), Vec2::new(w, 16.0));
                        let (fill, txtc, rad) = match r.kind {
                            RefKind::Local | RefKind::Synced => (col, Color32::BLACK, 8),
                            RefKind::Head => (FG, Color32::BLACK, 8),
                            RefKind::Remote => (Color32::TRANSPARENT, col, 8),
                            RefKind::Tag => (Color32::from_rgb(0x3A, 0x3F, 0x4A), FG, 2),
                        };
                        painter.rect(rr, CornerRadius::same(rad), fill, Stroke::new(1.2, if r.kind == RefKind::Tag { DIM } else { col }), StrokeKind::Inside);
                        painter.galley(Pos2::new(x + 6.0, cy(i) - gal.size().y / 2.0), gal, txtc);
                        x += w + 4.0;
                    }
                    painter.text(Pos2::new(x + 4.0, cy(i)), Align2::LEFT_CENTER, &c.subject, font.clone(), FG);
                    let rx = clip.right() - 8.0;
                    painter.text(Pos2::new(rx, cy(i)), Align2::RIGHT_CENTER, format!("{}  {}  {}", c.author, rel_time(c.time, self.now), &c.sha[..7]), small.clone(), DIM);
                }
            });
        });
        self.frame_ms = t0.elapsed().as_secs_f32() * 1000.0;
    }
}

fn cy_s(top: f32, start: usize, i: usize) -> f32 {
    top + (i as f32 - start as f32) * ROW_H + ROW_H / 2.0
}

fn main() -> eframe::Result {
    let g = graphcore::from_args();
    let now = g.commits.first().map(|c| c.time).unwrap_or(0) + 3600;
    let sel = g.head_row;
    eframe::run_native(
        "commit-graph · egui prototype",
        eframe::NativeOptions { renderer: if std::env::args().any(|a| a == "--glow") { eframe::Renderer::Glow } else { eframe::Renderer::Wgpu }, viewport: egui::ViewportBuilder::default().with_inner_size([1400.0, 900.0]), ..Default::default() },
        Box::new(move |cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(App { g, sel, show_tags: false, scroll_to: Some(sel), frame_ms: 0.0, now }))
        }),
    )
}
