use super::{
    Action, BattleAnimator, BattleSimulation, Entity, Living, SharedBattleResources, StatusDirector,
};
use crate::bindable::{EntityId, Movement};
use crate::structures::GenerationalIndex;
use framework::prelude::GameIO;

#[derive(Clone)]
pub struct TimeFreezeEntityBackup {
    entity_id: EntityId,
    action_index: Option<GenerationalIndex>,
    movement: Option<Movement>,
    animator: BattleAnimator,
    status_director: Option<StatusDirector>,
}

impl TimeFreezeEntityBackup {
    pub fn backup_and_prepare(
        simulation: &mut BattleSimulation,
        entity_id: EntityId,
        action_index: GenerationalIndex,
    ) -> Option<Self> {
        let entities = &mut simulation.entities;
        let Ok((entity, living)) =
            entities.query_one_mut::<(&mut Entity, Option<&mut Living>)>(entity_id.into())
        else {
            return None;
        };

        entity.time_frozen = false;

        // swap action index
        let old_action_index = entity.action_index;
        entity.action_index = Some(action_index);

        // back up movement
        let movement = std::mem::take(&mut entity.movement);

        // back up animator
        let animator = &mut simulation.animators[entity.animator_index];
        let animator_backup = animator.clone();

        // reset callbacks as they're already stored in the original animator
        animator.clear_callbacks();

        // backup status_director
        let status_director = living.map(|living| {
            // delete old status sprites
            for (_, index) in living.status_director.take_status_sprites() {
                entity.sprite_tree.remove(index);
            }

            std::mem::take(&mut living.status_director)
        });

        Some(Self {
            entity_id,
            action_index: old_action_index,
            movement,
            animator: animator_backup,
            status_director,
        })
    }

    pub fn restore(
        self,
        game_io: &GameIO,
        resources: &SharedBattleResources,
        simulation: &mut BattleSimulation,
    ) {
        // delete action if it still exists
        let entities = &mut simulation.entities;
        let Ok(entity) = entities.query_one_mut::<&mut Entity>(self.entity_id.into()) else {
            return;
        };

        if let Some(index) = entity.action_index {
            Action::delete_multi(game_io, resources, simulation, [index]);
        }

        // fully restore the entity
        let entities = &mut simulation.entities;
        let Ok((entity, living)) =
            entities.query_one_mut::<(&mut Entity, Option<&mut Living>)>(self.entity_id.into())
        else {
            return;
        };

        // freeze the entity again
        entity.time_frozen = true;

        // restore the action
        entity.action_index = self.action_index;

        // restore the movement
        entity.movement = self.movement;

        // restore animator
        simulation.animators[entity.animator_index] = self.animator;

        // restore statuses
        if let Some(living) = living {
            // merge to retain statuses applied during time freeze
            living.status_director.merge(self.status_director.unwrap());
        }
    }
}
