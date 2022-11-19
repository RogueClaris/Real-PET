use super::*;
use crate::bindable::Element;
use crate::lua_api::create_analytical_vm;
use crate::resources::{LocalAssetManager, ResourcePaths};
use std::cell::RefCell;

#[derive(Default, Clone)]
pub struct PlayerPackage {
    pub package_info: PackageInfo,
    pub name: String,
    pub health: i32,
    pub element: Element,
    pub description: String,
    pub icon_texture_path: String,
    pub preview_texture_path: String,
    pub overworld_animation_path: String,
    pub overworld_texture_path: String,
    pub mugshot_texture_path: String,
    pub mugshot_animation_path: String,
    pub emotions_texture_path: String,
}

impl Package for PlayerPackage {
    fn package_info(&self) -> &PackageInfo {
        &self.package_info
    }

    fn package_info_mut(&mut self) -> &mut PackageInfo {
        &mut self.package_info
    }

    fn load_new(assets: &LocalAssetManager, package_info: PackageInfo) -> Self {
        let package = RefCell::new(PlayerPackage::default());
        package.borrow_mut().package_info = package_info.clone();

        let lua = create_analytical_vm(assets, &package_info);

        let globals = lua.globals();
        let package_init: rollback_mlua::Function = match globals.get("package_init") {
            Ok(package_init) => package_init,
            _ => {
                log::error!(
                    "missing package_init() in {:?}",
                    ResourcePaths::shorten(&package_info.script_path)
                );
                return package.into_inner();
            }
        };

        let result = lua.scope(|scope| {
            crate::lua_api::inject_analytical_api(&lua, scope, assets, &package)?;
            crate::lua_api::query_dependencies(&lua);

            let package_table = lua.create_table()?;

            package_table.set(
                "declare_package_id",
                scope.create_function(|_, (_, id): (rollback_mlua::Table, String)| {
                    package.borrow_mut().package_info.id = id;
                    Ok(())
                })?,
            )?;

            package_table.set(
                "set_name",
                scope.create_function(|_, (_, name): (rollback_mlua::Table, String)| {
                    package.borrow_mut().name = name;
                    Ok(())
                })?,
            )?;

            package_table.set(
                "set_health",
                scope.create_function(|_, (_, health): (rollback_mlua::Table, i32)| {
                    package.borrow_mut().health = health;
                    Ok(())
                })?,
            )?;

            package_table.set(
                "set_element",
                scope.create_function(|_, (_, element): (rollback_mlua::Table, Element)| {
                    package.borrow_mut().element = element;
                    Ok(())
                })?,
            )?;

            package_table.set(
                "set_description",
                scope.create_function(|_, (_, description): (rollback_mlua::Table, String)| {
                    package.borrow_mut().description = description;
                    Ok(())
                })?,
            )?;

            package_table.set(
                "set_icon_texture_path",
                scope.create_function(|_, (_, path): (rollback_mlua::Table, String)| {
                    package.borrow_mut().icon_texture_path =
                        package_info.base_path.to_string() + &path;
                    Ok(())
                })?,
            )?;

            package_table.set(
                "set_preview_texture_path",
                scope.create_function(|_, (_, path): (rollback_mlua::Table, String)| {
                    package.borrow_mut().preview_texture_path =
                        package_info.base_path.to_string() + &path;
                    Ok(())
                })?,
            )?;

            package_table.set(
                "set_overworld_texture_path",
                scope.create_function(|_, (_, path): (rollback_mlua::Table, String)| {
                    package.borrow_mut().overworld_texture_path =
                        package_info.base_path.to_string() + &path;
                    Ok(())
                })?,
            )?;

            package_table.set(
                "set_overworld_animation_path",
                scope.create_function(|_, (_, path): (rollback_mlua::Table, String)| {
                    package.borrow_mut().overworld_animation_path =
                        package_info.base_path.to_string() + &path;
                    Ok(())
                })?,
            )?;

            package_table.set(
                "set_mugshot_texture_path",
                scope.create_function(|_, (_, path): (rollback_mlua::Table, String)| {
                    package.borrow_mut().mugshot_texture_path =
                        package_info.base_path.to_string() + &path;
                    Ok(())
                })?,
            )?;

            package_table.set(
                "set_mugshot_animation_path",
                scope.create_function(|_, (_, path): (rollback_mlua::Table, String)| {
                    package.borrow_mut().mugshot_animation_path =
                        package_info.base_path.to_string() + &path;
                    Ok(())
                })?,
            )?;

            package_table.set(
                "set_emotions_texture_path",
                scope.create_function(|_, (_, path): (rollback_mlua::Table, String)| {
                    package.borrow_mut().emotions_texture_path =
                        package_info.base_path.to_string() + &path;
                    Ok(())
                })?,
            )?;

            package_init.call(package_table)?;

            Ok(())
        });

        if let Err(e) = result {
            log::error!("{e}");
        }

        package.into_inner()
    }
}
