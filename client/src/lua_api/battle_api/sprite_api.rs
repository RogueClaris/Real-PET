use super::errors::{invalid_font_name, sprite_not_found};
use super::{BattleLuaApi, SPRITE_TABLE};
use crate::bindable::{GenerationalIndex, LuaColor, LuaVector, SpriteColorMode};
use crate::lua_api::helpers::{absolute_path, inherit_metatable};
use crate::render::ui::{FontStyle, TextStyle};
use crate::render::{SpriteNode, SpriteShaderEffect};
use crate::structures::TreeIndex;
use framework::prelude::Vec2;

pub fn inject_sprite_api(lua_api: &mut BattleLuaApi) {
    lua_api.add_dynamic_function(SPRITE_TABLE, "create_node", |api_ctx, lua, params| {
        let table: rollback_mlua::Table = lua.unpack_multi(params)?;

        let slot_index: GenerationalIndex = table.raw_get("#tree")?;
        let sprite_index: TreeIndex = table.raw_get("#sprite")?;

        let api_ctx = &mut *api_ctx.borrow_mut();
        let simulation = &mut api_ctx.simulation;

        let sprite_tree = simulation
            .sprite_trees
            .get_mut(slot_index)
            .ok_or_else(sprite_not_found)?;

        let child_index = sprite_tree
            .insert_child(
                sprite_index,
                SpriteNode::new(api_ctx.game_io, SpriteColorMode::Add),
            )
            .ok_or_else(sprite_not_found)?;

        let child_table = create_sprite_table(lua, slot_index, child_index, None)?;

        lua.pack_multi(child_table)
    });

    lua_api.add_dynamic_function(SPRITE_TABLE, "create_text_node", |api_ctx, lua, params| {
        let (table, font_name, text): (
            rollback_mlua::Table,
            rollback_mlua::String,
            rollback_mlua::String,
        ) = lua.unpack_multi(params)?;

        let Some(font_style) = FontStyle::from_name(font_name.to_str()?) else {
            return Err(invalid_font_name());
        };
        let text = &*text.to_string_lossy();

        let slot_index: GenerationalIndex = table.raw_get("#tree")?;
        let sprite_index: TreeIndex = table.raw_get("#sprite")?;

        let api_ctx = &mut *api_ctx.borrow_mut();
        let game_io = api_ctx.game_io;
        let simulation = &mut api_ctx.simulation;

        let sprite_tree = simulation
            .sprite_trees
            .get_mut(slot_index)
            .ok_or_else(sprite_not_found)?;

        let text_style = TextStyle::new(game_io, font_style);
        let child_index = sprite_tree
            .insert_text_child(game_io, sprite_index, &text_style, text)
            .ok_or_else(sprite_not_found)?;

        let child_table = create_sprite_table(lua, slot_index, child_index, None)?;

        lua.pack_multi(child_table)
    });

    lua_api.add_dynamic_function(SPRITE_TABLE, "remove_node", |api_ctx, lua, params| {
        let (table, child_table): (rollback_mlua::Table, rollback_mlua::Table) =
            lua.unpack_multi(params)?;

        let slot_index: GenerationalIndex = table.raw_get("#tree")?;
        let sprite_index: TreeIndex = table.raw_get("#sprite")?;

        let child_slot_index: GenerationalIndex = child_table.raw_get("#tree")?;
        let sprite_child_index: TreeIndex = child_table.raw_get("#sprite")?;

        if slot_index != child_slot_index {
            return lua.pack_multi(());
        }

        let mut api_ctx = api_ctx.borrow_mut();
        let simulation = &mut api_ctx.simulation;

        let sprite_tree = simulation
            .sprite_trees
            .get_mut(slot_index)
            .ok_or_else(sprite_not_found)?;

        // verify existence of sprite
        let sprite_node_node = sprite_tree
            .get_node(sprite_index)
            .ok_or_else(sprite_not_found)?;

        // verify the child is a child of the sprite
        if !sprite_node_node.children().contains(&sprite_child_index) {
            return lua.pack_multi(());
        }

        sprite_tree.remove(sprite_child_index);

        lua.pack_multi(())
    });

    getter(lua_api, "texture", |node, _, _: ()| {
        Ok(node.texture_path().to_string())
    });

    lua_api.add_dynamic_function(SPRITE_TABLE, "set_texture", move |api_ctx, lua, params| {
        let (table, path): (rollback_mlua::Table, String) = lua.unpack_multi(params)?;
        let path = absolute_path(lua, path)?;

        let slot_index: GenerationalIndex = table.raw_get("#tree")?;
        let sprite_index: TreeIndex = table.raw_get("#sprite")?;

        let api_ctx = &mut *api_ctx.borrow_mut();
        let simulation = &mut api_ctx.simulation;

        let sprite_tree = simulation
            .sprite_trees
            .get_mut(slot_index)
            .ok_or_else(sprite_not_found)?;

        let sprite_node = sprite_tree
            .get_mut(sprite_index)
            .ok_or_else(sprite_not_found)?;

        sprite_node.set_texture(api_ctx.game_io, path);

        if let Ok(animator_index) = table.raw_get::<_, GenerationalIndex>("#anim") {
            if let Some(animator) = simulation.animators.get_mut(animator_index) {
                animator.apply(sprite_node);
            }
        }

        lua.pack_multi(())
    });

    getter(lua_api, "visible", |node, _, _: ()| Ok(node.visible()));
    setter(lua_api, "set_visible", |node, _, visible: bool| {
        node.set_visible(visible);
        Ok(())
    });
    setter(lua_api, "reveal", |node, _, _: ()| {
        node.set_visible(true);
        Ok(())
    });
    setter(lua_api, "hide", |node, _, _: ()| {
        node.set_visible(false);
        Ok(())
    });

    getter(lua_api, "layer", |node, _, _: ()| Ok(node.layer()));
    setter(lua_api, "set_layer", |node, _, layer| {
        node.set_layer(layer);
        Ok(())
    });

    //   "add_tags", [](WeakWrapper<SpriteProxyNode>& node, std::initializer_list<std::string> tags) {
    //     node.Unwrap()->AddTags(tags);
    //   },
    //   "remove_tags", [](WeakWrapper<SpriteProxyNode>& node, std::initializer_list<std::string> tags) {
    //     node.Unwrap()->RemoveTags(tags);
    //   },
    //   "has_tag", [](WeakWrapper<SpriteProxyNode>& node, const std::string& tag) -> bool{
    //     return node.Unwrap()->HasTag(tag);
    //   },
    //   "find_child_nodes_with_tags", [](WeakWrapper<SpriteProxyNode>& node, std::vector<std::string> tags) {
    //     auto nodes = node.Unwrap()->GetChildNodesWithTag(tags);
    //     std::vector<WeakWrapper<SceneNode>> result;
    //     result.reserve(nodes.size());

    //     for (auto node : nodes) {
    //       result.push_back(WeakWrapper(node));
    //     }

    //     return sol::as_table(result);
    //   },

    lua_api.add_dynamic_function(SPRITE_TABLE, "children", move |api_ctx, lua, params| {
        let table: rollback_mlua::Table = lua.unpack_multi(params)?;

        let slot_index: GenerationalIndex = table.raw_get("#tree")?;
        let sprite_index: TreeIndex = table.raw_get("#sprite")?;

        let api_ctx = &mut *api_ctx.borrow_mut();
        let simulation = &mut api_ctx.simulation;

        let sprite_tree = simulation
            .sprite_trees
            .get_mut(slot_index)
            .ok_or_else(sprite_not_found)?;

        let sprite_tree_node = sprite_tree
            .get_node(sprite_index)
            .ok_or_else(sprite_not_found)?;

        // create table
        let children_table = lua.create_table()?;

        for &child_index in sprite_tree_node.children() {
            // todo: associate animators?
            children_table.push(create_sprite_table(lua, slot_index, child_index, None)?)?;
        }

        lua.pack_multi(children_table)
    });

    getter(lua_api, "offset", |node, _, _: ()| {
        Ok(LuaVector::from(node.offset()))
    });
    setter(lua_api, "set_offset", |node, _, offset: (f32, f32)| {
        node.set_offset(offset.into());
        Ok(())
    });

    getter(lua_api, "origin", |node, _, _: ()| {
        Ok(LuaVector::from(node.origin()))
    });
    setter(lua_api, "set_origin", |node, _, origin: (f32, f32)| {
        node.set_origin(origin.into());
        Ok(())
    });

    getter(lua_api, "scale", |node, _, _: ()| {
        Ok(LuaVector::from(node.scale()))
    });
    setter(lua_api, "set_scale", |node, _, scale: (f32, f32)| {
        node.set_scale(scale.into());
        Ok(())
    });

    setter(lua_api, "size", |node, _, _: ()| {
        Ok(LuaVector::from(node.size()))
    });
    setter(lua_api, "set_size", |node, _, size: (f32, f32)| {
        node.set_size(size.into());
        Ok(())
    });

    setter(lua_api, "width", |node, _, _: ()| Ok(node.size().x));
    setter(lua_api, "set_width", |node, _, width| {
        let height = node.size().y;
        node.set_size(Vec2::new(width, height));
        Ok(())
    });

    setter(lua_api, "height", |node, _, _: ()| Ok(node.size().y));
    setter(lua_api, "set_height", |node, _, height| {
        let width = node.size().x;
        node.set_size(Vec2::new(width, height));
        Ok(())
    });

    getter(lua_api, "color", |node, _, _: ()| {
        Ok(LuaColor::from(node.color()))
    });
    setter(lua_api, "set_color", |node, _, color: LuaColor| {
        node.set_color(color.into());
        Ok(())
    });

    getter(
        lua_api,
        "color_mode",
        |node, _, _: ()| Ok(node.color_mode()),
    );
    setter(lua_api, "set_color_mode", |node, _, mode| {
        node.set_color_mode(mode);
        Ok(())
    });

    getter(lua_api, "palette", |node, _, _: ()| {
        Ok(node.palette_path().map(String::from))
    });
    lua_api.add_dynamic_function(SPRITE_TABLE, "set_palette", move |api_ctx, lua, params| {
        let (table, path): (rollback_mlua::Table, Option<String>) = lua.unpack_multi(params)?;
        let path = path.map(|path| absolute_path(lua, path)).transpose()?;

        let slot_index: GenerationalIndex = table.raw_get("#tree")?;
        let sprite_index: TreeIndex = table.raw_get("#sprite")?;

        let api_ctx = &mut *api_ctx.borrow_mut();
        let simulation = &mut api_ctx.simulation;

        let sprite_tree = simulation
            .sprite_trees
            .get_mut(slot_index)
            .ok_or_else(sprite_not_found)?;

        let sprite_node = sprite_tree
            .get_mut(sprite_index)
            .ok_or_else(sprite_not_found)?;

        sprite_node.set_palette(api_ctx.game_io, path);

        lua.pack_multi(())
    });

    getter(
        lua_api,
        "never_flip",
        |node, _, _: ()| Ok(node.never_flip()),
    );
    setter(
        lua_api,
        "set_never_flip",
        |node, _, never_flip: Option<bool>| {
            node.set_never_flip(never_flip.unwrap_or(true));
            Ok(())
        },
    );

    getter(lua_api, "shader_effect", |node, _, _: ()| {
        Ok(node.shader_effect())
    });
    setter(
        lua_api,
        "set_shader_effect",
        |node, _, effect: SpriteShaderEffect| {
            node.set_shader_effect(effect);
            Ok(())
        },
    );

    setter(
        lua_api,
        "use_root_shader",
        |node, _, enable: Option<bool>| {
            node.set_using_root_shader(enable.unwrap_or(true));
            Ok(())
        },
    );

    setter(
        lua_api,
        "use_parent_shader",
        |node, _, enable: Option<bool>| {
            node.set_using_root_shader(enable.unwrap_or(true));
            Ok(())
        },
    );
}

pub fn create_sprite_table(
    lua: &rollback_mlua::Lua,
    slot_index: GenerationalIndex,
    sprite_index: TreeIndex,
    animator_index: Option<GenerationalIndex>,
) -> rollback_mlua::Result<rollback_mlua::Table> {
    let table = lua.create_table()?;
    table.raw_set("#tree", slot_index)?;
    table.raw_set("#sprite", sprite_index)?;

    if let Some(index) = animator_index {
        table.raw_set("#anim", index)?;
    }

    inherit_metatable(lua, SPRITE_TABLE, &table)?;

    Ok(table)
}

fn getter<F, P, R>(lua_api: &mut BattleLuaApi, name: &str, callback: F)
where
    R: for<'lua> rollback_mlua::IntoLua<'lua>,
    P: for<'lua> rollback_mlua::FromLuaMulti<'lua>,
    F: for<'lua> Fn(&SpriteNode, &'lua rollback_mlua::Lua, P) -> rollback_mlua::Result<R> + 'static,
{
    lua_api.add_dynamic_function(SPRITE_TABLE, name, move |api_ctx, lua, params| {
        let (table, param): (rollback_mlua::Table, P) = lua.unpack_multi(params)?;

        let slot_index: GenerationalIndex = table.raw_get("#tree")?;
        let sprite_index: TreeIndex = table.raw_get("#sprite")?;

        let mut api_ctx = api_ctx.borrow_mut();
        let simulation = &mut api_ctx.simulation;

        let sprite_tree = simulation
            .sprite_trees
            .get_mut(slot_index)
            .ok_or_else(sprite_not_found)?;

        let sprite_node = sprite_tree.get(sprite_index).ok_or_else(sprite_not_found)?;

        lua.pack_multi(callback(sprite_node, lua, param)?)
    });
}

fn setter<F, P, R>(lua_api: &mut BattleLuaApi, name: &str, callback: F)
where
    R: for<'lua> rollback_mlua::IntoLuaMulti<'lua>,
    P: for<'lua> rollback_mlua::FromLuaMulti<'lua>,
    F: for<'lua> Fn(&mut SpriteNode, &'lua rollback_mlua::Lua, P) -> rollback_mlua::Result<R>
        + 'static,
{
    lua_api.add_dynamic_function(SPRITE_TABLE, name, move |api_ctx, lua, params| {
        let (table, param): (rollback_mlua::Table, P) = lua.unpack_multi(params)?;

        let slot_index: GenerationalIndex = table.raw_get("#tree")?;
        let sprite_index: TreeIndex = table.raw_get("#sprite")?;

        let mut api_ctx = api_ctx.borrow_mut();
        let simulation = &mut api_ctx.simulation;

        let sprite_tree = simulation
            .sprite_trees
            .get_mut(slot_index)
            .ok_or_else(sprite_not_found)?;

        let sprite_node = sprite_tree
            .get_mut(sprite_index)
            .ok_or_else(sprite_not_found)?;

        lua.pack_multi(callback(sprite_node, lua, param)?)
    });
}
