pub mod types;
pub mod update;

mod detail;
mod node;

pub use crate::spatial::math::vector::{V3c, V3cf32};
pub use types::{
    Albedo, Contree, ContreeEntry, VoxelData,
};

use crate::{
    contree::{
        detail::child_sectant_for,
        types::{ChunkData, VoxelChildren, VoxelContent, ContreeError, PaletteIndexValues},
    },
    object_pool::{empty_marker, ObjectPool},
    spatial::{
        math::{flat_projection, matrix_index_for},
        Cube,
    },
};
use std::{collections::HashMap, hash::Hash};

#[cfg(feature = "serialization")]
use serde::{de::DeserializeOwned, Serialize};

#[cfg(feature = "bytecode")]
use bendy::{decoding::FromBencode, encoding::ToBencode};

//####################################################################################
//     ███████      █████████  ███████████ ███████████   ██████████ ██████████
//   ███░░░░░███   ███░░░░░███░█░░░███░░░█░░███░░░░░███ ░░███░░░░░█░░███░░░░░█
//  ███     ░░███ ███     ░░░ ░   ░███  ░  ░███    ░███  ░███  █ ░  ░███  █ ░
// ░███      ░███░███             ░███     ░██████████   ░██████    ░██████
// ░███      ░███░███             ░███     ░███░░░░░███  ░███░░█    ░███░░█
// ░░███     ███ ░░███     ███    ░███     ░███    ░███  ░███ ░   █ ░███ ░   █
//  ░░░███████░   ░░█████████     █████    █████   █████ ██████████ ██████████
//    ░░░░░░░      ░░░░░░░░░     ░░░░░    ░░░░░   ░░░░░ ░░░░░░░░░░ ░░░░░░░░░░
//  ██████████ ██████   █████ ███████████ ███████████   █████ █████
// ░░███░░░░░█░░██████ ░░███ ░█░░░███░░░█░░███░░░░░███ ░░███ ░░███
//  ░███  █ ░  ░███░███ ░███ ░   ░███  ░  ░███    ░███  ░░███ ███
//  ░██████    ░███░░███░███     ░███     ░██████████    ░░█████
//  ░███░░█    ░███ ░░██████     ░███     ░███░░░░░███    ░░███
//  ░███ ░   █ ░███  ░░█████     ░███     ░███    ░███     ░███
//  ██████████ █████  ░░█████    █████    █████   █████    █████
// ░░░░░░░░░░ ░░░░░    ░░░░░    ░░░░░    ░░░░░   ░░░░░    ░░░░░
//####################################################################################
impl<'a, T: VoxelData> From<(&'a Albedo, &'a T)> for ContreeEntry<'a, T> {
    fn from((albedo, data): (&'a Albedo, &'a T)) -> Self {
        ContreeEntry::Complex(albedo, data)
    }
}

#[macro_export]
macro_rules! voxel_data {
    ($data:expr) => {
        ContreeEntry::Informative($data)
    };
    () => {
        ContreeEntry::Empty
    };
}

impl<'a, T: VoxelData> From<&'a Albedo> for ContreeEntry<'a, T> {
    fn from(albedo: &'a Albedo) -> Self {
        ContreeEntry::Visual(albedo)
    }
}

impl<'a, T: VoxelData> ContreeEntry<'a, T> {
    pub fn albedo(&self) -> Option<&'a Albedo> {
        match self {
            ContreeEntry::Empty => None,
            ContreeEntry::Visual(albedo) => Some(albedo),
            ContreeEntry::Informative(_) => None,
            ContreeEntry::Complex(albedo, _) => Some(albedo),
        }
    }

    pub fn data(&self) -> Option<&'a T> {
        match self {
            ContreeEntry::Empty => None,
            ContreeEntry::Visual(_) => None,
            ContreeEntry::Informative(data) => Some(data),
            ContreeEntry::Complex(_, data) => Some(data),
        }
    }

    pub fn is_none(&self) -> bool {
        match self {
            ContreeEntry::Empty => true,
            ContreeEntry::Visual(albedo) => albedo.is_transparent(),
            ContreeEntry::Informative(data) => data.is_empty(),
            ContreeEntry::Complex(albedo, data) => albedo.is_transparent() && data.is_empty(),
        }
    }

    pub fn is_some(&self) -> bool {
        !self.is_none()
    }
}

//####################################################################################
//     ███████      █████████  ███████████ ███████████   ██████████ ██████████
//   ███░░░░░███   ███░░░░░███░█░░░███░░░█░░███░░░░░███ ░░███░░░░░█░░███░░░░░█
//  ███     ░░███ ███     ░░░ ░   ░███  ░  ░███    ░███  ░███  █ ░  ░███  █ ░
// ░███      ░███░███             ░███     ░██████████   ░██████    ░██████
// ░███      ░███░███             ░███     ░███░░░░░███  ░███░░█    ░███░░█
// ░░███     ███ ░░███     ███    ░███     ░███    ░███  ░███ ░   █ ░███ ░   █
//  ░░░███████░   ░░█████████     █████    █████   █████ ██████████ ██████████
//    ░░░░░░░      ░░░░░░░░░     ░░░░░    ░░░░░   ░░░░░ ░░░░░░░░░░ ░░░░░░░░░░
//####################################################################################
pub(crate) const OOB_SECTANT: u8 = 64;
pub(crate) const BOX_NODE_DIMENSION: usize = 4;
pub(crate) const BOX_NODE_CHILDREN_COUNT: usize = 64;

impl<
        #[cfg(all(feature = "bytecode", feature = "serialization"))] T: FromBencode
            + ToBencode
            + Serialize
            + DeserializeOwned
            + Default
            + Eq
            + Clone
            + Hash
            + VoxelData,
        #[cfg(all(feature = "bytecode", not(feature = "serialization")))] T: FromBencode + ToBencode + Default + Eq + Clone + Hash + VoxelData,
        #[cfg(all(not(feature = "bytecode"), feature = "serialization"))] T: Serialize + DeserializeOwned + Default + Eq + Clone + Hash + VoxelData,
        #[cfg(all(not(feature = "bytecode"), not(feature = "serialization")))] T: Default + Eq + Clone + Hash + VoxelData,
    > Contree<T>
{
    /// converts the data structure to a byte representation
    #[cfg(feature = "bytecode")]
    pub fn to_bytes(&self) -> Vec<u8> {
        self.to_bencode()
            .expect("Failed to serialize Octree to Bytes")
    }

    /// parses the data structure from a byte string
    #[cfg(feature = "bytecode")]
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self::from_bencode(&bytes).expect("Failed to serialize Octree from Bytes")
    }

    /// saves the data structure to the given file path
    #[cfg(feature = "bytecode")]
    pub fn save(&self, path: &str) -> Result<(), std::io::Error> {
        use std::fs::File;
        use std::io::Write;
        let mut file = File::create(path)?;
        file.write_all(&self.to_bytes())?;
        Ok(())
    }

    /// loads the data structure from the given file path
    #[cfg(feature = "bytecode")]
    pub fn load(path: &str) -> Result<Self, std::io::Error> {
        use std::fs::File;
        use std::io::Read;
        let mut file = File::open(path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        Ok(Self::from_bytes(bytes))
    }

    /// creates an contree with the given size
    /// * `chunk_dimension` - must be one of `(2^x)` and smaller than the size of the contree
    /// * `size` - must be `chunk_dimension * (4^x)`, e.g: chunk_dimension == 2 --> size can be 8,32,128...
    pub fn new(size: u32, chunk_dimension: u32) -> Result<Self, ContreeError> {
        if 0 == size || (chunk_dimension as f32).log(2.0).fract() != 0.0 {
            return Err(ContreeError::InvalidChunkDimension(chunk_dimension));
        }
        if chunk_dimension > size
            || 0 == size
            || (size as f32 / chunk_dimension as f32).log(4.0).fract() != 0.0
        {
            return Err(ContreeError::InvalidSize(size));
        }
        if size < (chunk_dimension * BOX_NODE_DIMENSION as u32) {
            return Err(ContreeError::InvalidStructure(
                "Octree size must be larger, than BOX_NODE_DIMENSION * chunk dimension".into(),
            ));
        }
        let node_count_estimation = (size / chunk_dimension).pow(3);
        let mut nodes = ObjectPool::with_capacity(node_count_estimation.min(1024) as usize);
        let root_node_key = nodes.push(VoxelContent::Nothing); // The first element is the root Node
        assert!(root_node_key == 0);
        Ok(Self {
            auto_simplify: true,
            contree_size: size,
            chunk_dim: chunk_dimension,
            nodes,
            node_children: vec![VoxelChildren::default()],
            voxel_color_palette: vec![],
            voxel_data_palette: vec![],
            map_to_color_index_in_palette: HashMap::new(),
            map_to_data_index_in_palette: HashMap::new(),
        })
    }

    /// Getter function for the contree
    /// * Returns immutable reference to the data at the given position, if there is any
    pub fn get(&self, position: &V3c<u32>) -> ContreeEntry<T> {
        VoxelContent::pix_get_ref(
            &self.get_internal(
                Self::ROOT_NODE_KEY as usize,
                Cube::root_bounds(self.contree_size as f32),
                position,
            ),
            &self.voxel_color_palette,
            &self.voxel_data_palette,
        )
    }

    /// Internal Getter function for the contree, to be able to call get from within the tree itself
    /// * Returns immutable reference to the data of the given node at the given position, if there is any
    fn get_internal(
        &self,
        mut current_node_key: usize,
        mut current_bounds: Cube,
        position: &V3c<u32>,
    ) -> PaletteIndexValues {
        let position = V3c::from(*position);
        if !current_bounds.contains(&position) {
            return empty_marker();
        }

        loop {
            match self.nodes.get(current_node_key) {
                VoxelContent::Nothing => return empty_marker(),
                VoxelContent::Leaf(chunks) => {
                    // In case chunk_dimension == contree size, the root node can not be a leaf...
                    debug_assert!(self.chunk_dim < self.contree_size);

                    // Hash the position to the target child
                    let child_sectant_at_position = child_sectant_for(&current_bounds, &position);

                    // If the child exists, query it for the voxel
                    match &chunks[child_sectant_at_position as usize] {
                        ChunkData::Empty => {
                            return empty_marker();
                        }
                        ChunkData::Parted(chunk) => {
                            current_bounds =
                                Cube::child_bounds_for(&current_bounds, child_sectant_at_position);
                            let mat_index = matrix_index_for(
                                &current_bounds,
                                &V3c::from(position),
                                self.chunk_dim,
                            );
                            let mat_index = flat_projection(
                                mat_index.x as usize,
                                mat_index.y as usize,
                                mat_index.z as usize,
                                self.chunk_dim as usize,
                            );
                            if !VoxelContent::pix_points_to_empty(
                                &chunk[mat_index],
                                &self.voxel_color_palette,
                                &self.voxel_data_palette,
                            ) {
                                return chunk[mat_index];
                            }
                            return empty_marker();
                        }
                        ChunkData::Solid(voxel) => {
                            return *voxel;
                        }
                    }
                }
                VoxelContent::UniformLeaf(chunk) => match chunk {
                    ChunkData::Empty => {
                        return empty_marker();
                    }
                    ChunkData::Parted(chunk) => {
                        let mat_index =
                            matrix_index_for(&current_bounds, &V3c::from(position), self.chunk_dim);
                        let mat_index = flat_projection(
                            mat_index.x as usize,
                            mat_index.y as usize,
                            mat_index.z as usize,
                            self.chunk_dim as usize,
                        );
                        return chunk[mat_index];
                    }
                    ChunkData::Solid(voxel) => {
                        return *voxel;
                    }
                },
                VoxelContent::Internal(occupied_bits) => {
                    // Hash the position to the target child
                    let child_sectant_at_position = child_sectant_for(&current_bounds, &position);
                    let child_at_position =
                        self.node_children[current_node_key].child(child_sectant_at_position);

                    // There is a valid child at the given position inside the node, recurse into it
                    if self.nodes.key_is_valid(child_at_position as usize) {
                        debug_assert_ne!(
                            0,
                            occupied_bits & (0x01 << child_sectant_at_position),
                            "Node[{:?}] under {:?} \n has a child(node[{:?}]) in sectant[{:?}](global position: {:?}), which is incompatible with the occupancy bitmap: {:#10X}; \n child node: {:?}; child node children: {:?};",
                            current_node_key,
                            current_bounds,
                            self.node_children[current_node_key].child(child_sectant_at_position),
                            child_sectant_at_position,
                            position, occupied_bits,
                            self.nodes.get(self.node_children[current_node_key].child(child_sectant_at_position)),
                            self.node_children[self.node_children[current_node_key].child(child_sectant_at_position)]
                        );
                        current_node_key = child_at_position as usize;
                        current_bounds =
                            Cube::child_bounds_for(&current_bounds, child_sectant_at_position);
                    } else {
                        return empty_marker();
                    }
                }
            }
        }
    }

    /// Tells the radius of the area covered by the contree
    pub fn get_size(&self) -> u32 {
        self.contree_size
    }
}
