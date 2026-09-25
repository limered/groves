use graphcore::{Graph, PALETTE, RefKind, rel_time};
use iced::widget::canvas::{self, Frame, Geometry, Path, Stroke, Text};
use iced::widget::{column, container, row, text, Canvas};
use iced::{keyboard, mouse, Color, Element, Font, Length, Point, Rectangle, Renderer, Size, Subscription, Theme};
use std::cell::Cell;
use std::sync::Arc;
use std::time::Instant;

const ROW_H: f32 = 22.0;
const LANE_W: f32 = 14.0;

fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb8(r, g, b)
}
fn pal(c: u16) -> Color {
    let (r, g, b) = PALETTE[c as usize % PALETTE.len()];
    rgb(r, g, b)
}

#[derive(Debug, Clone)]
enum Msg {
    Scroll(f32),
    Select(usize),
    Move(isize),
    Head,
    ToggleTags,
    Viewport(f32),
}

struct App {
    g: Arc<Graph>,
    sel: usize,
    offset: f32,
    view_h: f32,
    show_tags: bool,
    now: i64,
    draw_ms: Arc<Cell<f32>>,
}

struct Pane<'a> {
    app: &'a App,
}

impl<'a> canvas::Program<Msg> for Pane<'a> {
    type State = f32; // last seen height

    fn update(&self, state: &mut f32, event: &canvas::Event, bounds: Rectangle, cursor: mouse::Cursor) -> Option<canvas::Action<Msg>> {
        if (*state - bounds.height).abs() > 0.5 {
            *state = bounds.height;
            return Some(canvas::Action::publish(Msg::Viewport(bounds.height)));
        }
        match event {
            canvas::Event::Mouse(mouse::Event::WheelScrolled { delta }) if cursor.is_over(bounds) => {
                let dy = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => -y * ROW_H * 3.0,
                    mouse::ScrollDelta::Pixels { y, .. } => -y,
                };
                Some(canvas::Action::publish(Msg::Scroll(dy)).and_capture())
            }
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let p = cursor.position_in(bounds)?;
                let r = ((p.y + self.app.offset) / ROW_H) as usize;
                Some(canvas::Action::publish(Msg::Select(r)).and_capture())
            }
            _ => None,
        }
    }

    fn draw(&self, _s: &f32, renderer: &Renderer, _t: &Theme, bounds: Rectangle, _c: mouse::Cursor) -> Vec<Geometry> {
        let t0 = Instant::now();
        let a = self.app;
        let g = &a.g;
        let n = g.commits.len();
        let mut f = Frame::new(renderer, bounds.size());
        let bg = rgb(0x1B, 0x1E, 0x24);
        f.fill_rectangle(Point::ORIGIN, bounds.size(), bg);
        let start = (a.offset / ROW_H).floor().max(0.0) as usize;
        let end = (start + (bounds.height / ROW_H) as usize + 2).min(n);
        let y = |i: usize| i as f32 * ROW_H - a.offset + ROW_H / 2.0;
        let x = |l: u16| 8.0 + l as f32 * LANE_W;
        for i in start..end {
            let fill = if i == a.sel { rgb(0x2D, 0x3A, 0x55) } else if i % 2 == 0 { bg } else { rgb(0x1F, 0x22, 0x29) };
            f.fill_rectangle(Point::new(0.0, y(i) - ROW_H / 2.0), Size::new(bounds.width, ROW_H), fill);
        }
        for i in start.saturating_sub(1)..end {
            for e in g.gap_edges(i) {
                let (x0, y0, x1, y1) = (x(e.src), y(i), x(e.dst), y(i + 1));
                let path = Path::new(|b| {
                    b.move_to(Point::new(x0, y0));
                    if x0 == x1 {
                        b.line_to(Point::new(x1, y1));
                    } else {
                        b.bezier_curve_to(Point::new(x0, y0 + ROW_H * 0.55), Point::new(x1, y1 - ROW_H * 0.55), Point::new(x1, y1));
                    }
                });
                f.stroke(&path, Stroke::default().with_width(2.0).with_color(pal(e.color)));
            }
        }
        let graph_w = g.max_lanes.min(12) as f32 * LANE_W + 10.0;
        let fg = rgb(0xD8, 0xDC, 0xE3);
        let dim = rgb(0x80, 0x87, 0x93);
        for i in start..end {
            let c = &g.commits[i];
            let col = pal(g.color[i]);
            let p = Point::new(x(g.lane[i]), y(i));
            if c.parents.len() > 1 {
                f.fill(&Path::circle(p, 3.5), col);
            } else {
                f.fill(&Path::circle(p, 5.0), bg);
                f.stroke(&Path::circle(p, 4.5), Stroke::default().with_width(2.0).with_color(col));
            }
            let mut px = graph_w;
            for r in &c.refs {
                if r.kind == RefKind::Tag && !a.show_tags { continue }
                let label = if r.kind == RefKind::Synced { format!("{} ⇅", r.name) } else { r.name.clone() };
                // no cheap text measurement in canvas: approximate width
                let w = label.chars().count() as f32 * 6.6 + 12.0;
                let rr = Path::rounded_rectangle(Point::new(px, y(i) - 8.0), Size::new(w, 16.0), if r.kind == RefKind::Tag { 2.0 } else { 8.0 }.into());
                let (fill, tc) = match r.kind {
                    RefKind::Local | RefKind::Synced => (Some(col), Color::BLACK),
                    RefKind::Head => (Some(fg), Color::BLACK),
                    RefKind::Remote => (None, col),
                    RefKind::Tag => (Some(rgb(0x3A, 0x3F, 0x4A)), fg),
                };
                if let Some(fc) = fill { f.fill(&rr, fc) }
                f.stroke(&rr, Stroke::default().with_width(1.2).with_color(if r.kind == RefKind::Tag { dim } else { col }));
                f.fill_text(Text { content: label, position: Point::new(px + 6.0, y(i)), color: tc, size: 11.5.into(), align_y: iced::alignment::Vertical::Center, ..Text::default() });
                px += w + 4.0;
            }
            f.fill_text(Text { content: c.subject.clone(), position: Point::new(px + 4.0, y(i)), color: fg, size: 13.0.into(), align_y: iced::alignment::Vertical::Center, ..Text::default() });
            f.fill_text(Text {
                content: format!("{}  {}  {}", c.author, rel_time(c.time, a.now), &c.sha[..7]),
                position: Point::new(bounds.width - 8.0, y(i)),
                color: dim,
                size: 11.5.into(),
                align_x: iced::widget::text::Alignment::Right,
                align_y: iced::alignment::Vertical::Center,
                ..Text::default()
            });
        }
        a.draw_ms.set(t0.elapsed().as_secs_f32() * 1000.0);
        vec![f.into_geometry()]
    }
}

impl App {
    fn clamp(&mut self) {
        let max = (self.g.commits.len() as f32 * ROW_H - self.view_h).max(0.0);
        self.offset = self.offset.clamp(0.0, max);
    }
    fn reveal(&mut self) {
        let y = self.sel as f32 * ROW_H;
        if y < self.offset || y + ROW_H > self.offset + self.view_h {
            self.offset = y - self.view_h / 2.0;
        }
        self.clamp();
    }
    fn update(&mut self, m: Msg) {
        let n = self.g.commits.len();
        match m {
            Msg::Scroll(d) => { self.offset += d; self.clamp() }
            Msg::Select(r) => self.sel = r.min(n - 1),
            Msg::Move(d) => { self.sel = (self.sel as isize + d).clamp(0, n as isize - 1) as usize; self.reveal() }
            Msg::Head => { self.sel = self.g.head_row; self.reveal() }
            Msg::ToggleTags => self.show_tags = !self.show_tags,
            Msg::Viewport(h) => { self.view_h = h; self.reveal() }
        }
    }
    fn view(&self) -> Element<'_, Msg> {
        let c = &self.g.commits[self.sel];
        let dim = rgb(0x80, 0x87, 0x93);
        let mut files = column![text("Changed files (stub)").color(dim)].spacing(2);
        for fl in ["src/Backend/Api/Controllers/TenantController.cs", "src/Frontend/app/tenant/settings.tsx", "Documentation/feature.md"] {
            files = files.push(text(format!("M  {fl}")).font(Font::MONOSPACE).size(12));
        }
        let detail = container(
            column![
                text(&c.subject).size(15),
                text(format!("{}  ·  {} ago", c.author, rel_time(c.time, self.now))).color(dim),
                text(&c.sha).font(Font::MONOSPACE).size(11).color(dim),
                files,
            ]
            .spacing(6),
        )
        .padding(10)
        .width(340)
        .height(Length::Fill);
        let status = text(format!(
            "iced · {} commits · {} lanes · load {:.0} ms · layout {:.0} ms · canvas draw {:.2} ms · tags {} (t) · j/k h PgUp/PgDn",
            self.g.commits.len(), self.g.max_lanes, self.g.load_ms, self.g.layout_ms, self.draw_ms.get(),
            if self.show_tags { "on" } else { "off" }
        ))
        .size(11)
        .color(dim);
        column![
            row![Canvas::new(Pane { app: self }).width(Length::Fill).height(Length::Fill), detail].height(Length::Fill),
            container(status).padding(4),
        ]
        .into()
    }
    fn subscription(&self) -> Subscription<Msg> {
        keyboard::listen().filter_map(|e| match e {
            keyboard::Event::KeyPressed { key, .. } => match key.as_ref() {
                keyboard::Key::Character("j") | keyboard::Key::Named(keyboard::key::Named::ArrowDown) => Some(Msg::Move(1)),
                keyboard::Key::Character("k") | keyboard::Key::Named(keyboard::key::Named::ArrowUp) => Some(Msg::Move(-1)),
                keyboard::Key::Named(keyboard::key::Named::PageDown) => Some(Msg::Move(40)),
                keyboard::Key::Named(keyboard::key::Named::PageUp) => Some(Msg::Move(-40)),
                keyboard::Key::Named(keyboard::key::Named::End) => Some(Msg::Move(isize::MAX / 2)),
                keyboard::Key::Character("h") | keyboard::Key::Named(keyboard::key::Named::Home) => Some(Msg::Head),
                keyboard::Key::Character("t") => Some(Msg::ToggleTags),
                _ => None,
            },
            _ => None,
        })
    }
}

fn main() -> iced::Result {
    iced::application(
        || {
            let g = graphcore::from_args();
            let now = g.commits.first().map(|c| c.time).unwrap_or(0) + 3600;
            let sel = g.head_row;
            App { g: Arc::new(g), sel, offset: 0.0, view_h: 800.0, show_tags: false, now, draw_ms: Arc::new(Cell::new(0.0)) }
        },
        App::update,
        App::view,
    )
    .title("commit-graph · iced prototype")
    .subscription(App::subscription)
    .theme(|_: &App| Theme::Dark)
    .window_size(Size::new(1400.0, 900.0))
    .run()
}
