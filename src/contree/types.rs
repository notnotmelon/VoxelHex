use std::{error::Error, hash::Hash, u64};

#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

/// error types during usage or creation of the contree
#[derive(Debug)]
pub enum ContreeError {
    /// Octree creation was attempted with an invalid structure parameter ( refer to error )
    InvalidStructure(Box<dyn Error>),

    /// Octree query was attempted with an invalid position
    InvalidPosition { x: u32, y: u32, z: u32 },
}

/// Color properties of a voxel
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Albedo {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

pub(crate) type VoxelData = u32;
pub const AIR: VoxelData = 0;

/// Sparse 64Tree of Voxels. Branches indefinitely until reaching a homogenous Contree or air.
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum Contree {
    Leaf(VoxelData),
    Node(ContreeNode)
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct ContreeNode {
    pub(crate) mip: Albedo,
    pub(crate) occupancy: u64,
    pub(crate) children: Box<[Option<Contree>; 64]>,
}

impl Contree {
    /// Subdivides the leaf into multiple identicial nodes. Does nothing if this is not a leaf.
    #[inline]
    fn subdivide(&mut self) {
        let children: [Option<Contree>; 64] = match self {
            Contree::Leaf(_) => {
                let mut children = [
                    None, None, None, None, None, None, None, None,
                    None, None, None, None, None, None, None, None,
                    None, None, None, None, None, None, None, None,
                    None, None, None, None, None, None, None, None,
                    None, None, None, None, None, None, None, None,
                    None, None, None, None, None, None, None, None,
                    None, None, None, None, None, None, None, None,
                    None, None, None, None, None, None, None, None,
                ];
                for i in 0..64 {
                    children[i] = Some(self.clone());
                }
                children
            }
            Contree::Node(_) => return,
        };

        *self = Contree::Node(ContreeNode{
            mip: Albedo { r: 0, g: 0, b: 0, a: 0 },
            occupancy: u64::MAX,
            children: Box::from(children),
        });
    }

    /// Recursively optimizes compaction of children nodes.
    #[inline]
    fn recursive_simplify(&mut self) {
        todo!();
    }

    #[inline]
    fn recalculate_occupancy_bits(&mut self) {
        let mut occupancy = 0;
        let mut homogeneous_material = Some(0);
        match self {
            Contree::Node(node) => {
                let mut bit = 1;
                for node in node.children.iter() {
                    match node {
                        Some(Contree::Leaf(node)) => {
                            let node = *node;
                            if node == AIR {
                                homogeneous_material = None;
                                bit *= 2;
                                continue;
                            } else if homogeneous_material == Some(0) {
                                homogeneous_material = Some(node);
                            } else if homogeneous_material != Some(node) {
                                homogeneous_material = None;
                            }
                            occupancy &= bit;
                        },
                        Some(Contree::Node(_)) => {
                            occupancy &= bit;
                            homogeneous_material = None;
                        },
                        None => {
                            homogeneous_material = None;
                        }
                    }
                    bit *= 2;
                }
                node.occupancy = occupancy;
            },
            Contree::Leaf(_) => return,
        }

        match occupancy {
            0 => {
                *self = Contree::Leaf(AIR);
            },
            u64::MAX => {
                if let Some(homogeneous_material) = homogeneous_material {
                    debug_assert!(homogeneous_material != AIR);
                    *self = Contree::Leaf(homogeneous_material);
                }
            },
            _ => {}
        }
    }

    #[inline]
    fn set_voxel(&mut self, voxel: VoxelData, i: usize) {
        self.subdivide();
        match self {
            Contree::Node(node) => {
                node.children[i] = Some(Contree::Leaf(voxel));
            },
            Contree::Leaf(_) => unreachable!(),
        };
        self.recalculate_occupancy_bits();
    }

    fn set_voxels(&mut self, voxels: [VoxelData; 64]) {
        *self = Contree::Node(ContreeNode{
            mip: Albedo { r: 0, g: 0, b: 0, a: 0 },
            occupancy: u64::MAX,
            children: Box::new(voxels.map(|voxel| Some(Contree::Leaf(voxel)))),
        });
        self.recalculate_occupancy_bits();
    }
}
