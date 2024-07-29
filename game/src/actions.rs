use bevy::prelude::*;

use crate::level::LoadNextLevel;

pub struct ActionPlugin;

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ActionPlan::default())
            .observe(add_action)
            .observe(remove_action)
            .observe(reset_action_plan)
            .observe(reset_action_plan_on_level_load);
    }
}

#[derive(Debug, Clone, Copy, Event, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Action {
    Forward,
    Right,
    Backward,
    Left,
    Nothing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CWRotation {
    Zero,
    Ninety,
    OneEighty,
    TwoSeventy,
}

impl CWRotation {
    pub fn to_combinator(self) -> fn(&Action) -> Action {
        match self {
            CWRotation::Zero => |action| *action,
            CWRotation::Ninety => Action::rotate_cw,
            CWRotation::OneEighty => Action::rotate_180,
            CWRotation::TwoSeventy => Action::rotate_ccw,
        }
    }
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Forward => write!(f, "↑"),
            Action::Right => write!(f, "→"),
            Action::Backward => write!(f, "↓"),
            Action::Left => write!(f, "←"),
            Action::Nothing => write!(f, "N/A"),
        }
    }
}

impl Action {
    pub fn rotate_cw(&self) -> Self {
        match self {
            Action::Forward => Action::Right,
            Action::Right => Action::Backward,
            Action::Backward => Action::Left,
            Action::Left => Action::Forward,
            Action::Nothing => Action::Nothing,
        }
    }

    pub fn rotate_ccw(&self) -> Self {
        match self {
            Action::Forward => Action::Left,
            Action::Right => Action::Forward,
            Action::Backward => Action::Right,
            Action::Left => Action::Backward,
            Action::Nothing => Action::Nothing,
        }
    }

    pub fn rotate_180(&self) -> Self {
        match self {
            Action::Forward => Action::Backward,
            Action::Right => Action::Left,
            Action::Backward => Action::Forward,
            Action::Left => Action::Right,
            Action::Nothing => Action::Nothing,
        }
    }

    pub fn cw_rotation(&self, target: Action) -> CWRotation {
        match (self, target) {
            (Action::Forward, Action::Forward) => CWRotation::Zero,
            (Action::Forward, Action::Right) => CWRotation::Ninety,
            (Action::Forward, Action::Backward) => CWRotation::OneEighty,
            (Action::Forward, Action::Left) => CWRotation::TwoSeventy,
            (Action::Right, Action::Forward) => CWRotation::TwoSeventy,
            (Action::Right, Action::Right) => CWRotation::Zero,
            (Action::Right, Action::Backward) => CWRotation::Ninety,
            (Action::Right, Action::Left) => CWRotation::OneEighty,
            (Action::Backward, Action::Forward) => CWRotation::OneEighty,
            (Action::Backward, Action::Right) => CWRotation::TwoSeventy,
            (Action::Backward, Action::Backward) => CWRotation::Zero,
            (Action::Backward, Action::Left) => CWRotation::Ninety,
            (Action::Left, Action::Forward) => CWRotation::Ninety,
            (Action::Left, Action::Right) => CWRotation::OneEighty,
            (Action::Left, Action::Backward) => CWRotation::TwoSeventy,
            (Action::Left, Action::Left) => CWRotation::Zero,
            (_, _) => CWRotation::Zero,
        }
    }
}

impl From<Action> for String {
    fn from(value: Action) -> Self {
        match value {
            Action::Forward => "Forward".into(),
            Action::Backward => "Backward".into(),
            Action::Left => "Left".into(),
            Action::Right => "Right".into(),
            Action::Nothing => "N/A".into(),
        }
    }
}

#[derive(Debug, Clone, Event, Deref)]
pub struct AddAction(pub Action);

#[derive(Debug, Clone, Copy, Event, Deref)]
pub struct RemoveAction(pub usize);

#[derive(
    Debug, Default, Clone, Resource, Deref, DerefMut, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
pub struct ActionPlan(pub Vec<Action>);

impl ActionPlan {
    pub fn phase_iter(&self) -> impl Iterator<Item = Vec<Action>> {
        let actions = self.clone().0;

        (1..=actions.len()).map(move |i| {
            let mut phase = actions.clone();
            phase.rotate_right(i);
            phase
        })
    }

    pub fn canonicalize_phase(&self) -> Self {
        Self(self.phase_iter().min().unwrap())
    }

    pub fn mirror(&self) -> Self {
        ActionPlan(
            self.iter()
                .map(|action| match action {
                    action @ (Action::Forward | Action::Backward | Action::Nothing) => *action,
                    Action::Left => Action::Right,
                    Action::Right => Action::Left,
                })
                .collect(),
        )
    }

    pub fn canonicalize_mirror(&self) -> Self {
        let mirror = self.mirror();

        if self < &mirror {
            self.clone()
        } else {
            mirror
        }
    }

    pub fn canonicalize_rotation(&self) -> Self {
        if self.is_empty() {
            return Self::default();
        }

        let rotate = self[0].cw_rotation(Action::Forward).to_combinator();

        Self(self.iter().map(rotate).collect())
    }

    pub fn canonicalize(&self) -> Self {
        self.canonicalize_rotation()
            .canonicalize_mirror()
            .canonicalize_phase()
    }
}

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

#[derive(Debug, Clone, Copy, Event)]
pub struct ResetActionPlan;

fn reset_action_plan(_trigger: Trigger<ResetActionPlan>, mut action_plan: ResMut<ActionPlan>) {
    action_plan.clear();
}

fn reset_action_plan_on_level_load(_trigger: Trigger<LoadNextLevel>, mut commands: Commands) {
    commands.trigger(ResetActionPlan);
}
