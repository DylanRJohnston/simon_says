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
    // You need to own a Box<dyn FnOnce> to be able to call it, Bevy ECS normally only lets us borrow data
    // and calling FnOnce requires ownership
    // Using Option lets us take ownership of Box<dyn FnOnce> from the ECS with Option::take
    pub command: Option<Box<dyn FnOnce(&mut Commands) + Send + Sync + 'static>>,
    pub delay: Timer,
}

impl DelayedCommand {
    pub fn new(secs: f32, command: impl FnOnce(&mut Commands) + Send + Sync + 'static) -> Self {
        Self {
            command: Some(Box::new(command)),
            delay: Timer::new(Duration::from_secs_f32(secs), TimerMode::Once),
        }
    }
}

fn run_delayed_commands(
    mut commands: Commands,
    mut delayed_commands: Query<(Entity, &mut DelayedCommand)>,
    time: Res<Time>,
) {
    for (entity, mut delayed_command) in &mut delayed_commands {
        if !delayed_command.delay.tick(time.delta()).just_finished() {
            continue;
        }

        let Some(delayed_command) = delayed_command.command.take() else {
            continue;
        };

        (delayed_command)(&mut commands);
        commands.entity(entity).despawn_recursive();
    }
}

pub trait DelayedCommandExt {
    fn delayed(
        &mut self,
        secs: f32,
        command: impl FnOnce(&mut Commands) + Send + Sync + 'static,
    ) -> EntityCommands<'_>;
}

impl DelayedCommandExt for Commands<'_, '_> {
    fn delayed(
        &mut self,
        secs: f32,
        command: impl FnOnce(&mut Commands) + Send + Sync + 'static,
    ) -> EntityCommands<'_> {
        self.spawn(DelayedCommand::new(secs, command))
    }
}
