use std::time::Duration;

use bevy::prelude::Resource;
use tracing::trace;

use crate::tick::Tick;

#[derive(Clone)]
pub struct TickConfig {
    pub tick_duration: Duration,
}

impl TickConfig {
    pub fn new(tick_duration: Duration) -> Self {
        Self { tick_duration }
    }
}

/// Manages the tick for the host system. Ticks are incremented by one every time
/// the [`bevy::prelude::FixedUpdate`] schedule runs
#[derive(Resource)]
pub struct TickManager {
    /// Tick configuration
    pub config: TickConfig,
    /// Current tick (sequence number of the FixedUpdate schedule)
    tick: Tick,
}

impl TickManager {
    pub fn from_config(config: TickConfig) -> Self {
        Self {
            config,
            tick: Tick(0),
        }
    }

    // NOTE: this is public just for integration testing purposes
    #[doc(hidden)]
    pub fn increment_tick(&mut self) {
        self.tick += 1;
        trace!(new_tick = ?self.tick, "incremented client tick")
    }
    pub(crate) fn set_tick_to(&mut self, tick: Tick) {
        self.tick = tick;
    }

    pub fn current_tick(&self) -> Tick {
        self.tick
    }
}
