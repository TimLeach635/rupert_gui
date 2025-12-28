use std::f64::consts::PI;

use egui::{Color32, Frame, Grid, Vec2};
use egui_plot::{Legend, Line, Plot, PlotPoints};

use crate::widgets::polygon::Polygon;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct RupertApp {
    // Example stuff:
    label: String,

    #[serde(skip)]
    outer_polygon: Polygon,

    #[serde(skip)]
    inner_polygon: Polygon,
}

impl Default for RupertApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            outer_polygon: Polygon::default(),
            inner_polygon: Polygon::default(),
        }
    }
}

impl RupertApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
    }
}

impl eframe::App for RupertApp {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::MenuBar::new().ui(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("Rupert");

            self.outer_polygon.ui_control(ui);

            ui.separator();

            Grid::new("polygon_holder").show(ui, |ui| {
                ui.label("Polygon A (outer)");
                ui.label("Polygon B (inner)");
                ui.end_row();

                Frame::canvas(ui.style()).show(ui, |ui| {
                    self.outer_polygon.ui_content(ui);
                });
                Frame::canvas(ui.style()).show(ui, |ui| {
                    self.inner_polygon.ui_content(ui);
                });
                ui.end_row();

                ui.vertical(|ui| {
                    self.outer_polygon.ui_readout(ui);
                });
                ui.vertical(|ui| {
                    self.inner_polygon.ui_readout(ui);
                });
                ui.end_row();
            });

            ui.separator();

            // TODO: Split this out into its own widget
            {
                // Values we need
                let mut centroid = Vec2::ZERO;
                let vertices = self.inner_polygon.vertices();
                for vertex in &vertices {
                    centroid += vertex.to_vec2();
                }
                centroid /= vertices.len() as f32;
                let vertices: Vec<Vec2> = vertices.iter().map(|v| v.to_vec2() - centroid).collect();

                // Generate x and y values
                let num_points: usize = 400;
                let xs: Vec<f64> = (0..num_points)
                    .map(|i| (i as f64) * 2.0 * PI / (num_points as f64))
                    .collect();
                let yss: Vec<Vec<f64>> = vertices
                    .iter()
                    .map(|v| {
                        xs.iter()
                            .map(|&x| (v.x as f64) * x.cos() + (v.y as f64) * x.sin())
                            .collect()
                    })
                    .collect();
                let mut max_ys: Vec<f64> = Vec::new();
                for idx in 0..xs.len() {
                    let mut max = f64::NEG_INFINITY;
                    for ys in &yss {
                        if ys[idx] > max {
                            max = ys[idx];
                        }
                    }
                    max_ys.push(max);
                }
                let lines: Vec<Line<'_>> = yss
                    .iter()
                    .enumerate()
                    .map(|(idx, ys)| {
                        Line::new(
                            format!("j = {}", idx + 1),
                            xs.iter()
                                .zip(ys.iter())
                                .map(|(&x, &y)| [x, y])
                                .collect::<PlotPoints<'_>>(),
                        )
                        .color(
                            ui.style().visuals.weak_text_color.unwrap_or(
                                ui.style()
                                    .visuals
                                    .text_color()
                                    .gamma_multiply(ui.style().visuals.weak_text_alpha),
                            ),
                        )
                    })
                    .collect();
                let max_line = Line::new(
                    "Maximum",
                    xs.iter()
                        .zip(max_ys.iter())
                        .map(|(&x, &y)| [x, y])
                        .collect::<PlotPoints<'_>>(),
                )
                .color(Color32::from_rgb(200, 100, 100));

                Plot::new("cosine_plot_test")
                    .width(600.0)
                    .height(300.0)
                    .legend(Legend::default())
                    .show(ui, |plot_ui| {
                        for line in lines {
                            plot_ui.line(line);
                        }
                        plot_ui.line(max_line);
                    });
            }

            ui.separator();

            ui.add(egui::github_link_file!(
                "https://github.com/TimLeach635/rupert_gui/blob/main/",
                "Source code."
            ));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
