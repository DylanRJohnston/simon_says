use bevy::prelude::*;

use crate::player::LevelCompleted;

pub struct ActionPlugin;

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ActionPlan::default())
            .observe(add_action)
            .observe(remove_action)
            .observe(reset_action_plan);
    }
}

#[derive(Debug, Clone, Copy, Event, PartialEq, Eq, Hash)]
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

#[derive(Debug, Default, Clone, Resource, Deref, DerefMut, PartialEq, Eq, Hash)]
pub struct ActionPlan(pub Vec<Action>);

impl ActionPlan {
    pub fn isomorphic_under_rotation(&self, other: &ActionPlan) -> bool {
        if self.len() != other.len() {
            return false;
        }

        if self.is_empty() && other.is_empty() {
            return true;
        }

        let first = self[0].cw_rotation(other[0]);

        for (a, b) in self.iter().zip(other.iter()).skip(1) {
            if a.cw_rotation(*b) != first {
                return false;
            }
        }

        true
    }

    pub fn isomorphic_under_phase_shift(&self, other: &ActionPlan) -> bool {
        if self.len() != other.len() {
            return false;
        }

        if self.is_empty() && other.is_empty() {
            return true;
        }

        unimplemented!();
    }

    pub fn isomorphic(&self, other: &ActionPlan) -> bool {
        unimplemented!()
    }

    pub fn canonicalize(&mut self) {
        if self.is_empty() {
            return;
        }

        let first = self[0];
        let rotate = first.cw_rotation(Action::Forward).to_combinator();

        self.iter_mut().for_each(|action| *action = rotate(action));
    }
}

fn add_action(trigger: Trigger<AddAction>, mut action_plan: ResMut<ActionPlan>) {
    tracing::info!("reacting to add action");
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

fn reset_action_plan(_trigger: Trigger<LevelCompleted>, mut action_plan: ResMut<ActionPlan>) {
    action_plan.clear();
}

#[cfg(test)]
mod test {
    use bevy::utils::hashbrown::HashSet;
    use proptest::prelude::*;

    use crate::actions::{Action, ActionPlan};

    fn any_action() -> impl Strategy<Value = Action> {
        prop_oneof![
            Just(Action::Forward),
            Just(Action::Backward),
            Just(Action::Left),
            Just(Action::Right),
        ]
    }

    proptest! {

        #[test]
        fn isomorphic_under_rotation(
            mut action_plan in prop::collection::vec(any_action(), 0..10),
            rotation_amount in 0..4
        ) {
            let original = action_plan.clone();

            for _ in 0..=rotation_amount {
                action_plan.iter_mut().for_each(|action| *action = action.rotate_cw());
            }

            prop_assert!(ActionPlan(original).isomorphic_under_rotation(&ActionPlan(action_plan)));
        }

        #[ignore]
        #[test]
        fn isomorphic_under_phase_shift(
            action_plan in prop::collection::vec(any_action(), 0..10),
        ) {
            let mut action_plan = ActionPlan(action_plan);
            let original = action_plan.clone();

            for _ in 0..original.len() {
                action_plan.rotate_right(1);

                prop_assert!(original.isomorphic_under_phase_shift(&action_plan));
            }

        }
    }

    #[test]
    pub fn all_novel_action_plans_under_rotation() {
        let mut plans = HashSet::new();

        // All one action plans are rotationally isomorphic to Forward
        let action_plan = ActionPlan(vec![Action::Forward]);
        plans.insert(action_plan.clone());

        // All two actions plans are rotational / mirror isomorphic to (Forward, Backward) or (Forward, Right)
        for action in [Action::Backward, Action::Right] {
            let mut action_plan = action_plan.clone();
            action_plan.push(action);

            plans.insert(action_plan.clone());

            for action in [
                Action::Forward,
                Action::Right,
                Action::Backward,
                Action::Left,
            ] {
                let mut action_plan = action_plan.clone();
                action_plan.push(action);
                plans.insert(action_plan.clone());
            }
        }

        println!("Plan Count: {count}, {plans:?}", count = plans.len());

        // for outer_plan in plans.clone() {
        //     if
        // }
    }
}
