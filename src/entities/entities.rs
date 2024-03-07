use std::f32::consts::PI;
use std::path::{Path, PathBuf};

use glam::{Vec2, Vec3, Vec4};
use hashbrown::HashMap;

use crate::assets::texture::{Sampler, Texture2D};
use crate::assets::{assets, ldtk};
use crate::context::VisContext;
use crate::entities::sprite::Sprite;
use crate::entities::transform2d::Transform2D;
use crate::utils::{Guid, GuidGenerator};

//A collection of entities that represents a set of worlds.
pub struct Worlds {
    worlds: HashMap<Guid, hecs::World>,
    generator: GuidGenerator,
    current_world: Option<Guid>,
}

impl Default for Worlds {
    fn default() -> Self {
        Self { worlds: HashMap::new(), generator: GuidGenerator::new(), current_world: None }
    }
}

impl Worlds {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_world(&mut self, world: hecs::World) -> Guid {
        let guid = self.generator.generate();
        self.worlds.insert(guid, world);
        guid
    }

    pub fn get_world(&mut self, guid: Guid) -> Option<&hecs::World> {
        self.worlds.get(&guid)
    }

    pub fn get_mut(&mut self) -> Option<&mut hecs::World> {
        if let Some(guid) = self.current_world {
            self.worlds.get_mut(&guid)
        } else {
            None
        }
    }

    pub fn get(&mut self) -> Option<&hecs::World> {
        if let Some(guid) = self.current_world {
            self.worlds.get(&guid)
        } else {
            None
        }
    }

    pub fn start_world(&mut self, guid: Guid) {
        self.current_world = Some(guid);
    }

    pub fn from_ldtk_file<P: AsRef<Path>>(
        context: &VisContext, loc: &Option<PathBuf>, assets: &mut assets::Assets, ldtk_file_path: P,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let file_content = std::fs::read(&ldtk_file_path)?;
        let project: ldtk::Project = serde_json::from_slice(&file_content)?;

        assert_eq!(project.worlds.len(), 0, "Ldtk Multi-worlds setting is not supported");
        assert_eq!(project.levels.len(), 1, "Cannot have more than one level in a ldtk file");

        assert_eq!(
            project.json_version, "1.5.3",
            "Ldtk version {} is not supported - only 1.5.3 is supported",
            project.json_version
        );

        let level = &project.levels[0];

        let li = match &level.layer_instances {
            Some(li) => li,
            None => return Err("Level has no layer instances".into()),
        };

        let layer_z_coord_offset = 0.99 / li.len() as f32;
        let mut layer_z = 1.0;

        let mut world = hecs::World::new();
        for layer in li {
            let (layer_texture, layer_texture_info) =
                match (&layer.tileset_rel_path, layer.tileset_def_uid) {
                    (Some(rp), Some(id)) => {
                        let tileset_path = tileset_filepath(&ldtk_file_path, loc, &rp)?;

                        let texture_info = project
                            .defs
                            .tilesets
                            .iter()
                            .find(|t| t.uid == id)
                            .ok_or(format!("Tileset with id {} not found in ldtk file", id))?;

                        let texture: assets::Ptr<Texture2D> =
                            assets.request_asset(tileset_path.to_string_lossy(), 0);

                        (texture, texture_info)
                    }
                    _ => return Err("Layer has no tileset".into()),
                };

            for tile in layer.grid_tiles.iter() {
                // Calculate scale from c_wid and c_hei
                let scale_x = 1.0 / (layer.c_wid as f32);
                let scale_y = 1.0 / (layer.c_hei as f32);
                let scale = scale_x.min(scale_y);
                debug_assert!((0.0..=1.0).contains(&scale), "scale out of bounds");

                // Calculate x and y coordinates from tile position

                let x_grid_pos = (layer.px_total_offset_x + tile.px[0]) / layer.grid_size;
                let y_grid_pos = (layer.px_total_offset_y + tile.px[1]) / layer.grid_size;

                let x_coord = x_grid_pos as f32 * scale * 2.0 - 1.0;
                let y_coord = y_grid_pos as f32 * scale * 2.0;

                let transform = Transform2D::new(
                    context,
                    Vec3::new(x_coord, -y_coord, layer_z),
                    PI,
                    // Scale:
                    Vec2::new(scale, scale),
                );

                // This is definitely correct:
                let fanta = Sprite::new(
                    context,
                    layer_texture,
                    Vec4::new(1.0, 1.0, 1.0, tile.a as f32),
                    Some(&tile.coords_8(
                        layer.grid_size,
                        layer_texture_info.px_wid as f32,
                        layer_texture_info.px_hei as f32,
                    )),
                    Some(Sampler::new(context)),
                );

                world.spawn((transform, fanta));
            }

            for _entity in layer.entity_instances.iter() {}

            layer_z -= layer_z_coord_offset;
        }

        let mut worlds = Worlds::new();
        let guid = worlds.add_world(world);
        worlds.start_world(guid);
        Ok(worlds)
    }
}

fn tileset_filepath<P1: AsRef<Path>, P2: AsRef<Path>>(
    ldtk_file_path: &P1, loc: &Option<PathBuf>, tileset_relative_path: &P2,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut path = std::path::Path::new(ldtk_file_path.as_ref());
    if let Some(prefix) = &loc {
        path = path.strip_prefix(prefix)?;
    }

    let parent = path.parent().ok_or("Cannot get parent of ldtk file path")?;
    // For the tileset_relative_path, we replace the extension (e.g. .png) with .fur
    let tileset_relative_path = tileset_relative_path.as_ref();
    let tileset_relative_path = tileset_relative_path.with_extension("fur");

    Ok(parent.join(tileset_relative_path))
}
