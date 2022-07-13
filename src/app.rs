use egui::{Align2, Color32, FontId, MultiTouchInfo, PointerButton, Pos2, RichText, Stroke, Ui};

// Scrlol wheel zoom sensitivity. Larger numbers are less sensitive.
const MOUSE_SCROLL_SENSITIVITY: f32 = 100.0;

// Position where options ui is centered
const OPTIONS_UI_ANCHOR_LOCATION: [f32; 2] = [10.0, 10.0];

enum ColorMode {
    Monochrome(String),
    Length(String),
    Radial(String),
}

pub struct TimesCircleApp {
    first_frame: bool,
    paused: bool,
    center: (f32, f32),
    offset: (f32, f32),
    zoom: f32,
    rotation: f32,
    num_points: usize,
    multiplier: f32,
    step_size: f32,
    stroke: f32,
    color: Color32,
    background_color: Color32,
    color_mode: ColorMode,
}

impl eframe::App for TimesCircleApp {
    // Called whenever frame needs to be redrawn, maybe several times a second
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Animate times circle
        if !self.paused && self.multiplier < self.num_points as f32 && self.multiplier > 0.0 {
            self.multiplier += self.step_size;
            ctx.request_repaint();
        }

        self.ui(ctx);
    }
}

impl TimesCircleApp {
    // Initialize app state
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // _cc.egui_ctx.request_repaint();
        TimesCircleApp {
            first_frame: true,
            paused: true,
            center: (0.0, 0.0),
            offset: (0.0, 0.0),
            zoom: 0.85,
            rotation: std::f32::consts::PI,
            num_points: 500,
            multiplier: 2.0,
            step_size: 0.1,
            stroke: 0.3,
            color: Color32::from_rgb(0, 0, 0),
            background_color: Color32::from_rgb(255, 255, 255),
            color_mode: ColorMode::Monochrome("Monochrome".to_string()),
        }
    }

    fn ui(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Paint times circle
            self.times_circle(ui);

            if ui.ui_contains_pointer() || self.first_frame {
                self.handle_mouse(ctx);

                self.first_frame = false;
                ctx.request_repaint();
            }

            // Handle multitouch gestures for mobile
            if let Some(multi_touch) = ui.ctx().multi_touch() {
                self.handle_multitouch(multi_touch);
            }

            // Display options Ui
            egui::Window::new("Settings")
                .collapsible(true)
                .auto_sized()
                .anchor(Align2::LEFT_TOP, OPTIONS_UI_ANCHOR_LOCATION)
                .show(ctx, |ui| {
                    self.options_ui(ui);
                });
        });
    }

    fn options_ui(&mut self, ui: &mut Ui) {
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
        ui.add(
            egui::Slider::new(&mut self.step_size, 0.0..=1.0)
                .text("Step Size")
                .min_decimals(1)
                .max_decimals(3),
        );

        // Stroke width slider
        ui.add(
            egui::Slider::new(&mut self.stroke, 0.0..=1.0)
                .text("Stroke Width")
                .max_decimals(2),
        );

        // Color mode
        ui.horizontal(|ui| {
            ui.label("Color Mode");
            let (color_mode_text, next_mode) = match &self.color_mode {
                ColorMode::Monochrome(label) => (label, ColorMode::Length("Length".to_string())),
                ColorMode::Length(label) => (label, ColorMode::Radial("Radial".to_string())),
                ColorMode::Radial(label) => {
                    (label, ColorMode::Monochrome("Monochrome".to_string()))
                }
            };
            if ui.selectable_label(false, color_mode_text).clicked() {
                self.color_mode = next_mode;
            };
        });

        // Stroke color picker
        ui.horizontal(|ui| {
            ui.label("Stroke Color");
            ui.color_edit_button_srgba(&mut self.color);
        });

        // Background color picker
        ui.horizontal(|ui| {
            ui.label("Background Color");
            ui.color_edit_button_srgba(&mut self.background_color);
        });

        // Playback buttons
        ui.horizontal(|ui| {
            if ui.button("▶").clicked() {
                self.paused = false;
            }
            if ui.button("■").clicked() {
                self.paused = true;
            }
        });
    }

    fn times_circle(&mut self, ui: &mut Ui) {
        // Calculate radius of circle from screen size
        let radius: f32 = if self.center.1 < self.center.0 {
            self.center.1 * self.zoom
        } else {
            self.center.0 * self.zoom
        };

        // Generate evenly spaced points around the circumference of a circle
        let points: Vec<Pos2> = generate_points(self.num_points, self.rotation, radius);

        // Draw lines between points
        for i in 0..self.num_points {
            // Find the point to connect to
            let j = ((i as f32) * self.multiplier) as usize % self.num_points;

            // Transform to world coords
            let p1 = Pos2 {
                x: points[i].x + self.center.0 + self.offset.0,
                y: points[i].y + self.center.1 + self.offset.1,
            };
            let p2 = Pos2 {
                x: points[j].x + self.center.0 + self.offset.0,
                y: points[j].y + self.center.1 + self.offset.1,
            };

            match self.color_mode {
                ColorMode::Monochrome(_) => ui
                    .painter()
                    .line_segment([p1, p2], Stroke::new(self.stroke, self.color)),
                ColorMode::Length(_) => ui
                    .painter()
                    .line_segment([p1, p2], Stroke::new(self.stroke, Color32::DARK_GREEN)),
                ColorMode::Radial(_) => ui
                    .painter()
                    .line_segment([p1, p2], Stroke::new(self.stroke, Color32::DARK_BLUE)),
            }
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
        )
    }

    // Rename to mouse controls?
    fn handle_mouse(&mut self, ctx: &egui::Context) {
        // Calculate center of current screen
        self.center = (
            // This should probably be moved out of this function
            (ctx.available_rect().max.x - ctx.available_rect().min.x) / 2.0,
            (ctx.available_rect().max.y - ctx.available_rect().min.y) / 2.0,
        );

        // Calculate zoom
        self.zoom = f32::max(
            0.85,
            self.zoom + ctx.input().scroll_delta.y / MOUSE_SCROLL_SENSITIVITY,
        );

        // Allow to drag circle around with mouse
        if ctx.input().pointer.button_down(PointerButton::Primary) {
            self.offset.0 += ctx.input().pointer.delta().x;
            self.offset.1 += ctx.input().pointer.delta().y;
        }
    }

    fn handle_multitouch(&mut self, multi_touch: MultiTouchInfo) {
        self.zoom *= multi_touch.zoom_delta;
        self.rotation += multi_touch.rotation_delta;
        self.offset.0 += multi_touch.translation_delta.x;
        self.offset.1 += multi_touch.translation_delta.y;
    }
}

// Generate the coordinates of the points on the circle
fn generate_points(num_points: usize, start_angle: f32, radius: f32) -> Vec<Pos2> {
    let n: f32 = num_points as f32;
    let mut points: Vec<Pos2> = Vec::with_capacity(num_points);
    let mut angle: f32 = start_angle;
    for _ in 0..num_points {
        let point = Pos2 {
            x: radius * angle.cos(),
            y: radius * angle.sin(),
        };
        angle += std::f32::consts::TAU / n;
        points.push(point);
    }
    points
}
