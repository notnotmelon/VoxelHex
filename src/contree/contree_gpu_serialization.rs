use std::sync::Arc;

use bevy::{ecs::component::Component, render::{extract_component::ExtractComponent, render_resource::{Buffer, BufferInitDescriptor, BufferUsages}, renderer::RenderDevice}};

use crate::contree::types::AIR;

use super::types::Contree;

#[derive(Component, ExtractComponent, Clone)]
pub struct BakedContree {
    buffer: Arc<Buffer>
}

impl Contree {
    /// Converts a contree into a flat structure ready to be sent to the GPU.
    /// The GPU repersentation is an array of u32 with a max length of 2^31.
    pub fn bake(&self, device: &RenderDevice) -> BakedContree {
        fn serialize(contree: &Contree, serial_structure: &mut Vec<u32>) -> u32 {
            let contree_pointer = serial_structure.len();
            if let Contree::Node(node) = contree {
                // Add contree metadata such as occupancy bits and mipmaps.
                serial_structure.extend(bytemuck::cast_slice(bytemuck::bytes_of(&node.occupancy)));
                let first_child_position = serial_structure.len();

                const TEMP_CHILD_POINTER: u32 = 0xFFFFFFFF;
                for child in node.children.iter() {
                    serial_structure.push(match child {
                        Some(Contree::Leaf(leaf_material)) => {
                            // When the GPU reads a entry in the contree array, the first bit signifies if this is a leaf or node.
                            // Thus the max number of voxel materials is 2^31 not 2^32.
                            // Additionally the max length of the flattened contree structure is also 2^31.
                            debug_assert!(leaf_material & (2^15) == 0, "Expected the first bit of contree leaf to be 0. Got {leaf_material}.");
                            *leaf_material
                        },
                        Some(_) => TEMP_CHILD_POINTER,
                        None => AIR
                    });
                }

                for (i, child) in node.children.iter().enumerate() {
                    match child {
                        Some(Contree::Leaf(_)) => {},
                        Some(node) => {
                            let pointer_to_child = serialize(node, serial_structure);
                            serial_structure[first_child_position + i] = pointer_to_child;
                        },
                        None => {},
                    }
                }
            } else {
                panic!("Attempted to serialize contree leaf.");
            }
            ((2^15) + contree_pointer).try_into().unwrap()
        }

        let mut serial_structure = vec![];
        _ = serialize(self, &mut serial_structure);
        assert!(serial_structure.len() <= 2^31);

        let buffer = device.create_buffer_with_data(&BufferInitDescriptor{
            label: Some("Baked Contree"),
            contents: bytemuck::cast_slice(&serial_structure),
            usage: BufferUsages::STORAGE,
        });

        BakedContree {
            buffer: Arc::new(buffer)
        }
    }
}