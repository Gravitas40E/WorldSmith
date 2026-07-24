//! Minimal visualization plugin that composes snapshot bridging and rendering.

use worldsmith_state::SimulationSnapshot;

use crate::{bridge::SnapshotBridge, renderer::SceneRenderer};

/// Minimal plugin that ties a [`SnapshotBridge`] and a [`SceneRenderer`]
/// together.
///
/// The plugin does not own engine integration or ECS entities.
pub struct VisualizationPlugin<B, R> {
    bridge: B,
    renderer: R,
}

impl<B, R> VisualizationPlugin<B, R>
where
    B: SnapshotBridge,
    R: SceneRenderer,
{
    /// Creates a new visualization plugin.
    pub fn new(bridge: B, renderer: R) -> Self {
        Self { bridge, renderer }
    }

    /// Converts a simulation snapshot into a render scene and renders it.
    ///
    /// Takes `&mut self` because renderers are typically stateful (cached
    /// handles, command buffers, output buffers). This preserves the
    /// renderer-agnostic trait boundary.
    pub fn render_snapshot(&mut self, snapshot: &SimulationSnapshot) {
        let scene = self.bridge.build_scene(snapshot);
        self.renderer.render(&scene);
    }
}
