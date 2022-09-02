use egui::{
    Align2, Color32, CursorIcon, FontId, MultiTouchInfo, PointerButton, Pos2, RichText, Stroke, Ui,
};

enum ColorMode {
    Monochrome(String),
    Length(String),
    Radial(String),
}

pub struct TimesCircleApp {
    first_frame: bool,
    center: Pos2,
    offset: Pos2,
    zoom: f32,
    rotation: f32,
    paused: bool,
    num_points: usize,
    multiplier: f32,
    step_size: f32,
    stroke: f32,
    color: Color32,
    background_color: Color32,
    color_mode: ColorMode,
    show_points: bool,
}

impl eframe::App for TimesCircleApp {
    // Called whenever frame needs to be redrawn, maybe several times a second
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Calculate center of window (may change as window is resized)
        self.center = Pos2 {
            x: (ctx.available_rect().max.x - ctx.available_rect().min.x) / 2.0,
            y: (ctx.available_rect().max.y - ctx.available_rect().min.y) / 2.0,
        };

        // Draw ui
        self.ui(ctx);

        // Request a repaint (update gets called again immediately after this)
        // When animating circle
        if !self.paused && self.multiplier < self.num_points as f32 && self.multiplier > 0.0 {
            self.multiplier += self.step_size;
            ctx.request_repaint();
        }
    }
}

impl TimesCircleApp {
    // Initialize app state
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        TimesCircleApp {
            first_frame: true,
            center: Pos2 { x: 0.0, y: 0.0 },
            offset: Pos2 { x: 0.0, y: 0.0 },
            zoom: 0.85,
            rotation: std::f32::consts::PI,
            paused: true,
            num_points: 10,
            multiplier: 2.0,
            step_size: 0.1,
            stroke: 1.0,
            color: Color32::from_rgb(0, 0, 0),
            background_color: Color32::from_rgb(255, 255, 255),
            color_mode: ColorMode::Monochrome("Monochrome".to_string()),
            show_points: true,
        }
    }

    fn ui(&mut self, ctx: &egui::Context) {
        let frame = egui::Frame::default().fill(self.background_color);
        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            // Handle mouse controls
            if ui.ui_contains_pointer() || self.first_frame {
                self.handle_mouse_inputs(ctx);

                // Use first frame to fix bug
                self.first_frame = false;
                ctx.request_repaint();
            }

            // Handle multitouch gestures for mobile
            if let Some(multi_touch) = ui.ctx().multi_touch() {
                self.handle_multitouch_inputs(multi_touch);
            }

            // Display options Ui
            egui::Window::new("Options")
                .collapsible(true)
                .auto_sized()
                .anchor(Align2::LEFT_TOP, [10.0, 10.0])
                .show(ctx, |ui| {
                    self.options_ui(ui);
                });

            // Paint times circle
            self.paint_times_circle(ui);
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
                .min_decimals(2)
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

        // Show points radio
        ui.horizontal(|ui| {
            ui.label("Show Points");
            ui.radio_value(&mut self.show_points, true, "Yes");
            ui.radio_value(&mut self.show_points, false, "No");
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

    fn paint_times_circle(&mut self, ui: &mut Ui) {
        // Calculate radius of circle from screen size
        let radius: f32 = if self.center.y < self.center.x {
            self.center.y * self.zoom
        } else {
            self.center.x * self.zoom
        };

        // Generate evenly spaced points around the circumference of a circle
        let points: Vec<Pos2> = TimesCircleApp::generate_points(self.num_points, self.rotation);

        if (self.show_points) {
            self.draw_perimeter_points(radius, &points, ui);
        }

        // FIXME This could be refactored
        // Draw lines between points
        for i in 0..self.num_points {
            // Find the point to connect to
            let j = ((i as f32) * self.multiplier) as usize % self.num_points;

            // Transform to world coords
            let p1 = Pos2 {
                x: points[i].x * radius + self.center.x + self.offset.x,
                y: points[i].y * radius + self.center.y + self.offset.y,
            };
            let p2 = Pos2 {
                x: points[j].x * radius + self.center.x + self.offset.x,
                y: points[j].y * radius + self.center.y + self.offset.y,
            };

            // TODO implement other color modes
            // Draw line with using appropriate color mode
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
                x: self.center.x + self.offset.x,
                y: self.center.y + self.offset.y,
            },
            radius,
            Color32::TRANSPARENT,
            Stroke::new(self.stroke, self.color),
        )
    }

    fn draw_perimeter_points(&mut self, radius: f32, points: &Vec<Pos2>, ui: &mut Ui) {
        // Draw points
        for i in 0..self.num_points {
            let p = Pos2 {
                x: points[i].x * radius + self.center.x + self.offset.x,
                y: points[i].y * radius + self.center.y + self.offset.y,
            };
            ui.painter().circle(p, 2.0, Color32::BLACK, Stroke::new(self.stroke, Color32::DARK_BLUE));
        }
    }

    fn handle_mouse_inputs(&mut self, ctx: &egui::Context) {
        // Allow to drag circle around with mouse
        if ctx.input().pointer.button_down(PointerButton::Primary) {
            ctx.output().cursor_icon = CursorIcon::Grab;
            self.offset.x += ctx.input().pointer.delta().x;
            self.offset.y += ctx.input().pointer.delta().y;
        }

        // Zoom
        if let Some(pos) = ctx.pointer_hover_pos() {
            let factor = ctx.input().zoom_delta();
            self.zoom *= factor;

            // Change offset to zoom into mouse location
            self.offset.x += (self.center.x + self.offset.x - pos.x) * (factor - 1.0);
            self.offset.y += (self.center.y + self.offset.y - pos.y) * (factor - 1.0);
        }
    }

    fn handle_multitouch_inputs(&mut self, multi_touch: MultiTouchInfo) {
        self.zoom *= multi_touch.zoom_delta;
        self.rotation += multi_touch.rotation_delta;
        self.offset.x += multi_touch.translation_delta.x;
        self.offset.y += multi_touch.translation_delta.y;
    }

    // Generate evenly spaced points around a circle of radius 1, starting at given start_angle
    fn generate_points(num_points: usize, start_angle: f32) -> Vec<Pos2> {
        let n: f32 = num_points as f32;
        let mut points: Vec<Pos2> = Vec::with_capacity(num_points);
        let mut angle: f32 = start_angle;
        for _ in 0..num_points {
            let point = Pos2 {
                x: f32::cos(angle),
                y: f32::sin(angle),
            };
            angle += std::f32::consts::TAU / n;
            points.push(point);
        }
        points
    }
}