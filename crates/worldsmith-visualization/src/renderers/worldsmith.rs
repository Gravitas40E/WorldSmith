//! Composed Bevy plugin for the full WorldSmith visualization stack.

use bevy::prelude::*;

use crate::{BevyViewerPlugin, BevyVisualizationPlugin};

/// Convenience plugin that registers both the visualization pipeline
/// and the interactive viewer in the correct composition order.
///
/// This plugin guarantees:
/// - `BevySceneRenderer` resource exists before viewer systems run.
/// - Camera and light are spawned once.
/// - Viewer controls, labels, and FPS overlay are available immediately.
///
/// To customize startup, add your own `Startup` systems *before* this
/// plugin so they run first, or disable the default visualization plugin
/// and assemble the resources manually.
pub struct WorldSmithVisualizationPlugin;

impl Plugin for WorldSmithVisualizationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            // Core renderer must come first so the viewer can query its
            // resource and camera entities.
            BevyVisualizationPlugin,
            BevyViewerPlugin,
        ));
    }
}
