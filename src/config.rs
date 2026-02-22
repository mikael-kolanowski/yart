use std::path::PathBuf;

use serde::Deserialize;
use serde::de::{self, Deserializer};

use crate::math::Vec3;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub camera: CameraConfig,
    pub renderer: RendererConfig,
    pub image: ImageConfig,
    pub materials: Vec<MaterialConfig>,
    pub objects: Vec<ObjectConfig>,
}

#[derive(Debug, Deserialize)]
pub struct CameraConfig {
    #[serde(deserialize_with = "deserialize_aspect_ratio")]
    pub aspect_ratio: f64,
    pub field_of_view: u32,
    #[serde(deserialize_with = "deserialize_vec3")]
    pub position: Vec3,
    #[serde(deserialize_with = "deserialize_vec3")]
    pub look_at: Vec3,
}

#[derive(Debug, Deserialize)]
pub struct RendererConfig {
    pub samples_per_pixel: u32,
    pub max_bounces: u32,
}

#[derive(Debug, Deserialize)]
pub struct ImageConfig {
    pub width: u32,
    pub output: PathBuf,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum MaterialConfig {
    #[serde(rename = "lambertian")]
    Lambertian {
        name: String,
        #[serde(deserialize_with = "deserialize_vec3")]
        albedo: Vec3,
    },

    #[serde(rename = "metal")]
    Metal {
        name: String,
        #[serde(deserialize_with = "deserialize_vec3")]
        albedo: Vec3,
        fuzz: f64,
    },
    #[serde(rename = "normal_vis")]
    NormalVisualization { name: String },
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum ObjectConfig {
    #[serde(rename = "sphere")]
    Sphere {
        #[serde(deserialize_with = "deserialize_vec3")]
        position: Vec3,
        radius: f64,
        material: String,
    },
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
        return Err(de::Error::custom("aspect ratio height must not be zero"))?;
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
        ))?;
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
