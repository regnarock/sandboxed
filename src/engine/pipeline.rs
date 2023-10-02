use std::borrow::Cow;

use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_graph,
        render_resource::*,
        renderer::{RenderContext, RenderDevice},
    },
};

use super::{image::VoxelsRenderImage, SIM_SIZE, WORKGROUP_SIZE};

#[derive(Resource)]
pub struct VoxelShadersPipeline {
    init_pipeline: CachedComputePipelineId,
    update_pipeline: CachedComputePipelineId,
    texture_bind_group_layout: BindGroupLayout,
}

impl FromWorld for VoxelShadersPipeline {
    fn from_world(world: &mut World) -> Self {
        let texture_bind_group_layout =
            world
                .resource::<RenderDevice>()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("Game of Life Bind Group Layout"),
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::WriteOnly,
                            format: TextureFormat::Rgba8Unorm,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    }],
                });

        let pipeline_cache = world.resource::<PipelineCache>();
        let shader = world
            .resource::<AssetServer>()
            .load("shaders/sandboxed.wgsl");

        let init_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            shader: shader.clone(),
            shader_defs: vec![],
            layout: vec![texture_bind_group_layout.clone()],
            entry_point: Cow::from("init"),
            push_constant_ranges: Vec::new(),
            label: Some(std::borrow::Cow::Borrowed("Sandboxed Init Pipeline")),
        });
        let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            shader,
            shader_defs: vec![],
            layout: vec![texture_bind_group_layout.clone()],
            entry_point: Cow::from("update"),
            push_constant_ranges: Vec::new(),
            label: Some(std::borrow::Cow::Borrowed("Sandboxed Update Pipeline")),
        });

        VoxelShadersPipeline {
            texture_bind_group_layout,
            init_pipeline,
            update_pipeline,
        }
    }
}

#[derive(Resource)]
struct VoxelImageBindGroup(pub BindGroup);

pub fn queue_bind_group(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    pipeline: Res<VoxelShadersPipeline>,
    gpu_images: Res<RenderAssets<Image>>,
    game_of_life_image: Res<VoxelsRenderImage>,
) {
    let view = &gpu_images[&game_of_life_image.0];
    let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: Some("Game of Life Bind Group"),
        layout: &pipeline.texture_bind_group_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: BindingResource::TextureView(&view.texture_view),
        }],
    });
    commands.insert_resource(VoxelImageBindGroup(bind_group));
}

// TODO: FORMAT: separate in different file (think of better organisation)
pub enum VoxelSandboxState {
    Loading,
    Init,
    Update,
}

pub struct VoxelSandboxNode {
    state: VoxelSandboxState,
}

impl Default for VoxelSandboxNode {
    fn default() -> Self {
        Self {
            state: VoxelSandboxState::Loading,
        }
    }
}

impl render_graph::Node for VoxelSandboxNode {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<VoxelShadersPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        // if the corresponding pipeline has loaded, transition to the next stage
        match self.state {
            VoxelSandboxState::Loading => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.init_pipeline)
                {
                    self.state = VoxelSandboxState::Init;
                }
            }
            VoxelSandboxState::Init => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.update_pipeline)
                {
                    self.state = VoxelSandboxState::Update;
                }
            }
            VoxelSandboxState::Update => {}
        }
    }

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let texture_bind_group = &world.resource::<VoxelImageBindGroup>().0;
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<VoxelShadersPipeline>();
        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

        pass.set_bind_group(0, texture_bind_group, &[]);
        match self.state {
            VoxelSandboxState::Update | VoxelSandboxState::Loading => {}
            VoxelSandboxState::Init => {
                let init_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.init_pipeline)
                    .unwrap();
                pass.set_pipeline(init_pipeline);
                pass.dispatch_workgroups(
                    SIM_SIZE.0 / WORKGROUP_SIZE,
                    SIM_SIZE.1 / WORKGROUP_SIZE,
                    1,
                );
            }
        }
        Ok(())
    }
}
