use egui::{Color32, FontId, PointerButton, Pos2, RichText, Stroke, Ui};

pub struct TimesCircleApp {
    paused: bool,
    center: (f32, f32),
    offset: (f32, f32),
    zoom: f32,
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
            center: (0.0, 0.0),
            offset: (0.0, 0.0),
            zoom: 0.85,
            num_points: 500,
            multiplier: 2.0,
            step_size: 0.1,
            stroke: 0.3,
            color: Color32::from_rgb(0, 0, 0),
        }
    }
}

impl eframe::App for TimesCircleApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update the current state of the app based on the user controls
        self.controls(ctx);

        // Paint Ui
        self.ui(ctx);
    }
}

impl TimesCircleApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }

    fn ui(&mut self, ctx: &egui::Context) {
        if egui::CentralPanel::default()
            .show(ctx, |ui| {
                // Display options Ui
                self.options_ui(ui);

                // Paint times circle
                self.times_circle(ui);
            })
            .response
            .dragged()
        {};
    }

    fn options_ui(&mut self, ui: &mut Ui) {
        egui::CollapsingHeader::new("Options").show(ui, |ui| {
            // p Mod n text
            let p_mod_n =
                RichText::new(format!("{:.2} Mod {}", self.multiplier, self.num_points).as_str())
                    .font(FontId::proportional(20.0));
            ui.label(p_mod_n);

            // Num points slider
            ui.add(egui::Slider::new(&mut self.num_points, 0..=10000).text("Points"));

            // Multiplier slider
            ui.add(
                egui::Slider::new(&mut self.multiplier, 0.0..=self.num_points as f32)
                    .text("Multiplier")
                    .min_decimals(1)
                    .max_decimals(2),
            );

            // Step size slider
            ui.horizontal(|ui| {
                ui.add(
                    egui::Slider::new(&mut self.step_size, 0.0..=1.0)
                        .text("Step Size")
                        .min_decimals(1)
                        .max_decimals(3),
                )
            });

            // Light/Dark mode buttons
            egui::widgets::global_dark_light_mode_buttons(ui);

            ui.label("Stroke");
            // Stroke width slider
            ui.horizontal(|ui| {
                ui.add(
                    egui::Slider::new(&mut self.stroke, 0.0..=1.0)
                        .text("Width")
                        .max_decimals(2),
                )
            });

            // Stroke Color picker
            ui.horizontal(|ui| {
                ui.label("Color");
                ui.color_edit_button_srgba(&mut self.color);
            });

            // Playback buttons
            ui.horizontal(|ui| {
                if ui.button("▶").clicked() {
                    self.paused = false;
                }
                if ui.button("■").clicked() {
                    self.paused = true;
                }

                // if ui
                //     .button(RichText::new("⏺").color(Color32::DARK_RED))
                //     .clicked()
                // {
                //     self.paused = false;
                // }
            });
        });
    }

    fn times_circle(&mut self, ui: &mut Ui) {
        // If not paused, increment multiplier, request redraw and
        // Else, draw
        if !self.paused && self.multiplier < self.num_points as f32 && self.multiplier >= 0.0 {
            self.multiplier += self.step_size;
            ui.ctx().request_repaint();
        }

        // Calculate radius of circle from screen size
        let radius: f32 = if self.center.1 < self.center.0 {
            self.center.1 * self.zoom
        } else {
            self.center.0 * self.zoom
        };

        // Generate evenly spaced points around the circumference of a circle
        let points: Vec<Pos2> = generate_points(self.num_points, radius);

        // Draw lines between points
        for i in 0..self.num_points {
            // Find the point to connect to
            let j = ((i as f32) * self.multiplier) as usize % self.num_points;

            // Transform to world coords
            let p1 = Pos2 {
                x: (points[i].x + self.center.0 + self.offset.0),
                y: (points[i].y + self.center.1 + self.offset.1),
            };
            let p2 = Pos2 {
                x: (points[j].x + self.center.0) + self.offset.0,
                y: (points[j].y + self.center.1) + self.offset.1,
            };

            // Draw line
            ui.painter()
                .line_segment([p1, p2], Stroke::new(self.stroke, self.color));
        }

        // Draw circle
        ui.painter().circle(
            Pos2 {
                x: self.center.0 + self.offset.0,
                y: self.center.1 + self.offset.1,
            },
            radius,
            Color32::TRANSPARENT,
            Stroke::new(self.stroke, self.color),
        );
    }

    fn controls(&mut self, ctx: &egui::Context) {
        // Calculate center of current screen
        self.center = (
            (ctx.available_rect().max.x - ctx.available_rect().min.x) / 2.0,
            (ctx.available_rect().max.y - ctx.available_rect().min.y) / 2.0,
        );

        // Calculate zoom
        self.zoom += ctx.input().scroll_delta.y / 60.0;
        if self.zoom < 0.0001 {
            self.zoom = 0.0001;
        }

        // Allow to drag circle around with mouse
        if ctx.input().pointer.button_down(PointerButton::Primary) {
            self.offset.0 += ctx.input().pointer.delta().x;
            self.offset.1 += ctx.input().pointer.delta().y;
        }
    }
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
