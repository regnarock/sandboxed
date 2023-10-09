use bevy::{
    prelude::{Res, Resource},
    render::{
        extract_resource::ExtractResource,
        render_resource::{Buffer, BufferInitDescriptor, BufferUsages},
        renderer::RenderDevice,
    },
};

use super::{SIM_SIZE, SIM_VOXELS};

#[derive(Resource, Clone, ExtractResource, Debug)]
pub struct VoxelPipelineBuffers {
    /// uniform, used for constants
    pub dimentions: Buffer,
    /// storage
    pub in_out_double_buffers: Vec<Buffer>,
    pub(in crate::engine) double_indices: DoubleBufferIndices,
}

impl VoxelPipelineBuffers {
    pub fn new(device: Res<RenderDevice>) -> Self {
        // Initialize constants (configuration, parameters... etc)
        let dimentions = create_uniform_buffer(
            &device,
            &[SIM_SIZE.0, SIM_SIZE.1],
            Some("Voxels image dimensions Uniform"),
        );

        // Initialize double buffers for the simulation data
        let initial_life_data = vec![0u32; 2 * SIM_VOXELS];
        let in_out_double_buffers = (0..2)
            .map(|i| {
                create_storage_buffer_with_data(
                    &device,
                    &initial_life_data,
                    Some(&format!("Voxel Engine Storage Buffer {i}")),
                )
            })
            .collect::<Vec<_>>();

        VoxelPipelineBuffers {
            dimentions,
            in_out_double_buffers,
            double_indices: DoubleBufferIndices::default(),
        }
    }
}

/// To help us maintain the current and next buffer indices
#[derive(Clone, Debug)]
pub(in crate::engine) struct DoubleBufferIndices {
    pub current: usize,
    pub next: usize,
}

impl Default for DoubleBufferIndices {
    fn default() -> Self {
        Self {
            current: 0,
            next: 1,
        }
    }
}

impl DoubleBufferIndices {
    // Sould only ever be called once per rendering tick
    pub fn swap(&mut self) {
        self.current = self.next;
        self.next = (self.next + 1) % 2;
    }
}

pub(in crate::engine) fn create_uniform_buffer<T: bytemuck::Pod + bytemuck::Zeroable>(
    device: &RenderDevice,
    data: &[T],
    label: Option<&str>,
) -> Buffer {
    device.create_buffer_with_data(&BufferInitDescriptor {
        label,
        contents: bytemuck::cast_slice(data),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    })
}

pub(in crate::engine) fn create_storage_buffer_with_data<T: bytemuck::Pod + bytemuck::Zeroable>(
    device: &RenderDevice,
    data: &[T],
    label: Option<&str>,
) -> Buffer {
    device.create_buffer_with_data(&BufferInitDescriptor {
        label,
        contents: bytemuck::cast_slice(data),
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
    })
}
