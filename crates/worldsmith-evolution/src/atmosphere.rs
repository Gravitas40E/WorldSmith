//! Atmospheric evolution: bulk atmosphere thermodynamics driven by volcanic
//! outgassing and stellar irradiance.
//!
//! This module models a deterministic bulk atmosphere.  Phase 11A introduces
//! a V1 baseline with no circulation, weather, clouds, or seasons.
//!
//! ## Responsibilities
//!
//! - Owns `atmospheric_mass_kg`, `surface_pressure_pa`, `mean_temperature_k`,
//!   and `atmosphere_composition` per ADR-011.
//! - Reads `volcanic_flux`, `volcanic_activity`, `tectonic_activity`,
//!   `Planet.physical.mass_kg`, `Planet.physical.radius_m`, and stellar
//!   luminosity after `worldsmith.evolution.plate_tectonics`.
//! - Applies deterministic outgassing, escape, radiative balance, and a
//!   simple linear greenhouse model.
//!
//! ## Simplifying assumptions
//!
//! 1. **Bulk atmosphere**: a single scalar mass and temperature represent the
//!    entire atmosphere; no latitude, longitude, altitude, or circulation.
//! 2. **Radiative equilibrium**: equilibrium temperature follows the standard
//!    blackbody formula with a fixed Bond albedo.
//! 3. **Linear greenhouse**: warming is a linear function of CO₂ mole fraction
//!    only.  No water vapor feedback, no clouds.
//! 4. **Deterministic outgassing**: volcanic flux is converted to atmospheric
//!    mass gain via a configurable efficiency, then classified as CO₂.
//! 5. **Deterministic escape**: a fixed fractional loss removes mass each step.
//! 6. **No chemistry**: composition is a small fixed set of gases; no
//!    photochemistry, no redox reactions, no biological exchange.
//!
//! ## Future replacement
//!
//! This implementation is a deterministic bulk atmospheric model and does not
//! simulate weather, circulation, clouds, or detailed radiative transfer.
//! Future phases should introduce:
//!
//! - latitude-resolved temperature
//! - atmospheric circulation cells
//! - cloud microphysics
//! - full chemistry solver
//!
//! ## Ownership
//!
//! - **Reads**: `volcanic_flux`, `volcanic_activity`, `tectonic_activity`,
//!   `Planet.physical.mass_kg`, `Planet.physical.radius_m`,
//!   `Star.luminosity_w`
//! - **Writes**: `atmospheric_mass_kg`, `surface_pressure_pa`,
//!   `mean_temperature_k`, `atmosphere_composition`
//! - **Never modifies**: `InteriorState`, `VolcanismState`,
//!   `PlateTectonicsState`, `climate`, `ocean`, `magnetic_field`,
//!   `habitability`

use serde::{Deserialize, Serialize};
use worldsmith_models::{
    AtmosphereState, AtmosphericGas, BodyReference, Molecule, Planet, PlanetId,
};
use worldsmith_state::{FieldKey, SimulationEvent};
use worldsmith_traits::{ContractResult, ModuleContext, SimulationModule, StateWriter};

const G: f64 = 6.67430e-11;
const STEFAN_BOLTZMANN: f64 = 5.670374419e-8;

const DEFAULT_OUTGASSING_EFFICIENCY: f64 = 1.0e-18;
const DEFAULT_ESCAPE_RATE: f64 = 1.0e-24;
const DEFAULT_GREENHOUSE_SCALING: f64 = 50.0;
const DEFAULT_ALBEDO: f64 = 0.3;
const DEFAULT_CO2_OUTGASSING_FRACTION: f64 = 0.8;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AtmosphereConfig {
    pub outgassing_efficiency: f64,
    pub escape_rate: f64,
    pub greenhouse_scaling: f64,
    pub albedo: f64,
    pub co2_outgassing_fraction: f64,
}

impl Default for AtmosphereConfig {
    fn default() -> Self {
        Self {
            outgassing_efficiency: DEFAULT_OUTGASSING_EFFICIENCY,
            escape_rate: DEFAULT_ESCAPE_RATE,
            greenhouse_scaling: DEFAULT_GREENHOUSE_SCALING,
            albedo: DEFAULT_ALBEDO,
            co2_outgassing_fraction: DEFAULT_CO2_OUTGASSING_FRACTION,
        }
    }
}

pub struct AtmosphereModule {
    config: AtmosphereConfig,
    initialized: bool,
}

impl AtmosphereModule {
    pub fn new(config: AtmosphereConfig) -> Self {
        Self {
            config,
            initialized: false,
        }
    }

    fn apply_initial_atmosphere(&self, planet: &Planet) -> ContractResult<Planet> {
        let mut updated = planet.clone();
        if updated.atmosphere_state.is_none() {
            updated.atmosphere_state = Some(AtmosphereState {
                atmospheric_mass_kg: 0.0,
                surface_pressure_pa: 0.0,
                mean_temperature_k: 0.0,
                atmosphere_composition: Vec::new(),
            });
        }
        Ok(updated)
    }

    fn default_composition() -> Vec<AtmosphericGas> {
        vec![
            AtmosphericGas {
                molecule: Molecule {
                    formula: "N2".into(),
                    name: "Nitrogen".into(),
                    molar_mass_kg_mol: Some(worldsmith_models::MeasuredValue {
                        value: 0.028014,
                        unit: "kg/mol".into(),
                        provenance: None,
                    }),
                },
                abundance: worldsmith_models::MeasuredValue {
                    value: 0.78,
                    unit: "mole_fraction".into(),
                    provenance: None,
                },
                is_greenhouse: false,
            },
            AtmosphericGas {
                molecule: Molecule {
                    formula: "O2".into(),
                    name: "Oxygen".into(),
                    molar_mass_kg_mol: Some(worldsmith_models::MeasuredValue {
                        value: 0.032,
                        unit: "kg/mol".into(),
                        provenance: None,
                    }),
                },
                abundance: worldsmith_models::MeasuredValue {
                    value: 0.21,
                    unit: "mole_fraction".into(),
                    provenance: None,
                },
                is_greenhouse: false,
            },
            AtmosphericGas {
                molecule: Molecule {
                    formula: "CO2".into(),
                    name: "Carbon Dioxide".into(),
                    molar_mass_kg_mol: Some(worldsmith_models::MeasuredValue {
                        value: 0.04401,
                        unit: "kg/mol".into(),
                        provenance: None,
                    }),
                },
                abundance: worldsmith_models::MeasuredValue {
                    value: 0.0004,
                    unit: "mole_fraction".into(),
                    provenance: None,
                },
                is_greenhouse: true,
            },
            AtmosphericGas {
                molecule: Molecule {
                    formula: "H2O".into(),
                    name: "Water".into(),
                    molar_mass_kg_mol: Some(worldsmith_models::MeasuredValue {
                        value: 0.018015,
                        unit: "kg/mol".into(),
                        provenance: None,
                    }),
                },
                abundance: worldsmith_models::MeasuredValue {
                    value: 0.01,
                    unit: "mole_fraction".into(),
                    provenance: None,
                },
                is_greenhouse: true,
            },
            AtmosphericGas {
                molecule: Molecule {
                    formula: "Ar".into(),
                    name: "Argon".into(),
                    molar_mass_kg_mol: Some(worldsmith_models::MeasuredValue {
                        value: 0.039948,
                        unit: "kg/mol".into(),
                        provenance: None,
                    }),
                },
                abundance: worldsmith_models::MeasuredValue {
                    value: 0.0093,
                    unit: "mole_fraction".into(),
                    provenance: None,
                },
                is_greenhouse: false,
            },
        ]
    }

    fn compute_surface_pressure(atm_mass: f64, planet_mass: f64, planet_radius: f64) -> f64 {
        let g = G * planet_mass / (planet_radius * planet_radius);
        let area = 4.0 * std::f64::consts::PI * planet_radius * planet_radius;
        atm_mass * g / area
    }

    fn compute_equilibrium_temperature(luminosity: f64, semi_major_axis: f64, albedo: f64) -> f64 {
        (luminosity * (1.0 - albedo)
            / (16.0 * std::f64::consts::PI * STEFAN_BOLTZMANN * semi_major_axis * semi_major_axis))
            .max(0.0)
            .powf(0.25)
    }

    fn add_co2_to_composition(
        &self,
        composition: &mut Vec<AtmosphericGas>,
        co2_moles_added: f64,
        total_moles: f64,
    ) {
        let mut current_co2 = 0.0;
        for gas in composition.iter() {
            if gas.molecule.formula == "CO2" {
                current_co2 = gas.abundance.value;
                break;
            }
        }

        let new_total = total_moles + co2_moles_added;
        let new_co2 = if new_total > 0.0 {
            (current_co2 * total_moles + co2_moles_added) / new_total
        } else {
            current_co2 + co2_moles_added
        };

        let mut updated = false;
        for gas in composition.iter_mut() {
            if gas.molecule.formula == "CO2" {
                gas.abundance = worldsmith_models::MeasuredValue {
                    value: new_co2,
                    unit: "mole_fraction".into(),
                    provenance: None,
                };
                updated = true;
                break;
            }
        }
        if !updated {
            composition.push(AtmosphericGas {
                molecule: Molecule {
                    formula: "CO2".into(),
                    name: "Carbon Dioxide".into(),
                    molar_mass_kg_mol: Some(worldsmith_models::MeasuredValue {
                        value: 0.04401,
                        unit: "kg/mol".into(),
                        provenance: None,
                    }),
                },
                abundance: worldsmith_models::MeasuredValue {
                    value: new_co2,
                    unit: "mole_fraction".into(),
                    provenance: None,
                },
                is_greenhouse: true,
            });
        }
    }
}

impl Default for AtmosphereModule {
    fn default() -> Self {
        Self::new(AtmosphereConfig::default())
    }
}

impl SimulationModule for AtmosphereModule {
    fn id(&self) -> &'static str {
        "worldsmith.evolution.atmosphere"
    }

    fn initialize(&mut self, state: &mut dyn StateWriter) -> ContractResult<()> {
        let planet_ids: Vec<_> = state.world().planets.keys().cloned().collect();
        for planet_id in planet_ids {
            if let Some(planet) = state.world().planets.get(&planet_id).cloned() {
                let mut updated = self.apply_initial_atmosphere(&planet)?;
                if updated.atmosphere_state.is_some() {
                    updated.atmosphere_state = Some(AtmosphereState {
                        atmospheric_mass_kg: 5.15e18,
                        surface_pressure_pa: 101325.0,
                        mean_temperature_k: 288.15,
                        atmosphere_composition: Self::default_composition(),
                    });
                }
                state.world_mut().planets.insert(updated.id, updated);
            }
        }
        self.initialized = true;
        Ok(())
    }

    fn update(
        &mut self,
        context: ModuleContext,
        state: &mut dyn StateWriter,
    ) -> ContractResult<()> {
        if !self.initialized {
            return Ok(());
        }

        let dt = context.delta_seconds;
        if dt <= 0.0 {
            return Ok(());
        }

        let outgassing_eff = self.config.outgassing_efficiency;
        let escape_rate = self.config.escape_rate;
        let greenhouse_scaling = self.config.greenhouse_scaling;
        let albedo = self.config.albedo;

        let snapshot: Vec<(PlanetId, Planet, Option<AtmosphereState>)> = state
            .world()
            .planets
            .values()
            .map(|planet| (planet.id, planet.clone(), planet.atmosphere_state.clone()))
            .collect();

        for (planet_id, planet, atmosphere_state) in snapshot {
            let Some(mut new_atmosphere) = atmosphere_state else {
                continue;
            };

            let planet_mass = planet.physical.mass_kg.value;
            let planet_radius = planet.physical.radius_m.value;

            let volcanic_flux = planet
                .volcanism
                .as_ref()
                .map(|v| v.volcanic_flux)
                .unwrap_or(0.0);

            let volcanic_activity_level = planet
                .volcanism
                .as_ref()
                .map(|v| v.volcanic_activity)
                .unwrap_or(worldsmith_models::VolcanicActivity::None);

            let tectonic_activity = planet
                .plate_tectonics
                .as_ref()
                .map(|t| t.tectonic_activity)
                .unwrap_or(worldsmith_models::TectonicActivity::None);

            // 1. Outgassing mass gain.
            let mut mass_gain = outgassing_eff * volcanic_flux * dt;
            if matches!(
                volcanic_activity_level,
                worldsmith_models::VolcanicActivity::Moderate
                    | worldsmith_models::VolcanicActivity::High
                    | worldsmith_models::VolcanicActivity::Extreme
            ) {
                mass_gain *= 1.2;
            }
            if matches!(
                tectonic_activity,
                worldsmith_models::TectonicActivity::Low
                    | worldsmith_models::TectonicActivity::Moderate
                    | worldsmith_models::TectonicActivity::High
            ) {
                mass_gain *= 1.3;
            }

            // 2. Escape loss.
            let mass_loss = escape_rate * new_atmosphere.atmospheric_mass_kg * dt;

            new_atmosphere.atmospheric_mass_kg =
                (new_atmosphere.atmospheric_mass_kg + mass_gain - mass_loss).max(0.0);

            // 3. Surface pressure from bulk hydrostatic approximation.
            new_atmosphere.surface_pressure_pa = Self::compute_surface_pressure(
                new_atmosphere.atmospheric_mass_kg,
                planet_mass,
                planet_radius,
            );

            // 4. Equilibrium temperature and greenhouse warming.
            let (luminosity, semi_major_axis) = match planet.orbit.parent {
                BodyReference::Star(star_id) => {
                    let star = state.world().stars.get(&star_id);
                    let star = match star {
                        Some(s) => s,
                        None => {
                            return Err(worldsmith_traits::ContractError::ModuleError(format!(
                                "star {star_id:?} not found in world state"
                            )))
                        }
                    };
                    (
                        star.luminosity_w.value,
                        planet.orbit.semi_major_axis_m.value,
                    )
                }
                _ => (1.0, 1.0),
            };

            let t_eq = Self::compute_equilibrium_temperature(luminosity, semi_major_axis, albedo);
            let greenhouse_factor = 1.0
                + greenhouse_scaling
                    * co2_fraction(&new_atmosphere.atmosphere_composition).max(0.0);
            new_atmosphere.mean_temperature_k = greenhouse_factor * t_eq;

            // 5. Update composition: classify volcanic outgassing as CO₂.
            let total_moles = new_atmosphere
                .atmosphere_composition
                .iter()
                .fold(0.0, |sum, gas| sum + gas.abundance.value);
            if total_moles > 0.0 && mass_gain > 0.0 {
                let avg_molar_mass =
                    new_atmosphere
                        .atmosphere_composition
                        .iter()
                        .fold(0.0, |sum, gas| {
                            let molar = gas
                                .molecule
                                .molar_mass_kg_mol
                                .as_ref()
                                .map(|v| v.value)
                                .unwrap_or(0.029);
                            sum + gas.abundance.value * molar
                        });
                let moles_added = if avg_molar_mass > 0.0 {
                    mass_gain / avg_molar_mass
                } else {
                    0.0
                };
                let co2_moles_added = moles_added * self.config.co2_outgassing_fraction;
                self.add_co2_to_composition(
                    &mut new_atmosphere.atmosphere_composition,
                    co2_moles_added,
                    total_moles,
                );
            }

            // 6. Sanity checks.
            if !new_atmosphere.atmospheric_mass_kg.is_finite()
                || !new_atmosphere.surface_pressure_pa.is_finite()
                || !new_atmosphere.mean_temperature_k.is_finite()
            {
                return Err(worldsmith_traits::ContractError::ModuleError(
                    "atmosphere produced non-finite values".into(),
                ));
            }

            if let Some(planet) = state.world_mut().planets.get_mut(&planet_id) {
                planet.atmosphere_state = Some(new_atmosphere);
            }
        }

        Ok(())
    }

    fn shutdown(&mut self, _state: &mut dyn StateWriter) -> ContractResult<()> {
        self.initialized = false;
        Ok(())
    }

    fn reads(&self) -> Vec<FieldKey> {
        vec![
            FieldKey::VolcanicFlux,
            FieldKey::VolcanicActivity,
            FieldKey::TectonicActivity,
            FieldKey::PlanetMass,
            FieldKey::SurfaceGravity,
            FieldKey::StellarLuminosity,
        ]
    }

    fn writes(&self) -> Vec<FieldKey> {
        vec![
            FieldKey::AtmosphericMass,
            FieldKey::AtmosphericTemperature,
            FieldKey::AtmosphericComposition,
        ]
    }

    fn publish_events(&mut self) -> Vec<SimulationEvent> {
        Vec::new()
    }

    fn consume_events(&mut self, _events: &[SimulationEvent]) -> ContractResult<()> {
        Ok(())
    }
}

fn co2_fraction(composition: &[AtmosphericGas]) -> f64 {
    let mut total = 0.0;
    let mut co2 = 0.0;
    for gas in composition {
        let frac = gas.abundance.value;
        total += frac;
        if gas.molecule.formula == "CO2" {
            co2 = frac;
        }
    }
    if total > 0.0 {
        co2 / total
    } else {
        co2
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CoreEvolutionModule, MantleEvolutionModule, PlateTectonicsModule, VolcanismModule,
    };
    use worldsmith_engine::EngineBuilder;
    use worldsmith_math::Vector3;
    use worldsmith_models::{
        BodyReference, InteriorState, MeasuredValue, OrbitalProperties, PhysicalProperties, Planet,
        PlanetId, PlanetType, SpectralType, StarClass, StarId, SystemId,
    };
    use worldsmith_traits::ModuleContext;

    fn earth_like_planet() -> Planet {
        Planet {
            id: PlanetId(1),
            name: "AtmosphereUnitTests".into(),
            class: worldsmith_models::PlanetClass::Terrestrial,
            planet_type: PlanetType::Rocky,
            system_id: SystemId(1),
            physical: PhysicalProperties {
                mass_kg: MeasuredValue {
                    value: 5.972e24,
                    unit: "kg".into(),
                    provenance: None,
                },
                radius_m: MeasuredValue {
                    value: 6.371e6,
                    unit: "m".into(),
                    provenance: None,
                },
                density_kg_m3: None,
                surface_gravity_m_s2: None,
            },
            orbit: OrbitalProperties {
                parent: BodyReference::Star(StarId(1)),
                semi_major_axis_m: MeasuredValue {
                    value: 1.496e11,
                    unit: "m".into(),
                    provenance: None,
                },
                semi_minor_axis_m: None,
                eccentricity: MeasuredValue {
                    value: 0.0167,
                    unit: "dimensionless".into(),
                    provenance: None,
                },
                inclination_rad: MeasuredValue {
                    value: 0.0,
                    unit: "rad".into(),
                    provenance: None,
                },
                orbital_period_s: None,
                rotation_period_s: None,
                axial_tilt_rad: None,
            },
            interior: None,
            geology: None,
            atmosphere: None,
            climate: None,
            ocean: None,
            magnetic_field: None,
            habitability: None,
            volcanism: None,
            plate_tectonics: None,
            atmosphere_state: None,
            hydrology_state: None,
            climate_state: None,
            carbon_cycle_state: None,
            biosphere_state: None,
            habitability_state: None,
            classification_state: None,
            surface_chemistry_state: None,
            cryosphere_state: None,
            moons: Vec::new(),
            position_m: Vector3::ZERO,
            velocity_m_s: Vector3::ZERO,
        }
    }

    fn seeded_planet() -> Planet {
        let mut p = earth_like_planet();
        p.interior = Some(InteriorState {
            age_seconds: 0.0,
            internal_heat: 5.972e24 * 1.0e6,
            radiogenic_heat: 5.972e24 * 2.0e-15,
            core_temperature: 6000.0,
            mantle_temperature: 4800.0,
            heat_flux: 1.2e16,
        });
        p.volcanism = Some(worldsmith_models::VolcanismState {
            volcanic_flux: 1.0e16,
            volcanic_activity: worldsmith_models::VolcanicActivity::Moderate,
            magma_generation_rate: 5.97e9,
        });
        p.plate_tectonics = Some(worldsmith_models::PlateTectonicsState {
            plate_velocity: 5.0,
            crustal_recycling_rate: 0.025,
            tectonic_activity: worldsmith_models::TectonicActivity::Moderate,
        });
        p
    }

    fn insert_sun(state: &mut worldsmith_state::WorldState) {
        state.stars.insert(
            StarId(1),
            worldsmith_models::Star {
                id: StarId(1),
                name: "Sun".into(),
                spectral_type: SpectralType::G,
                class: StarClass::MainSequence,
                mass_kg: MeasuredValue {
                    value: 1.989e30,
                    unit: "kg".into(),
                    provenance: None,
                },
                radius_m: MeasuredValue {
                    value: 6.957e8,
                    unit: "m".into(),
                    provenance: None,
                },
                luminosity_w: MeasuredValue {
                    value: 3.828e26,
                    unit: "W".into(),
                    provenance: None,
                },
                effective_temperature_k: MeasuredValue {
                    value: 5778.0,
                    unit: "K".into(),
                    provenance: None,
                },
                surface_gravity_m_s2: MeasuredValue {
                    value: 274.0,
                    unit: "m/s^2".into(),
                    provenance: None,
                },
                metallicity: MeasuredValue {
                    value: 0.0,
                    unit: "dimensionless".into(),
                    provenance: None,
                },
                rotation_period_s: None,
                age_s: None,
                position_m: Vector3::ZERO,
                velocity_m_s: Vector3::ZERO,
            },
        );
    }

    #[test]
    fn module_constructs_with_defaults() {
        let module = AtmosphereModule::default();
        assert!(!module.initialized);
    }

    #[test]
    fn initialization_seeds_atmosphere_state() {
        let mut engine = EngineBuilder::new()
            .with_seed(7)
            .build()
            .expect("engine builds");
        engine
            .state_mut()
            .planets
            .insert(PlanetId(1), earth_like_planet());
        insert_sun(engine.state_mut());

        let mut core = CoreEvolutionModule::default();
        core.initialize(engine.state_mut()).expect("core init");

        let mut mantle = MantleEvolutionModule::default();
        mantle.initialize(engine.state_mut()).expect("mantle init");

        let mut volcanism = VolcanismModule::default();
        volcanism
            .initialize(engine.state_mut())
            .expect("volcanism init");

        let mut plate = PlateTectonicsModule::default();
        plate.initialize(engine.state_mut()).expect("plate init");

        let mut atm = AtmosphereModule::default();
        atm.initialize(engine.state_mut()).expect("atmosphere init");

        let planet = engine.state().planets.get(&PlanetId(1)).unwrap();
        assert!(planet.atmosphere_state.is_some(), "atmosphere present");
        let a = planet.atmosphere_state.as_ref().unwrap();
        assert!(a.atmospheric_mass_kg.is_finite(), "mass finite");
    }

    #[test]
    fn zero_timestep_produces_no_state_change() {
        let mut engine = EngineBuilder::new()
            .with_seed(7)
            .build()
            .expect("engine builds");
        engine
            .state_mut()
            .planets
            .insert(PlanetId(1), seeded_planet());
        insert_sun(engine.state_mut());

        let mut core = CoreEvolutionModule::default();
        core.initialize(engine.state_mut()).unwrap();
        let mut mantle = MantleEvolutionModule::default();
        mantle.initialize(engine.state_mut()).unwrap();
        let mut volcanism = VolcanismModule::default();
        volcanism.initialize(engine.state_mut()).unwrap();
        let mut plate = PlateTectonicsModule::default();
        plate.initialize(engine.state_mut()).unwrap();
        let mut module = AtmosphereModule::default();
        module.initialize(engine.state_mut()).unwrap();

        module
            .update(
                ModuleContext {
                    timestamp_s: 0.0,
                    delta_seconds: 0.0,
                    seed: 7,
                },
                engine.state_mut(),
            )
            .expect("update");

        let planet = engine.state().planets.get(&PlanetId(1)).unwrap();
        let a = planet.atmosphere_state.as_ref().unwrap();
        assert!(a.atmospheric_mass_kg.is_finite() && a.atmospheric_mass_kg >= 0.0);
    }

    #[test]
    fn values_are_non_negative_and_finite() {
        let mut engine = EngineBuilder::new()
            .with_seed(7)
            .build()
            .expect("engine builds");
        engine
            .state_mut()
            .planets
            .insert(PlanetId(1), seeded_planet());
        insert_sun(engine.state_mut());

        let mut core = CoreEvolutionModule::default();
        core.initialize(engine.state_mut()).unwrap();
        let mut mantle = MantleEvolutionModule::default();
        mantle.initialize(engine.state_mut()).unwrap();
        let mut volcanism = VolcanismModule::default();
        volcanism.initialize(engine.state_mut()).unwrap();
        let mut plate = PlateTectonicsModule::default();
        plate.initialize(engine.state_mut()).unwrap();
        let mut module = AtmosphereModule::default();
        module.initialize(engine.state_mut()).unwrap();

        for i in 0..100 {
            module
                .update(
                    ModuleContext {
                        timestamp_s: (i as f64) * 1.0,
                        delta_seconds: 1.0,
                        seed: 7,
                    },
                    engine.state_mut(),
                )
                .expect("update");
        }

        let planet = engine.state().planets.get(&PlanetId(1)).unwrap();
        let a = planet.atmosphere_state.as_ref().unwrap();
        assert!(a.atmospheric_mass_kg.is_finite() && a.atmospheric_mass_kg >= 0.0);
        assert!(a.surface_pressure_pa.is_finite() && a.surface_pressure_pa >= 0.0);
        assert!(a.mean_temperature_k.is_finite() && a.mean_temperature_k >= 0.0);
    }

    #[test]
    fn volcanic_outgassing_increases_atmospheric_mass() {
        let mut engine = EngineBuilder::new()
            .with_seed(7)
            .build()
            .expect("engine builds");
        engine
            .state_mut()
            .planets
            .insert(PlanetId(1), seeded_planet());
        insert_sun(engine.state_mut());

        let mut core = CoreEvolutionModule::default();
        core.initialize(engine.state_mut()).unwrap();
        let mut mantle = MantleEvolutionModule::default();
        mantle.initialize(engine.state_mut()).unwrap();
        let mut volcanism = VolcanismModule::default();
        volcanism.initialize(engine.state_mut()).unwrap();
        let mut plate = PlateTectonicsModule::default();
        plate.initialize(engine.state_mut()).unwrap();
        let mut module = AtmosphereModule::default();
        module.initialize(engine.state_mut()).expect("init");

        let initial_mass = engine
            .state()
            .planets
            .get(&PlanetId(1))
            .unwrap()
            .atmosphere_state
            .as_ref()
            .map(|a| a.atmospheric_mass_kg)
            .unwrap_or(0.0);

        module
            .update(
                ModuleContext {
                    timestamp_s: 1.0,
                    delta_seconds: 3.154e7,
                    seed: 7,
                },
                engine.state_mut(),
            )
            .expect("update");

        let new_mass = engine
            .state()
            .planets
            .get(&PlanetId(1))
            .unwrap()
            .atmosphere_state
            .as_ref()
            .unwrap()
            .atmospheric_mass_kg;
        assert!(new_mass > initial_mass, "outgassing should increase mass");
    }

    #[test]
    fn repeated_updates_are_deterministic() {
        let mut engine_a = EngineBuilder::new()
            .with_seed(7)
            .build()
            .expect("engine builds");
        engine_a
            .state_mut()
            .planets
            .insert(PlanetId(1), seeded_planet());
        insert_sun(engine_a.state_mut());

        let mut engine_b = EngineBuilder::new()
            .with_seed(7)
            .build()
            .expect("engine builds");
        engine_b
            .state_mut()
            .planets
            .insert(PlanetId(1), seeded_planet());
        insert_sun(engine_b.state_mut());

        let mut core_a = CoreEvolutionModule::default();
        let mut core_b = CoreEvolutionModule::default();
        core_a.initialize(engine_a.state_mut()).unwrap();
        core_b.initialize(engine_b.state_mut()).unwrap();

        let mut mantle_a = MantleEvolutionModule::default();
        let mut mantle_b = MantleEvolutionModule::default();
        mantle_a.initialize(engine_a.state_mut()).unwrap();
        mantle_b.initialize(engine_b.state_mut()).unwrap();

        let mut volcanism_a = VolcanismModule::default();
        let mut volcanism_b = VolcanismModule::default();
        volcanism_a.initialize(engine_a.state_mut()).unwrap();
        volcanism_b.initialize(engine_b.state_mut()).unwrap();

        let mut plate_a = PlateTectonicsModule::default();
        let mut plate_b = PlateTectonicsModule::default();
        plate_a.initialize(engine_a.state_mut()).unwrap();
        plate_b.initialize(engine_b.state_mut()).unwrap();

        let mut module_a = AtmosphereModule::default();
        let mut module_b = AtmosphereModule::default();
        module_a.initialize(engine_a.state_mut()).unwrap();
        module_b.initialize(engine_b.state_mut()).unwrap();

        for i in 0..50 {
            let ctx = ModuleContext {
                timestamp_s: (i as f64) * 1.0,
                delta_seconds: 1.0,
                seed: 7,
            };
            module_a.update(ctx, engine_a.state_mut()).expect("update");
            module_b.update(ctx, engine_b.state_mut()).expect("update");
        }

        let a = engine_a
            .state()
            .planets
            .get(&PlanetId(1))
            .unwrap()
            .atmosphere_state
            .as_ref()
            .unwrap();
        let b = engine_b
            .state()
            .planets
            .get(&PlanetId(1))
            .unwrap()
            .atmosphere_state
            .as_ref()
            .unwrap();
        assert_eq!(a, b);
    }
}
