use bevy::{
    core_pipeline::core_2d::Transparent2d,
    ecs::system::lifetimeless::{Read, SRes},
    render::{
        mesh::GpuBufferInfo,
        render_phase::{RenderCommand, RenderCommandResult},
        render_resource::PipelineCache,
        view::ViewUniformOffset,
    },
};

use super::{
    chunk::RenderChunkStorage,
    extract::ExtractedTilemap,
    queue::TileViewBindGroup,
    uniform::{DynamicUniformComponent, TilemapUniform}, resources::TilemapBindGroups,
};

pub type DrawTilemap = (
    SetPipeline,
    SetTilemapViewBindGroup<0>,
    SetTilemapUniformBindGroup<1>,
    SetTilemapColorTextureBindGroup<2>,
    DrawTileMesh,
);

pub type DrawTilemapPureColor = (
    SetPipeline,
    SetTilemapViewBindGroup<0>,
    SetTilemapUniformBindGroup<1>,
    DrawTileMesh,
);

pub struct SetPipeline;
impl RenderCommand<Transparent2d> for SetPipeline {
    type Param = SRes<PipelineCache>;

    type ViewWorldQuery = ();

    type ItemWorldQuery = ();

    #[inline]
    fn render<'w>(
        item: &Transparent2d,
        _view: bevy::ecs::query::ROQueryItem<'w, Self::ViewWorldQuery>,
        _entity: bevy::ecs::query::ROQueryItem<'w, Self::ItemWorldQuery>,
        pipeline_cache: bevy::ecs::system::SystemParamItem<'w, '_, Self::Param>,
        pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let pipeline_cache = pipeline_cache.into_inner();
        if let Some(pipeline) = pipeline_cache.get_render_pipeline(item.pipeline) {
            pass.set_render_pipeline(pipeline);
            RenderCommandResult::Success
        } else {
            pipeline_cache.get_render_pipeline_state(item.pipeline);
            println!("Failed to get render pipeline!");
            panic!();
        }
    }
}

pub struct SetTilemapViewBindGroup<const I: usize>;
impl<const I: usize> RenderCommand<Transparent2d> for SetTilemapViewBindGroup<I> {
    type Param = ();

    type ViewWorldQuery = (Read<ViewUniformOffset>, Read<TileViewBindGroup>);

    type ItemWorldQuery = ();

    fn render<'w>(
        _item: &Transparent2d,
        (view_uniform_offset, view_bind_group): bevy::ecs::query::ROQueryItem<
            'w,
            Self::ViewWorldQuery,
        >,
        _entity: bevy::ecs::query::ROQueryItem<'w, Self::ItemWorldQuery>,
        _param: bevy::ecs::system::SystemParamItem<'w, '_, Self::Param>,
        pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        pass.set_bind_group(I, &view_bind_group.value, &[view_uniform_offset.offset]);

        RenderCommandResult::Success
    }
}

pub struct SetTilemapUniformBindGroup<const I: usize>;
impl<const I: usize> RenderCommand<Transparent2d> for SetTilemapUniformBindGroup<I> {
    type Param = SRes<TilemapBindGroups>;

    type ViewWorldQuery = ();

    type ItemWorldQuery = (
        Read<ExtractedTilemap>,
        Read<DynamicUniformComponent<TilemapUniform>>,
    );

    fn render<'w>(
        _item: &Transparent2d,
        _view: bevy::ecs::query::ROQueryItem<'w, Self::ViewWorldQuery>,
        (tilemap, uniform_data): bevy::ecs::query::ROQueryItem<'w, Self::ItemWorldQuery>,
        bind_groups: bevy::ecs::system::SystemParamItem<'w, '_, Self::Param>,
        pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Some(tilemap_uniform_bind_group) = bind_groups
            .into_inner()
            .tilemap_uniforms
            .get(&tilemap.id)
        {
            pass.set_bind_group(I, tilemap_uniform_bind_group, &[uniform_data.index]);
            RenderCommandResult::Success
        } else {
            println!("Failed to get tilemap uniform bind group!");
            RenderCommandResult::Failure
        }
    }
}

pub struct SetTilemapColorTextureBindGroup<const I: usize>;
impl<const I: usize> RenderCommand<Transparent2d> for SetTilemapColorTextureBindGroup<I> {
    type Param = SRes<TilemapBindGroups>;

    type ViewWorldQuery = ();

    type ItemWorldQuery = Read<ExtractedTilemap>;

    #[inline]
    fn render<'w>(
        _item: &Transparent2d,
        _view: bevy::ecs::query::ROQueryItem<'w, Self::ViewWorldQuery>,
        tilemap: bevy::ecs::query::ROQueryItem<'w, Self::ItemWorldQuery>,
        bind_groups: bevy::ecs::system::SystemParamItem<'w, '_, Self::Param>,
        pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Some(texture) = &tilemap.texture {
            pass.set_bind_group(
                I,
                bind_groups
                    .into_inner()
                    .colored_textures
                    .get(texture.handle())
                    .unwrap(),
                &[],
            );
        }

        RenderCommandResult::Success
    }
}

pub struct DrawTileMesh;
impl RenderCommand<Transparent2d> for DrawTileMesh {
    type Param = SRes<RenderChunkStorage>;

    type ViewWorldQuery = ();

    type ItemWorldQuery = Read<ExtractedTilemap>;

    fn render<'w>(
        _item: &Transparent2d,
        _view: bevy::ecs::query::ROQueryItem<'w, Self::ViewWorldQuery>,
        tilemap: bevy::ecs::query::ROQueryItem<'w, Self::ItemWorldQuery>,
        render_chunks: bevy::ecs::system::SystemParamItem<'w, '_, Self::Param>,
        pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Some(chunks) = render_chunks.into_inner().get_chunks(tilemap.id) {
            for chunk in chunks.iter().rev() {
                let Some(c) = chunk else {
                    continue;
                };

                if !c.visible {
                    continue;
                }

                if let Some(gpu_mesh) = &c.gpu_mesh {
                    pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
                    match &gpu_mesh.buffer_info {
                        GpuBufferInfo::Indexed {
                            buffer,
                            count,
                            index_format,
                        } => {
                            pass.set_index_buffer(buffer.slice(..), 0, *index_format);
                            pass.draw_indexed(0..*count, 0, 0..1);
                        }
                        GpuBufferInfo::NonIndexed => {
                            pass.draw(0..gpu_mesh.vertex_count, 0..1);
                        }
                    }
                }
            }
        }

        RenderCommandResult::Success
    }
}