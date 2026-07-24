//! Bridge from simulation snapshots to render scenes.
//!
//! The default implementation extracts authoritative world-space positions
//! directly from [`SimulationSnapshot`]. No orbital projection or placeholder
//! coordinates are used.

use worldsmith_models::{Moon, Planet, Star, StellarSystem};
use worldsmith_state::SimulationSnapshot;

use crate::scene::{BodyCategory, ColorHint, RenderScene, SceneBody};

/// Converts a [`SimulationSnapshot`] into a [`RenderScene`].
///
/// Implementations may choose different extraction strategies or LOD policies.
pub trait SnapshotBridge {
    fn build_scene(&self, snapshot: &SimulationSnapshot) -> RenderScene;
}

/// Default snapshot bridge that extracts render data directly from
/// [`SimulationSnapshot`].
///
/// World-space positions are taken from the simulation layer. Visualization
/// performs no orbital propagation.
pub struct DefaultSnapshotBridge;

impl DefaultSnapshotBridge {
    /// Builds a body for a stellar system.
    fn build_stellar_system_body(&self, system: &StellarSystem) -> SceneBody {
        SceneBody {
            id: system.id.0.to_string(),
            name: system.name.clone(),
            category: BodyCategory::StellarSystem,
            position_m: [
                system.position_m.x,
                system.position_m.y,
                system.position_m.z,
            ],
            radius_m: 0.0, // TODO: replace with system scale once defined by models.
            color_hint: ColorHint::Classification,
        }
    }

    /// Builds a body for a star.
    fn build_star_body(&self, star: &Star) -> SceneBody {
        SceneBody {
            id: star.id.0.to_string(),
            name: star.name.clone(),
            category: BodyCategory::Star,
            position_m: [star.position_m.x, star.position_m.y, star.position_m.z],
            radius_m: star.radius_m.value,
            color_hint: ColorHint::Classification,
        }
    }

    /// Builds a body for a planet using authoritative simulation coordinates.
    fn build_planet_body(&self, planet: &Planet) -> SceneBody {
        SceneBody {
            id: planet.id.0.to_string(),
            name: planet.name.clone(),
            category: BodyCategory::Planet,
            position_m: [
                planet.position_m.x,
                planet.position_m.y,
                planet.position_m.z,
            ],
            radius_m: planet.physical.radius_m.value,
            color_hint: ColorHint::Classification,
        }
    }

    /// Builds a body for a moon using authoritative simulation coordinates.
    fn build_moon_body(&self, moon: &Moon) -> SceneBody {
        SceneBody {
            id: moon.id.0.to_string(),
            name: moon.name.clone(),
            category: BodyCategory::Moon,
            position_m: [moon.position_m.x, moon.position_m.y, moon.position_m.z],
            radius_m: moon.physical.radius_m.value,
            color_hint: ColorHint::Classification,
        }
    }

    /// Collects all renderable bodies from a snapshot.
    fn collect_bodies(&self, snapshot: &SimulationSnapshot) -> Vec<SceneBody> {
        let total = snapshot.stellar.systems.len()
            + snapshot.stellar.stars.len()
            + snapshot.planets.len()
            + snapshot.moons.len();
        let mut bodies = Vec::with_capacity(total);

        for system in &snapshot.stellar.systems {
            bodies.push(self.build_stellar_system_body(system));
        }

        for star in &snapshot.stellar.stars {
            bodies.push(self.build_star_body(star));
        }

        for planet_snapshot in &snapshot.planets {
            bodies.push(self.build_planet_body(&planet_snapshot.planet));
        }

        for moon in &snapshot.moons {
            bodies.push(self.build_moon_body(moon));
        }

        bodies
    }
}

impl SnapshotBridge for DefaultSnapshotBridge {
    fn build_scene(&self, snapshot: &SimulationSnapshot) -> RenderScene {
        RenderScene {
            timestamp_s: snapshot.timestamp_s,
            bodies: self.collect_bodies(snapshot),
        }
    }
}
