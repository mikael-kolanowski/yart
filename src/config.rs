use std::fs;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;
use serde::Serializer;
use serde::de::{self, Deserializer};

use crate::math::{Point3, Vec3};

fn serialize_vec3<S>(v: &Vec3, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!("{}, {}, {}", v.x, v.y, v.z))
}

fn serialize_point3<S>(p: &Point3, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!("{}, {}, {}", p.0.x, p.0.y, p.0.z))
}

fn serialize_aspect_ratio<S>(ar: &f64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let (w, h) = if *ar > 0.0 {
        (16.0 * ar, 16.0)
    } else {
        (16.0, 16.0 / *ar)
    };
    serializer.serialize_str(&format!("{}:{}", w as u32, h as u32))
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub camera: CameraConfig,
    pub renderer: RendererConfig,
    pub image: ImageConfig,
    pub materials: Vec<MaterialConfig>,
    pub objects: Vec<ObjectConfig>,
    pub sky: SkyConfig,
    #[serde(default)]
    pub viewport: ViewportConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CameraConfig {
    #[serde(
        serialize_with = "serialize_aspect_ratio",
        deserialize_with = "deserialize_aspect_ratio"
    )]
    pub aspect_ratio: f64,
    pub field_of_view: u32,
    #[serde(
        serialize_with = "serialize_point3",
        deserialize_with = "deserialize_point3"
    )]
    pub position: Point3,
    #[serde(
        serialize_with = "serialize_point3",
        deserialize_with = "deserialize_point3"
    )]
    pub look_at: Point3,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RendererConfig {
    pub samples_per_pixel: u32,
    pub max_bounces: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ImageConfig {
    pub width: u32,
    pub output: PathBuf,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct ViewportConfig {
    pub width: u32,
    pub samples_per_pixel: u32,
    pub max_bounces: u32,
}

impl Default for ViewportConfig {
    fn default() -> Self {
        Self {
            width: 200,
            samples_per_pixel: 5,
            max_bounces: 6,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum SkyConfig {
    #[serde(rename = "linear-gradient")]
    LinearGradient {
        #[serde(
            serialize_with = "serialize_vec3",
            deserialize_with = "deserialize_vec3"
        )]
        from: Vec3,
        #[serde(
            serialize_with = "serialize_vec3",
            deserialize_with = "deserialize_vec3"
        )]
        to: Vec3,
    },

    #[serde(rename = "solid")]
    Solid {
        #[serde(
            serialize_with = "serialize_vec3",
            deserialize_with = "deserialize_vec3"
        )]
        color: Vec3,
    },
}

#[derive(Debug, Deserialize, Clone, Serialize)]
#[serde(tag = "type")]
pub enum MaterialConfig {
    #[serde(rename = "lambertian")]
    Lambertian {
        name: String,
        #[serde(
            serialize_with = "serialize_vec3",
            deserialize_with = "deserialize_vec3"
        )]
        albedo: Vec3,
    },

    #[serde(rename = "metal")]
    Metal {
        name: String,
        #[serde(
            serialize_with = "serialize_vec3",
            deserialize_with = "deserialize_vec3"
        )]
        albedo: Vec3,
        fuzz: f64,
    },
    #[serde(rename = "normal_vis")]
    NormalVisualization { name: String },
}

#[derive(Debug, Deserialize, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ObjectConfig {
    #[serde(rename = "sphere")]
    Sphere {
        #[serde(
            serialize_with = "serialize_vec3",
            deserialize_with = "deserialize_vec3"
        )]
        position: Vec3,
        radius: f64,
        material: String,
    },
    #[serde(rename = "triangle")]
    Triangle {
        #[serde(
            serialize_with = "serialize_point3",
            deserialize_with = "deserialize_point3"
        )]
        p1: Point3,
        #[serde(
            serialize_with = "serialize_point3",
            deserialize_with = "deserialize_point3"
        )]
        p2: Point3,
        #[serde(
            serialize_with = "serialize_point3",
            deserialize_with = "deserialize_point3"
        )]
        p3: Point3,
        material: String,
    },
    #[serde(rename = "mesh")]
    Mesh { path: PathBuf, material: String },
}

impl Config {
    pub fn from_path(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: Self = toml::from_str(&contents)?;
        Ok(config)
    }

    pub fn save_to_file(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let contents = toml::to_string_pretty(&self)?;
        fs::write(path, contents)?;
        Ok(())
    }
}

impl MaterialConfig {
    pub fn name(&self) -> &str {
        match self {
            MaterialConfig::Lambertian { name, .. } => name,
            MaterialConfig::Metal { name, .. } => name,
            MaterialConfig::NormalVisualization { name } => name,
        }
    }
}

fn deserialize_aspect_ratio<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    let (width, height) = s
        .split_once(':')
        .ok_or_else(|| de::Error::custom("aspect ratio must be of form W:H"))?;

    let width: f64 = width
        .parse()
        .map_err(|_| de::Error::custom("invalid width in aspect ratio"))?;

    let height: f64 = height
        .parse()
        .map_err(|_| de::Error::custom("invalid height in aspect ratio"))?;

    if height == 0.0 {
        return Err(de::Error::custom("aspect ratio height must not be zero"));
    }

    Ok(width / height)
}

fn deserialize_vec3<'de, D>(deserializer: D) -> Result<Vec3, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    let parts: Vec<_> = s.split(",").map(|part| part.trim()).collect();

    if parts.len() != 3 {
        return Err(de::Error::custom(
            "vectors should have exactly three components",
        ));
    }

    let x: f64 = parts[0]
        .parse()
        .map_err(|_| de::Error::custom("vector components must be floating point numbers"))?;
    let y: f64 = parts[1]
        .parse()
        .map_err(|_| de::Error::custom("vector components must be floating point numbers"))?;
    let z: f64 = parts[2]
        .parse()
        .map_err(|_| de::Error::custom("vector components must be floating point numbers"))?;

    let v = Vec3::new(x, y, z);
    Ok(v)
}

fn deserialize_point3<'de, D>(deserializer: D) -> Result<Point3, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_vec3(deserializer).map(Point3)
}
