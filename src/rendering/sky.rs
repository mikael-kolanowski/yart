use crate::color::Color;
use crate::math::Lerp;
use crate::math::Ray;

pub trait SkyBox {
    fn color(&self, ray: Ray) -> Color;
}

pub struct LinearGradientSkyBox {
    pub from: Color,
    pub to: Color,
}

impl SkyBox for LinearGradientSkyBox {
    fn color(&self, ray: Ray) -> Color {
        let t = 0.5 * (ray.direction.normalized().y + 1.0);
        Color::lerp(self.from, self.to, t)
    }
}

pub struct SolidColorSkyBox {
    pub color: Color,
}

impl SkyBox for SolidColorSkyBox {
    fn color(&self, _ray: Ray) -> Color {
        self.color
    }
}
