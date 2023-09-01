use crate::bindable::SpriteColorMode;
use crate::packages::{PackageId, PackageNamespace};
use crate::render::ui::{
    FontStyle, SceneTitle, ScrollTracker, SubSceneFrame, Text, TextStyle, Textbox, TextboxQuestion,
    UiInputTracker,
};
use crate::render::{Animator, AnimatorLoopMode, Background, Camera, FrameTime, SpriteColorQueue};
use crate::resources::*;
use crate::saves::InstalledSwitchDrive;
use framework::prelude::*;
use packets::structures::SwitchDriveSlot;
use std::borrow::Cow;
use std::collections::HashSet;

enum Event {
    Leave,
}

#[derive(Clone, Copy)]
enum SwitchDriveOption {
    Add,
    Remove,
}

#[derive(Clone)]
struct CompactDrivePackageInfo {
    id: PackageId,
    name: String,
    slot: Option<SwitchDriveSlot>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    ListSelection,
}

pub struct ManageSwitchDriveScene {
    camera: Camera,
    background: Background,
    frame: SubSceneFrame,
    animator: Animator,
    information_box_sprite: Sprite,
    information_text: Text,
    input_tracker: UiInputTracker,
    scroll_tracker: ScrollTracker,
    slot: Option<SwitchDriveSlot>,
    tracked_invalid: HashSet<(Cow<'static, PackageId>, Option<SwitchDriveSlot>)>,
    state: State,
    textbox: Textbox,
    time: FrameTime,
    packages: Vec<CompactDrivePackageInfo>,
    event_sender: flume::Sender<Event>,
    event_receiver: flume::Receiver<Event>,
    next_scene: NextScene,
}

impl ManageSwitchDriveScene {
    pub fn new(game_io: &GameIO) -> Self {
        let globals = game_io.resource::<Globals>().unwrap();
        let assets = &globals.assets;

        let restrictions = &globals.restrictions;

        // load drive part packages
        let mut packages: Vec<_> = globals
            .augment_packages
            .packages_with_override(PackageNamespace::Local)
            .filter(|package| package.slot.is_some())
            .filter(|package| {
                restrictions.validate_package_tree(game_io, package.package_info.triplet())
            })
            .map(|package| CompactDrivePackageInfo {
                id: package.package_info.id.clone(),
                name: package.name.clone(),
                slot: package.slot,
            })
            .collect();

        packages.sort_by(|a, b| a.id.cmp(&b.id));

        let mut animator = Animator::load_new(assets, ResourcePaths::CUSTOMIZE_UI_ANIMATION);

        let mut information_box_sprite = assets.new_sprite(game_io, ResourcePaths::CUSTOMIZE_UI);
        animator.set_state("TEXTBOX");
        animator.apply(&mut information_box_sprite);

        let information_bounds = Rect::from_corners(
            animator.point("TEXT_START").unwrap_or_default(),
            animator.point("TEXT_END").unwrap_or_default(),
        ) - animator.origin();

        let information_text = Text::new(game_io, FontStyle::Thin)
            .with_bounds(information_bounds)
            .with_shadow_color(TEXT_DARK_SHADOW_COLOR);

        // scroll tracker
        let scroll_tracker = ScrollTracker::new(game_io, 4)
            .with_view_margin(1)
            .with_total_items(packages.len())
            .with_custom_cursor(
                game_io,
                ResourcePaths::TEXTBOX_CURSOR_ANIMATION,
                ResourcePaths::TEXTBOX_CURSOR,
            );

        let (event_sender, event_receiver) = flume::unbounded();

        Self {
            camera: Camera::new_ui(game_io),
            background: Background::new_character_scene(game_io),
            frame: SubSceneFrame::new(game_io).with_everything(true),
            animator,
            information_box_sprite,
            information_text,
            input_tracker: UiInputTracker::new(),
            scroll_tracker,
            slot: Option::Some(SwitchDriveSlot::Head),
            tracked_invalid: HashSet::new(),
            state: State::ListSelection,
            textbox: Textbox::new_navigation(game_io).with_position(RESOLUTION_F * 0.5),
            time: 0,
            packages,
            event_sender,
            event_receiver,
            next_scene: NextScene::None,
        }
    }

    /// Failed placement will provide a list of overlapping positions
    fn add_drive_part(
        &mut self,
        game_io: &mut GameIO,
        info: CompactDrivePackageInfo,
    ) -> Option<Vec<(usize, usize)>> {
        // TODO: validate the drive slot
        // let mut conflicts = Vec::new();

        // if !conflicts.is_empty() {
        //     return Some(conflicts);
        // }

        // actual placement
        // save drive parts
        let globals = game_io.resource_mut::<Globals>().unwrap();
        let global_save = &mut globals.global_save;

        let installed_drive = InstalledSwitchDrive {
            package_id: info.id,
            slot: info.slot.unwrap(),
        };

        global_save
            .installed_drive_parts
            .insert(global_save.selected_character.clone(), installed_drive);

        global_save.save();

        None
    }

    fn handle_input(&mut self, game_io: &mut GameIO) {
        let globals = game_io.resource::<Globals>().unwrap();

        let index = self.scroll_tracker.selected_index();

        if let Some(part) = self.packages.get(index) {
            if self.input_tracker.is_active(Input::Confirm) {
                let clone = part.clone();

                let success = self.add_drive_part(game_io, clone).is_none();

                let globals = game_io.resource::<Globals>().unwrap();

                if success {
                    globals.audio.play_sound(&globals.sfx.cursor_select);
                } else {
                    globals.audio.play_sound(&globals.sfx.cursor_error);
                }
            } else if self.input_tracker.is_active(Input::Cancel) {
                // self.uninstall(game_io, part);

                self.state = State::ListSelection;

                globals.audio.play_sound(&globals.sfx.cursor_cancel);
            }
        }
        match self.state {
            State::ListSelection => {
                if self.input_tracker.is_active(Input::Cancel) {
                    let event_sender = self.event_sender.clone();

                    let question = TextboxQuestion::new(
                        String::from("Quit customizing and return to menu?"),
                        move |yes| {
                            if yes {
                                event_sender.send(Event::Leave).unwrap();
                            }
                        },
                    );

                    self.textbox.push_interface(question);
                    self.textbox.open();
                } else if self.input_tracker.is_active(Input::Left) {
                    let y = self.scroll_tracker.selected_index() - self.scroll_tracker.top_index();

                    let globals = game_io.resource::<Globals>().unwrap();
                    globals.audio.play_sound(&globals.sfx.cursor_select);
                } else {
                    // handle scrolling
                    let prev_index = self.scroll_tracker.selected_index();

                    self.scroll_tracker
                        .handle_vertical_input(&self.input_tracker);

                    if self.input_tracker.is_active(Input::End) {
                        let last_index = self.scroll_tracker.total_items() - 1;
                        self.scroll_tracker.set_selected_index(last_index);
                    }

                    // sfx
                    let selected_index = self.scroll_tracker.selected_index();

                    if prev_index != selected_index {
                        let globals = game_io.resource::<Globals>().unwrap();
                        globals.audio.play_sound(&globals.sfx.cursor_move);

                        // self.update_text(game_io);
                    }

                    // handle confirm
                    if self.input_tracker.is_active(Input::Confirm)
                        && self.packages.get(selected_index).is_some()
                    {
                        // selected a part
                        let package = self.packages.remove(selected_index);
                        self.scroll_tracker.set_total_items(self.packages.len());
                        let globals = game_io.resource::<Globals>().unwrap();

                        globals.audio.play_sound(&globals.sfx.cursor_select);
                    }
                }
            }
        }
    }

    fn handle_events(&mut self, game_io: &GameIO) {
        while let Ok(event) = self.event_receiver.try_recv() {
            match event {
                Event::Leave => {
                    let transition = crate::transitions::new_sub_scene_pop(game_io);
                    self.next_scene = NextScene::new_pop().with_transition(transition);
                }
            }
        }
    }
}

impl Scene for ManageSwitchDriveScene {
    fn next_scene(&mut self) -> &mut NextScene {
        &mut self.next_scene
    }

    fn update(&mut self, game_io: &mut GameIO) {
        self.background.update();
        self.input_tracker.update(game_io);
        self.time += 1;

        self.textbox.update(game_io);

        if self.textbox.is_complete() {
            self.textbox.close();
        }

        if !game_io.is_in_transition() && !self.textbox.is_open() {
            self.handle_input(game_io);
        }

        self.handle_events(game_io);
    }

    fn draw(&mut self, game_io: &mut GameIO, render_pass: &mut RenderPass) {
        self.background.draw(game_io, render_pass);

        let mut sprite_queue =
            SpriteColorQueue::new(game_io, &self.camera, SpriteColorMode::Multiply);

        // draw package list
        let mut recycled_sprite =
            Sprite::new(game_io, self.information_box_sprite.texture().clone());

        let selected_index = if self.state == State::ListSelection {
            Some(self.scroll_tracker.selected_index())
        } else {
            None
        };

        self.animator.set_state("OPTION");
        let mut offset = self.animator.point("START").unwrap_or_default();
        let offset_jump = self.animator.point("STEP").unwrap_or_default();

        let text_bounds = Rect::from_corners(
            self.animator.point("TEXT_START").unwrap_or_default(),
            self.animator.point("TEXT_END").unwrap_or_default(),
        );
        let mut text_style = TextStyle::new(game_io, FontStyle::Thick)
            .with_shadow_color(TEXT_DARK_SHADOW_COLOR)
            .with_bounds(text_bounds);

        if self.scroll_tracker.top_index() == 0 {
            offset += offset_jump;
        }

        text_style.bounds += offset - offset_jump;

        for i in self.scroll_tracker.view_range() {
            recycled_sprite.set_position(offset);
            text_style.bounds += offset_jump;
            offset += offset_jump;

            // draw package option
            if Some(i) == selected_index {
                self.animator.set_state("OPTION_BLINK");
                self.animator.set_loop_mode(AnimatorLoopMode::Loop);
                self.animator.sync_time(self.time);
            } else {
                self.animator.set_state("OPTION");
            }

            self.animator.apply(&mut recycled_sprite);
            sprite_queue.draw_sprite(&recycled_sprite);

            let compact_info = &self.packages[i];
            text_style.draw(game_io, &mut sprite_queue, &compact_info.name)
        }

        // draw information text
        sprite_queue.draw_sprite(&self.information_box_sprite);
        self.information_text.draw(game_io, &mut sprite_queue);

        // draw frame
        self.frame.draw(&mut sprite_queue);
        SceneTitle::new("CUSTOMIZE").draw(game_io, &mut sprite_queue);

        // draw textbox
        self.textbox.draw(game_io, &mut sprite_queue);

        render_pass.consume_queue(sprite_queue);
    }
}
