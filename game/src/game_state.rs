use bevy::prelude::*;
use bevy_pkv::PkvStore;

use crate::ui::challenges::ChallengeState;

pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ChallengeState::new())
            .insert_resource(PkvStore::new("DylanRJohnston", "SimonSays"))
            .add_systems(Startup, setup)
            .add_systems(Update, save_state)
            .observe(reset_challenge_state);
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub enum GameState {
    #[default]
    Loading,
    MainMenu,
    InGame,
    Paused,
}

#[derive(SubStates, Clone, PartialEq, Eq, Debug, Hash, Default)]
#[source(GameState = GameState::InGame)]
pub enum LevelState {
    #[default]
    Loading,
    Loaded,
    Unloading,
}

fn setup(pkv: Res<PkvStore>, mut state: ResMut<ChallengeState>) {
    if let Ok(from_storage) = pkv.get::<ChallengeState>("challenge_state") {
        *state = from_storage;
    }
}

fn save_state(mut pkv: ResMut<PkvStore>, state: Res<ChallengeState>) {
    if !state.is_changed() {
        return;
    }

    if let Err(err) = pkv.set("challenge_state", &*state) {
        tracing::error!(?err, "failed to save challenge state");
    }
}

#[derive(Debug, Clone, Copy, Event)]
pub struct ResetChallengeState;

fn reset_challenge_state(
    _trigger: Trigger<ResetChallengeState>,
    mut state: ResMut<ChallengeState>,
) {
    *state = ChallengeState::new();
}
