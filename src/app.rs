use egui::{Color32, FontId, Pos2, RichText, Stroke};

pub struct TimesCircleApp {
    paused: bool,
    num_points: usize,
    multiplier: f32,
    step_size: f32,
    stroke: f32,
    color: Color32,
}

impl Default for TimesCircleApp {
    fn default() -> Self {
        Self {
            paused: true,
            num_points: 500,
            multiplier: 2.0,
            step_size: 0.1,
            stroke: 0.30,
            color: Color32::from_rgb(0, 0, 0),
        }
    }
}

impl TimesCircleApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customized the look at feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        Default::default()
    }
}

impl eframe::App for TimesCircleApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let my_frame = egui::containers::Frame {
            fill: Color32::LIGHT_GRAY,
            ..Default::default()
        };
        egui::CentralPanel::default()
            .frame(my_frame)
            .show(ctx, |ui| {
                egui::CollapsingHeader::new("Options").show(ui, |ui| {
                    ui.label(generate_title(
                        format!("{:.5} Mod {}", self.multiplier, self.num_points).as_str(),
                    ));

                    // ui.horizontal(|ui| {
                    //     ui.label("Points");
                    //     ui.add(egui::DragValue::new(&mut self.num_points).speed(1));
                    // });

                    ui.add(
                        egui::Slider::new(&mut self.num_points, 0..=10000)
                            .text("Times")
                            .max_decimals(5),
                    )
                    .clicked();

                    ui.add(
                        egui::Slider::new(&mut self.multiplier, 0.0..=self.num_points as f32)
                            .text("Times")
                            .max_decimals(5),
                    )
                    .clicked();

                    ui.label(generate_title("Animation"));

                    ui.horizontal(|ui| {
                        if ui.button("▶").clicked() {
                            self.paused = false;
                        }
                        if ui.button("■").clicked() {
                            self.paused = true;
                        }

                        if ui
                            .button(RichText::new("⏺").color(Color32::DARK_RED))
                            .clicked()
                        {
                            self.paused = false;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Step Size");
                        if ui.button("-").clicked() {
                            self.step_size -= 0.01;
                        }
                        ui.add(
                            egui::DragValue::new(&mut self.step_size)
                                .speed(0.01)
                                .min_decimals(2)
                                .max_decimals(5),
                        );
                        if ui.button("+").clicked() {
                            self.step_size += 0.01;
                        }
                    });

                    // ui.horizontal(|ui| {
                    //     ui.label("Steps per second");
                    //     ui.add(egui::DragValue::new(&mut self.stroke).speed(0.1).max_decimals(2));
                    // });

                    ui.label(generate_title("Style"));
                    ui.horizontal(|ui| {
                        ui.label("Stroke");
                        if ui.button("-").clicked() {
                            self.stroke -= 0.1;
                        }
                        ui.add(
                            egui::DragValue::new(&mut self.stroke)
                                .speed(0.1)
                                .max_decimals(2),
                        );
                        if ui.button("+").clicked() {
                            self.stroke += 0.1;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Color");
                        ui.color_edit_button_srgba(&mut self.color);
                    });
                });

                if !self.paused
                    && self.multiplier < self.num_points as f32
                    && self.multiplier >= 0.0
                {
                    self.multiplier += self.step_size;
                    ui.ctx().request_repaint();
                    // std::thread::sleep(std::time::Duration::from_millis(30));
                }

                let center: (f32, f32) = (
                    (ctx.available_rect().max.x - ctx.available_rect().min.x) / 2.0,
                    (ctx.available_rect().max.y - ctx.available_rect().min.y) / 2.0,
                );

                let margin: f32 = 30.0;

                let points = generate_points(self.num_points, center.1 - margin);

                for i in 0..self.num_points {
                    let j = ((i as f32) * self.multiplier) as usize % self.num_points;
                    let p1 = Pos2 {
                        x: points[i].x + center.0,
                        y: points[i].y + center.1,
                    };
                    let p2 = Pos2 {
                        x: points[j].x + center.0,
                        y: points[j].y + center.1,
                    };

                    ui.painter().line_segment(
                        [p1, p2],
                        Stroke::new(
                            self.stroke,
                            Color32::from_rgb(
                                self.color[0] as u8,
                                self.color[1] as u8,
                                self.color[2] as u8,
                            ),
                        ),
                    );
                }

                ui.painter().circle(
                    Pos2 {
                        x: center.0,
                        y: center.1,
                    },
                    center.1 - margin,
                    Color32::TRANSPARENT,
                    Stroke::new(
                        self.stroke,
                        Color32::from_rgb(
                            self.color[0] as u8,
                            self.color[1] as u8,
                            self.color[2] as u8,
                        ),
                    ),
                );
            });
    }
}

fn generate_title(title: &str) -> RichText {
    RichText::new(title).font(FontId::proportional(20.0))
}

// Generate the coordinates of the points on the circle
fn generate_points(num_points: usize, radius: f32) -> Vec<Pos2> {
    let n: f32 = num_points as f32;
    let mut points: Vec<Pos2> = Vec::with_capacity(num_points);
    let mut angle: f32 = std::f32::consts::PI;
    for _ in 0..num_points {
        let point = Pos2 {
            x: radius * angle.cos(),
            y: radius * angle.sin(),
        };
        points.push(point);
        angle += std::f32::consts::TAU / n;
    }
    points
}
