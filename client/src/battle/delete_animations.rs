use super::{Artifact, BattleCallback, BattleSimulation, Component, Entity, Living};
use crate::bindable::EntityId;
use crate::resources::Globals;
use framework::prelude::GameIO;

/// requires Living component
pub fn delete_player_animation(game_io: &GameIO, simulation: &mut BattleSimulation, id: EntityId) {
    let sfx = &game_io.resource::<Globals>().unwrap().sfx;
    simulation.play_sound(game_io, &sfx.player_deleted);

    let (entity, living) = simulation
        .entities
        .query_one_mut::<(&mut Entity, &Living)>(id.into())
        .unwrap();

    let x = entity.x;
    let y = entity.y;

    // flinch
    let player_animator = &mut simulation.animators[entity.animator_index];
    let callbacks = player_animator.set_state(living.flinch_anim_state.as_ref().unwrap());
    simulation.pending_callbacks.extend(callbacks);

    let player_root_node = entity.sprite_tree.root_mut();
    player_animator.apply(player_root_node);

    player_animator.disable();

    // create transformation shine artifact
    Component::create_player_deletion(simulation, id);

    let artifact_id = Artifact::create_transformation_shine(game_io, simulation);
    let artifact_entity = simulation
        .entities
        .query_one_mut::<&mut Entity>(artifact_id.into())
        .unwrap();

    artifact_entity.x = x;
    artifact_entity.y = y;
    artifact_entity.pending_spawn = true;

    let animator = &mut simulation.animators[artifact_entity.animator_index];

    animator.on_complete(BattleCallback::new(
        move |game_io, resources, simulation, _| {
            simulation.mark_entity_for_erasure(game_io, resources, artifact_id);
        },
    ));
}

pub fn delete_character_animation(
    simulation: &mut BattleSimulation,
    id: EntityId,
    explosion_count: Option<usize>,
) {
    let explosion_count = explosion_count.unwrap_or(2);
    let entity = simulation
        .entities
        .query_one_mut::<&Entity>(id.into())
        .unwrap();

    simulation.animators[entity.animator_index].disable();

    Component::create_character_deletion(simulation, id, explosion_count);
}
