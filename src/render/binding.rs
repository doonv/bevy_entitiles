use bevy::{
    asset::Handle,
    ecs::{component::Component, entity::Entity, system::Resource, world::FromWorld},
    render::{
        render_resource::{
            BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
            BindGroupLayoutEntry, BindingResource, BindingType, BufferBindingType,
            SamplerBindingType, ShaderStages, ShaderType, TextureSampleType, TextureViewDimension,
        },
        renderer::RenderDevice,
        texture::Image,
        view::ViewUniform,
    },
    utils::{EntityHashMap, HashMap},
};

use super::{
    buffer::{
        PerTilemapBuffersStorage, TilemapStorageBuffers, TilemapUniform, TilemapUniformBuffer,
        UniformBuffer,
    },
    extract::ExtractedTilemap,
    pipeline::EntiTilesPipeline,
    texture::TilemapTexturesStorage,
};

#[derive(Component)]
pub struct TilemapViewBindGroup {
    pub value: BindGroup,
}

#[derive(Resource, Default)]
pub struct TilemapBindGroups {
    pub tilemap_uniform_buffer: Option<BindGroup>,
    pub tilemap_storage_buffers: EntityHashMap<Entity, BindGroup>,
    pub colored_textures: HashMap<Handle<Image>, BindGroup>,
}

impl TilemapBindGroups {
    pub fn bind_uniform_buffers(
        &mut self,
        render_device: &RenderDevice,
        uniform_buffers: &mut TilemapUniformBuffer,
        entitiles_pipeline: &EntiTilesPipeline,
    ) {
        let Some(uniform_buffer) = uniform_buffers.binding() else {
            return;
        };

        self.tilemap_uniform_buffer = Some(render_device.create_bind_group(
            Some("tilemap_uniform_buffers_bind_group"),
            &entitiles_pipeline.uniform_buffers_layout,
            &[BindGroupEntry {
                binding: 0,
                resource: uniform_buffer,
            }],
        ));
    }

    pub fn bind_storage_buffers(
        &mut self,
        render_device: &RenderDevice,
        storage_buffers: &mut TilemapStorageBuffers,
        entitiles_pipeline: &EntiTilesPipeline,
    ) {
        storage_buffers
            .bindings()
            .into_iter()
            .for_each(|(tilemap, resource)| {
                self.tilemap_storage_buffers.insert(
                    tilemap,
                    render_device.create_bind_group(
                        Some("tilemap_storage_bind_group"),
                        &entitiles_pipeline.storage_buffers_layout,
                        &[BindGroupEntry {
                            binding: 0,
                            resource,
                        }],
                    ),
                );
            });
    }

    /// Returns is_pure_color
    pub fn queue_textures(
        &mut self,
        tilemap: &ExtractedTilemap,
        render_device: &RenderDevice,
        textures_storage: &TilemapTexturesStorage,
        entitile_pipeline: &EntiTilesPipeline,
    ) -> bool {
        let Some(tilemap_texture) = &tilemap.texture else {
            return true;
        };

        let Some(texture) = textures_storage.get_texture(tilemap_texture.handle()) else {
            return !textures_storage.contains(tilemap_texture.handle());
        };

        if !self.colored_textures.contains_key(tilemap_texture.handle()) {
            self.colored_textures.insert(
                tilemap_texture.clone_weak(),
                render_device.create_bind_group(
                    Some("color_texture_bind_group"),
                    &entitile_pipeline.color_texture_layout,
                    &[
                        BindGroupEntry {
                            binding: 0,
                            resource: BindingResource::TextureView(&texture.texture_view),
                        },
                        BindGroupEntry {
                            binding: 1,
                            resource: BindingResource::Sampler(&texture.sampler),
                        },
                    ],
                ),
            );
        }

        false
    }
}

#[derive(Resource)]
pub struct TilemapBindGroupLayouts {
    pub view_layout: BindGroupLayout,
    pub tilemap_uniforms_layout: BindGroupLayout,
    pub tilemap_storage_layout: BindGroupLayout,
    pub color_texture_layout: BindGroupLayout,
}

impl FromWorld for TilemapBindGroupLayouts {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let view_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("tilemap_view_layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: Some(ViewUniform::min_size()),
                },
                count: None,
            }],
        });

        let tilemap_uniforms_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("tilemap_uniforms_layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: Some(TilemapUniform::min_size()),
                    },
                    count: None,
                }],
            });

        let tilemap_storage_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("tilemap_storage_layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(i32::min_size()),
                    },
                    count: None,
                }],
            });

        #[cfg(not(feature = "atlas"))]
        let color_texture_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("color_texture_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2Array,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });
        
        #[cfg(feature = "atlas")]
        let color_texture_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("color_texture_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        Self {
            view_layout,
            tilemap_uniforms_layout,
            tilemap_storage_layout,
            color_texture_layout,
        }
    }
}
