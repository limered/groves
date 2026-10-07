fn measure(label: &str, ui: &mut egui::Ui, total_rows: usize, auto_shrink: bool, show: bool) {
    ui.heading("Groves");
    let available = ui.available_height();
    let mut area = egui::ScrollArea::vertical();
    if auto_shrink {
        area = area.auto_shrink([false, false]);
    }
    let out = if show {
        area.show(ui, |ui| {
            for _ in 0..total_rows.min(50) {
                ui.monospace("a1b2c3d Fix lane reuse");
            }
        })
    } else {
        area.show_rows(ui, 22.0, total_rows, |ui, range| {
            for _ in range {
                ui.monospace("a1b2c3d Fix lane reuse");
            }
        })
    };
    println!(
        "{label:28} rows={total_rows:5} available_h={available:7.1} area_h={:7.1} content_h={:9.1}",
        out.response.rect.height(),
        out.content_size.y
    );
}

#[test]
fn scroll_sizing() {
    egui::__run_test_ui(|ui| {
        ui.set_max_width(800.0);
        ui.set_max_height(600.0);
        egui::CentralPanel::default().show(ui, |ui| {
            measure("show_rows, default", ui, 10000, false, false);
            measure("show_rows, small list", ui, 10, false, false);
            measure("show_rows, small+no_shrink", ui, 10, true, false);
            measure("plain show, big", ui, 10000, false, true);
            measure("plain show, small", ui, 10, false, true);
        });
    });
}
