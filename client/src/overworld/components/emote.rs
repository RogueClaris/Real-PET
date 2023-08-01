use super::{ActorAttachment, Animator, AttachmentLayer};
use crate::overworld::OverworldArea;
use crate::render::{AnimatorLoopMode, FrameTime};
use framework::prelude::*;

const MAX_LIFETIME: FrameTime = 60 * 5;

pub struct Emote {
    pub lifetime: FrameTime,
}

impl Emote {
    pub fn spawn_or_recycle(area: &mut OverworldArea, parent_entity: hecs::Entity, emote_id: &str) {
        let entities = &mut area.entities;

        for (_, (emote, attachment, animator)) in
            entities.query_mut::<(&mut Emote, &ActorAttachment, &mut Animator)>()
        {
            if attachment.actor_entity != parent_entity {
                continue;
            }

            // update existing emote
            emote.lifetime = 0;
            animator.set_state(emote_id);
            animator.set_loop_mode(AnimatorLoopMode::Loop);
            return;
        }

        Self::spawn(area, parent_entity, emote_id);
    }

    pub fn spawn(area: &mut OverworldArea, parent_entity: hecs::Entity, emote_id: &str) {
        let entities = &mut area.entities;

        let mut animator = area.emote_animator.clone();
        animator.set_state(emote_id);
        animator.set_loop_mode(AnimatorLoopMode::Loop);

        let sprite = area.emote_sprite.clone();
        let offset = Self::resolve_offset(entities, parent_entity).unwrap_or_else(|| {
            let Ok(parent_animator) = entities.query_one_mut::<&Animator>(parent_entity) else {
                return Vec2::ZERO;
            };

            Vec2::new(0.0, -parent_animator.origin().y)
        });
        let attachment_layer = AttachmentLayer(-1);
        let attachment = ActorAttachment {
            actor_entity: parent_entity,
            point: None,
        };

        area.entities.spawn((
            Emote { lifetime: 0 },
            animator,
            sprite,
            offset,
            attachment_layer,
            attachment,
            Vec3::ZERO,
        ));
    }

    pub fn system(area: &mut OverworldArea) {
        // handle timers
        let mut pending_deletion = Vec::new();

        let entities = &mut area.entities;

        for (entity, emote) in entities.query_mut::<&mut Emote>() {
            emote.lifetime += 1;

            if emote.lifetime > MAX_LIFETIME {
                pending_deletion.push(entity);
            }
        }

        for entity in pending_deletion {
            let _ = entities.despawn(entity);
        }

        // move to "EMOTE" point
        let mut emote_query = entities.query::<(&Emote, &ActorAttachment, &mut Vec2)>();
        for (_, (_, attachment, offset)) in emote_query.into_iter() {
            if let Some(new_offset) = Self::resolve_offset(entities, attachment.actor_entity) {
                *offset = new_offset;
            }
        }
    }

    fn resolve_offset(entities: &hecs::World, parent_entity: hecs::Entity) -> Option<Vec2> {
        let Ok(mut parent_query) = entities.query_one::<&Animator>(parent_entity) else {
            return None;
        };

        let Some(parent_animator) = parent_query.get() else {
            return None;
        };

        parent_animator
            .point("EMOTE")
            .map(|point| point - parent_animator.origin())
    }
}
