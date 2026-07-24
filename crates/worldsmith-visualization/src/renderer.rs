//! Scene renderer interface.

use crate::scene::RenderScene;

/// Abstraction for consuming a [`RenderScene`].
///
/// Implementors decide how the scene is rendered: wgpu, Bevy, debug text,
/// export, or headless batch output.
///
/// # Allocation note
///
/// `render` receives the scene by reference, but be aware that the
/// renderer-agnostic boundary means a concrete renderer cannot share an
/// internal scene handle with the bridge without coupling to snapshot
/// internals. Cloning the scene here is an intentional architectural
/// tradeoff: it keeps [`SceneRenderer`] independent of
/// `SimulationSnapshot` and `DefaultSnapshotBridge` lifetimes. See
/// [`BevySceneRenderer`](crate::renderers::BevySceneRenderer) for one
/// implementation that absorbs the clone into an ECS flush.
pub trait SceneRenderer {
    /// Render or process the provided scene.
    fn render(&mut self, scene: &RenderScene);
}
