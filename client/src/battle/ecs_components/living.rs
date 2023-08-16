use crate::battle::*;
use crate::bindable::*;
use crate::resources::*;
use framework::prelude::*;
use std::collections::HashMap;

#[derive(Clone)]
pub struct Living {
    pub hit: bool, // used for flashing white
    pub hitbox_enabled: bool,
    pub counterable: bool,
    pub health: i32,
    pub max_health: i32,
    pub intangibility: Intangibility,
    pub defense_rules: Vec<DefenseRule>,
    pub flinch_anim_state: Option<String>,
    pub status_director: StatusDirector,
    pub status_callbacks: HashMap<HitFlags, Vec<BattleCallback>>,
    pub hit_callbacks: Vec<BattleCallback<HitProperties>>,
    pub countered_callback: BattleCallback,
}

impl Default for Living {
    fn default() -> Self {
        Self {
            hit: false,
            hitbox_enabled: true,
            counterable: false,
            health: 0,
            max_health: 0,
            intangibility: Intangibility::default(),
            defense_rules: Vec::new(),
            flinch_anim_state: None,
            status_director: StatusDirector::default(),
            status_callbacks: HashMap::new(),
            hit_callbacks: Vec::new(),
            countered_callback: BattleCallback::default(),
        }
    }
}

impl Living {
    pub fn set_health(&mut self, mut health: i32) {
        health = health.max(0);

        if self.max_health == 0 {
            self.max_health = health;
        }

        self.health = health.min(self.max_health);
    }

    pub fn register_status_callback(&mut self, hit_flag: HitFlags, callback: BattleCallback) {
        if let Some(callbacks) = self.status_callbacks.get_mut(&hit_flag) {
            callbacks.push(callback);
        } else {
            self.status_callbacks.insert(hit_flag, vec![callback]);
        }
    }

    pub fn register_hit_callback(&mut self, callback: BattleCallback<HitProperties>) {
        self.hit_callbacks.push(callback);
    }

    pub fn process_hit(
        game_io: &GameIO,
        resources: &SharedBattleResources,
        simulation: &mut BattleSimulation,
        entity_id: EntityId,
        mut hit_props: HitProperties,
    ) {
        let entities = &mut simulation.entities;
        let Ok((entity, living)) =
            entities.query_one_mut::<(&Entity, &mut Living)>(entity_id.into())
        else {
            return;
        };

        let time_is_frozen = entity.time_frozen_count > 0;
        let tile_pos = (entity.x, entity.y);

        let defense_rules = living.defense_rules.clone();

        // filter statuses through defense rules
        DefenseJudge::filter_statuses(
            game_io,
            resources,
            simulation,
            &mut hit_props,
            &defense_rules,
        );

        if time_is_frozen {
            hit_props.flags |= HitFlag::SHAKE;
            hit_props.context.flags |= HitFlag::NO_COUNTER;
        }

        let entities = &mut simulation.entities;
        let entity = entities.query_one_mut::<&Entity>(entity_id.into()).unwrap();

        let original_damage = hit_props.damage;

        // super effective bonus
        if hit_props.is_super_effective(entity.element) {
            hit_props.damage += original_damage;
        }

        // tile bonus
        let tile = simulation.field.tile_at_mut(tile_pos).unwrap();
        let tile_state = &simulation.tile_states[tile.state_index()];
        let bonus_damage_callback = tile_state.calculate_bonus_damage_callback.clone();

        hit_props.damage += bonus_damage_callback.call(
            game_io,
            resources,
            simulation,
            (hit_props.clone(), original_damage),
        );

        let entities = &mut simulation.entities;
        let Ok((entity, living)) =
            entities.query_one_mut::<(&Entity, &mut Living)>(entity_id.into())
        else {
            return;
        };

        // apply damage
        living.set_health(living.health - hit_props.damage);

        if hit_props.flags & HitFlag::IMPACT != 0 {
            // used for flashing white
            living.hit = true
        }

        let status_registry = &resources.status_registry;

        // handle counter
        if living.counterable
            && !living.status_director.is_inactionable(status_registry)
            && (hit_props.flags & HitFlag::IMPACT) == HitFlag::IMPACT
            && (hit_props.context.flags & HitFlag::NO_COUNTER) == 0
        {
            living.status_director.apply_status(HitFlag::PARALYZE, 150);
            living.counterable = false;

            // notify self
            let self_callback = living.countered_callback.clone();
            simulation.pending_callbacks.push(self_callback);

            // notify aggressor
            let aggressor_id = hit_props.context.aggressor;

            let notify_aggressor =
                BattleCallback::new(move |game_io, resources, simulation, ()| {
                    let entities = &mut simulation.entities;
                    let Ok(aggressor_entity) =
                        entities.query_one_mut::<&Entity>(aggressor_id.into())
                    else {
                        return;
                    };

                    let callback = aggressor_entity.counter_callback.clone();
                    callback.call(game_io, resources, simulation, entity_id);

                    // play counter sfx if the attack was caused by the local player
                    if simulation.local_player_id == aggressor_id {
                        let globals = game_io.resource::<Globals>().unwrap();
                        simulation.play_sound(game_io, &globals.sfx.counter_hit);
                    }
                });

            simulation.pending_callbacks.push(notify_aggressor);
        }

        // apply statuses
        let status_director = &mut living.status_director;
        status_director.apply_hit_flags(status_registry, hit_props.flags);

        // store callbacks
        let hit_callbacks = living.hit_callbacks.clone();

        // handle drag
        if hit_props.drags() && entity.movement.is_none() {
            let can_move_to_callback = entity.can_move_to_callback.clone();
            let delta: IVec2 = hit_props.drag.direction.i32_vector().into();

            let mut dest = IVec2::new(entity.x, entity.y);
            let mut duration = 0;

            for _ in 0..hit_props.drag.count {
                dest += delta;

                let tile_exists = simulation.field.tile_at_mut(dest.into()).is_some();

                if !tile_exists
                    || !can_move_to_callback.call(game_io, resources, simulation, dest.into())
                {
                    dest -= delta;
                    break;
                }

                duration += DRAG_PER_TILE_DURATION;
            }

            if duration != 0 {
                let entity = (simulation.entities)
                    .query_one_mut::<&mut Entity>(entity_id.into())
                    .unwrap();

                entity.movement = Some(Movement::slide(dest.into(), duration));
            }
        }

        for callback in hit_callbacks {
            callback.call(game_io, resources, simulation, hit_props.clone());
        }

        simulation.call_pending_callbacks(game_io, resources);
    }
}
