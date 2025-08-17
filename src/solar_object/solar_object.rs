use std::{collections::HashMap, fs, hash::RandomState};

use image::DynamicImage;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct SolarObject {
    pub name: String,
    pub radius_km: f64,
    pub distance_from_parent_km: f64,
    pub orbital_period_days: Option<f64>,
    pub rotation_period_days: f64,
    pub rotation_axis: [f64; 3],
    pub texture_image: DynamicImage,
    pub children: Vec<SolarObject>,
}

#[derive(Debug, Clone, Deserialize)]
struct SolarObjectListRaw {
    #[serde(rename = "Body")]
    bodies: Vec<SolarObjectRaw>,
}

#[derive(Debug, Clone, Deserialize)]
struct SolarObjectRaw {
    name: String,
    parent: Option<String>,
    radius_km: f64,
    avg_distance_km: Option<f64>,
    orbital_period_days: Option<f64>,
    rotation_period_hours: f64,
    axis: [f64; 3],
    texture: String, // Path to image
}

fn load_recursive(parent: &mut SolarObject, map: &mut HashMap<String, SolarObjectRaw>) {
    let names = map
        .iter()
        .filter(|(_, raw)| {
            raw.parent.as_ref().expect("Only the sun is without parent") == &parent.name
        })
        .map(|(name, _)| name.clone())
        .collect::<Vec<_>>();
    for name in names {
        let body = map.remove(&name).expect("It exists");
        parent.children.push(body.clone().into());
        load_recursive(parent.children.last_mut().unwrap(), map);
    }
}

/// # Panics
///
/// Panics upon first error. Loading should succeed without any fails. Otherwise data may not be
/// properly set.
pub fn load_solar_objects(path: &str) -> SolarObject {
    let toml_str = fs::read_to_string(path).unwrap();
    let objects: SolarObjectListRaw = toml::from_str(&toml_str).unwrap();
    let mut map: HashMap<_, _, RandomState> =
        HashMap::from_iter(objects.bodies.into_iter().map(|obj| {
            let name = obj.name.clone();
            (name, obj)
        }));
    let mut sun: SolarObject = map.remove("Sun").expect("Sun is defined").into();
    load_recursive(&mut sun, &mut map);
    sun
}

impl From<SolarObjectRaw> for SolarObject {
    fn from(raw: SolarObjectRaw) -> Self {
        let texture_image =
            image::open(format!("resources/{}", raw.texture)).expect("Failed to load texture");
        Self {
            name: raw.name,
            radius_km: raw.radius_km / 10000.0,
            distance_from_parent_km: raw.avg_distance_km.unwrap_or(0.0) / 10000.0,
            orbital_period_days: raw.orbital_period_days,
            rotation_period_days: raw.rotation_period_hours / 24.0,
            rotation_axis: raw.axis,
            texture_image,
            children: Vec::new(),
        }
    }
}
