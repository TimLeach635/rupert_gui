use std::f32::consts::PI;

use egui::{
    Color32, Pos2, Rect, Sense, Shape, Stroke, Vec2, emath,
    epaint::{self, PathShape},
    pos2,
};

pub struct Polygon {
    vertices: Vec<Pos2>,
    stroke: Stroke,
    fill: Color32,
    bounding_box_stroke: Stroke,
}

impl Default for Polygon {
    fn default() -> Self {
        let offset = 100.0;

        Self {
            vertices: vec![
                pos2(50.0 + offset, 0.0 + offset),
                pos2(-25.0 + offset, 3.0f32.sqrt() * 25.0 + offset),
                pos2(-25.0 + offset, -3.0f32.sqrt() * 25.0 + offset),
            ],
            stroke: Stroke::new(1.0, Color32::from_rgb(25, 200, 100)),
            fill: Color32::from_rgb(50, 100, 150).linear_multiply(0.25),
            bounding_box_stroke: Stroke::new(0.0, Color32::LIGHT_GREEN.linear_multiply(0.25)),
        }
    }
}

impl Polygon {
    pub fn regular(n_vertices: usize) -> Self {
        let offset = 100.0;
        let radius = 50.0;
        let angle_step = 2.0 * PI / (n_vertices as f32);
        let mut vertices: Vec<Pos2> = Vec::new();
        for idx in 0..n_vertices {
            let angle = angle_step * (idx as f32);
            vertices.push(pos2(
                offset + angle.cos() * radius,
                offset + angle.sin() * radius,
            ));
        }

        Self {
            vertices,
            ..Default::default()
        }
    }
    pub fn with_style(stroke: Stroke, fill: Color32) -> Self {
        Self {
            stroke,
            fill,
            ..Default::default()
        }
    }
    pub fn regular_with_style(n_vertices: usize, stroke: Stroke, fill: Color32) -> Self {
        let mut result = Self::regular(n_vertices);
        result.stroke = stroke;
        result.fill = fill;
        result
    }

    pub fn centered_vertices(&self) -> Vec<Pos2> {
        let mut centroid = Vec2::ZERO;
        for v in &self.vertices {
            centroid += v.to_vec2();
        }
        centroid /= self.vertices.len() as f32;

        self.vertices.iter().map(|&a| a - centroid).collect()
    }

    pub fn ui_content(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let (response, painter) = ui.allocate_painter(Vec2::new(200.0, 200.0), Sense::hover());

        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.size()),
            response.rect,
        );

        let vertex_radius = 8.0;

        let vertex_handles: Vec<Shape> = self
            .vertices
            .iter_mut()
            .enumerate()
            .map(|(i, point)| {
                let size = Vec2::splat(2.0 * vertex_radius);

                let point_in_screen = to_screen.transform_pos(*point);
                let point_rect = Rect::from_center_size(point_in_screen, size);
                let point_id = response.id.with(i);
                let point_response = ui.interact(point_rect, point_id, Sense::drag());

                *point += point_response.drag_delta();
                *point = to_screen.from().clamp(*point);

                let point_in_screen = to_screen.transform_pos(*point);
                let stroke = ui.style().interact(&point_response).fg_stroke;

                Shape::circle_stroke(point_in_screen, vertex_radius, stroke)
            })
            .collect();

        let points_in_screen: Vec<Pos2> = self.vertices.iter().map(|p| to_screen * *p).collect();

        let shape = PathShape::convex_polygon(points_in_screen, self.fill, self.stroke);
        painter.add(epaint::RectShape::stroke(
            shape.visual_bounding_rect(),
            0.0,
            self.bounding_box_stroke,
            egui::StrokeKind::Outside,
        ));
        painter.add(shape);

        painter.extend(vertex_handles);

        response
    }

    pub fn ui_readout(&mut self, ui: &mut egui::Ui) {
        for (idx, vertex) in self.vertices.iter().enumerate() {
            ui.label(format!(
                "Vertex {}: ({:.2}, {:.2})",
                idx + 1,
                vertex.x,
                vertex.y
            ));
        }
    }
}
