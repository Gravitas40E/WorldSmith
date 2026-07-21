//! Scientific data models shared by WorldSmith simulation crates.
//!
//! This crate contains data only: identifiers, measured properties, scientific
//! metadata, and composable celestial body models. Simulation behavior belongs
//! in simulation crates and engine contracts.

use serde::{Deserialize, Serialize};
use worldsmith_math::Vector3;

macro_rules! id_type {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        pub struct $name(pub u64);
    };
}

id_type!(/// Strong identifier for a stellar system.
SystemId);
id_type!(/// Strong identifier for a star.
StarId);
id_type!(/// Strong identifier for a planet.
PlanetId);
id_type!(/// Strong identifier for a moon.
MoonId);
id_type!(/// Strong identifier for any celestial body when parentage is generic.
BodyId);

/// Reference to a parent object in an orbital hierarchy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BodyReference {
    /// Parent is a star.
    Star(StarId),
    /// Parent is a planet.
    Planet(PlanetId),
    /// Parent is a moon, enabling future nested moon systems.
    Moon(MoonId),
    /// Parent is an unresolved generic body.
    Body(BodyId),
}

/// Confidence and provenance metadata for a scientific property.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ScientificProvenance {
    /// Optional equation, model name, or literature source used to derive the value.
    pub source_equation: Option<String>,
    /// Named input variables and serialized values used by the source equation.
    pub input_variables: Vec<NamedValue>,
    /// Confidence in `[0, 1]` assigned by the producing model or data source.
    pub confidence: Option<f64>,
    /// Human-readable scientific notes for explainability.
    pub notes: Vec<String>,
    /// Future references such as DOI, URL, paper key, or dataset identifier.
    pub references: Vec<String>,
}

/// Named scalar metadata value with an optional unit label.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NamedValue {
    /// Variable name as used by the model or equation.
    pub name: String,
    /// Numeric value stored in the stated unit.
    pub value: f64,
    /// Unit label, usually SI.
    pub unit: Option<String>,
}

/// Numeric value with unit text and optional scientific provenance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeasuredValue {
    /// Numeric magnitude.
    pub value: f64,
    /// Unit label. Internal physical values should prefer SI units.
    pub unit: String,
    /// Optional scientific provenance.
    pub provenance: Option<ScientificProvenance>,
}

/// Stellar spectral sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpectralType {
    /// O-type star.
    O,
    /// B-type star.
    B,
    /// A-type star.
    A,
    /// F-type star.
    F,
    /// G-type star.
    G,
    /// K-type star.
    K,
    /// M-type star.
    M,
    /// Brown dwarf or substellar object.
    BrownDwarf,
    /// White dwarf remnant.
    WhiteDwarf,
    /// Neutron star remnant.
    NeutronStar,
    /// Future extension point.
    Other,
}

/// Broad stellar classification used by system models.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StarClass {
    /// Main sequence hydrogen-burning star.
    MainSequence,
    /// Giant star.
    Giant,
    /// Supergiant star.
    Supergiant,
    /// White dwarf remnant.
    WhiteDwarf,
    /// Brown dwarf.
    BrownDwarf,
    /// Stellar remnant.
    Remnant,
    /// Future extension point.
    Other,
}

/// Planet mass/radius class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanetClass {
    /// Mercury-like small rocky body.
    Terrestrial,
    /// Larger rocky body.
    SuperEarth,
    /// Volatile-rich sub-Neptune.
    MiniNeptune,
    /// Neptune-like ice giant.
    IceGiant,
    /// Jupiter-like gas giant.
    GasGiant,
    /// Dwarf planet.
    Dwarf,
    /// Future extension point.
    Other,
}

/// Planetary composition or observational type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanetType {
    /// Rocky silicate-metal world.
    Rocky,
    /// Ocean-dominated world.
    Ocean,
    /// Ice-rich world.
    Ice,
    /// Gas-dominated world.
    Gas,
    /// Carbon-rich world.
    Carbon,
    /// Lava or magma world.
    Lava,
    /// Future extension point.
    Other,
}

/// Global climate category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClimateType {
    /// No dominant climate assigned.
    Unknown,
    /// Frozen surface conditions.
    Frozen,
    /// Temperate climate regime.
    Temperate,
    /// Arid climate regime.
    Arid,
    /// Tropical or warm wet climate regime.
    Tropical,
    /// Runaway greenhouse regime.
    RunawayGreenhouse,
    /// Future extension point.
    Other,
}

/// Broad atmosphere category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AtmosphereType {
    /// No durable atmosphere.
    None,
    /// Trace or exosphere-like atmosphere.
    Trace,
    /// Thin atmosphere.
    Thin,
    /// Earth-like pressure range.
    Standard,
    /// Dense atmosphere.
    Dense,
    /// Gas giant envelope.
    GasEnvelope,
    /// Future extension point.
    Other,
}

/// Common planetary surface material category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SurfaceMaterial {
    /// Silicate rock.
    SilicateRock,
    /// Metallic surface exposure.
    Metal,
    /// Water ice.
    WaterIce,
    /// Carbon dioxide ice.
    CarbonDioxideIce,
    /// Liquid water.
    LiquidWater,
    /// Basaltic lava.
    Lava,
    /// Organic-rich material.
    Organics,
    /// Future extension point.
    Other,
}

/// Ocean coverage and composition category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OceanType {
    /// No stable ocean.
    None,
    /// Surface liquid water ocean.
    Water,
    /// Subsurface liquid water ocean.
    SubsurfaceWater,
    /// Hydrocarbon ocean.
    Hydrocarbon,
    /// Magma ocean.
    Magma,
    /// Future extension point.
    Other,
}

/// Cloud particle or coverage category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloudType {
    /// No major clouds.
    None,
    /// Water clouds.
    Water,
    /// Carbon dioxide clouds.
    CarbonDioxide,
    /// Ammonia clouds.
    Ammonia,
    /// Sulfuric acid clouds.
    SulfuricAcid,
    /// Future extension point.
    Other,
}

/// Weather regime category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeatherType {
    /// No dominant weather.
    None,
    /// Stable low-variability weather.
    Calm,
    /// Persistent winds.
    Windy,
    /// Storm dominated.
    Stormy,
    /// Precipitation dominated.
    Precipitating,
    /// Future extension point.
    Other,
}

/// Biological or system life stage used by habitability models.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifeStage {
    /// No life inferred.
    None,
    /// Prebiotic chemistry.
    Prebiotic,
    /// Microbial life.
    Microbial,
    /// Complex multicellular life.
    Complex,
    /// Technological civilization.
    Technological,
    /// Future extension point.
    Other,
}

/// Explainable habitability rating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HabitabilityRating {
    /// Not assessed.
    Unknown,
    /// Hostile to known life.
    Hostile,
    /// Marginally habitable.
    Marginal,
    /// Potentially habitable.
    Potential,
    /// Likely habitable.
    Favorable,
    /// Future extension point.
    Other,
}

/// Plate tectonic activity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TectonicActivity {
    /// No active tectonics.
    None,
    /// Low tectonic activity.
    Low,
    /// Earth-like activity.
    Moderate,
    /// High activity.
    High,
    /// Future extension point.
    Other,
}

/// Volcanic activity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VolcanicActivity {
    /// No active volcanism.
    None,
    /// Low volcanism.
    Low,
    /// Moderate volcanism.
    Moderate,
    /// High volcanism.
    High,
    /// Catastrophic resurfacing.
    Extreme,
    /// Future extension point.
    Other,
}

/// Stellar body data with physical and kinematic fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Star {
    /// Strong star identifier.
    pub id: StarId,
    /// Human-readable display name.
    pub name: String,
    /// Spectral type.
    pub spectral_type: SpectralType,
    /// Broad stellar class.
    pub class: StarClass,
    /// Stellar mass in kilograms.
    pub mass_kg: MeasuredValue,
    /// Stellar radius in meters.
    pub radius_m: MeasuredValue,
    /// Luminosity in watts.
    pub luminosity_w: MeasuredValue,
    /// Effective temperature in kelvin.
    pub effective_temperature_k: MeasuredValue,
    /// Surface gravity in meters per second squared.
    pub surface_gravity_m_s2: MeasuredValue,
    /// Metallicity relative to solar abundance or chosen scale.
    pub metallicity: MeasuredValue,
    /// Rotation period in seconds.
    pub rotation_period_s: Option<MeasuredValue>,
    /// Stellar age in seconds.
    pub age_s: Option<MeasuredValue>,
    /// Barycentric position in meters.
    pub position_m: Vector3,
    /// Barycentric velocity in meters per second.
    pub velocity_m_s: Vector3,
}

/// Binary star pair with optional mutual orbit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BinaryStar {
    /// Primary star.
    pub primary: Star,
    /// Secondary star.
    pub secondary: Star,
    /// Relative binary orbit.
    pub orbit: Option<OrbitalProperties>,
}

/// Stellar system containing stars and child celestial bodies.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StellarSystem {
    /// Strong system identifier.
    pub id: SystemId,
    /// Display name.
    pub name: String,
    /// Stars belonging to the system.
    pub stars: Vec<Star>,
    /// Planet identifiers orbiting within this system.
    pub planets: Vec<PlanetId>,
    /// Barycentric position in meters.
    pub position_m: Vector3,
    /// Barycentric velocity in meters per second.
    pub velocity_m_s: Vector3,
}

/// Reusable orbital element data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrbitalProperties {
    /// Parent body being orbited.
    pub parent: BodyReference,
    /// Semi-major axis in meters.
    pub semi_major_axis_m: MeasuredValue,
    /// Semi-minor axis in meters.
    pub semi_minor_axis_m: Option<MeasuredValue>,
    /// Eccentricity, dimensionless.
    pub eccentricity: MeasuredValue,
    /// Inclination in radians.
    pub inclination_rad: MeasuredValue,
    /// Orbital period in seconds.
    pub orbital_period_s: Option<MeasuredValue>,
    /// Rotation period in seconds.
    pub rotation_period_s: Option<MeasuredValue>,
    /// Axial tilt in radians.
    pub axial_tilt_rad: Option<MeasuredValue>,
}

/// Bulk physical properties of a planet or moon.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhysicalProperties {
    /// Mass in kilograms.
    pub mass_kg: MeasuredValue,
    /// Mean radius in meters.
    pub radius_m: MeasuredValue,
    /// Mean density in kilograms per cubic meter.
    pub density_kg_m3: Option<MeasuredValue>,
    /// Surface gravity in meters per second squared.
    pub surface_gravity_m_s2: Option<MeasuredValue>,
}

/// Atmospheric composition and vertical structure.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AtmosphericProperties {
    /// Broad atmosphere type.
    pub atmosphere_type: AtmosphereType,
    /// Surface pressure in pascals.
    pub pressure_pa: Option<MeasuredValue>,
    /// Near-surface density in kilograms per cubic meter.
    pub density_kg_m3: Option<MeasuredValue>,
    /// Scale height in meters.
    pub scale_height_m: Option<MeasuredValue>,
    /// Atmospheric layers from surface upward.
    pub layers: Vec<AtmosphericLayer>,
    /// Atmospheric gas composition.
    pub composition: Vec<AtmosphericGas>,
    /// Cloud coverage fraction in `[0, 1]`.
    pub cloud_coverage: Option<MeasuredValue>,
    /// Greenhouse gases tracked for radiative models.
    pub greenhouse_gases: Vec<AtmosphericGas>,
}

/// Named atmospheric layer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AtmosphericLayer {
    /// Layer name.
    pub name: String,
    /// Base altitude in meters.
    pub base_altitude_m: MeasuredValue,
    /// Top altitude in meters.
    pub top_altitude_m: MeasuredValue,
    /// Representative temperature in kelvin.
    pub temperature_k: Option<MeasuredValue>,
}

/// Chemical element data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Element {
    /// Atomic number.
    pub atomic_number: u8,
    /// Chemical symbol.
    pub symbol: String,
    /// Element name.
    pub name: String,
    /// Atomic mass in unified atomic mass units.
    pub atomic_mass_u: Option<MeasuredValue>,
}

/// Molecule data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Molecule {
    /// Formula, such as `H2O`.
    pub formula: String,
    /// Common name.
    pub name: String,
    /// Molar mass in kilograms per mole.
    pub molar_mass_kg_mol: Option<MeasuredValue>,
}

/// Material data for surfaces and interiors.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Material {
    /// Material name.
    pub name: String,
    /// Surface material category.
    pub surface_material: SurfaceMaterial,
    /// Bulk abundance as mass, volume, or mole fraction.
    pub abundance: Option<MeasuredValue>,
}

/// Chemical compound data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Compound {
    /// Compound molecule.
    pub molecule: Molecule,
    /// Constituent elements.
    pub elements: Vec<Element>,
    /// Relative abundance.
    pub abundance: Option<MeasuredValue>,
}

/// Atmospheric gas abundance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AtmosphericGas {
    /// Gas molecule.
    pub molecule: Molecule,
    /// Mole fraction or partial pressure, depending on unit.
    pub abundance: MeasuredValue,
    /// Whether this gas contributes to greenhouse forcing in future models.
    pub is_greenhouse: bool,
}

/// Geological interior and surface data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeologicalProperties {
    /// Core model.
    pub core: Option<CoreProperties>,
    /// Mantle model.
    pub mantle: Option<MantleProperties>,
    /// Crust model.
    pub crust: Option<CrustProperties>,
    /// Surface materials.
    pub surface_materials: Vec<Material>,
    /// Plate tectonic system.
    pub plate_system: Option<PlateSystem>,
    /// Heat flow in watts per square meter.
    pub heat_flow_w_m2: Option<MeasuredValue>,
    /// Volcanic activity level.
    pub volcanism: VolcanicActivity,
}

/// Planetary core data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoreProperties {
    /// Core radius in meters.
    pub radius_m: Option<MeasuredValue>,
    /// Core materials.
    pub materials: Vec<Material>,
    /// Whether a liquid outer core is represented.
    pub has_liquid_outer_core: bool,
}

/// Mantle data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MantleProperties {
    /// Mantle thickness in meters.
    pub thickness_m: Option<MeasuredValue>,
    /// Mantle materials.
    pub materials: Vec<Material>,
}

/// Crust data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrustProperties {
    /// Mean crust thickness in meters.
    pub mean_thickness_m: Option<MeasuredValue>,
    /// Crust materials.
    pub materials: Vec<Material>,
}

/// Plate tectonic system data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlateSystem {
    /// Tectonic activity category.
    pub activity: TectonicActivity,
    /// Estimated number of major plates.
    pub major_plate_count: Option<u32>,
}

/// Climate state data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClimateProperties {
    /// Global climate type.
    pub climate_type: ClimateType,
    /// Average surface temperature in kelvin.
    pub average_temperature_k: Option<MeasuredValue>,
    /// Latitude or regional temperature bands.
    pub temperature_bands: Vec<TemperatureBand>,
    /// Wind regime.
    pub wind: Option<WindProperties>,
    /// Relative humidity fraction in `[0, 1]`.
    pub humidity: Option<MeasuredValue>,
    /// Surface ice coverage fraction in `[0, 1]`.
    pub ice_coverage: Option<MeasuredValue>,
    /// Seasonal model summary.
    pub seasons: Vec<Season>,
}

/// Temperature band for regional climate summaries.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemperatureBand {
    /// Band label.
    pub name: String,
    /// Minimum latitude in radians.
    pub min_latitude_rad: f64,
    /// Maximum latitude in radians.
    pub max_latitude_rad: f64,
    /// Average temperature in kelvin.
    pub average_temperature_k: MeasuredValue,
}

/// Wind state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WindProperties {
    /// Average wind speed in meters per second.
    pub average_speed_m_s: Option<MeasuredValue>,
    /// Prevailing direction vector in local/world coordinates.
    pub prevailing_direction: Option<Vector3>,
    /// Weather type associated with this wind regime.
    pub weather_type: WeatherType,
}

/// Seasonal summary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Season {
    /// Season name.
    pub name: String,
    /// Duration in seconds.
    pub duration_s: MeasuredValue,
    /// Average temperature in kelvin.
    pub average_temperature_k: Option<MeasuredValue>,
}

/// Ocean and hydrosphere data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OceanProperties {
    /// Ocean type.
    pub ocean_type: OceanType,
    /// Surface coverage fraction in `[0, 1]`.
    pub coverage: Option<MeasuredValue>,
    /// Average depth in meters.
    pub average_depth_m: Option<MeasuredValue>,
    /// Ocean composition materials or compounds.
    pub composition: Vec<Compound>,
}

/// Magnetic field data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MagneticFieldProperties {
    /// Equatorial or representative field strength in tesla.
    pub field_strength_t: Option<MeasuredValue>,
    /// Pole orientation vector.
    pub pole_orientation: Option<Vector3>,
    /// Magnetosphere radius in meters.
    pub magnetosphere_radius_m: Option<MeasuredValue>,
}

/// Explainable habitability assessment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HabitabilityProperties {
    /// Overall habitability rating.
    pub rating: HabitabilityRating,
    /// Positive scientific factors.
    pub positive_factors: Vec<String>,
    /// Negative scientific factors.
    pub negative_factors: Vec<String>,
    /// Scientific notes supporting the assessment.
    pub scientific_notes: Vec<String>,
    /// Confidence in `[0, 1]`.
    pub confidence: Option<f64>,
    /// Life stage, if any.
    pub life_stage: LifeStage,
}

/// Planet composed from focused property models.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Planet {
    /// Strong planet identifier.
    pub id: PlanetId,
    /// Display name.
    pub name: String,
    /// Planet class.
    pub class: PlanetClass,
    /// Planet type.
    pub planet_type: PlanetType,
    /// Parent stellar system.
    pub system_id: SystemId,
    /// Physical data.
    pub physical: PhysicalProperties,
    /// Orbit data.
    pub orbit: OrbitalProperties,
    /// Geological data.
    pub geology: Option<GeologicalProperties>,
    /// Atmospheric data.
    pub atmosphere: Option<AtmosphericProperties>,
    /// Climate data.
    pub climate: Option<ClimateProperties>,
    /// Ocean data.
    pub ocean: Option<OceanProperties>,
    /// Magnetic field data.
    pub magnetic_field: Option<MagneticFieldProperties>,
    /// Habitability data.
    pub habitability: Option<HabitabilityProperties>,
    /// Child moon identifiers.
    pub moons: Vec<MoonId>,
}

/// Moon represented as an independent celestial body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Moon {
    /// Strong moon identifier.
    pub id: MoonId,
    /// Display name.
    pub name: String,
    /// Parent body.
    pub parent: BodyReference,
    /// Physical data.
    pub physical: PhysicalProperties,
    /// Orbit data.
    pub orbit: OrbitalProperties,
    /// Geological data.
    pub geology: Option<GeologicalProperties>,
    /// Atmospheric data.
    pub atmosphere: Option<AtmosphericProperties>,
    /// Nested child moons.
    pub moons: Vec<MoonId>,
}
