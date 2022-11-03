use super::BattleCallback;
use crate::bindable::AnimatorPlaybackMode;
use crate::render::{Animator, AnimatorLoopMode, DerivedFrame, SpriteNode};
use crate::resources::Globals;
use framework::prelude::{GameIO, Vec2};
use std::collections::HashMap;

#[derive(Clone)]
pub struct BattleAnimator {
    complete_callbacks: Vec<BattleCallback>,
    interrupt_callbacks: Vec<BattleCallback>,
    frame_callbacks: HashMap<usize, Vec<BattleCallback>>,
    animator: Animator,
    enabled: bool,
}

impl BattleAnimator {
    pub fn new() -> Self {
        Self {
            complete_callbacks: Vec::new(),
            interrupt_callbacks: Vec::new(),
            frame_callbacks: HashMap::new(),
            animator: Animator::new(),
            enabled: true,
        }
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn load(&mut self, game_io: &GameIO<Globals>, path: &str) -> Vec<BattleCallback> {
        self.animator.load(&game_io.globals().assets, path);

        self.complete_callbacks.clear();
        self.frame_callbacks.clear();

        std::mem::take(&mut self.interrupt_callbacks)
    }

    #[must_use]
    pub fn copy_from(&mut self, other: &Self) -> Vec<BattleCallback> {
        self.animator.copy_from(&other.animator);

        if let Some(state) = self.current_state() {
            // activate interrupt callbacks and clear other listeners by resetting state
            // resets progress as well
            self.set_state(&state.to_string())
        } else {
            Vec::new()
        }
    }

    pub fn on_complete(&mut self, callback: BattleCallback) {
        self.complete_callbacks.push(callback);
    }

    pub fn on_interrupt(&mut self, callback: BattleCallback) {
        self.interrupt_callbacks.push(callback);
    }

    pub fn on_frame(&mut self, frame_index: usize, callback: BattleCallback) {
        if let Some(callbacks) = self.frame_callbacks.get_mut(&frame_index) {
            callbacks.push(callback);
        } else {
            self.frame_callbacks.insert(frame_index, vec![callback]);
        }
    }

    pub fn set_loop_mode(&mut self, mode: AnimatorLoopMode) {
        self.animator.set_loop_mode(mode)
    }

    pub fn set_playback_mode(&mut self, mode: AnimatorPlaybackMode) {
        match mode {
            AnimatorPlaybackMode::Once => {
                self.animator.set_loop_mode(AnimatorLoopMode::Once);
                self.animator.set_reversed(false);
            }
            AnimatorPlaybackMode::Loop => {
                self.animator.set_loop_mode(AnimatorLoopMode::Loop);
                self.animator.set_reversed(false);
            }
            AnimatorPlaybackMode::Bounce => {
                self.animator.set_loop_mode(AnimatorLoopMode::Bounce);
                self.animator.set_reversed(false);
            }
            AnimatorPlaybackMode::Reverse => {
                self.animator.set_loop_mode(AnimatorLoopMode::Once);
                self.animator.set_reversed(true);
            }
        }
    }

    pub fn set_reversed(&mut self, reversed: bool) {
        self.animator.set_reversed(reversed)
    }

    pub fn origin(&self) -> Vec2 {
        self.animator.origin()
    }

    pub fn point(&self, name: &str) -> Option<Vec2> {
        self.animator.point(name)
    }

    pub fn is_complete(&self) -> bool {
        self.animator.is_complete()
    }

    pub fn current_state(&self) -> Option<&str> {
        self.animator.current_state()
    }

    pub fn has_state(&self, state: &str) -> bool {
        self.animator.has_state(state)
    }

    #[must_use]
    pub fn set_state(&mut self, state: &str) -> Vec<BattleCallback> {
        self.animator.set_state(state);

        self.complete_callbacks.clear();
        self.frame_callbacks.clear();

        std::mem::take(&mut self.interrupt_callbacks)
    }

    #[must_use]
    pub fn update(&mut self) -> Vec<BattleCallback> {
        let mut pending_callbacks = Vec::new();

        if !self.enabled || self.animator.is_complete() {
            return pending_callbacks;
        }

        let previous_frame = self.animator.current_frame_index();
        let previous_loop_count = self.animator.loop_count();

        self.animator.update();

        let current_frame = self.animator.current_frame_index();

        if previous_frame != current_frame {
            if let Some(callbacks) = self.frame_callbacks.get(&current_frame) {
                pending_callbacks.extend(callbacks.iter().cloned());
            }
        }

        if self.animator.is_complete() || previous_loop_count != self.animator.loop_count() {
            pending_callbacks.extend(self.complete_callbacks.clone());
        }

        if self.animator.is_complete() {
            self.interrupt_callbacks.clear();
        }

        pending_callbacks
    }

    pub fn apply(&self, sprite_node: &mut SpriteNode) {
        sprite_node.apply_animation(&self.animator);
    }

    pub fn derive_state(
        &mut self,
        original_state: &str,
        frame_derivation: Vec<DerivedFrame>,
    ) -> String {
        let new_state = Animator::generate_state_id(original_state);

        self.animator
            .derive_state(&new_state, original_state, frame_derivation);

        new_state
    }
}