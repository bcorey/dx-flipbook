use dioxus::html::geometry::{euclid::Vector2D, ClientSpace};

#[derive(Clone, PartialEq, Debug)]
pub struct RectData {
    size: Vector2D<f64, ClientSpace>,
    position: Vector2D<f64, ClientSpace>,
}

impl RectData {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            size: Vector2D::new(width, height),
            position: Vector2D::new(x, y),
        }
    }

    pub fn interpolate_to(&self, t: f32, to: &Self) -> Self {
        let size = self.size.lerp(to.size, t.into());
        let position = self.position.lerp(to.position, t.into());

        Self { size, position }
    }

    pub fn to_css(&self) -> String {
        format!(
            "width: {}px; height: {}px; left: {}px; top: {}px;",
            self.size.x, self.size.y, self.position.x, self.position.y
        )
    }
}
