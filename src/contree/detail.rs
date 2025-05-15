use crate::{
    contree::{
        types::{Albedo, Contree, VoxelChildren, VoxelContent, VoxelData},
        ChunkData, Cube, V3c, BOX_NODE_CHILDREN_COUNT, BOX_NODE_DIMENSION,
    },
    object_pool::empty_marker,
    spatial::{
        lut::SECTANT_OFFSET_LUT,
        math::{flat_projection, offset_sectant},
    },
};
use bendy::{decoding::FromBencode, encoding::ToBencode};
use num_traits::Zero;
use std::{
    hash::Hash,
    ops::{Add, Div},
};

/// Returns with the sectant value(i.e. index) of the child for the given position
pub(crate) fn child_sectant_for(bounds: &Cube, position: &V3c<f32>) -> u8 {
    debug_assert!(
        bounds.contains(position),
        "Position {:?}, out of {:?}",
        position,
        bounds
    );
    offset_sectant(&(*position - bounds.min_position), bounds.size)
}

impl<T: Zero + PartialEq> VoxelData for T {
    fn is_empty(&self) -> bool {
        *self == T::zero()
    }
}

//####################################################################################
//     █████████   █████       ███████████  ██████████ ██████████      ███████
//   ███░░░░░███ ░░███       ░░███░░░░░███░░███░░░░░█░░███░░░░███   ███░░░░░███
//  ░███    ░███  ░███        ░███    ░███ ░███  █ ░  ░███   ░░███ ███     ░░███
//  ░███████████  ░███        ░██████████  ░██████    ░███    ░███░███      ░███
//  ░███░░░░░███  ░███        ░███░░░░░███ ░███░░█    ░███    ░███░███      ░███
//  ░███    ░███  ░███      █ ░███    ░███ ░███ ░   █ ░███    ███ ░░███     ███
//  █████   █████ ███████████ ███████████  ██████████ ██████████   ░░░███████░
// ░░░░░   ░░░░░ ░░░░░░░░░░░ ░░░░░░░░░░░  ░░░░░░░░░░ ░░░░░░░░░░      ░░░░░░░
//####################################################################################

impl Albedo {
    pub fn with_red(mut self, r: u8) -> Self {
        self.r = r;
        self
    }

    pub fn with_green(mut self, g: u8) -> Self {
        self.g = g;
        self
    }

    pub fn with_blue(mut self, b: u8) -> Self {
        self.b = b;
        self
    }

    pub fn with_alpha(mut self, a: u8) -> Self {
        self.a = a;
        self
    }

    pub fn is_transparent(&self) -> bool {
        self.a == 0
    }

    pub fn distance_from(&self, other: &Albedo) -> f32 {
        let distance_r = self.r as f32 - other.r as f32;
        let distance_g = self.g as f32 - other.g as f32;
        let distance_b = self.b as f32 - other.b as f32;
        let distance_a = self.a as f32 - other.a as f32;
        (distance_r.powf(2.) + distance_g.powf(2.) + distance_b.powf(2.) + distance_a.powf(2.))
            .sqrt()
    }
}

impl From<u32> for Albedo {
    fn from(value: u32) -> Self {
        let a = (value & 0x000000FF) as u8;
        let b = ((value & 0x0000FF00) >> 8) as u8;
        let g = ((value & 0x00FF0000) >> 16) as u8;
        let r = ((value & 0xFF000000) >> 24) as u8;

        Albedo::default()
            .with_red(r)
            .with_green(g)
            .with_blue(b)
            .with_alpha(a)
    }
}

impl Add for Albedo {
    type Output = Albedo;
    fn add(self, other: Albedo) -> Albedo {
        Albedo {
            r: self.r + other.r,
            g: self.g + other.g,
            b: self.b + other.b,
            a: self.a + other.a,
        }
    }
}

impl Div<f32> for Albedo {
    type Output = Albedo;
    fn div(self, divisor: f32) -> Albedo {
        Albedo {
            r: (self.r as f32 / divisor).round() as u8,
            g: (self.g as f32 / divisor).round() as u8,
            b: (self.b as f32 / divisor).round() as u8,
            a: (self.a as f32 / divisor).round() as u8,
        }
    }
}

impl Zero for Albedo {
    fn zero() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        }
    }
    fn is_zero(&self) -> bool {
        self.is_empty()
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
impl<T> Contree<T>
where
    T: Default + Clone + Eq + Hash + VoxelData,
{
    /// The root node is always the first item
    pub(crate) const ROOT_NODE_KEY: u32 = 0;
}

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
    /// Returns with true if Node is empty at the given target sectant
    pub(crate) fn node_empty_at(&self, node_key: usize, target_sectant: u8) -> bool {
        match self.nodes.get(node_key) {
            VoxelContent::Nothing => true,
            VoxelContent::Leaf(chunks) => match &chunks[target_sectant as usize] {
                ChunkData::Empty => true,
                ChunkData::Solid(voxel) => VoxelContent::pix_points_to_empty(
                    voxel,
                    &self.voxel_color_palette,
                    &self.voxel_data_palette,
                ),
                ChunkData::Parted(_chunk) => {
                    if let Some(data) = chunks[target_sectant as usize].get_homogeneous_data() {
                        VoxelContent::pix_points_to_empty(
                            data,
                            &self.voxel_color_palette,
                            &self.voxel_data_palette,
                        )
                    } else {
                        false
                    }
                }
            },
            VoxelContent::UniformLeaf(chunk) => match chunk {
                ChunkData::Empty => true,
                ChunkData::Solid(voxel) => VoxelContent::pix_points_to_empty(
                    voxel,
                    &self.voxel_color_palette,
                    &self.voxel_data_palette,
                ),
                ChunkData::Parted(chunk) => {
                    let check_start = V3c::from(
                        (SECTANT_OFFSET_LUT[target_sectant as usize] * self.chunk_dim as f32)
                            .floor(),
                    );
                    let check_size =
                        (self.chunk_dim as f32 / BOX_NODE_DIMENSION as f32).max(1.) as usize;
                    for x in check_start.x..(check_start.x + check_size) {
                        for y in check_start.y..(check_start.y + check_size) {
                            for z in check_start.z..(check_start.z + check_size) {
                                if !VoxelContent::pix_points_to_empty(
                                    &chunk[flat_projection(x, y, z, self.chunk_dim as usize)],
                                    &self.voxel_color_palette,
                                    &self.voxel_data_palette,
                                ) {
                                    return false;
                                }
                            }
                        }
                    }
                    true
                }
            },
            VoxelContent::Internal(_occupied_bits) => {
                debug_assert!(
                    !matches!(
                        self.node_children[node_key],
                        VoxelChildren::OccupancyBitmap(_)
                    ),
                    "Expected for internal node to not have OccupancyBitmap as assigned child: {:?}",
                    self.node_children[node_key],
                );
                for child_sectant in 0..BOX_NODE_CHILDREN_COUNT {
                    let child_key = self.node_children[node_key].child(target_sectant);
                    if self.nodes.key_is_valid(child_key)
                        && !self.node_empty_at(child_key, child_sectant as u8)
                    {
                        return false;
                    }
                }
                true
            }
        }
    }

    /// Compares the contents of the given node keys to see if they match
    /// Invalid keys count as empty content
    /// Returns with true if the 2 keys have equivalaent values
    pub(crate) fn compare_nodes(&self, node_key_left: usize, node_key_right: usize) -> bool {
        if self.nodes.key_is_valid(node_key_left) != self.nodes.key_is_valid(node_key_right) {
            return false;
        }

        if self.nodes.key_is_valid(node_key_left) {
            // both keys are valid, compare their contents
            return self
                .nodes
                .get(node_key_left)
                .compare(self.nodes.get(node_key_right));
        }
        true
    }

    /// Subdivides the node into multiple nodes. It guarantees that there will be a child at the target sectant
    /// * `node_key` - The key of the node to subdivide. It must be a leaf
    /// * `target_sectant` - The sectant that must have a child
    pub(crate) fn subdivide_leaf_to_nodes(&mut self, node_key: usize, target_sectant: usize) {
        // Since the node is expected to be a leaf, by default it is supposed that it is fully occupied
        let mut node_content = VoxelContent::Internal(
            if let VoxelChildren::OccupancyBitmap(occupied_bits) = self.node_children[node_key] {
                occupied_bits
            } else {
                panic!(
                    "Expected node to have OccupancyBitmap(_), instead of {:?}",
                    self.node_children[node_key]
                )
            },
        );
        std::mem::swap(&mut node_content, self.nodes.get_mut(node_key));
        let mut node_new_children = [empty_marker(); BOX_NODE_CHILDREN_COUNT];
        match node_content {
            VoxelContent::Nothing | VoxelContent::Internal(_) => {
                panic!("Non-leaf node expected to be Leaf")
            }
            VoxelContent::Leaf(mut chunks) => {
                // All contained chunks shall be converted to leaf nodes
                for sectant in 0..BOX_NODE_CHILDREN_COUNT {
                    let mut chunk = ChunkData::Empty;
                    std::mem::swap(&mut chunk, &mut chunks[sectant]);

                    if !chunk.contains_nothing(&self.voxel_color_palette, &self.voxel_data_palette)
                        || sectant == target_sectant
                    // Push in a new child even if the chunk is empty for the target sectant
                    {
                        // Push in the new(placeholder) child
                        node_new_children[sectant] = self.nodes.push(VoxelContent::Nothing) as u32;
                        // Potentially Resize node children array to accomodate the new child
                        self.node_children.resize(
                            self.node_children
                                .len()
                                .max(node_new_children[sectant] as usize + 1),
                            VoxelChildren::default(),
                        );
                    }

                    match chunk {
                        ChunkData::Empty => {}
                        ChunkData::Solid(voxel) => {
                            // Set the occupancy bitmap for the new leaf child node
                            self.node_children[node_new_children[sectant] as usize] =
                                VoxelChildren::OccupancyBitmap(u64::MAX);
                            *self.nodes.get_mut(node_new_children[sectant] as usize) =
                                VoxelContent::UniformLeaf(ChunkData::Solid(voxel));
                        }
                        ChunkData::Parted(chunk) => {
                            // Calculcate the occupancy bitmap for the new leaf child node
                            // As it is a higher resolution, than the current bitmap, it needs to be bruteforced
                            self.node_children[node_new_children[sectant] as usize] =
                                VoxelChildren::OccupancyBitmap(
                                    chunks[sectant].calculate_occupied_bits(
                                        self.chunk_dim as usize,
                                        &self.voxel_color_palette,
                                        &self.voxel_data_palette,
                                    ),
                                );
                            *self.nodes.get_mut(node_new_children[sectant] as usize) =
                                VoxelContent::UniformLeaf(ChunkData::Parted(chunk.clone()));
                        }
                    }
                }
            }
            VoxelContent::UniformLeaf(chunk) => {
                // The leaf will be divided into 64 chunks, and the contents will be mapped from the current chunk
                match chunk {
                    ChunkData::Empty => {
                        // Push in an empty leaf child to the target sectant ( that will be populated later )
                        // But nothing else to do, as the Uniform leaf is empty!
                        node_new_children[target_sectant] =
                            self.nodes.push(VoxelContent::Nothing) as u32;
                        self.node_children.resize(
                            self.node_children
                                .len()
                                .max(node_new_children[target_sectant] as usize + 1),
                            VoxelChildren::default(),
                        );
                        self.node_children[node_new_children[target_sectant] as usize] =
                            VoxelChildren::OccupancyBitmap(0);
                    }
                    ChunkData::Solid(voxel) => {
                        // Push in all solid children for child sectants
                        for new_child in node_new_children.iter_mut().take(BOX_NODE_CHILDREN_COUNT)
                        {
                            *new_child = self
                                .nodes
                                .push(VoxelContent::UniformLeaf(ChunkData::Solid(voxel)))
                                as u32;
                            self.node_children.resize(
                                self.node_children.len().max(*new_child as usize + 1),
                                VoxelChildren::default(),
                            );
                            self.node_children[*new_child as usize] =
                                VoxelChildren::OccupancyBitmap(u64::MAX);
                        }
                    }
                    ChunkData::Parted(chunk) => {
                        // Each chunk is mapped to take up one subsection of the current data
                        let children_chunks = Self::dilute_chunk_data(chunk, self.chunk_dim);
                        for (sectant, new_chunk) in children_chunks.into_iter().enumerate() {
                            // Push in the new child
                            let child_occupied_bits = ChunkData::calculate_chunk_occupied_bits(
                                &new_chunk,
                                self.chunk_dim as usize,
                                &self.voxel_color_palette,
                                &self.voxel_data_palette,
                            );
                            node_new_children[sectant] = self
                                .nodes
                                .push(VoxelContent::UniformLeaf(ChunkData::Parted(new_chunk)))
                                as u32;

                            // Potentially Resize node children array to accomodate the new child
                            self.node_children.resize(
                                self.node_children
                                    .len()
                                    .max(node_new_children[sectant] as usize + 1),
                                VoxelChildren::default(),
                            );

                            // Set the occupancy bitmap for the new leaf child node
                            self.node_children[node_new_children[sectant] as usize] =
                                VoxelChildren::OccupancyBitmap(child_occupied_bits);
                        }
                    }
                }
            }
        }
        self.node_children[node_key] = VoxelChildren::Children(node_new_children);
    }

    /// Tries to create a chunk from the given node if possible. WARNING: Data loss may occur
    pub(crate) fn try_chunk_from_node(&self, node_key: usize) -> ChunkData {
        if !self.nodes.key_is_valid(node_key) {
            return ChunkData::Empty;
        }
        match self.nodes.get(node_key) {
            VoxelContent::Nothing | VoxelContent::Internal(_) | VoxelContent::Leaf(_) => {
                ChunkData::Empty
            }

            VoxelContent::UniformLeaf(chunk) => chunk.clone(),
        }
    }

    /// Erase all children of the node under the given key, and set its children to "No children"
    pub(crate) fn deallocate_children_of(&mut self, node: usize) {
        if !self.nodes.key_is_valid(node) {
            return;
        }
        let mut to_deallocate = Vec::new();
        if let Some(children) = self.node_children[node].iter() {
            for child in children {
                if self.nodes.key_is_valid(*child as usize) {
                    to_deallocate.push(*child as usize);
                }
            }
            for child in to_deallocate {
                self.deallocate_children_of(child); // Recursion should be fine as depth is not expceted to be more, than 32
                self.nodes.free(child);
                self.node_children[child] = VoxelChildren::NoChildren;
            }
        }
    }

    /// Calculates the occupied bits of a Node; For empty nodes(Nodecontent::Nothing) as well;
    /// As they might be empty by fault and to correct them the occupied bits is required.
    pub(crate) fn stored_occupied_bits(&self, node_key: usize) -> u64 {
        match self.nodes.get(node_key) {
            VoxelContent::Leaf(_) | VoxelContent::UniformLeaf(_) => {
                match self.node_children[node_key] {
                    VoxelChildren::OccupancyBitmap(occupied_bits) => occupied_bits,
                    VoxelChildren::NoChildren => 0,
                    VoxelChildren::Children(children) => {
                        debug_assert!(
                            false,
                            "Expected node[{node_key}] to not have children.\nnode:{:?}\nchildren: {:?}",
                            self.nodes.get(node_key),
                            children
                        );
                        0
                    }
                }
            }
            VoxelContent::Nothing => 0,
            VoxelContent::Internal(occupied_bits) => *occupied_bits,
        }
    }

    /// Stores the given occupied bits for the given node based on key
    pub(crate) fn store_occupied_bits(&mut self, node_key: usize, new_occupied_bits: u64) {
        match self.nodes.get_mut(node_key) {
            VoxelContent::Internal(occupied_bits) => *occupied_bits = new_occupied_bits,
            VoxelContent::Nothing => {
                self.node_children[node_key] = VoxelChildren::OccupancyBitmap(new_occupied_bits)
            }
            VoxelContent::Leaf(_) | VoxelContent::UniformLeaf(_) => {
                match self.node_children[node_key] {
                    VoxelChildren::NoChildren => {
                        self.node_children[node_key] =
                            VoxelChildren::OccupancyBitmap(new_occupied_bits)
                    }
                    VoxelChildren::OccupancyBitmap(ref mut occupied_bits) => {
                        *occupied_bits = new_occupied_bits;
                    }
                    VoxelChildren::Children(_) => panic!(
                        "Expected Leaf node to have OccupancyBitmap instead of {:?}",
                        self.node_children[node_key]
                    ),
                }
            }
        }
    }
}
