use crate::{contree::BOX_NODE_CHILDREN_COUNT, object_pool::ObjectPool};
use std::{collections::HashMap, error::Error, hash::Hash};

#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

/// error types during usage or creation of the contree
#[derive(Debug)]
pub enum ContreeError {
    /// Octree creation was attempted with an invalid contree size
    InvalidSize(u32),

    /// Octree creation was attempted with an invalid brick dimension
    InvalidBrickDimension(u32),

    /// Octree creation was attempted with an invalid structure parameter ( refer to error )
    InvalidStructure(Box<dyn Error>),

    /// Octree query was attempted with an invalid position
    InvalidPosition { x: u32, y: u32, z: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContreeEntry<'a, T: VoxelData> {
    /// No information available in contree query
    Empty,

    /// Albedo data is available in contree query
    Visual(&'a Albedo),

    /// User data is avaliable in contree query
    Informative(&'a T),

    /// Both user data and color information is available in contree query
    Complex(&'a Albedo, &'a T),
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub(crate) enum BrickData {
    /// Brick is empty
    Empty,

    /// Brick is an NxNxN matrix, size is determined by the parent entity
    Parted(Vec<PaletteIndexValues>),

    /// Brick is a single item T, which takes up the entirety of the brick
    Solid(PaletteIndexValues),
}

#[derive(Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub(crate) enum VoxelContent
{
    /// Node is empty
    #[default]
    Nothing,

    /// Internal node + cache data to store the occupancy of the enclosed nodes
    Internal(u64),

    /// Node contains 64 children, each with their own brickdata
    Leaf([BrickData; BOX_NODE_CHILDREN_COUNT]),

    /// Node has one child, which takes up the entirety of the node with its brick data
    UniformLeaf(BrickData),
}

#[derive(Default, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub(crate) enum VoxelChildren {
    #[default]
    NoChildren,
    Children([u32; BOX_NODE_CHILDREN_COUNT]),
    OccupancyBitmap(u64), // In case of leaf nodes
}

/// Trait for User Defined Voxel Data
pub trait VoxelData {
    /// Determines if the voxel is to be hit by rays in the raytracing algorithms
    fn is_empty(&self) -> bool;
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

pub(crate) type PaletteIndexValues = u32;

/// Sparse 64Tree of Voxel Bricks, where each leaf node contains a brick of voxels.
/// A Brick is a 3 dimensional matrix, each element of it containing a voxel.
/// A Brick can be indexed directly, as opposed to the contree which is essentially a
/// tree-graph where each node has 64 children.
#[cfg_attr(feature = "serialization", derive(Serialize))]
#[derive(Clone)]
pub struct Contree<T = u32>
where
    T: Default + Clone + Eq + Hash,
{
    /// Size of one brick in a leaf node (dim^3)
    pub(crate) brick_dim: u32,

    /// Extent of the contree
    pub(crate) contree_size: u32,

    /// Storing data at each position through palette index values
    pub(crate) nodes: ObjectPool<VoxelContent>,

    /// Node Connections
    pub(crate) node_children: Vec<VoxelChildren>,

    /// The albedo colors used by the contree. Maximum 65535 colors can be used at once
    /// because of a limitation on GPU raytracing, to spare space index values refering the palettes
    /// are stored on 2 Bytes
    pub(crate) voxel_color_palette: Vec<Albedo>, // referenced by @nodes
    pub(crate) voxel_data_palette: Vec<T>, // referenced by @nodes

    /// Cache variable to help find colors inside the color palette
    #[cfg_attr(feature = "serialization", serde(skip_serializing, skip_deserializing))]
    pub(crate) map_to_color_index_in_palette: HashMap<Albedo, usize>,

    /// Cache variable to help find user data in the palette
    #[cfg_attr(feature = "serialization", serde(skip_serializing, skip_deserializing))]
    pub(crate) map_to_data_index_in_palette: HashMap<T, usize>,

    /// Feature flag to enable/disable simplification attempts during contree update operations
    pub auto_simplify: bool,
}
