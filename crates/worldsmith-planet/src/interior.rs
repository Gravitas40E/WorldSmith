//! Interior differentiation, heat budget, and magnetic dynamo foundations.

use serde::{Deserialize, Serialize};
use worldsmith_math::{constants, Vector3};
use worldsmith_models::{
    CoreProperties, CrustProperties, GeologicalProperties, MagneticFieldProperties,
    MantleProperties, Material, MeasuredValue, PlateSystem, SurfaceMaterial, TectonicActivity,
    VolcanicActivity,
};

use crate::errors::{PlanetFormationError, PlanetFormationResult};

/// Derived interior layer and heat model for an evolved planet.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InteriorModel {
    /// Iron-rich core mass fraction.
    pub core_fraction: f64,
    /// Core radius in meters.
    pub core_radius_m: f64,
    /// Mantle thickness in meters.
    pub mantle_thickness_m: f64,
    /// Crust thickness in meters.
    pub crust_thickness_m: f64,
    /// Whether a liquid outer core can support a dynamo.
    pub has_liquid_outer_core: bool,
    /// Heat budget by source.
    pub heat_budget: HeatBudget,
    /// Geological properties ready for `Planet`.
    pub geology: GeologicalProperties,
    /// Magnetic field properties ready for `Planet`.
    pub magnetic_field: MagneticFieldProperties,
}

/// Internal heat sources in watts and representative internal temperature.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HeatBudget {
    /// Residual accretion heat in watts.
    pub primordial_heat_w: f64,
    /// Radiogenic heat in watts.
    pub radioactive_heat_w: f64,
    /// Tidal heat placeholder in watts.
    pub tidal_heat_w: f64,
    /// Total heat budget in watts.
    pub total_heat_w: f64,
    /// Secular cooling rate in kelvin per gigayear.
    pub cooling_rate_k_gyr: f64,
    /// Representative internal temperature in kelvin.
    pub internal_temperature_k: f64,
}

/// Differentiates a planet into core, mantle, crust, heat, geology, and magnetic field.
pub fn differentiate_interior(
    mass_kg: f64,
    radius_m: f64,
    metal_fraction: f64,
    water_fraction: f64,
    age_gyr: f64,
    rotation_period_s: Option<f64>,
) -> PlanetFormationResult<InteriorModel> {
    if mass_kg <= 0.0 || radius_m <= 0.0 {
        return Err(PlanetFormationError::InvalidEvolution(
            "planet mass and radius must be positive".to_string(),
        ));
    }
    let earth_masses = mass_kg / constants::EARTH_MASS;
    let core_fraction = metal_fraction.clamp(0.05, 0.65);
    let core_radius_m = radius_m * core_fraction.powf(1.0 / 3.0);
    let crust_thickness_m = radius_m * (0.006 + 0.012 / earth_masses.cbrt()).clamp(0.003, 0.04);
    let mantle_thickness_m = (radius_m - core_radius_m - crust_thickness_m).max(0.0);
    let heat_budget = heat_budget(mass_kg, age_gyr, water_fraction);
    let has_liquid_outer_core =
        core_fraction > 0.18 && heat_budget.internal_temperature_k > 1_600.0;
    let rotation_factor = rotation_period_s
        .map(|s| (86_400.0 / s).clamp(0.1, 5.0))
        .unwrap_or(1.0);
    let dynamo = if has_liquid_outer_core {
        (heat_budget.total_heat_w / 4.7e13).sqrt() * rotation_factor
    } else {
        0.0
    };
    let field_strength_t = (31.2e-6 * dynamo).clamp(0.0, 250.0e-6);
    let magnetosphere_radius_m = radius_m * (1.0 + dynamo.powf(1.0 / 3.0) * 8.0);
    let tectonics =
        if earth_masses > 0.5 && heat_budget.total_heat_w > 1.0e13 && water_fraction > 0.001 {
            TectonicActivity::Moderate
        } else if heat_budget.total_heat_w > 3.0e12 {
            TectonicActivity::Low
        } else {
            TectonicActivity::None
        };
    let volcanism = if heat_budget.total_heat_w > 5.0e13 {
        VolcanicActivity::High
    } else if heat_budget.total_heat_w > 1.0e13 {
        VolcanicActivity::Moderate
    } else if heat_budget.total_heat_w > 1.0e12 {
        VolcanicActivity::Low
    } else {
        VolcanicActivity::None
    };

    Ok(InteriorModel {
        core_fraction,
        core_radius_m,
        mantle_thickness_m,
        crust_thickness_m,
        has_liquid_outer_core,
        heat_budget,
        geology: GeologicalProperties {
            core: Some(CoreProperties {
                radius_m: Some(measured(
                    core_radius_m,
                    "m",
                    "R_core = R * core_fraction^(1/3)",
                )),
                materials: vec![material("Iron-rich core", SurfaceMaterial::Metal)],
                has_liquid_outer_core,
            }),
            mantle: Some(MantleProperties {
                thickness_m: Some(measured(
                    mantle_thickness_m,
                    "m",
                    "mantle thickness from differentiated layer radii",
                )),
                materials: vec![material("Silicate mantle", SurfaceMaterial::SilicateRock)],
            }),
            crust: Some(CrustProperties {
                mean_thickness_m: Some(measured(
                    crust_thickness_m,
                    "m",
                    "mass-scaled crust thickness approximation",
                )),
                materials: surface_materials(water_fraction, volcanism),
            }),
            surface_materials: surface_materials(water_fraction, volcanism),
            plate_system: Some(PlateSystem {
                activity: tectonics,
                major_plate_count: if tectonics == TectonicActivity::None {
                    None
                } else {
                    Some(7)
                },
            }),
            heat_flow_w_m2: Some(measured(
                heat_budget.total_heat_w / (4.0 * std::f64::consts::PI * radius_m.powi(2)),
                "W m^-2",
                "heat flow = total heat / surface area",
            )),
            volcanism,
        },
        magnetic_field: MagneticFieldProperties {
            field_strength_t: Some(measured(
                field_strength_t,
                "T",
                "Earth-relative dynamo scaling from heat and rotation",
            )),
            pole_orientation: Some(Vector3::Z),
            magnetosphere_radius_m: Some(measured(
                magnetosphere_radius_m,
                "m",
                "magnetopause radius proxy from field strength",
            )),
        },
    })
}

fn heat_budget(mass_kg: f64, age_gyr: f64, water_fraction: f64) -> HeatBudget {
    let earth_masses = mass_kg / constants::EARTH_MASS;
    let primordial_heat_w = 2.5e13 * earth_masses.powf(1.2) * (-age_gyr / 4.5).exp();
    let radioactive_heat_w = 2.0e13 * earth_masses * (-age_gyr / 6.0).exp();
    let tidal_heat_w = 5.0e11 * water_fraction.clamp(0.0, 0.5);
    let total_heat_w = primordial_heat_w + radioactive_heat_w + tidal_heat_w;
    let internal_temperature_k = 1_200.0 + 4_000.0 * (total_heat_w / 5.0e13).sqrt().clamp(0.0, 2.0);
    HeatBudget {
        primordial_heat_w,
        radioactive_heat_w,
        tidal_heat_w,
        total_heat_w,
        cooling_rate_k_gyr: 120.0 / earth_masses.cbrt().max(0.4),
        internal_temperature_k,
    }
}

fn surface_materials(water_fraction: f64, volcanism: VolcanicActivity) -> Vec<Material> {
    let mut materials = vec![material("Basaltic crust", SurfaceMaterial::SilicateRock)];
    if volcanism == VolcanicActivity::High {
        materials.push(material("Lava plains", SurfaceMaterial::Lava));
    }
    if water_fraction > 0.02 {
        materials.push(material(
            "Surface ice and hydrated minerals",
            SurfaceMaterial::WaterIce,
        ));
    }
    materials
}

fn material(name: &str, surface_material: SurfaceMaterial) -> Material {
    Material {
        name: name.to_string(),
        surface_material,
        abundance: None,
    }
}

fn measured(value: f64, unit: &str, equation: &str) -> MeasuredValue {
    MeasuredValue {
        value,
        unit: unit.to_string(),
        provenance: Some(worldsmith_models::ScientificProvenance {
            source_equation: Some(equation.to_string()),
            input_variables: Vec::new(),
            confidence: Some(0.6),
            notes: vec!["WorldSmith simplified planetary evolution approximation".to_string()],
            references: vec!["Parameterized terrestrial planet evolution model".to_string()],
        }),
    }
}
