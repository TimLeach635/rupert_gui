use std::f32::consts::PI;

use egui::{Color32, Frame, Grid, Stroke, Vec2};
use itertools::{Itertools as _, repeat_n};
use nalgebra::{Matrix2x3, Matrix3x2, Vector2, Vector3, matrix, vector};

use crate::widgets::{poly_fit_display::PolyFitDisplay, polygon::Polygon};

enum Fit {
    NoFit,
    Fit {
        angle: f32,
        translation: Vector2<f32>,
    },
}

#[derive(Clone)]
pub struct PlotData {
    pub xs: Vec<f32>,
    pub yss: Vec<Vec<f32>>,
    pub min_ys: Vec<f32>,
}

struct FitData {
    fit: Fit,
    plot_data: PlotData,
}

pub struct FitCalculator {
    outer_polygon: Polygon,
    inner_polygon: Polygon,
    fit_data: FitData,
}

impl FitCalculator {
    fn calculate_data(outer_polygon: &Polygon, inner_polygon: &Polygon) -> FitData {
        // TODO: Refactor this, I don't like it

        // Calculate ns and cs
        let outer_vertices = outer_polygon.centered_vertices();
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
        let n_p: Vector3<f32> = vector![
            n_2.x * n_3.y - n_3.x * n_2.y,
            n_3.x * n_1.y - n_1.x * n_3.y,
            n_1.x * n_2.y - n_2.x * n_1.y,
        ];
        assert!(n_p[0] >= 0.0);
        assert!(n_p[1] >= 0.0);
        assert!(n_p[2] >= 0.0);
        let n_mat: Matrix3x2<f32> = matrix![
            n_1.x, n_1.y;
            n_2.x, n_2.y;
            n_3.x, n_3.y;
        ];

        // Values we need
        let inner_vertices = inner_polygon.centered_vertices().clone();

        // Generate x and y values
        let num_points: usize = 400;
        let xs: Vec<f32> = (0..num_points)
            .map(|i| (i as f32) * 2.0 * PI / (num_points as f32))
            .collect();

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

        let mut yss: Vec<Vec<f32>> = vec![Vec::new(); index_choices.len()];
        let mut min_ys: Vec<f32> = Vec::new();
        let mut max_min_y = f32::NEG_INFINITY;
        let mut max_min_x: Option<f32> = None; // The value of x that maximises `max_min`
        let mut max_min_indices: Option<Vec<usize>> = None; // TODO: This is messy and I hate it

        for &x in &xs {
            let mut min_y = f32::INFINITY;
            let mut min_indices: Option<Vec<usize>> = None;
            for (index_idx, index_choice) in index_choices.iter().enumerate() {
                // Each "index choice" is a vector of length N_A, where each element is an
                // index from 0 to (N_B - 1) indicating which choice of "j" (i.e. which inner
                // polygon vertex) to use for the corresponding curve.
                // We then plug all those choices into the dot product with the critical
                // region's normal vector.
                // In the triangle case, this is only a single plane with a single normal, so
                // there is only one dot product, but the number of planes grows with the
                // number of sides of the polygons (roughly with the cube of N_A).
                // TODO: Generalise this to N_A > 3
                assert_eq!(index_choice.len(), 3);
                let mut y: f32 = 0.0;
                for (i, &j) in index_choice.iter().enumerate() {
                    let cos_multiplier = ns[i].dot(inner_vertices[j].to_vec2());
                    let sin_multiplier =
                        ns[i].x * inner_vertices[j].y - inner_vertices[j].x * ns[i].y;
                    let curve_value = cos_multiplier * x.cos() + sin_multiplier * x.sin();
                    let c_value = cs[i] - curve_value;
                    let dot_product_term = n_p[i] * c_value;
                    y += dot_product_term;
                }

                // Keeping track of the mins and maxes
                if y < min_y {
                    min_y = y;
                    min_indices = Some(index_choice.clone());
                }

                yss[index_idx].push(y);
            }

            min_ys.push(min_y);
            if min_y > max_min_y {
                max_min_x = Some(x);
                max_min_y = min_y;
                max_min_indices = min_indices.clone();
            }
        }

        // Our "corner" can be calculated from the max-mins we stored earlier
        // However, this is done very messily and I want to sort it out
        let x_1: f32 = {
            let i = 0;
            let j = max_min_indices.clone().unwrap()[i];
            let cos_multiplier = ns[i].dot(inner_vertices[j].to_vec2());
            let sin_multiplier = ns[i].x * inner_vertices[j].y - inner_vertices[j].x * ns[i].y;
            let curve_value = cos_multiplier * max_min_x.unwrap().cos()
                + sin_multiplier * max_min_x.unwrap().sin();
            cs[i] - curve_value
        };
        let x_2: f32 = {
            let i = 1;
            let j = max_min_indices.clone().unwrap()[i];
            let cos_multiplier = ns[i].dot(inner_vertices[j].to_vec2());
            let sin_multiplier = ns[i].x * inner_vertices[j].y - inner_vertices[j].x * ns[i].y;
            let curve_value = cos_multiplier * max_min_x.unwrap().cos()
                + sin_multiplier * max_min_x.unwrap().sin();
            cs[i] - curve_value
        };
        let x_3: f32 = {
            let i = 2;
            let j = max_min_indices.clone().unwrap()[i];
            let cos_multiplier = ns[i].dot(inner_vertices[j].to_vec2());
            let sin_multiplier = ns[i].x * inner_vertices[j].y - inner_vertices[j].x * ns[i].y;
            let curve_value = cos_multiplier * max_min_x.unwrap().cos()
                + sin_multiplier * max_min_x.unwrap().sin();
            cs[i] - curve_value
        };
        let corner: Vector3<f32> = vector![x_1, x_2, x_3];
        let x_p = corner - (corner.dot(&n_p) * n_p);
        let n_pseudoinverse: Matrix2x3<f32> = n_mat
            .pseudo_inverse(0.0001)
            .expect("Should be able to compute pseudoinverse");

        // Finally!
        let t: Vector2<f32> = n_pseudoinverse * x_p;

        // If this minimum line ever rises above zero, there is a fit!
        // By finding the value of x that attains this maximum, we are in some way recording
        // the "best" fit, for some definition of "best".
        // I give it a buffer to avoid the floating point flickering that I was getting.
        let does_fit = max_min_y > 0.01;
        let fit: Fit = if does_fit {
            Fit::Fit {
                angle: max_min_x.unwrap(),
                translation: t,
            }
        } else {
            Fit::NoFit
        };

        FitData {
            fit,
            plot_data: PlotData { xs, yss, min_ys },
        }
    }

    pub fn new() -> Self {
        let outer = Polygon::default();
        let inner = Polygon::with_style(
            Stroke::new(1.0, Color32::from_rgb(200, 25, 100)),
            Color32::from_rgb(100, 50, 150).linear_multiply(0.25),
        );
        let data = Self::calculate_data(&outer, &inner);
        Self {
            outer_polygon: outer,
            inner_polygon: inner,
            fit_data: data,
        }
    }

    // TODO: Better practice to return an egui::Response, so do so.
    pub fn ui_content(&mut self, ui: &mut egui::Ui) /* -> egui::Response */
    {
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
            let poly_fit_display = if let Fit::Fit {
                angle,
                translation: t,
            } = self.fit_data.fit
            {
                let translation = Vec2::new(t[0], t[1]);

                // TODO: The PolyFitDisplay also performs centering on the vertices, so we are
                // duplicating work here. Decide where it makes most sense to perform that
                // operation, and refactor.
                PolyFitDisplay::with_fit(
                    &self.outer_polygon.centered_vertices(),
                    &self.inner_polygon.centered_vertices(),
                    angle,
                    translation,
                )
            } else {
                PolyFitDisplay::without_fit(
                    &self.outer_polygon.centered_vertices(),
                    &self.inner_polygon.centered_vertices(),
                )
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
            ui.vertical(|ui| {
                if let Fit::Fit {
                    angle: _,
                    translation,
                } = self.fit_data.fit
                {
                    ui.label("Polygon fits!");
                    ui.label(format!("t: ({:.2}, {:.2})", translation.x, translation.y));
                } else {
                    ui.label("No fit");
                }
            });
            ui.end_row();
        });

        // Recalculate
        self.fit_data = Self::calculate_data(&self.outer_polygon, &self.inner_polygon);
    }

    pub fn plot_data(&self) -> PlotData {
        self.fit_data.plot_data.clone()
    }
}
