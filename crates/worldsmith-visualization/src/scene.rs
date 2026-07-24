//! Render-ready scene representation.
//!
//! All positions in this structure are authoritative barycentric coordinates
//! supplied by the simulation layer. No orbital math is performed here.

/// Internal render-ready scene built from simulation data.
///
/// This is the visualization crate's private scene format. A concrete
/// [`super::SceneRenderer`] consumes this structure.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderScene {
    /// Simulation timestamp in seconds.
    pub timestamp_s: f64,
    /// Renderable bodies in this scene.
    pub bodies: Vec<SceneBody>,
}

/// A single renderable celestial body.
#[derive(Debug, Clone, PartialEq)]
pub struct SceneBody {
    /// Display name.
    pub name: String,
    /// Stable origin identifier as a string.
    pub id: String,
    /// Body classification for renderers.
    pub category: BodyCategory,
    /// Barycentric position in meters.
    pub position_m: [f64; 3],
    /// Mean radius in meters.
    pub radius_m: f64,
    /// Color hint for the renderer.
    pub color_hint: ColorHint,
}

/// Broad celestial body category used by renderers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodyCategory {
    /// Stellar system barycenter.
    StellarSystem,
    /// Star.
    Star,
    /// Planet.
    Planet,
    /// Moon.
    Moon,
}

/// Color hint for a renderable body.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorHint {
    /// Use the body's classification or spectral type for color lookup.
    Classification,
    /// Use an approximate temperature-derived color.
    Temperature,
    /// Explicit RGB color in linear space [0, 1].
    Rgb { r: f32, g: f32, b: f32 },
}
