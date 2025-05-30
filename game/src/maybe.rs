use bevy::{
    ecs::{
        bundle::{Bundle, BundleEffect, DynamicBundle},
        component::{
            ComponentId, Components, ComponentsRegistrator, RequiredComponents, StorageType,
        },
        world::EntityWorldMut,
    },
    ptr::OwningPtr,
};

pub struct Maybe<B>(pub Option<B>);

unsafe impl<B: Bundle> Bundle for Maybe<B> {
    fn get_component_ids(components: &Components, ids: &mut impl FnMut(Option<ComponentId>)) {
        <() as Bundle>::get_component_ids(components, ids);
    }

    fn register_required_components(
        components: &mut ComponentsRegistrator,
        required_components: &mut RequiredComponents,
    ) {
        <() as Bundle>::register_required_components(components, required_components);
    }

    fn component_ids(components: &mut ComponentsRegistrator, ids: &mut impl FnMut(ComponentId)) {
        <() as Bundle>::component_ids(components, ids);
    }
}

impl<B: Bundle> DynamicBundle for Maybe<B> {
    type Effect = Self;

    fn get_components(self, func: &mut impl FnMut(StorageType, OwningPtr<'_>)) -> Self::Effect {
        <() as DynamicBundle>::get_components((), func);
        self
    }
}

impl<B: Bundle> BundleEffect for Maybe<B> {
    fn apply(self, entity: &mut EntityWorldMut) {
        let Maybe(Some(bundle)) = self else {
            return;
        };

        entity.insert(bundle);
    }
}

pub trait MaybeBundleExt<B> {
    fn into_bundle(self) -> Maybe<B>;
}

impl<B: Bundle> MaybeBundleExt<B> for Option<B> {
    fn into_bundle(self) -> Maybe<B> {
        Maybe(self)
    }
}
