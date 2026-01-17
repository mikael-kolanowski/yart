use std::ops;

use super::Lerp;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);
    pub const ONES: Self = Self::new(1.0, 1.0, 1.0);

    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Vec3 { x, y, z }
    }

    pub fn dot(self, rhs: Vec3) -> f64 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub fn length(self) -> f64 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt()
    }

    pub fn length_squared(self) -> f64 {
        self.x.powi(2) + self.y.powi(2) + self.z.powi(2)
    }

    pub fn normalized(self) -> Self {
        let len = self.length();
        return Vec3::new(self.x / len, self.y / len, self.z / len);
    }

    pub fn to_point(&self) -> Point3 {
        return Point3 {
            x: self.x,
            y: self.y,
            z: self.z,
        };
    }
}

impl Lerp<Vec3> for Vec3 {
    fn lerp(start: Vec3, end: Vec3, t: f64) -> Vec3 {
        (1.0 - t) * start + t * end
    }
}

impl ops::Add<Vec3> for Vec3 {
    type Output = Vec3;
    fn add(self, rhs: Vec3) -> Vec3 {
        return Vec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        };
    }
}

impl ops::Sub<Vec3> for Vec3 {
    type Output = Vec3;
    fn sub(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl ops::Mul<f64> for Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: f64) -> Vec3 {
        Vec3 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl ops::Mul<Vec3> for f64 {
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: rhs.x * self,
            y: rhs.y * self,
            z: rhs.z * self,
        }
    }
}

impl ops::Div<f64> for Vec3 {
    type Output = Vec3;
    fn div(self, rhs: f64) -> Self::Output {
        Vec3 {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl ops::Neg for Vec3 {
    type Output = Vec3;
    fn neg(self) -> Self::Output {
        Vec3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

pub type Point3 = Vec3;

#[cfg(test)]
pub mod tests {
    use super::Vec3;

    #[test]
    fn test_add() {
        let u = Vec3::new(1.0, 2.0, 3.0);
        let v = Vec3::new(4.0, 5.0, 6.0);
        let expected = Vec3::new(5.0, 7.0, 9.0);
        assert_eq!(u + v, expected);
    }

    #[test]
    fn test_sub() {
        let u = Vec3::new(4.0, 5.0, 6.0);
        let v = Vec3::new(1.0, 2.0, 3.0);
        let expected = Vec3::new(3.0, 3.0, 3.0);
        assert_eq!(u - v, expected);
    }

    #[test]
    fn test_scale() {
        let u = Vec3::new(4.0, 5.0, 6.0);
        assert_eq!(u * 2.0, Vec3::new(8.0, 10.0, 12.0));
    }

    #[test]
    fn test_dot() {
        let u = Vec3::new(1.0, 2.0, 3.0);
        let v = Vec3::new(1.0, 1.0, 1.0);
        let expected = 6.0;
        assert_eq!(u.dot(v), expected);
    }

    #[test]
    fn test_negate() {
        assert_eq!(-Vec3::ZERO, Vec3::ZERO);

        assert_eq!(-Vec3::new(1.0, 2.0, 3.0), Vec3::new(-1.0, -2.0, -3.0));
    }
}
