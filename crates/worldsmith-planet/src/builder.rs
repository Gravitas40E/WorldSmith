//! High-level deterministic planet formation builder.

use serde::{Deserialize, Serialize};
use worldsmith_math::constants;
use worldsmith_models::{
    BodyReference, MeasuredValue, NamedValue, OrbitalProperties, PhysicalProperties, Planet,
    PlanetId, ScientificProvenance, StarId, SystemId,
};
use worldsmith_rng::RngStream;
use worldsmith_state::EventPayload;

use crate::{
    accretion::{accrete_planetesimals, AccretionSummary},
    classification::classify_embryo,
    disk::ProtoplanetaryDisk,
    embryo::{bulk_density_kg_m3, PlanetaryEmbryo},
    errors::{PlanetFormationError, PlanetFormationResult},
    migration::{migrate_embryo, MigrationModel, MigrationRecord},
    planetesimal::{generate_planetesimals, Planetesimal},
};

/// Inputs controlling deterministic planet formation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormationConfig {
    /// Parent system id.
    pub system_id: SystemId,
    /// Parent star id used by generated orbital models.
    pub parent_star_id: StarId,
    /// First assigned planet id.
    pub first_planet_id: u64,
    /// Number of initial planetesimals.
    pub planetesimal_count: usize,
    /// Minimum embryo promotion mass in kilograms.
    pub promotion_mass_kg: f64,
    /// Migration model.
    pub migration: MigrationModel,
}

impl Default for FormationConfig {
    fn default() -> Self {
        Self {
            system_id: SystemId(1),
            parent_star_id: StarId(1),
            first_planet_id: 1,
            planetesimal_count: 192,
            promotion_mass_kg: 2.5e22,
            migration: MigrationModel::default(),
        }
    }
}

impl FormationConfig {
    /// Validates deterministic formation settings.
    pub fn validate(&self) -> PlanetFormationResult<()> {
        if self.planetesimal_count == 0 {
            return Err(PlanetFormationError::InvalidDiskMass(
                "planetesimal_count must be greater than zero".to_string(),
            ));
        }
        if !self.promotion_mass_kg.is_finite() || self.promotion_mass_kg <= 0.0 {
            return Err(PlanetFormationError::InvalidDiskMass(
                "promotion_mass_kg must be positive and finite".to_string(),
            ));
        }
        Ok(())
    }
}

/// Complete deterministic formation output.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanetFormationOutput {
    /// Disk used for formation.
    pub disk: ProtoplanetaryDisk,
    /// Generated planetesimals.
    pub planetesimals: Vec<Planetesimal>,
    /// Accretion result.
    pub accretion: AccretionSummary,
    /// Migration records.
    pub migration_records: Vec<MigrationRecord>,
    /// Final planet models.
    pub planets: Vec<Planet>,
    /// Engine events corresponding to formation milestones.
    pub events: Vec<EventPayload>,
}

/// Builder that transforms stellar inputs and a disk into planets.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanetFormationBuilder {
    config: FormationConfig,
    disk: Option<ProtoplanetaryDisk>,
    seed: u64,
}

impl PlanetFormationBuilder {
    /// Creates a builder with default formation settings.
    pub fn new() -> Self {
        Self {
            config: FormationConfig::default(),
            disk: None,
            seed: 0,
        }
    }

    /// Sets formation configuration.
    pub fn config(mut self, config: FormationConfig) -> Self {
        self.config = config;
        self
    }

    /// Sets deterministic seed.
    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Provides a precomputed disk.
    pub fn disk(mut self, disk: ProtoplanetaryDisk) -> Self {
        self.disk = Some(disk);
        self
    }

    /// Builds planet formation output.
    pub fn build(self) -> PlanetFormationResult<PlanetFormationOutput> {
        self.config.validate()?;
        let disk = self
            .disk
            .unwrap_or_else(|| ProtoplanetaryDisk::from_star(1.0, 1.0, 0.0134, 1.0));
        disk.validate()?;
        let mut rng = RngStream::new(self.seed).derive("planet-formation");
        let planetesimals = generate_planetesimals(&disk, self.config.planetesimal_count, &mut rng);
        let mut accretion =
            accrete_planetesimals(planetesimals.clone(), self.config.promotion_mass_kg);
        let mut migration_records = Vec::new();
        for embryo in &mut accretion.embryos {
            migration_records.push(migrate_embryo(
                embryo,
                disk.gas_fraction,
                self.config.migration,
            ));
        }
        let planets = accretion
            .embryos
            .iter()
            .enumerate()
            .map(|(index, embryo)| {
                embryo_to_planet(
                    self.config.system_id,
                    self.config.parent_star_id,
                    PlanetId(self.config.first_planet_id + index as u64),
                    embryo,
                )
            })
            .collect::<Vec<_>>();
        let mut events = vec![EventPayload::DiskCreated {
            system_id: self.config.system_id,
        }];
        events.extend(
            planetesimals
                .iter()
                .map(|body| EventPayload::PlanetesimalCreated { local_id: body.id }),
        );
        events.extend(
            accretion
                .embryos
                .iter()
                .map(|embryo| EventPayload::EmbryoCreated {
                    local_id: embryo.id,
                }),
        );
        events.extend(accretion.collisions.iter().map(|collision| {
            EventPayload::CollisionOccurred {
                embryo_id: collision.embryo_id,
                body_id: collision.body_id,
            }
        }));
        events.extend(planets.iter().map(|planet| EventPayload::PlanetCreated {
            planet_id: planet.id,
        }));
        events.extend(
            planets
                .iter()
                .map(|planet| EventPayload::PlanetClassificationChanged {
                    planet_id: planet.id,
                }),
        );
        events.extend(planets.iter().map(|planet| EventPayload::PlanetMigrated {
            planet_id: planet.id,
        }));
        Ok(PlanetFormationOutput {
            disk,
            planetesimals,
            accretion,
            migration_records,
            planets,
            events,
        })
    }
}

impl Default for PlanetFormationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

fn embryo_to_planet(
    system_id: SystemId,
    parent_star_id: StarId,
    id: PlanetId,
    embryo: &PlanetaryEmbryo,
) -> Planet {
    let (class, planet_type) = classify_embryo(embryo);
    let density = bulk_density_kg_m3(embryo.composition);
    let surface_gravity =
        constants::GRAVITATIONAL_CONSTANT * embryo.mass_kg / embryo.radius_m.powi(2);
    Planet {
        id,
        name: format!("Planet {}", id.0),
        class,
        planet_type,
        system_id,
        physical: PhysicalProperties {
            mass_kg: measured(embryo.mass_kg, "kg", "sum of accreted planetesimal masses"),
            radius_m: measured(
                embryo.radius_m,
                "m",
                "radius from mass and composition-derived density",
            ),
            density_kg_m3: Some(measured(
                density,
                "kg m^-3",
                "composition-weighted bulk density",
            )),
            surface_gravity_m_s2: Some(measured(surface_gravity, "m s^-2", "g = G M / R^2")),
        },
        orbit: OrbitalProperties {
            parent: BodyReference::Star(parent_star_id),
            semi_major_axis_m: measured(
                embryo.orbital_distance_au * constants::ASTRONOMICAL_UNIT,
                "m",
                "migrated embryo orbital distance",
            ),
            semi_minor_axis_m: None,
            eccentricity: measured(
                0.02,
                "dimensionless",
                "low-eccentricity post-accretion placeholder",
            ),
            inclination_rad: measured(0.0, "rad", "coplanar disk approximation"),
            orbital_period_s: None,
            rotation_period_s: None,
            axial_tilt_rad: None,
        },
        geology: None,
        atmosphere: None,
        climate: None,
        ocean: None,
        magnetic_field: None,
        habitability: None,
        moons: Vec::new(),
    }
}

fn measured(value: f64, unit: &str, equation: &str) -> MeasuredValue {
    MeasuredValue {
        value,
        unit: unit.to_string(),
        provenance: Some(ScientificProvenance {
            source_equation: Some(equation.to_string()),
            input_variables: vec![NamedValue {
                name: "value".to_string(),
                value,
                unit: Some(unit.to_string()),
            }],
            confidence: Some(0.65),
            notes: vec!["WorldSmith simplified deterministic planet formation".to_string()],
            references: vec!["MMSN-inspired disk and feeding-zone approximation".to_string()],
        }),
    }
}
