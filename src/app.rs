use egui::Color32;
use egui_plot::{Line, Plot, PlotPoints};

use crate::widgets::fit_calculator::FitCalculator;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct RupertApp {
    // Example stuff:
    label: String,

    #[serde(skip)]
    fit_calculator: FitCalculator,
}

impl Default for RupertApp {
    fn default() -> Self {
        Self {
            label: "Hello World!".to_owned(),
            fit_calculator: FitCalculator::new(),
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
            ui.label(
                "Use the handles below to customise the shapes of the polygons, \
                and explore how that affects the ability of one to fit within the other.",
            );

            ui.separator();

            self.fit_calculator.ui_content(ui);

            ui.separator();

            let plot_data = self.fit_calculator.plot_data();
            // Create plot lines
            let lines: Vec<Line<'_>> = plot_data
                .yss
                .iter()
                .enumerate()
                .map(|(idx, ys)| {
                    Line::new(
                        format!("j = {}", idx + 1),
                        plot_data
                            .xs
                            .iter()
                            .zip(ys.iter())
                            .map(|(&x, &y)| [x as f64, y as f64])
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
            let min_line = Line::new(
                "Minimum",
                plot_data
                    .xs
                    .iter()
                    .zip(plot_data.min_ys.iter())
                    .map(|(&x, &y)| [x as f64, y as f64])
                    .collect::<PlotPoints<'_>>(),
            )
            .color(Color32::from_rgb(200, 100, 100));

            Plot::new("cosine_plot_test")
                .width(600.0)
                .height(300.0)
                .show(ui, |plot_ui| {
                    for line in lines {
                        plot_ui.line(line);
                    }
                    plot_ui.line(min_line);
                });

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
