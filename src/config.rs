use serde::Deserialize;
use serde::de::{self, Deserializer};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub camera: CameraConfig,
    pub renderer: RendererConfig,
}

#[derive(Debug, Deserialize)]
pub struct CameraConfig {
    pub image_width: u32,
    #[serde(deserialize_with = "deserialize_aspect_ratio")]
    pub aspect_ratio: f64,
}

#[derive(Debug, Deserialize)]
pub struct RendererConfig {
    pub samples_per_pixel: u32,
    pub max_bounces: u32,
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
