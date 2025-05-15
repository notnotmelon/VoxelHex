pub mod clear;
pub mod insert;

#[cfg(test)]
mod tests;

use crate::{
    contree::{
        child_sectant_for,
        types::{ContreeEntry, ChunkData, VoxelChildren, VoxelContent, PaletteIndexValues},
        Albedo, Contree, VoxelData, BOX_NODE_CHILDREN_COUNT, BOX_NODE_DIMENSION,
    },
    object_pool::empty_marker,
    spatial::{
        lut::SECTANT_OFFSET_LUT,
        math::{
            flat_projection, matrix_index_for, octant_in_sectants, offset_sectant, vector::V3c,
        },
        update_size_within, Cube,
    },
};
use num_traits::Zero;
use std::{fmt::Debug, hash::Hash};

#[cfg(feature = "bytecode")]
use bendy::{decoding::FromBencode, encoding::ToBencode};

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
    //####################################################################################
    // ███████████    █████████   █████       ██████████ ███████████ ███████████ ██████████
    // ░░███░░░░░███  ███░░░░░███ ░░███       ░░███░░░░░█░█░░░███░░░█░█░░░███░░░█░░███░░░░░█
    //  ░███    ░███ ░███    ░███  ░███        ░███  █ ░ ░   ░███  ░ ░   ░███  ░  ░███  █ ░
    //  ░██████████  ░███████████  ░███        ░██████       ░███        ░███     ░██████
    //  ░███░░░░░░   ░███░░░░░███  ░███        ░███░░█       ░███        ░███     ░███░░█
    //  ░███         ░███    ░███  ░███      █ ░███ ░   █    ░███        ░███     ░███ ░   █
    //  █████        █████   █████ ███████████ ██████████    █████       █████    ██████████
    // ░░░░░        ░░░░░   ░░░░░ ░░░░░░░░░░░ ░░░░░░░░░░    ░░░░░       ░░░░░    ░░░░░░░░░░
    //####################################################################################
    /// Updates the stored palette by adding the new colors and data in the given entry
    /// Since unused colors are not removed from the palette, possible "pollution" is possible,
    /// where unused colors remain in the palette.
    /// * Returns with the resulting PaletteIndexValues Entry
    pub(crate) fn add_to_palette(&mut self, entry: &ContreeEntry<T>) -> PaletteIndexValues {
        match entry {
            ContreeEntry::Empty => empty_marker::<PaletteIndexValues>(),
            ContreeEntry::Visual(albedo) => {
                if **albedo == Albedo::zero() {
                    return empty_marker();
                }
                let potential_new_albedo_index = self.map_to_color_index_in_palette.keys().len();
                let albedo_index = if let std::collections::hash_map::Entry::Vacant(e) =
                    self.map_to_color_index_in_palette.entry(**albedo)
                {
                    e.insert(potential_new_albedo_index);
                    self.voxel_color_palette.push(**albedo);
                    potential_new_albedo_index
                } else {
                    self.map_to_color_index_in_palette[albedo]
                };
                debug_assert!(
                    albedo_index < u16::MAX as usize,
                    "Albedo color palette overflow!"
                );
                VoxelContent::pix_visual(albedo_index as u16)
            }
            ContreeEntry::Informative(data) => {
                if data.is_empty() {
                    return empty_marker();
                }
                let potential_new_data_index = self.map_to_data_index_in_palette.keys().len();
                let data_index = if let std::collections::hash_map::Entry::Vacant(e) =
                    self.map_to_data_index_in_palette.entry((*data).clone())
                {
                    e.insert(potential_new_data_index);
                    self.voxel_data_palette.push((*data).clone());
                    potential_new_data_index
                } else {
                    self.map_to_data_index_in_palette[data]
                };
                debug_assert!(
                    data_index < u16::MAX as usize,
                    "Data color palette overflow!"
                );
                VoxelContent::pix_informal(data_index as u16)
            }
            ContreeEntry::Complex(albedo, data) => {
                if **albedo == Albedo::zero() {
                    return self.add_to_palette(&ContreeEntry::Informative(*data));
                } else if data.is_empty() {
                    return self.add_to_palette(&ContreeEntry::Visual(albedo));
                }
                let potential_new_albedo_index = self.map_to_color_index_in_palette.keys().len();
                let albedo_index = if let std::collections::hash_map::Entry::Vacant(e) =
                    self.map_to_color_index_in_palette.entry(**albedo)
                {
                    e.insert(potential_new_albedo_index);
                    self.voxel_color_palette.push(**albedo);
                    potential_new_albedo_index
                } else {
                    self.map_to_color_index_in_palette[albedo]
                };
                let potential_new_data_index = self.map_to_data_index_in_palette.keys().len();
                let data_index = if let std::collections::hash_map::Entry::Vacant(e) =
                    self.map_to_data_index_in_palette.entry((*data).clone())
                {
                    e.insert(potential_new_data_index);
                    self.voxel_data_palette.push((*data).clone());
                    potential_new_data_index
                } else {
                    self.map_to_data_index_in_palette[data]
                };
                debug_assert!(
                    albedo_index < u16::MAX as usize,
                    "Albedo color palette overflow!"
                );
                debug_assert!(
                    data_index < u16::MAX as usize,
                    "Data color palette overflow!"
                );
                VoxelContent::pix_complex(albedo_index as u16, data_index as u16)
            }
        }
        // find color in the palette is present, add if not
    }

    //####################################################################################
    //  █████       ██████████   █████████   ███████████
    // ░░███       ░░███░░░░░█  ███░░░░░███ ░░███░░░░░░█
    //  ░███        ░███  █ ░  ░███    ░███  ░███   █ ░
    //  ░███        ░██████    ░███████████  ░███████
    //  ░███        ░███░░█    ░███░░░░░███  ░███░░░█
    //  ░███      █ ░███ ░   █ ░███    ░███  ░███  ░
    //  ███████████ ██████████ █████   █████ █████
    // ░░░░░░░░░░░ ░░░░░░░░░░ ░░░░░   ░░░░░ ░░░░░
    //  █████  █████ ███████████  ██████████     █████████   ███████████ ██████████
    // ░░███  ░░███ ░░███░░░░░███░░███░░░░███   ███░░░░░███ ░█░░░███░░░█░░███░░░░░█
    //  ░███   ░███  ░███    ░███ ░███   ░░███ ░███    ░███ ░   ░███  ░  ░███  █ ░
    //  ░███   ░███  ░██████████  ░███    ░███ ░███████████     ░███     ░██████
    //  ░███   ░███  ░███░░░░░░   ░███    ░███ ░███░░░░░███     ░███     ░███░░█
    //  ░███   ░███  ░███         ░███    ███  ░███    ░███     ░███     ░███ ░   █
    //  ░░████████   █████        ██████████   █████   █████    █████    ██████████
    //   ░░░░░░░░   ░░░░░        ░░░░░░░░░░   ░░░░░   ░░░░░    ░░░░░    ░░░░░░░░░░
    //####################################################################################
    /// Updates the given node to be a Leaf, and inserts the provided data for it.
    /// It will update a whole node, or maximum one chunk. Chunk update range is starting from the position,
    /// goes up to the extent of the chunk. Does not set occupancy bitmap of the given node.
    /// * Returns with the size of the actual update
    pub(crate) fn leaf_update(
        &mut self,
        overwrite_if_empty: bool,
        node_key: usize,
        node_bounds: &Cube,
        target_bounds: &Cube,
        target_child_sectant: usize,
        position: &V3c<u32>,
        size: u32,
        target_content: PaletteIndexValues,
    ) -> usize {
        // Update the leaf node, if it is possible as is, and if it's even needed to update
        // and decide if the node content needs to be divided into chunks, and the update function to be called again
        match self.nodes.get_mut(node_key) {
            VoxelContent::Leaf(chunks) => {
                // In case chunk_dimension == contree size, the 0 can not be a leaf...
                debug_assert!(self.chunk_dim < self.contree_size);
                match &mut chunks[target_child_sectant] {
                    //If there is no chunk in the target position of the leaf, create one
                    ChunkData::Empty => {
                        // Create a new empty chunk at the given sectant
                        let mut new_chunk = vec![
                            empty_marker::<PaletteIndexValues>();
                            self.chunk_dim.pow(3) as usize
                        ];
                        // update the new empty chunk at the given position
                        let update_size = Self::update_chunk(
                            overwrite_if_empty,
                            &mut new_chunk,
                            target_bounds,
                            self.chunk_dim,
                            *position,
                            size,
                            &target_content,
                        );
                        chunks[target_child_sectant] = ChunkData::Parted(new_chunk);
                        update_size
                    }
                    ChunkData::Solid(voxel) => {
                        // In case the data doesn't match the current contents of the node, it needs to be subdivided
                        let update_size;
                        if (VoxelContent::pix_points_to_empty(
                            &target_content,
                            &self.voxel_color_palette,
                            &self.voxel_data_palette,
                        ) && !VoxelContent::pix_points_to_empty(
                            voxel,
                            &self.voxel_color_palette,
                            &self.voxel_data_palette,
                        )) || (!VoxelContent::pix_points_to_empty(
                            &target_content,
                            &self.voxel_color_palette,
                            &self.voxel_data_palette,
                        ) && *voxel != target_content)
                        {
                            // create new chunk and update it at the given position
                            let mut new_chunk = vec![*voxel; self.chunk_dim.pow(3) as usize];
                            update_size = Self::update_chunk(
                                overwrite_if_empty,
                                &mut new_chunk,
                                target_bounds,
                                self.chunk_dim,
                                *position,
                                size,
                                &target_content,
                            );
                            chunks[target_child_sectant] = ChunkData::Parted(new_chunk);
                        } else {
                            // Since the Voxel already equals the data to be set, no need to update anything
                            update_size = 0;
                        }
                        update_size
                    }
                    ChunkData::Parted(chunk) => {
                        // Simply update the chunk at the given position
                        Self::update_chunk(
                            overwrite_if_empty,
                            chunk,
                            target_bounds,
                            self.chunk_dim,
                            *position,
                            size,
                            &target_content,
                        )
                    }
                }
            }
            VoxelContent::UniformLeaf(ref mut mat) => {
                match mat {
                    ChunkData::Empty => {
                        debug_assert_eq!(
                            self.node_children[node_key],
                            VoxelChildren::OccupancyBitmap(0),
                            "Expected Node OccupancyBitmap(0) for empty leaf node instead of {:?}",
                            self.node_children[node_key]
                        );
                        if !VoxelContent::pix_points_to_empty(
                            &target_content,
                            &self.voxel_color_palette,
                            &self.voxel_data_palette,
                        ) {
                            let mut new_leaf_content: [ChunkData;
                                BOX_NODE_CHILDREN_COUNT] =
                                vec![ChunkData::Empty; BOX_NODE_CHILDREN_COUNT]
                                    .try_into()
                                    .unwrap();

                            // Add a chunk to the target sectant and update with the given data
                            let mut new_chunk = vec![
                                self.add_to_palette(&ContreeEntry::Empty);
                                self.chunk_dim.pow(3) as usize
                            ];
                            let update_size = Self::update_chunk(
                                overwrite_if_empty,
                                &mut new_chunk,
                                target_bounds,
                                self.chunk_dim,
                                *position,
                                size,
                                &target_content,
                            );
                            new_leaf_content[target_child_sectant] = ChunkData::Parted(new_chunk);
                            *self.nodes.get_mut(node_key) = VoxelContent::Leaf(new_leaf_content);
                            return update_size;
                        }
                    }
                    ChunkData::Solid(voxel) => {
                        debug_assert!(
                            !VoxelContent::pix_points_to_empty(voxel, &self.voxel_color_palette, &self.voxel_data_palette)
                                && (self.node_children[node_key]
                                    == VoxelChildren::OccupancyBitmap(u64::MAX))
                                || VoxelContent::pix_points_to_empty(voxel, &self.voxel_color_palette, &self.voxel_data_palette)
                                    && (self.node_children[node_key]
                                        == VoxelChildren::OccupancyBitmap(0)),
                            "Expected Node occupancy bitmap({:?}) to align for Solid Voxel Chunk in Uniform Leaf, which is {}",
                            self.node_children[node_key],
                            if VoxelContent::pix_points_to_empty(voxel, &self.voxel_color_palette, &self.voxel_data_palette) {
                                "empty"
                            } else {
                                "not empty"
                            }
                        );

                        // In case the data request doesn't match node content, it needs to be subdivided
                        if VoxelContent::pix_points_to_empty(
                            &target_content,
                            &self.voxel_color_palette,
                            &self.voxel_data_palette,
                        ) && VoxelContent::pix_points_to_empty(
                            voxel,
                            &self.voxel_color_palette,
                            &self.voxel_data_palette,
                        ) {
                            // Data request is to clear, it aligns with the voxel content,
                            // it's enough to update the node content in this case
                            *self.nodes.get_mut(node_key) = VoxelContent::Nothing;
                            return 0;
                        }

                        if !VoxelContent::pix_points_to_empty(
                            &target_content,
                            &self.voxel_color_palette,
                            &self.voxel_data_palette,
                        ) && *voxel != target_content
                            || (VoxelContent::pix_points_to_empty(
                                &target_content,
                                &self.voxel_color_palette,
                                &self.voxel_data_palette,
                            ) && !VoxelContent::pix_points_to_empty(
                                voxel,
                                &self.voxel_color_palette,
                                &self.voxel_data_palette,
                            ))
                        {
                            // Data request doesn't align with the voxel data
                            // create a voxel chunk and try to update with the given data
                            *mat = ChunkData::Parted(vec![
                                *voxel;
                                (self.chunk_dim * self.chunk_dim * self.chunk_dim)
                                    as usize
                            ]);

                            return self.leaf_update(
                                overwrite_if_empty,
                                node_key,
                                node_bounds,
                                target_bounds,
                                target_child_sectant,
                                position,
                                size,
                                target_content,
                            );
                        }

                        // data request aligns with node content
                        return 0;
                    }
                    ChunkData::Parted(chunk) => {
                        // Check if the voxel at the target position matches with the data update request
                        // The target position index is to be calculated from the node bounds,
                        // instead of the target bounds because the position should cover the whole leaf
                        // not just one chunk in it
                        let mat_index = matrix_index_for(node_bounds, position, self.chunk_dim);
                        let mat_index = flat_projection(
                            mat_index.x,
                            mat_index.y,
                            mat_index.z,
                            self.chunk_dim as usize,
                        );
                        if 1 < self.chunk_dim // ChunkData can only stay parted if chunk_dimension is above 1
                            && (
                                (
                                    VoxelContent::pix_points_to_empty(
                                        &target_content,
                                        &self.voxel_color_palette,
                                        &self.voxel_data_palette,
                                    )
                                    && VoxelContent::pix_points_to_empty(
                                        &chunk[mat_index],
                                        &self.voxel_color_palette,
                                        &self.voxel_data_palette
                                    )
                                )||(
                                    !VoxelContent::pix_points_to_empty(
                                        &target_content,
                                        &self.voxel_color_palette,
                                        &self.voxel_data_palette,
                                    )
                                    && chunk[mat_index] == target_content
                                )
                            )
                        {
                            // Target voxel matches with the data request, there's nothing to do!
                            return 0;
                        }

                        // If uniform leaf is the size of one chunk, the chunk is updated as is
                        if node_bounds.size <= self.chunk_dim as f32 && self.chunk_dim > 1 {
                            return Self::update_chunk(
                                overwrite_if_empty,
                                chunk,
                                node_bounds,
                                self.chunk_dim,
                                *position,
                                size,
                                &target_content,
                            );
                        }

                        // the data at the position inside the chunk doesn't match the given data,
                        // so the leaf needs to be divided into a NodeContent::Leaf(chunks)
                        let mut leaf_data: [ChunkData;
                            BOX_NODE_CHILDREN_COUNT] =
                            vec![ChunkData::Empty; BOX_NODE_CHILDREN_COUNT]
                                .try_into()
                                .unwrap();

                        // Each chunk is mapped to take up one subsection of the current data
                        let child_chunks =
                            Self::dilute_chunk_data(std::mem::take(chunk), self.chunk_dim);
                        let mut update_size = 0;
                        for (sectant, mut new_chunk) in child_chunks.into_iter().enumerate() {
                            // Also update the chunk if it is the target
                            if sectant == target_child_sectant {
                                update_size = Self::update_chunk(
                                    overwrite_if_empty,
                                    &mut new_chunk,
                                    target_bounds,
                                    self.chunk_dim,
                                    *position,
                                    size,
                                    &target_content,
                                );
                            }
                            leaf_data[sectant] = ChunkData::Parted(new_chunk);
                        }

                        *self.nodes.get_mut(node_key) = VoxelContent::Leaf(leaf_data);
                        debug_assert_ne!(
                            0, update_size,
                            "Expected Leaf node to be updated in operation"
                        );
                        return update_size;
                    }
                }
                self.leaf_update(
                    overwrite_if_empty,
                    node_key,
                    node_bounds,
                    target_bounds,
                    target_child_sectant,
                    position,
                    size,
                    target_content,
                )
            }
            VoxelContent::Internal(ocbits) => {
                // Warning: Calling leaf update to an internal node might induce data loss - see #69
                self.node_children[node_key] = VoxelChildren::OccupancyBitmap(*ocbits);
                *self.nodes.get_mut(node_key) = VoxelContent::Leaf(
                    (0..BOX_NODE_CHILDREN_COUNT)
                        .map(|sectant| {
                            self.try_chunk_from_node(
                                self.node_children[node_key].child(sectant as u8),
                            )
                        })
                        .collect::<Vec<_>>()
                        .try_into()
                        .unwrap(),
                );
                self.deallocate_children_of(node_key);
                self.leaf_update(
                    overwrite_if_empty,
                    node_key,
                    node_bounds,
                    target_bounds,
                    target_child_sectant,
                    position,
                    size,
                    target_content,
                )
            }
            VoxelContent::Nothing => {
                // Calling leaf update on Nothing is an odd thing to do..
                // But possible, if this call is mid-update
                // So let's try to gather all the information possible
                *self.nodes.get_mut(node_key) = VoxelContent::Leaf(
                    (0..BOX_NODE_CHILDREN_COUNT)
                        .map(|sectant| {
                            self.try_chunk_from_node(
                                self.node_children[node_key].child(sectant as u8),
                            )
                        })
                        .collect::<Vec<_>>()
                        .try_into()
                        .unwrap(),
                );
                self.deallocate_children_of(node_key);
                self.leaf_update(
                    overwrite_if_empty,
                    node_key,
                    node_bounds,
                    target_bounds,
                    target_child_sectant,
                    position,
                    size,
                    target_content,
                )
            }
        }
    }

    /// Calls the given function for every child position inside the given update range
    /// The function is called at least once
    /// * `node_bounds` - The bounds of the updated node
    /// * `position` - The position of the intended update
    /// * `update_size` - Range of the intended update starting from position
    /// * `target_size` - The size of one child inside the updated node
    /// * `fun` - The function to execute
    ///
    /// returns with update size
    fn execute_for_relevant_sectants<F: FnMut(V3c<u32>, u32, u8, &Cube)>(
        node_bounds: &Cube,
        position: &V3c<u32>,
        update_size: u32,
        target_size: f32,
        mut fun: F,
    ) -> usize {
        let children_updated_dimension =
            (update_size_within(node_bounds, position, update_size) as f32 / target_size).ceil()
                as u32;
        for x in 0..children_updated_dimension {
            for y in 0..children_updated_dimension {
                for z in 0..children_updated_dimension {
                    let shifted_position = V3c::from(*position)
                        + V3c::unit(target_size) * V3c::new(x as f32, y as f32, z as f32);
                    let target_child_sectant = child_sectant_for(node_bounds, &shifted_position);
                    let target_bounds = node_bounds.child_bounds_for(target_child_sectant);

                    // In case smaller chunk dimensions, it might happen that one update affects multiple sectants
                    // e.g. when a uniform leaf has a parted chunk of 2x2x2 --> Setting a value in one element
                    // affects multiple sectants. In these cases, the target size is 0.5, and positions
                    // also move inbetween voxels. Logically this is needed for e.g. setting the correct occupied bits
                    // for a given node. The worst case scenario is some cells are given a value multiple times,
                    // which is acceptable for the time being
                    let target_bounds = Cube {
                        min_position: target_bounds.min_position.floor(),
                        size: target_bounds.size.ceil(),
                    };
                    let (position_in_target, update_size_in_target) = if 0 == x && 0 == y && 0 == z
                    {
                        // Update starts from the start position, goes until end of first target cell
                        (
                            *position,
                            update_size_within(&target_bounds, position, update_size),
                        )
                    } else {
                        // Update starts from the start from update position projected onto target bound edge
                        let update_position = V3c::new(
                            position.x.max(target_bounds.min_position.x as u32),
                            position.y.max(target_bounds.min_position.y as u32),
                            position.z.max(target_bounds.min_position.z as u32),
                        );
                        let trimmed_update_vector =
                            *position + V3c::unit(update_size) - update_position;
                        let update_size_left = trimmed_update_vector
                            .x
                            .min(trimmed_update_vector.y)
                            .min(trimmed_update_vector.z);
                        (
                            update_position,
                            update_size_within(&target_bounds, &update_position, update_size_left),
                        )
                    };

                    fun(
                        position_in_target,
                        update_size_in_target,
                        target_child_sectant,
                        &target_bounds,
                    );
                }
            }
        }
        (target_size * children_updated_dimension as f32) as usize
    }

    //####################################################################################
    //  ███████████  ███████████   █████   █████████  █████   ████
    // ░░███░░░░░███░░███░░░░░███ ░░███   ███░░░░░███░░███   ███░
    //  ░███    ░███ ░███    ░███  ░███  ███     ░░░  ░███  ███
    //  ░██████████  ░██████████   ░███ ░███          ░███████
    //  ░███░░░░░███ ░███░░░░░███  ░███ ░███          ░███░░███
    //  ░███    ░███ ░███    ░███  ░███ ░░███     ███ ░███ ░░███
    //  ███████████  █████   █████ █████ ░░█████████  █████ ░░████
    // ░░░░░░░░░░░  ░░░░░   ░░░░░ ░░░░░   ░░░░░░░░░  ░░░░░   ░░░░
    //####################################################################################
    /// Provides an array of chunks, based on the given chunk data, with the same size of the original chunk,
    /// each voxel mapped as the new chunks were the children of the given chunk
    pub(crate) fn dilute_chunk_data<B>(
        chunk_data: Vec<B>,
        chunk_dim: u32,
    ) -> [Vec<B>; BOX_NODE_CHILDREN_COUNT]
    where
        B: Debug + Clone + Copy + PartialEq,
    {
        debug_assert_eq!(chunk_data.len(), chunk_dim.pow(3) as usize);

        if 1 == chunk_dim {
            debug_assert_eq!(chunk_data.len(), 1);
            return vec![chunk_data.clone(); BOX_NODE_CHILDREN_COUNT]
                .try_into()
                .unwrap();
        }

        if 2 == chunk_dim {
            debug_assert_eq!(chunk_data.len(), 8);
            return (0..BOX_NODE_CHILDREN_COUNT)
                .map(|sectant| {
                    vec![chunk_data[octant_in_sectants(sectant)]; chunk_dim.pow(3) as usize]
                })
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();
        };

        debug_assert!(chunk_data.len() <= BOX_NODE_CHILDREN_COUNT);
        let mut result: [Vec<B>; BOX_NODE_CHILDREN_COUNT] = (0..BOX_NODE_CHILDREN_COUNT)
            .map(|sectant| vec![chunk_data[sectant]; chunk_dim.pow(3) as usize])
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        // in case one child can be mapped 1:1 to an element in the chunk
        if 4 == chunk_dim {
            debug_assert_eq!(chunk_data.len(), BOX_NODE_CHILDREN_COUNT);
            return result;
        }

        // Generic case
        // Note: Each value in @result will be overwritten
        for sectant in 0..BOX_NODE_CHILDREN_COUNT {
            // Set the data of the new child
            let chunk_offset: V3c<usize> =
                V3c::from(SECTANT_OFFSET_LUT[sectant] * chunk_dim as f32);
            let new_chunk_flat_offset = flat_projection(
                chunk_offset.x,
                chunk_offset.y,
                chunk_offset.z,
                chunk_dim as usize,
            );
            let mut new_chunk_data =
                vec![chunk_data[new_chunk_flat_offset]; chunk_dim.pow(3) as usize];
            for x in 0..chunk_dim as usize {
                for y in 0..chunk_dim as usize {
                    for z in 0..chunk_dim as usize {
                        if x < BOX_NODE_DIMENSION
                            && y < BOX_NODE_DIMENSION
                            && z < BOX_NODE_DIMENSION
                        {
                            continue;
                        }
                        let new_chunk_flat_offset = flat_projection(x, y, z, chunk_dim as usize);
                        let chunk_flat_offset = flat_projection(
                            chunk_offset.x + x / BOX_NODE_DIMENSION,
                            chunk_offset.y + y / BOX_NODE_DIMENSION,
                            chunk_offset.z + z / BOX_NODE_DIMENSION,
                            chunk_dim as usize,
                        );
                        new_chunk_data[new_chunk_flat_offset] = chunk_data[chunk_flat_offset];
                    }
                }
            }
            result[sectant] = new_chunk_data;
        }
        result
    }

    /// Updates the content of the given chunk and its occupancy bitmap. Each components of mat_index must be smaller, than the size of the chunk.
    /// mat_index + size however need not be in bounds, the function will cut each component to fit inside the chunk.
    /// * `chunk` - mutable reference of the chunk to update
    /// * `mat_index` - the first position to update with the given data
    /// * `size` - the number of elements in x,y,z to update with the given data
    /// * `data` - the data  to update the chunk with. Erases data in case `None`
    /// * Returns with the size of the update
    fn update_chunk(
        overwrite_if_empty: bool,
        chunk: &mut [PaletteIndexValues],
        chunk_bounds: &Cube,
        chunk_dim: u32,
        position: V3c<u32>,
        size: u32,
        data: &PaletteIndexValues,
    ) -> usize {
        debug_assert!(
            chunk_bounds.contains(&(position.into())),
            "Expected position {:?} to be contained in chunk bounds {:?}",
            position,
            chunk_bounds
        );

        let mat_index = matrix_index_for(chunk_bounds, &position, chunk_dim);
        let update_size = (chunk_dim as usize - mat_index.x).min(size as usize);
        for x in mat_index.x..(mat_index.x + size as usize).min(chunk_dim as usize) {
            for y in mat_index.y..(mat_index.y + size as usize).min(chunk_dim as usize) {
                for z in mat_index.z..(mat_index.z + size as usize).min(chunk_dim as usize) {
                    let mat_index = flat_projection(x, y, z, chunk_dim as usize);
                    if overwrite_if_empty {
                        chunk[mat_index] = *data;
                    } else {
                        if VoxelContent::pix_color_is_some(data) {
                            chunk[mat_index] =
                                VoxelContent::pix_overwrite_color(chunk[mat_index], data);
                        }
                        if VoxelContent::pix_data_is_some(data) {
                            chunk[mat_index] =
                                VoxelContent::pix_overwrite_data(chunk[mat_index], data);
                        }
                    }
                }
            }
        }
        update_size
    }

    //####################################################################################
    //   █████████  █████ ██████   ██████ ███████████  █████       █████ ███████████ █████ █████
    //  ███░░░░░███░░███ ░░██████ ██████ ░░███░░░░░███░░███       ░░███ ░░███░░░░░░█░░███ ░░███
    // ░███    ░░░  ░███  ░███░█████░███  ░███    ░███ ░███        ░███  ░███   █ ░  ░░███ ███
    // ░░█████████  ░███  ░███░░███ ░███  ░██████████  ░███        ░███  ░███████     ░░█████
    //  ░░░░░░░░███ ░███  ░███ ░░░  ░███  ░███░░░░░░   ░███        ░███  ░███░░░█      ░░███
    //  ███    ░███ ░███  ░███      ░███  ░███         ░███      █ ░███  ░███  ░        ░███
    // ░░█████████  █████ █████     █████ █████        ███████████ █████ █████          █████
    //  ░░░░░░░░░  ░░░░░ ░░░░░     ░░░░░ ░░░░░        ░░░░░░░░░░░ ░░░░░ ░░░░░          ░░░░░
    //####################################################################################
    /// Updates the given node recursively to collapse nodes with uniform children into a leaf
    /// Returns with true if the given node was simplified
    pub(crate) fn simplify(&mut self, node_key: usize, recursive: bool) -> bool {
        if self.nodes.key_is_valid(node_key) {
            #[cfg(debug_assertions)]
            {
                if let VoxelContent::Internal(ocbits) = self.nodes.get(node_key) {
                    for sectant in 0..BOX_NODE_CHILDREN_COUNT as u8 {
                        if self.node_empty_at(node_key, sectant) {
                            debug_assert_eq!(
                                0,
                                *ocbits & (0x01 << sectant),
                                "Expected node[{:?}] ocbits({:#10X}) to represent child at sectant[{:?}]: \n{:?}",
                                node_key, ocbits, sectant,
                                self.nodes.get(self.node_children[node_key].child(sectant))
                            )
                        }
                    }
                }
            }

            match self.nodes.get_mut(node_key) {
                VoxelContent::Nothing => true,
                VoxelContent::UniformLeaf(chunk) => {
                    debug_assert!(
                        matches!(
                            self.node_children[node_key],
                            VoxelChildren::OccupancyBitmap(_)
                        ),
                        "Uniform leaf has {:?} instead of an Occupancy_bitmap(_)",
                        self.node_children[node_key]
                    );
                    match chunk {
                        ChunkData::Empty => true,
                        ChunkData::Solid(voxel) => {
                            if VoxelContent::pix_points_to_empty(
                                voxel,
                                &self.voxel_color_palette,
                                &self.voxel_data_palette,
                            ) {
                                debug_assert_eq!(
                                    0,
                                    if let VoxelChildren::OccupancyBitmap(occupied_bits) =
                                        self.node_children[node_key]
                                    {
                                        occupied_bits
                                    } else {
                                        0xD34D
                                    },
                                    "Solid empty voxel should have its occupied bits set to 0, instead of {:#10X}",
                                    if let VoxelChildren::OccupancyBitmap(occupied_bits) =
                                        self.node_children[node_key]
                                    {
                                        occupied_bits
                                    } else {
                                        0xD34D
                                    }
                                );
                                *self.nodes.get_mut(node_key) = VoxelContent::Nothing;
                                self.node_children[node_key] = VoxelChildren::NoChildren;
                                true
                            } else {
                                debug_assert_eq!(
                                    u64::MAX,
                                    if let VoxelChildren::OccupancyBitmap(occupied_bits) =
                                        self.node_children[node_key]
                                    {
                                        occupied_bits
                                    } else {
                                        0xD34D
                                    },
                                    "Solid full voxel should have its occupied bits set to u64::MAX, instead of {:#10X}",
                                    if let VoxelChildren::OccupancyBitmap(occupied_bits) =
                                        self.node_children[node_key]
                                    {
                                        occupied_bits
                                    } else {
                                        0xD34D
                                    }
                                );
                                false
                            }
                        }
                        ChunkData::Parted(_chunk) => {
                            if chunk.simplify(&self.voxel_color_palette, &self.voxel_data_palette) {
                                debug_assert!(
                                    self.node_children[node_key]
                                        == VoxelChildren::OccupancyBitmap(u64::MAX)
                                        || self.node_children[node_key]
                                            == VoxelChildren::OccupancyBitmap(0),
                                    "Expected chunk occuped bits( inside {:?}) to be either full or empty, becasue it could be simplified",
                                    self.node_children[node_key]
                                );
                                true
                            } else {
                                false
                            }
                        }
                    }
                }
                VoxelContent::Leaf(chunks) => {
                    #[cfg(debug_assertions)]
                    {
                        for (sectant, chunk) in
                            chunks.iter().enumerate().take(BOX_NODE_CHILDREN_COUNT)
                        {
                            if let ChunkData::Solid(_) | ChunkData::Empty = chunk {
                                // with solid and empty chunks, the relevant occupied bits should either be empty or full
                                if let VoxelChildren::OccupancyBitmap(occupied_bits) =
                                    self.node_children[node_key]
                                {
                                    let sectant_bitmask = 0x01 << sectant;
                                    debug_assert!(
                                        0 == occupied_bits & sectant_bitmask
                                            || sectant_bitmask == occupied_bits & sectant_bitmask,
                                        "Chunkdata at sectant[{:?}] doesn't match occupied bits: {:?} <> {:#10X}",
                                        sectant, chunk, occupied_bits,
                                    );
                                }
                            }
                        }
                    }

                    debug_assert!(
                        matches!(
                            self.node_children[node_key],
                            VoxelChildren::OccupancyBitmap(_),
                        ),
                        "Expected node child to be OccupancyBitmap(_) instead of {:?}",
                        self.node_children[node_key]
                    );

                    // Try to simplify chunks
                    let mut simplified = false;
                    let mut is_leaf_uniform_solid = true;
                    let mut uniform_solid_value = None;

                    for chunk in chunks.iter_mut().take(BOX_NODE_CHILDREN_COUNT) {
                        simplified |=
                            chunk.simplify(&self.voxel_color_palette, &self.voxel_data_palette);

                        if is_leaf_uniform_solid {
                            if let ChunkData::Solid(voxel) = chunk {
                                if let Some(ref uniform_solid_value) = uniform_solid_value {
                                    if *uniform_solid_value != voxel {
                                        is_leaf_uniform_solid = false;
                                    }
                                } else {
                                    uniform_solid_value = Some(voxel);
                                }
                            } else {
                                is_leaf_uniform_solid = false;
                            }
                        }
                    }

                    // Try to unite chunks into a solid chunk
                    let mut unified_chunk = ChunkData::Empty;
                    if is_leaf_uniform_solid {
                        debug_assert_ne!(uniform_solid_value, None);
                        debug_assert_eq!(
                            self.node_children[node_key],
                            VoxelChildren::OccupancyBitmap(u64::MAX),
                            "Expected Leaf with uniform solid value to have u64::MAX value"
                        );
                        *self.nodes.get_mut(node_key) = VoxelContent::UniformLeaf(ChunkData::Solid(
                            *uniform_solid_value.unwrap(),
                        ));
                        return true;
                    }

                    // Do not try to unite chunks into a uniform chunk
                    // since contents are not solid, it is not unifyable
                    // into a 1x1x1 chunk ( that's equivalent to a solid chunk )
                    if self.chunk_dim == 1 {
                        return false;
                    }

                    // Try to unite chunks into a Uniform parted chunk
                    let mut unified_chunk_data =
                        vec![empty_marker::<PaletteIndexValues>(); self.chunk_dim.pow(3) as usize];
                    let mut is_leaf_uniform = true;
                    const CHUNK_CELL_SIZE: usize = BOX_NODE_DIMENSION;
                    let superchunk_size = self.chunk_dim as f32 * BOX_NODE_DIMENSION as f32;
                    'chunk_process: for x in 0..self.chunk_dim {
                        for y in 0..self.chunk_dim {
                            for z in 0..self.chunk_dim {
                                let cell_start =
                                    V3c::new(x as f32, y as f32, z as f32) * CHUNK_CELL_SIZE as f32;
                                let ref_sectant =
                                    offset_sectant(&cell_start, superchunk_size) as usize;
                                let pos_in_child =
                                    cell_start - SECTANT_OFFSET_LUT[ref_sectant] * superchunk_size;
                                let ref_voxel = match &chunks[ref_sectant] {
                                    ChunkData::Empty => empty_marker(),
                                    ChunkData::Solid(voxel) => *voxel,
                                    ChunkData::Parted(chunk) => {
                                        chunk[flat_projection(
                                            pos_in_child.x as usize,
                                            pos_in_child.y as usize,
                                            pos_in_child.z as usize,
                                            self.chunk_dim as usize,
                                        )]
                                    }
                                };

                                for cx in 0..CHUNK_CELL_SIZE {
                                    for cy in 0..CHUNK_CELL_SIZE {
                                        for cz in 0..CHUNK_CELL_SIZE {
                                            if !is_leaf_uniform {
                                                break 'chunk_process;
                                            }
                                            let pos = cell_start
                                                + V3c::new(cx as f32, cy as f32, cz as f32);
                                            let sectant =
                                                offset_sectant(&pos, superchunk_size) as usize;
                                            let pos_in_child =
                                                pos - SECTANT_OFFSET_LUT[sectant] * superchunk_size;

                                            is_leaf_uniform &= match &chunks[sectant] {
                                                ChunkData::Empty => {
                                                    ref_voxel
                                                        == empty_marker::<PaletteIndexValues>()
                                                }
                                                ChunkData::Solid(voxel) => ref_voxel == *voxel,
                                                ChunkData::Parted(chunk) => {
                                                    ref_voxel
                                                        == chunk[flat_projection(
                                                            pos_in_child.x as usize,
                                                            pos_in_child.y as usize,
                                                            pos_in_child.z as usize,
                                                            self.chunk_dim as usize,
                                                        )]
                                                }
                                            };
                                        }
                                    }
                                }
                                // All voxel are the same in this cell! set value in unified chunk
                                unified_chunk_data[flat_projection(
                                    x as usize,
                                    y as usize,
                                    z as usize,
                                    self.chunk_dim as usize,
                                )] = ref_voxel;
                            }
                        }
                    }

                    // chunks can be represented as a uniform parted chunk matrix!
                    if is_leaf_uniform {
                        unified_chunk = ChunkData::Parted(unified_chunk_data);
                        simplified = true;
                    }

                    if !matches!(unified_chunk, ChunkData::Empty) {
                        *self.nodes.get_mut(node_key) = VoxelContent::UniformLeaf(unified_chunk);
                    }

                    simplified
                }
                VoxelContent::Internal(ocbits) => {
                    if 0 == *ocbits
                        || matches!(self.node_children[node_key], VoxelChildren::NoChildren)
                    {
                        if let VoxelContent::Nothing = self.nodes.get(node_key) {
                            return false;
                        }

                        *self.nodes.get_mut(node_key) = VoxelContent::Nothing;
                        return true;
                    }

                    debug_assert!(
                        matches!(self.node_children[node_key], VoxelChildren::Children(_)),
                        "Expected Internal node to have Children instead of {:?}",
                        self.node_children[node_key]
                    );
                    let child_keys =
                        if let VoxelChildren::Children(children) = self.node_children[node_key] {
                            children
                        } else {
                            return false;
                        };

                    // Try to simplify each child of the node
                    if recursive {
                        for child_key in child_keys.iter() {
                            self.simplify(*child_key as usize, true);
                        }
                    }

                    for sectant in 1..BOX_NODE_CHILDREN_COUNT {
                        if !self.compare_nodes(child_keys[0] as usize, child_keys[sectant] as usize)
                        {
                            return false;
                        }
                    }

                    // All children are the same!
                    // make the current node a leaf, erase the children
                    debug_assert!(matches!(
                        self.nodes.get(child_keys[0] as usize),
                        VoxelContent::Leaf(_) | VoxelContent::UniformLeaf(_)
                    ));
                    self.nodes.swap(node_key, child_keys[0] as usize);

                    // Deallocate children, and set correct occupancy bitmap
                    let new_node_children = self.node_children[child_keys[0] as usize];
                    self.deallocate_children_of(node_key);
                    self.node_children[node_key] = new_node_children;

                    // At this point there's no need to call simplify on the new leaf node
                    // because it's been attempted already on the data it copied from
                    true
                }
            }
        } else {
            // can't simplify invalid node
            false
        }
    }
}
