pub mod types {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Planet {
        pub id: String,
        pub name: String,
        pub class: String,
        pub planet_type: String,
        pub radius_m: f64,
        pub mass_kg: f64,
        pub gravity_m_s2: Option<f64>,
        pub stellar_class: Option<String>,
        pub temperature_k: Option<f64>,
        pub pressure_pa: Option<f64>,
        pub water_fraction: Option<f64>,
        pub ice_fraction: Option<f64>,
        pub atmospheric_mass_kg: Option<f64>,
        pub mean_temperature_k: Option<f64>,
        pub equilibrium_temperature_k: Option<f64>,
        pub planetary_albedo: Option<f64>,
        pub habitability_index: Option<f64>,
        pub habitability_class: Option<String>,
        pub primary_classification: Option<String>,
        pub secondary_classification: Option<String>,
        pub confidence_score: Option<f64>,
        pub classification_summary: Option<String>,
        pub age_seconds: Option<f64>,
        pub tick: Option<u64>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Snapshot {
        pub simulation_id: String,
        pub timestamp_s: f64,
        pub tick: u64,
        pub planets: Vec<Planet>,
    }
}
