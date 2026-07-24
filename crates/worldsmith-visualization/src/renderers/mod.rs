//! Renderer backend implementations.
//!
//! Concrete [`SceneRenderer`] implementations live here so that backend
//! types never leak into core abstractions.

pub mod bevy;
pub mod viewer;
pub mod worldsmith;
