use std::time::Duration;

use bevy::{ecs::system::EntityCommands, prelude::*};

pub struct DelayedCommandPlugin;

impl Plugin for DelayedCommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, run_delayed_commands);
    }
}

#[derive(Component)]
pub struct DelayedCommand {
    // This could be FnOnce if we did some mem/swap magic with a no-op closure
    pub command: Box<dyn FnMut(&mut Commands) + Send + Sync + 'static>,
    pub delay: Timer,
}

impl DelayedCommand {
    pub fn new(secs: f32, command: impl FnMut(&mut Commands) + Send + Sync + 'static) -> Self {
        Self {
            command: Box::new(command),
            delay: Timer::new(Duration::from_secs_f32(secs), TimerMode::Once),
        }
    }
}

fn run_delayed_commands(
    mut commands: Commands,
    mut delayed_commands: Query<(Entity, &mut DelayedCommand)>,
    time: Res<Time>,
) {
    for (entity, mut command) in &mut delayed_commands {
        if !command.delay.tick(time.delta()).just_finished() {
            continue;
        }

        (command.command)(&mut commands);
        commands.entity(entity).despawn_recursive();
    }
}

pub trait DelayedCommandExt {
    fn delayed(
        &mut self,
        secs: f32,
        command: impl FnMut(&mut Commands) + Send + Sync + 'static,
    ) -> EntityCommands<'_>;
}

impl DelayedCommandExt for Commands<'_, '_> {
    fn delayed(
        &mut self,
        secs: f32,
        command: impl FnMut(&mut Commands) + Send + Sync + 'static,
    ) -> EntityCommands<'_> {
        self.spawn(DelayedCommand::new(secs, command))
    }
}
