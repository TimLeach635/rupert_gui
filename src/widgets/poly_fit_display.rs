use egui::{
    Color32, Pos2, Rect, Sense, Stroke, Vec2,
    emath::{self, Rot2},
    epaint::PathShape,
};

pub enum PolyFitResult {
    NoFit,
    Fit { angle: f32, translation: Vec2 },
}

pub struct PolyFitDisplay {
    outer_vertices: Vec<Pos2>,
    inner_vertices: Vec<Pos2>,
    outer_stroke: Stroke,
    outer_fill: Color32,
    inner_stroke: Stroke,
    inner_fill: Color32,
    fit: PolyFitResult,

    canvas_size: f32,
}

impl PolyFitDisplay {
    pub fn with_fit(
        outer_vertices: &[Pos2],
        inner_vertices: &[Pos2],
        angle: f32,
        translation: Vec2,
    ) -> Self {
        Self {
            outer_vertices: outer_vertices.to_vec(),
            inner_vertices: inner_vertices.to_vec(),
            outer_stroke: Stroke::new(1.0, Color32::from_rgb(25, 200, 100)),
            outer_fill: Color32::from_rgb(50, 100, 150).linear_multiply(0.25),
            inner_stroke: Stroke::new(1.0, Color32::from_rgb(200, 25, 100)),
            inner_fill: Color32::from_rgb(100, 50, 150).linear_multiply(0.25),
            fit: PolyFitResult::Fit { angle, translation },
            canvas_size: 200.0,
        }
    }

    pub fn without_fit(outer_vertices: &[Pos2], inner_vertices: &[Pos2]) -> Self {
        Self {
            outer_vertices: outer_vertices.to_vec(),
            inner_vertices: inner_vertices.to_vec(),
            outer_stroke: Stroke::new(1.0, Color32::from_rgb(25, 200, 100)),
            outer_fill: Color32::from_rgb(50, 100, 150).linear_multiply(0.25),
            inner_stroke: Stroke::new(1.0, Color32::from_rgb(200, 25, 100)),
            inner_fill: Color32::from_rgb(100, 50, 150).linear_multiply(0.25),
            fit: PolyFitResult::NoFit,
            canvas_size: 200.0,
        }
    }

    fn centered_outer_vertices(&self) -> Vec<Pos2> {
        let mut centroid = Vec2::ZERO;
        for a in &self.outer_vertices {
            centroid += a.to_vec2();
        }
        centroid /= self.outer_vertices.len() as f32;
        let offset = Vec2::splat(self.canvas_size / 2.0) - centroid;

        self.outer_vertices.iter().map(|&a| a + offset).collect()
    }

    fn transformed_inner_vertices(&self) -> Option<Vec<Pos2>> {
        match self.fit {
            PolyFitResult::NoFit => None,
            PolyFitResult::Fit { angle, translation } => {
                let mut centroid = Vec2::ZERO;
                for b in &self.inner_vertices {
                    centroid += b.to_vec2();
                }
                centroid /= self.inner_vertices.len() as f32;

                // The maths is a bit different to the outer vertices, because we need to center
                // the shape at 0 (rather than the centre of the frame), then perform the rotation,
                // _then_ the translation, and only then can we offset it back to the centre of
                // the frame.
                let result = self
                    .inner_vertices
                    .iter()
                    .map(|&a| {
                        let centered = a.to_vec2() - centroid;
                        let rotated = Rot2::from_angle(-angle) * centered;
                        let translated = rotated + translation;
                        let centered_in_frame = translated + Vec2::splat(self.canvas_size / 2.0);
                        centered_in_frame.to_pos2()
                    })
                    .collect();
                Some(result)
            }
        }
    }

    pub fn ui_content(&self, ui: &mut egui::Ui) -> egui::Response {
        let (response, painter) = ui.allocate_painter(
            Vec2::new(self.canvas_size, self.canvas_size),
            Sense::hover(),
        );

        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.size()),
            response.rect,
        );

        let screen_outer_points: Vec<Pos2> = self
            .centered_outer_vertices()
            .iter()
            .map(|&a| to_screen * a)
            .collect();
        let outer_shape =
            PathShape::convex_polygon(screen_outer_points, self.outer_fill, self.outer_stroke);
        painter.add(outer_shape);

        if let PolyFitResult::Fit {
            angle: _,
            translation: _,
        } = self.fit
        {
            let screen_inner_points: Vec<Pos2> = self
                .transformed_inner_vertices()
                .unwrap()
                .iter()
                .map(|&b| to_screen * b)
                .collect();
            let inner_shape =
                PathShape::convex_polygon(screen_inner_points, self.inner_fill, self.inner_stroke);
            painter.add(inner_shape);
        }

        response
    }
}
