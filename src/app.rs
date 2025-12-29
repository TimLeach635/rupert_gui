use std::f64::consts::PI;

use egui::{Color32, Frame, Grid, Stroke, Vec2};
use egui_plot::{Line, Plot, PlotPoints};
use itertools::{Itertools, repeat_n};

use crate::widgets::{poly_fit_display::PolyFitDisplay, polygon::Polygon};

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
            inner_polygon: Polygon::with_style(
                Stroke::new(1.0, Color32::from_rgb(200, 25, 100)),
                Color32::from_rgb(100, 50, 150).linear_multiply(0.25),
            ),
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

            // TODO: Split this out into its own widget
            // Calculate ns and cs
            let outer_vertices = self.outer_polygon.vertices();
            let mut ns: Vec<Vec2> = Vec::new();
            let mut cs: Vec<f32> = Vec::new();
            for to_idx in 0..outer_vertices.len() {
                let mut from_idx = outer_vertices.len() - 1;
                if to_idx != 0 {
                    from_idx = to_idx - 1;
                }

                let edge = outer_vertices[to_idx].to_vec2() - outer_vertices[from_idx].to_vec2();
                // Compute the normal by rotating the edge clockwise 90 degrees and normalising
                // This works because we have defined the polygon to have its vertices defined
                // anti-clockwise
                // TODO: Make this method robust by detecting which way round the vertices are
                // defined
                let n = edge.rot90().normalized();
                let c = n.dot(outer_vertices[to_idx].to_vec2());

                ns.push(n);
                cs.push(c);
            }

            // Find the critical region
            // TODO: Generalise to N_A > 3
            let n_1 = ns[0];
            let n_2 = ns[1];
            let n_3 = ns[2];
            let critical_normal: Vec<f32> = vec![
                n_2.x * n_3.y - n_3.x * n_2.y,
                n_3.x * n_1.y - n_1.x * n_3.y,
                n_1.x * n_2.y - n_2.x * n_1.y,
            ];
            assert!(critical_normal[0] >= 0.0);
            assert!(critical_normal[1] >= 0.0);
            assert!(critical_normal[2] >= 0.0);

            // Values we need
            let mut centroid = Vec2::ZERO;
            let inner_vertices = self.inner_polygon.vertices();
            for vertex in &inner_vertices {
                centroid += vertex.to_vec2();
            }
            centroid /= inner_vertices.len() as f32;
            let bs: Vec<Vec2> = inner_vertices
                .iter()
                .map(|v| v.to_vec2() - centroid)
                .collect();

            // Generate x and y values
            let num_points: usize = 400;
            let xs: Vec<f64> = (0..num_points)
                .map(|i| (i as f64) * 2.0 * PI / (num_points as f64))
                .collect();

            let mut yss: Vec<Vec<f64>> = Vec::new();
            // We need to generate a potentially very large number of lines here: there are N_B
            // to the power of N_A different cosine curves that we need to consider.
            // We can generate a list of indices for these curves by getting "permutations with
            // replacement", which according to
            // https://docs.rs/itertools/latest/itertools/trait.Itertools.html#method.permutations
            // is done with the following:
            let index_choices = repeat_n(0..inner_vertices.len(), outer_vertices.len())
                .multi_cartesian_product()
                .collect_vec();
            // TODO: Generalise this assertion to (N_B)^(N_A)
            assert_eq!(index_choices.len(), 27);
            for index_choice in index_choices {
                // Each "index choice" is a vector of length N_A, where each element is an
                // index from 0 to (N_B - 1) indicating which choice of "j" (i.e. which inner
                // polygon vertex) to use for the corresponding curve.
                // We then plug all those choices into the dot product with the critical
                // region's normal vector.
                // In the triangle case, this is only a single plane with a single normal, so
                // there is only one dot product, but the number of planes grows with the
                // number of sides of the polygons (roughly with the cube of N_A).
                // TODO: Generalise this to N_A > 3
                assert_eq!(critical_normal.len(), 3);
                assert_eq!(index_choice.len(), 3);
                let mut ys: Vec<f64> = Vec::new();
                for &x in &xs {
                    let mut y: f64 = 0.0;
                    for (i, &j) in index_choice.iter().enumerate() {
                        let cos_multiplier = ns[i].dot(bs[j]);
                        let sin_multiplier = ns[i].x * bs[j].y - bs[j].x * ns[i].y;
                        let curve_value =
                            (cos_multiplier as f64) * x.cos() + (sin_multiplier as f64) * x.sin();
                        let c_value = (cs[i] as f64) - curve_value;
                        let dot_product_term = (critical_normal[i] as f64) * c_value;
                        y += dot_product_term;
                    }
                    ys.push(y);
                }
                yss.push(ys);
            }

            let mut min_ys: Vec<f64> = Vec::new();
            let mut max_min = f64::NEG_INFINITY;
            let mut maximiser: Option<f64> = None; // The value of x that maximises `max_min`
            for idx in 0..xs.len() {
                let mut min = f64::INFINITY;
                for ys in &yss {
                    if ys[idx] < min {
                        min = ys[idx];
                    }
                }
                if min > max_min {
                    max_min = min;
                    maximiser = Some(xs[idx]);
                }
                min_ys.push(min);
            }
            // If this minimum line ever rises above zero, there is a fit!
            // By finding the value of x that attains this maximum, we are in some way recording
            // the "best" fit, for some definition of "best".
            // I give it a buffer to avoid the floating point flickering that I was getting.
            let does_fit = max_min > 0.01;

            // Create plot lines
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
            let min_line = Line::new(
                "Minimum",
                xs.iter()
                    .zip(min_ys.iter())
                    .map(|(&x, &y)| [x, y])
                    .collect::<PlotPoints<'_>>(),
            )
            .color(Color32::from_rgb(200, 100, 100));

            // Show the polygons after we've already done the calculations
            Grid::new("polygon_holder").show(ui, |ui| {
                ui.label("Polygon A (outer)");
                ui.label("Polygon B (inner)");
                ui.label("Fit");
                ui.end_row();

                Frame::canvas(ui.style()).show(ui, |ui| {
                    self.outer_polygon.ui_content(ui);
                });
                Frame::canvas(ui.style()).show(ui, |ui| {
                    self.inner_polygon.ui_content(ui);
                });
                let poly_fit_display = match does_fit {
                    true => PolyFitDisplay::with_fit(
                        &outer_vertices,
                        &inner_vertices,
                        maximiser.unwrap() as f32,
                        Vec2::ZERO, // TODO: Compute translation
                    ),
                    false => PolyFitDisplay::without_fit(&outer_vertices, &inner_vertices),
                };
                Frame::canvas(ui.style()).show(ui, |ui| {
                    poly_fit_display.ui_content(ui);
                });
                ui.end_row();

                ui.vertical(|ui| {
                    self.outer_polygon.ui_readout(ui);
                });
                ui.vertical(|ui| {
                    self.inner_polygon.ui_readout(ui);
                });
                ui.label(match does_fit {
                    true => "Polygon fits!",
                    false => "No fit",
                });
                ui.end_row();
            });

            ui.separator();

            Plot::new("cosine_plot_test")
                .width(600.0)
                .height(300.0)
                .show(ui, |plot_ui| {
                    for line in lines {
                        plot_ui.line(line);
                    }
                    plot_ui.line(min_line);
                });
            // TODO: Widget split ends here

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
