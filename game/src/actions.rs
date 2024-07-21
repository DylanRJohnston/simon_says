use bevy::prelude::*;

pub struct ActionPlugin;

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ActionPlan::default())
            .observe(add_action)
            .observe(remove_action);
    }
}

#[derive(Debug, Clone, Copy, Component)]
pub enum Action {
    Forward,
    Backward,
    Left,
    Right,
}

impl From<Action> for String {
    fn from(value: Action) -> Self {
        match value {
            Action::Forward => "Forward".into(),
            Action::Backward => "Backward".into(),
            Action::Left => "Left".into(),
            Action::Right => "Right".into(),
        }
    }
}

#[derive(Debug, Clone, Event, Deref)]
pub struct AddAction(pub Action);

#[derive(Debug, Clone, Copy, Event, Deref)]
pub struct RemoveAction(pub usize);

#[derive(Debug, Default, Resource, Deref, DerefMut)]
pub struct ActionPlan(pub Vec<Action>);

fn add_action(trigger: Trigger<AddAction>, mut action_plan: ResMut<ActionPlan>) {
    action_plan.push(**trigger.event())
}

fn remove_action(trigger: Trigger<RemoveAction>, mut action_plan: ResMut<ActionPlan>) {
    let index = **trigger.event();

    if index > action_plan.len() {
        tracing::warn!(
            "attempted to remove action from invalid index: {index}, bounds: [0, {})",
            action_plan.len()
        );
        return;
    }

    action_plan.remove(index);
}
