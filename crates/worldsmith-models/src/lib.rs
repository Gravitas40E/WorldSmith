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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PlanetType {
    #[default]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ClimateType {
    /// No dominant climate assigned.
    #[default]
    Unknown,
    /// Frozen surface conditions.
    Frozen,
    /// Cold glaciated world.
    Cold,
    /// Temperate climate regime.
    Temperate,
    /// Arid climate regime.
    Arid,
    /// Tropical or warm wet climate regime.
    Tropical,
    /// Warm climate regime.
    Warm,
    /// Hot climate regime.
    Hot,
    /// Inferno-class surface temperatures.
    Inferno,
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

/// Habitability classification for V1 assessment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum HabitabilityClass {
    /// No known habitable characteristics.
    #[default]
    Hostile,
    /// Minimal conditions for extremophiles.
    Marginal,
    /// Potentially habitable for simple life.
    Habitable,
    /// Strong habitability indicators.
    HighlyHabitable,
    /// Exceptional conditions.
    Paradise,
}

/// Dominant limiting factor in habitability assessment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LimitingFactor {
    /// No significant limiting factor.
    #[default]
    None,
    /// Surface temperature too low.
    TooCold,
    /// Surface temperature too high.
    TooHot,
    /// Insufficient liquid water.
    TooDry,
    /// No atmosphere or pressure too low.
    NoAtmosphere,
    /// Extreme CO2 levels.
    ExtremeCO2,
    /// Insufficient biomass.
    LowBiomass,
    /// Global ice cover.
    GlobalIceCover,
}

/// Plate tectonic activity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TectonicActivity {
    /// No active tectonics.
    #[default]
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
/// Plate tectonics evolution state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PlateTectonicsState {
    /// Plate velocity in centimeters per year.
    pub plate_velocity: f64,
    /// Crustal recycling rate in arbitrary mass per second.
    pub crustal_recycling_rate: f64,
    /// Classified tectonic activity level.
    pub tectonic_activity: TectonicActivity,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum VolcanicActivity {
    /// No active volcanism.
    #[default]
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
/// Volcanic evolution state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct VolcanismState {
    /// Volcanic mass flux in kilograms per second.
    pub volcanic_flux: f64,
    /// Classified volcanic activity level.
    pub volcanic_activity: VolcanicActivity,
    /// Magma generation rate in kilograms per second.
    pub magma_generation_rate: f64,
}
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
/// Interior planetary state owned by evolution modules.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct InteriorState {
    /// Elapsed age of the planetary interior in seconds.
    pub age_seconds: f64,
    /// Core temperature in kelvin.
    pub core_temperature: f64,
    /// Mantle temperature in kelvin.
    pub mantle_temperature: f64,
    /// Remaining radiogenic heat in joules.
    pub radiogenic_heat: f64,
    /// Stored internal heat in joules.
    pub internal_heat: f64,
    /// Surface heat flux in watts per square meter.
    pub heat_flux: f64,
}
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
/// Deterministic bulk atmosphere state owned by the atmosphere module.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AtmosphereState {
    /// Total atmospheric mass in kilograms.
    pub atmospheric_mass_kg: f64,
    /// Surface pressure in pascals.
    pub surface_pressure_pa: f64,
    /// Mean atmospheric temperature in kelvin.
    pub mean_temperature_k: f64,
    /// Atmospheric gas composition as a list of abundance samples.
    pub atmosphere_composition: Vec<AtmosphericGas>,
}

/// Deterministic bulk hydrosphere state owned by the hydrology module.
///
/// This implementation models only planetary-scale water reservoirs.
/// No weather, precipitation, rivers, groundwater, or ocean circulation
/// is simulated in V1.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct HydrologyState {
    /// Total planetary water inventory in kilograms.
    pub total_water_mass_kg: f64,
    /// Liquid ocean water mass in kilograms.
    pub ocean_mass_kg: f64,
    /// Atmospheric water vapor mass in kilograms.
    pub atmospheric_water_mass_kg: f64,
    /// Surface and subsurface ice mass in kilograms.
    pub ice_mass_kg: f64,
    /// Fraction of total water that is liquid.
    pub liquid_water_fraction: f64,
}

/// Deterministic global climate state owned by the climate module.
///
/// This implementation represents a deterministic zero-dimensional
/// planetary energy balance model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ClimateState {
    /// Equilibrium temperature in kelvin before greenhouse offset.
    pub equilibrium_temperature_k: f64,
    /// Greenhouse warming offset applied above equilibrium.
    pub greenhouse_temperature_offset_k: f64,
    /// Planetary Bond albedo.
    pub planetary_albedo: f64,
    /// Deterministic climate classification.
    pub climate_classification: ClimateType,
}

/// Deterministic bulk carbon cycle state owned by the carbon cycle module.
///
/// This implementation models only planetary-scale carbon reservoirs and
/// fluxes between atmosphere, ocean, and lithosphere. No biology, no
/// carbonate chemistry, and no detailed geochemistry are simulated in V1.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CarbonCycleState {
    /// Atmospheric carbon mass in kilograms (primarily CO2).
    pub atmospheric_carbon_mass_kg: f64,
    /// Ocean dissolved inorganic carbon mass in kilograms.
    pub ocean_carbon_mass_kg: f64,
    /// Lithosphere carbon storage mass in kilograms.
    pub lithosphere_carbon_mass_kg: f64,
    /// Volcanic degassing flux into the atmosphere in kilograms per second.
    pub volcanic_carbon_flux_kg_per_s: f64,
    /// Silicate weathering removal flux in kilograms per second.
    pub weathering_flux_kg_per_s: f64,
    /// Net air-sea gas exchange flux in kilograms per second.
    pub ocean_exchange_flux_kg_per_s: f64,
    /// Atmospheric CO2 mole fraction (derived).
    pub atmospheric_co2_fraction: f64,
    /// Ratio of ocean to atmospheric carbon mass (derived).
    pub carbon_partition_ratio: f64,
    /// Weathering efficiency factor (derived).
    pub weathering_efficiency: f64,
}

/// Active ice reservoirs and cryosphere properties.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CryosphereState {
    /// Continental ice mass in kilograms.
    pub continental_ice_mass_kg: f64,
    /// Sea ice mass in kilograms.
    pub sea_ice_mass_kg: f64,
    /// Surface snow mass in kilograms.
    pub snow_mass_kg: f64,
    /// Fraction of land surface covered by permanent ice (derived) in [0, 1].
    pub permanent_ice_fraction: f64,
    /// Fraction of land surface covered by seasonal snow (derived) in [0, 1].
    pub seasonal_snow_fraction: f64,
    /// Ice melt rate in kilograms per second (derived).
    pub melt_rate_kg_per_s: f64,
    /// Ice freeze rate in kilograms per second (derived).
    pub freeze_rate_kg_per_s: f64,
    /// Total ice as a fraction of total planetary water (derived) in [0, 1].
    pub planetary_ice_fraction: f64,
    /// Cryosphere contribution to Bond albedo (derived) in [0, 1].
    pub cryosphere_albedo_modifier: f64,
    /// Sea-level contribution from ice melt relative to a reference (derived, meters).
    pub sea_level_offset_m: f64,
}

/// Planetary biomass and ecosystem properties.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct BiosphereState {
    /// Total planetary biomass in kilograms.
    pub total_biomass_kg: f64,
    /// Terrestrial (land) biomass in kilograms.
    pub terrestrial_biomass_kg: f64,
    /// Marine (ocean) biomass in kilograms.
    pub marine_biomass_kg: f64,
    /// Dead organic carbon stored in soils and sediments in kilograms.
    pub dead_organic_carbon_kg: f64,
    /// Planetary gross primary productivity in kilograms per second.
    pub productivity_rate_kg_per_s: f64,
    /// Planetary total respiration rate in kilograms per second.
    pub respiration_rate_kg_per_s: f64,
    /// Derived habitability factor in [0, 1].
    pub habitability_factor: f64,
    /// Fraction of land surface covered by vegetation (derived).
    pub vegetation_fraction: f64,
    /// Derived ocean productivity factor in [0, 1].
    pub ocean_productivity_factor: f64,
}

/// Bulk surface chemistry reservoirs and weathering properties.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SurfaceChemistryState {
    /// Bulk silicate reservoir mass in kilograms.
    pub silicate_mass_kg: f64,
    /// Carbonate reservoir mass in kilograms.
    pub carbonate_mass_kg: f64,
    /// Oxidized surface material mass in kilograms.
    pub oxidized_material_mass_kg: f64,
    /// Reduced surface material mass in kilograms.
    pub reduced_material_mass_kg: f64,
    /// Dissolved mineral mass in kilograms.
    pub dissolved_mineral_mass_kg: f64,
    /// Weathering flux in kilograms per second (derived).
    pub weathering_rate_kg_per_s: f64,
    /// Sedimentation flux in kilograms per second (derived).
    pub sedimentation_rate_kg_per_s: f64,
    /// Weathering intensity index (derived) in [0, 1].
    pub weathering_index: f64,
    /// Surface reactivity index (derived) in [0, 1].
    pub surface_reactivity: f64,
    /// Mineral availability index (derived) in [0, 1].
    pub mineral_availability: f64,
}

/// Deterministic planetary habitability assessment state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct HabitabilityState {
    /// Overall habitability score in [0, 1].
    pub overall_habitability_index: f64,
    /// Surface habitability score in [0, 1].
    pub surface_habitability_index: f64,
    /// Ocean habitability score in [0, 1].
    pub ocean_habitability_index: f64,
    /// Biological potential score in [0, 1].
    pub biological_potential_index: f64,
    /// Climate stability score in [0, 1].
    pub climate_stability_index: f64,
    /// Water availability score in [0, 1].
    pub water_availability_index: f64,
    /// Atmosphere suitability score in [0, 1].
    pub atmosphere_suitability_index: f64,
    /// Habitability classification.
    pub habitability_class: HabitabilityClass,
    /// Dominant limiting factor, if any.
    pub limiting_factor: Option<LimitingFactor>,
}

/// Primary planetary classification categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PrimaryClassification {
    #[default]
    Terrestrial,
    OceanWorld,
    IceWorld,
    DesertWorld,
    LavaWorld,
    RockyPlanet,
}

/// Secondary classification modifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SecondaryClassification {
    #[default]
    None,
    Temperate,
    Frozen,
    CarbonRich,
    HighBiomass,
    LowAtmosphere,
    DenseAtmosphere,
}

/// Hydrosphere category for classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum HydrosphereCategory {
    #[default]
    None,
    Liquid,
    Ice,
    Mixed,
    Dry,
}

/// Biosphere category for classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BiosphereCategory {
    #[default]
    None,
    LowBiomass,
    ModerateBiomass,
    HighBiomass,
    Dominant,
}

/// Deterministic planetary classification result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PlanetClassificationState {
    /// Primary classification category.
    pub primary_classification: PrimaryClassification,
    /// Secondary modifier, if any.
    pub secondary_classification: SecondaryClassification,
    /// Terrestrial type category.
    pub terrestrial_type: PlanetType,
    /// Climate category.
    pub climate_category: ClimateType,
    /// Hydrosphere category.
    pub hydrosphere_category: HydrosphereCategory,
    /// Biosphere category.
    pub biosphere_category: BiosphereCategory,
    /// Classification confidence score in [0, 1].
    pub confidence_score: f64,
    /// Human-readable classification summary.
    pub classification_summary: String,
    /// Notable planetary features.
    pub notable_features: Vec<String>,
}

/// Interior geological properties.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
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
    /// Atmospheric category and layers.
    pub atmosphere: Option<AtmosphericProperties>,
    /// Deterministic atmospheric evolution state.
    pub atmosphere_state: Option<AtmosphereState>,
    /// Deterministic hydrosphere evolution state.
    pub hydrology_state: Option<HydrologyState>,
    /// Deterministic global climate evolution state.
    pub climate_state: Option<ClimateState>,
    /// Deterministic carbon cycle evolution state.
    pub carbon_cycle_state: Option<CarbonCycleState>,
    /// Deterministic biosphere evolution state.
    pub biosphere_state: Option<BiosphereState>,
    /// Deterministic cryosphere evolution state.
    pub cryosphere_state: Option<CryosphereState>,
    /// Deterministic surface chemistry evolution state.
    pub surface_chemistry_state: Option<SurfaceChemistryState>,
    /// Deterministic habitability assessment state.
    pub habitability_state: Option<HabitabilityState>,
    /// Deterministic planetary classification state.
    pub classification_state: Option<PlanetClassificationState>,
    /// Interior thermal state.
    pub interior: Option<InteriorState>,
    /// Volcanic evolution state.
    pub volcanism: Option<VolcanismState>,
    /// Plate tectonic evolution state.
    pub plate_tectonics: Option<PlateTectonicsState>,
    /// Climate data.
    pub climate: Option<ClimateProperties>,
    /// Ocean data.
    pub ocean: Option<OceanProperties>,
    /// Magnetic field data.
    pub magnetic_field: Option<MagneticFieldProperties>,
    /// Habitability data.
    pub habitability: Option<HabitabilityProperties>,
    /// Barycentric position in meters.
    pub position_m: Vector3,
    /// Barycentric velocity in meters per second.
    pub velocity_m_s: Vector3,
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
    /// Deterministic atmospheric evolution state.
    pub atmosphere_state: Option<AtmosphereState>,
    /// Deterministic hydrosphere evolution state.
    pub hydrology_state: Option<HydrologyState>,
    /// Deterministic global climate evolution state.
    pub climate_state: Option<ClimateState>,
    /// Deterministic carbon cycle evolution state.
    pub carbon_cycle_state: Option<CarbonCycleState>,
    /// Barycentric position in meters.
    pub position_m: Vector3,
    /// Barycentric velocity in meters per second.
    pub velocity_m_s: Vector3,
    /// Nested child moons.
    pub moons: Vec<MoonId>,
}
