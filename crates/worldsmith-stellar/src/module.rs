//! Engine integration for deterministic stellar profiles.

use serde::{Deserialize, Serialize};
use worldsmith_models::{StarId, SystemId};
use worldsmith_state::{
    EventId, EventPayload, EventSource, EventTarget, FieldKey, SimulationEvent,
};
use worldsmith_traits::{
    ContractResult, ModuleContext, SimulationModule, SnapshotProducer, StateWriter,
};

use crate::{builder::StellarProfile, StarBuilder};

/// Configuration for the stellar simulation module.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StellarModuleConfig {
    /// Stellar system id that receives generated star data.
    pub system_id: SystemId,
    /// Star id for the primary star.
    pub star_id: StarId,
    /// Display name.
    pub name: String,
    /// Mass in solar masses.
    pub mass_solar: f64,
    /// Age in gigayears.
    pub age_gyr: f64,
    /// Metallicity mass fraction.
    pub metallicity: f64,
    /// Optional rotation period in days.
    pub rotation_days: Option<f64>,
}

impl Default for StellarModuleConfig {
    fn default() -> Self {
        Self {
            system_id: SystemId(1),
            star_id: StarId(1),
            name: "Primary".to_string(),
            mass_solar: 1.0,
            age_gyr: 4.57,
            metallicity: 0.0134,
            rotation_days: Some(25.4),
        }
    }
}

/// Simulation module that produces deterministic stellar data from configured inputs.
pub struct StellarModule {
    config: StellarModuleConfig,
    profile: Option<StellarProfile>,
    pending_events: Vec<SimulationEvent>,
    initialized: bool,
}

impl StellarModule {
    /// Creates a module from configuration.
    pub fn new(config: StellarModuleConfig) -> Self {
        Self {
            config,
            profile: None,
            pending_events: Vec::new(),
            initialized: false,
        }
    }

    /// Returns the latest stellar profile.
    pub fn profile(&self) -> Option<&StellarProfile> {
        self.profile.as_ref()
    }

    fn build_profile(&self) -> ContractResult<StellarProfile> {
        let mut builder = StarBuilder::new()
            .id(self.config.star_id)
            .name(self.config.name.clone())
            .mass_solar(self.config.mass_solar)
            .age_gyr(self.config.age_gyr)
            .metallicity(self.config.metallicity);
        if let Some(rotation_days) = self.config.rotation_days {
            builder = builder.rotation_days(rotation_days);
        }
        builder
            .build()
            .map_err(|err| worldsmith_traits::ContractError::InvalidInput(err.to_string()))
    }

    fn publish_payload(&mut self, payload: EventPayload, timestamp_s: f64) {
        self.pending_events.push(SimulationEvent {
            id: EventId(0),
            timestamp_s,
            source: EventSource::Module(self.id().to_string()),
            target: EventTarget::Star(self.config.star_id),
            payload,
        });
    }
}

impl Default for StellarModule {
    fn default() -> Self {
        Self::new(StellarModuleConfig::default())
    }
}

impl SimulationModule for StellarModule {
    fn id(&self) -> &'static str {
        "worldsmith.stellar"
    }

    fn name(&self) -> &'static str {
        "WorldSmith Stellar Module"
    }

    fn initialize(&mut self, state: &mut dyn StateWriter) -> ContractResult<()> {
        let profile = self.build_profile()?;
        state
            .world_mut()
            .stars
            .insert(profile.star.id, profile.star.clone());
        if let Some(system) = state
            .world_mut()
            .stellar_systems
            .get_mut(&self.config.system_id)
        {
            if !system.stars.iter().any(|star| star.id == profile.star.id) {
                system.stars.push(profile.star.clone());
            }
        }
        self.profile = Some(profile);
        self.initialized = true;
        self.publish_payload(
            EventPayload::StarCreated {
                star_id: self.config.star_id,
            },
            state.world().clock.elapsed_seconds(),
        );
        self.publish_payload(
            EventPayload::HabitableZoneChanged {
                star_id: self.config.star_id,
            },
            state.world().clock.elapsed_seconds(),
        );
        Ok(())
    }

    fn update(
        &mut self,
        context: ModuleContext,
        state: &mut dyn StateWriter,
    ) -> ContractResult<()> {
        if !self.initialized {
            return Ok(());
        }
        let profile = self.build_profile()?;
        let previous_luminosity = self.profile.as_ref().map(|p| p.luminosity_solar);
        state
            .world_mut()
            .stars
            .insert(profile.star.id, profile.star.clone());
        self.profile = Some(profile);
        self.publish_payload(
            EventPayload::StarUpdated {
                star_id: self.config.star_id,
            },
            context.timestamp_s,
        );
        self.publish_payload(
            EventPayload::StarAged {
                star_id: self.config.star_id,
            },
            context.timestamp_s,
        );
        if previous_luminosity != self.profile.as_ref().map(|p| p.luminosity_solar) {
            self.publish_payload(
                EventPayload::LuminosityChanged {
                    star_id: self.config.star_id,
                },
                context.timestamp_s,
            );
            self.publish_payload(
                EventPayload::HabitableZoneChanged {
                    star_id: self.config.star_id,
                },
                context.timestamp_s,
            );
        }
        Ok(())
    }

    fn shutdown(&mut self, _state: &mut dyn StateWriter) -> ContractResult<()> {
        self.initialized = false;
        Ok(())
    }

    fn reads(&self) -> Vec<FieldKey> {
        vec![FieldKey::StellarLuminosity]
    }

    fn writes(&self) -> Vec<FieldKey> {
        vec![FieldKey::StellarLuminosity, FieldKey::SurfaceGravity]
    }

    fn publish_events(&mut self) -> Vec<SimulationEvent> {
        std::mem::take(&mut self.pending_events)
    }

    fn consume_events(&mut self, _events: &[SimulationEvent]) -> ContractResult<()> {
        Ok(())
    }
}

impl SnapshotProducer for StellarModule {
    type Snapshot = Option<StellarProfile>;

    fn snapshot(&self) -> Self::Snapshot {
        self.profile.clone()
    }
}
