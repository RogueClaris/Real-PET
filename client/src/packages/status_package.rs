use super::*;
use crate::render::ui::{PackageListing, PackagePreviewData};
use crate::render::FrameTime;
use serde::Deserialize;

#[derive(Deserialize, Default)]
#[serde(default)]
struct StatusMeta {
    category: String,
    icon_texture_path: Option<String>,
    name: String,
    description: String,
    flag_name: String,
    mutual_exclusions: Vec<String>,
    blocks_actions: bool,
    blocks_mobility: bool,
    durations: Vec<FrameTime>,
}

#[derive(Default, Clone)]
pub struct StatusPackage {
    pub package_info: PackageInfo,
    pub icon_texture_path: Option<String>,
    pub name: String,
    pub description: String,
    pub flag_name: String,
    pub mutual_exclusions: Vec<String>,
    pub blocks_actions: bool,
    pub blocks_mobility: bool,
    pub durations: Vec<FrameTime>,
}

impl Package for StatusPackage {
    fn package_info(&self) -> &PackageInfo {
        &self.package_info
    }

    fn create_package_listing(&self) -> PackageListing {
        PackageListing {
            id: self.package_info.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            creator: String::new(),
            hash: self.package_info.hash,
            preview_data: PackagePreviewData::Status,
            dependencies: self.package_info.requirements.clone(),
        }
    }

    fn load_new(package_info: PackageInfo, package_table: toml::Table) -> Self {
        let mut package = Self {
            package_info,
            ..Default::default()
        };

        let meta: StatusMeta = match package_table.try_into() {
            Ok(toml) => toml,
            Err(e) => {
                log::error!("Failed to parse {:?}:\n{e}", package.package_info.toml_path);
                return package;
            }
        };

        if meta.category != "status" {
            log::error!(
                "Missing `category = \"status\"` in {:?}",
                package.package_info.toml_path
            );
        }

        let base_path = &package.package_info.base_path;

        package.name = meta.name;
        package.icon_texture_path = meta.icon_texture_path.map(|p| base_path.clone() + &p);
        package.description = meta.description;
        package.flag_name = meta.flag_name;
        package.mutual_exclusions = meta.mutual_exclusions;
        package.blocks_actions = meta.blocks_actions;
        package.blocks_mobility = meta.blocks_mobility;
        package.durations = meta.durations;

        if package.durations.is_empty() {
            package.durations.push(1);
        }

        package
    }
}
