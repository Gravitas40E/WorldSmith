//! WorldSmith visualization local architecture.
//!
//! This crate converts immutable `SimulationSnapshot` data into an internal
//! render-ready scene abstraction, then emits that scene through a pluggable
//! renderer interface. No simulation logic, UI chrome, or backend-specific
//! types belong here.
//!
//! The visualization layer consumes authoritative world-space coordinates
//! produced by the simulation. It performs no orbital projection, prediction,
//! or interpolation.
//!
//! When using the Bevy backend, [`WorldSmithVisualizationPlugin`] provides
//! a single entry point that composes the visualization renderer and the
//! interactive viewer in a safe ownership order.

pub mod bridge;
pub mod plugin;
pub mod renderer;
pub mod renderers;
pub mod scene;

pub use bridge::{DefaultSnapshotBridge, SnapshotBridge};
pub use plugin::VisualizationPlugin;
pub use renderer::SceneRenderer;
pub use renderers::bevy::BevyVisualizationPlugin;
pub use renderers::viewer::BevyViewerPlugin;
pub use renderers::worldsmith::WorldSmithVisualizationPlugin;
pub use scene::{BodyCategory, ColorHint, RenderScene, SceneBody};
