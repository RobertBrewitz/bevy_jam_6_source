use bevy::prelude::*;

use crate::routes::SyltRouterState;

pub struct SyltSystemSetPlugin;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum SyltGameSystemSet {
    Despawn,
    /**
     * Flush command queue and apply deferred commands before proceeding using apply_deferred.
     */
    Input,
    Update,
}

/// Whether or not the game is paused.
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[states(scoped_entities)]
pub struct SyltGamePauseState(pub bool);

/// A system set for systems that shouldn't run while the game is paused.
#[derive(SystemSet, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct SyltPausableSystems;

impl Plugin for SyltSystemSetPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                SyltGameSystemSet::Despawn,
                SyltGameSystemSet::Input,
                SyltGameSystemSet::Update,
            )
                .chain()
                .run_if(in_state(SyltRouterState::Game)),
        );
        app.add_systems(
            Update,
            ApplyDeferred
                .after(SyltGameSystemSet::Despawn)
                .before(SyltGameSystemSet::Input)
                .run_if(in_state(SyltRouterState::Game)),
        );

        // Pause state management
        app.init_state::<SyltGamePauseState>();
        app.configure_sets(
            Update,
            SyltPausableSystems.run_if(in_state(SyltGamePauseState(false))),
        );
    }
}
