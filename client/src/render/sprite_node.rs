use super::ui::{FontStyle, TextStyle};
use super::{Animator, SpriteColorQueue, SpriteShaderEffect};
use crate::bindable::SpriteColorMode;
use crate::resources::*;
use framework::prelude::*;
use smallvec::SmallVec;
use std::sync::Arc;

pub use crate::structures::{Tree, TreeIndex, TreeNode};

#[derive(Clone)]
pub struct SpriteNode {
    layer: i32,
    scale: Vec2,
    offset: Vec2,
    palette_path: String,
    palette: Option<Arc<Texture>>,
    texture_path: String,
    sprite: Sprite,
    color_mode: SpriteColorMode, // root node resets every frame
    using_parent_shader: bool,
    pixelate_with_alpha: bool,
    never_flip: bool,
    visible: bool,
    inherited_visible: bool,
}

impl SpriteNode {
    pub fn new(game_io: &GameIO, color_mode: SpriteColorMode) -> Self {
        let assets = &game_io.resource::<Globals>().unwrap().assets;

        let mut sprite = assets.new_sprite(game_io, ResourcePaths::BLANK);
        sprite.set_color(color_mode.default_color());

        Self {
            layer: 0,
            scale: Vec2::ONE,
            offset: Vec2::ZERO,
            palette_path: String::new(),
            palette: None,
            texture_path: ResourcePaths::BLANK.to_string(),
            sprite,
            color_mode,
            using_parent_shader: false,
            pixelate_with_alpha: false,
            never_flip: false,
            visible: true,
            inherited_visible: true,
        }
    }

    pub fn layer(&self) -> i32 {
        self.layer
    }

    /// Negative is closer, positive is farther
    pub fn set_layer(&mut self, layer: i32) {
        self.layer = layer;
    }

    pub fn offset(&self) -> Vec2 {
        self.offset
    }

    pub fn set_offset(&mut self, offset: Vec2) {
        self.offset = offset;
    }

    pub fn origin(&self) -> Vec2 {
        self.sprite.origin()
    }

    pub fn set_origin(&mut self, origin: Vec2) {
        self.sprite.set_origin(origin);
    }

    pub fn apply_animation(&mut self, animator: &Animator) {
        animator.apply(&mut self.sprite);
    }

    pub fn scale(&self) -> Vec2 {
        self.scale
    }

    pub fn set_scale(&mut self, scale: Vec2) {
        self.scale = scale;
    }

    pub fn size(&mut self) -> Vec2 {
        self.sprite.set_scale(self.scale);
        self.sprite.size()
    }

    pub fn set_size(&mut self, size: Vec2) {
        self.sprite.set_size(size);
        self.scale = self.sprite.scale();
    }

    pub fn visible(&self) -> bool {
        self.visible
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn inherited_visible(&self) -> bool {
        self.inherited_visible
    }

    pub fn never_flip(&self) -> bool {
        self.never_flip
    }

    pub fn set_never_flip(&mut self, never_flip: bool) {
        self.never_flip = never_flip;
    }

    pub fn using_parent_shader(&self) -> bool {
        self.using_parent_shader
    }

    pub fn set_using_parent_shader(&mut self, using_parent_shader: bool) {
        self.using_parent_shader = using_parent_shader;
    }

    pub fn color(&self) -> Color {
        self.sprite.color()
    }

    pub fn set_color(&mut self, color: Color) {
        self.sprite.set_color(color);
    }

    pub fn set_alpha(&mut self, alpha: f32) {
        let mut color = self.sprite.color();
        color.a = alpha;
        self.sprite.set_color(color);
    }

    pub fn color_mode(&self) -> SpriteColorMode {
        self.color_mode
    }

    pub fn set_color_mode(&mut self, mode: SpriteColorMode) {
        self.color_mode = mode;
    }

    pub fn pixelate_with_alpha(&self) -> bool {
        self.pixelate_with_alpha
    }

    pub fn set_pixelate_with_alpha(&mut self, enabled: bool) {
        self.pixelate_with_alpha = enabled;
    }

    pub fn palette_path(&self) -> Option<&str> {
        #[allow(clippy::question_mark)]
        if self.palette.is_none() {
            return None;
        }

        Some(&self.palette_path)
    }

    pub fn palette(&self) -> Option<&Arc<Texture>> {
        self.palette.as_ref()
    }

    pub fn set_palette(&mut self, game_io: &GameIO, path: Option<String>) {
        if let Some(path) = path {
            let assets = &game_io.resource::<Globals>().unwrap().assets;
            self.palette = Some(assets.texture(game_io, &path));
            self.palette_path = path;
        } else {
            self.palette = None;
            self.palette_path = String::new();
        }
    }

    pub fn texture(&self) -> &Arc<Texture> {
        self.sprite.texture()
    }

    pub fn texture_path(&self) -> &str {
        &self.texture_path
    }

    pub fn set_texture(&mut self, game_io: &GameIO, path: String) {
        let globals = game_io.resource::<Globals>().unwrap();

        self.sprite
            .set_texture(globals.assets.texture(game_io, &path));

        self.texture_path = path;
    }

    pub fn set_texture_direct(&mut self, texture: Arc<Texture>) {
        self.sprite.set_texture(texture);
        self.texture_path = String::new();
    }

    pub fn sprite(&self) -> &Sprite {
        &self.sprite
    }

    fn shader_effect(&self) -> SpriteShaderEffect {
        if self.palette.is_some() {
            if self.pixelate_with_alpha {
                SpriteShaderEffect::PixelatePalette
            } else {
                SpriteShaderEffect::Palette
            }
        } else if self.pixelate_with_alpha {
            SpriteShaderEffect::Pixelate
        } else {
            SpriteShaderEffect::Default
        }
    }
}

impl Tree<SpriteNode> {
    pub fn insert_text_child(
        &mut self,
        game_io: &GameIO,
        parent: TreeIndex,
        font_style: FontStyle,
        text: &str,
    ) -> Option<TreeIndex> {
        let text_node_index =
            self.insert_child(parent, SpriteNode::new(game_io, SpriteColorMode::Multiply))?;

        let globals = game_io.resource::<Globals>().unwrap();

        // add characters
        TextStyle::new(game_io, font_style).iterate(text, |frame, offset| {
            let mut char_node = SpriteNode::new(game_io, SpriteColorMode::Multiply);
            char_node.set_using_parent_shader(true);

            char_node.set_texture_direct(globals.font_texture.clone());
            frame.apply(&mut char_node.sprite);
            char_node.set_offset(offset);

            self.insert_child(text_node_index, char_node);
        });

        Some(text_node_index)
    }

    pub fn global_position(&self, mut index: TreeIndex) -> Vec2 {
        let mut position = Vec2::ZERO;

        if let Some(node) = self.get_node(index) {
            position += node.value().offset;

            if let Some(parent_index) = node.parent_index() {
                index = parent_index;
            } else {
                return position;
            }
        }

        while let Some(tree_node) = self.get_node(index) {
            let sprite_node = tree_node.value();
            position += sprite_node.offset();
            position += sprite_node.origin();

            if let Some(parent_index) = tree_node.parent_index() {
                index = parent_index;
            } else {
                break;
            }
        }

        position
    }

    /// Inherit position into sprite + visibility into inherited_visible, adapts scale + adjust for perspective
    fn inherit_from_parent(&mut self, root_offset: Vec2, flipped: bool) {
        struct InheritedProperties {
            offset: Vec2,
            visible: bool,
            flipped: bool,
            scale: Vec2,
        }

        let mut initial_scale = Vec2::ONE;

        if flipped {
            initial_scale.x = -1.0;
        }

        self.inherit(
            self.root_index(),
            InheritedProperties {
                offset: root_offset,
                visible: true,
                flipped,
                scale: initial_scale,
            },
            |node, inherited| {
                // calculate scale
                let mut scale = inherited.scale * node.scale;

                if inherited.flipped && node.never_flip {
                    // flip back if perspective is flipped
                    scale.x *= -1.0;
                }

                node.sprite.set_scale(scale);

                // calculate offset
                let offset = inherited.offset + node.offset * inherited.scale;

                node.sprite.set_position(offset);

                // calculate visibility
                node.inherited_visible = node.visible && inherited.visible;

                InheritedProperties {
                    offset,
                    visible: node.inherited_visible,
                    // stop passing flipped once never_flip is hit
                    flipped: inherited.flipped && !node.never_flip,
                    scale,
                }
            },
        );
    }

    pub fn draw(&mut self, sprite_queue: &mut SpriteColorQueue) {
        self.draw_with_offset(sprite_queue, Vec2::ZERO, false);
    }

    pub fn draw_with_offset(
        &mut self,
        sprite_queue: &mut SpriteColorQueue,
        root_offset: Vec2,
        flipped: bool,
    ) {
        let intial_shader_effect = sprite_queue.shader_effect();

        type SpriteVec<'a> = SmallVec<[&'a mut SpriteNode; 5]>;
        let mut sprite_nodes = SpriteVec::with_capacity(self.len());

        // offset each child by parent node
        self.inherit_from_parent(root_offset, flipped);

        // capture root values before mutable reference
        let root_node = self.root();
        let root_shader_effect = root_node.shader_effect();
        let root_palette = root_node.palette.clone();
        let root_color_mode = root_node.color_mode();
        let root_color = root_node.color();

        // sort nodes
        sprite_nodes.extend(self.values_mut());
        sprite_nodes.sort_by_key(|node| -node.layer());

        // draw nodes
        for node in sprite_nodes.iter_mut() {
            if !node.inherited_visible() {
                // could possibly filter earlier,
                // but not expecting huge trees of invisible nodes
                continue;
            }

            // resolve shader
            let shader_effect;
            let palette;
            let color_mode;
            let color;
            let original_color = node.color();

            if node.using_parent_shader() {
                shader_effect = root_shader_effect;
                palette = &root_palette;
                color_mode = root_color_mode;
                color = root_color;
            } else {
                shader_effect = node.shader_effect();
                palette = &node.palette;
                color_mode = node.color_mode();
                color = node.color();
            }

            sprite_queue.set_shader_effect(shader_effect);

            if let Some(texture) = palette.as_ref() {
                sprite_queue.set_palette(texture.clone());
            }

            sprite_queue.set_color_mode(color_mode);
            node.set_color(color);

            // finally drawing the sprite
            sprite_queue.draw_sprite(node.sprite());

            node.set_color(original_color);
        }

        sprite_queue.set_shader_effect(intial_shader_effect);
    }
}
