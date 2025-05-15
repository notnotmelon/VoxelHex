use crate::contree::{
    empty_marker,
    types::{
        Albedo, ChunkData, VoxelChildren, VoxelContent, PaletteIndexValues, VoxelData,
    },
    ContreeEntry, V3c, BOX_NODE_CHILDREN_COUNT,
};
use crate::spatial::math::{flat_projection, set_occupied_bitmap_value};
use std::{
    fmt::{Debug, Error, Formatter},
    matches,
};

//####################################################################################
//  ██████   █████    ███████    ██████████   ██████████
// ░░██████ ░░███   ███░░░░░███ ░░███░░░░███ ░░███░░░░░█
//  ░███░███ ░███  ███     ░░███ ░███   ░░███ ░███  █ ░
//  ░███░░███░███ ░███      ░███ ░███    ░███ ░██████
//  ░███ ░░██████ ░███      ░███ ░███    ░███ ░███░░█
//  ░███  ░░█████ ░░███     ███  ░███    ███  ░███ ░   █
//  █████  ░░█████ ░░░███████░   ██████████   ██████████
// ░░░░░    ░░░░░    ░░░░░░░    ░░░░░░░░░░   ░░░░░░░░░░
//    █████████  █████   █████ █████ █████       ██████████   ███████████   ██████████ ██████   █████
//   ███░░░░░███░░███   ░░███ ░░███ ░░███       ░░███░░░░███ ░░███░░░░░███ ░░███░░░░░█░░██████ ░░███
//  ███     ░░░  ░███    ░███  ░███  ░███        ░███   ░░███ ░███    ░███  ░███  █ ░  ░███░███ ░███
// ░███          ░███████████  ░███  ░███        ░███    ░███ ░██████████   ░██████    ░███░░███░███
// ░███          ░███░░░░░███  ░███  ░███        ░███    ░███ ░███░░░░░███  ░███░░█    ░███ ░░██████
// ░░███     ███ ░███    ░███  ░███  ░███      █ ░███    ███  ░███    ░███  ░███ ░   █ ░███  ░░█████
//  ░░█████████  █████   █████ █████ ███████████ ██████████   █████   █████ ██████████ █████  ░░█████
//   ░░░░░░░░░  ░░░░░   ░░░░░ ░░░░░ ░░░░░░░░░░░ ░░░░░░░░░░   ░░░░░   ░░░░░ ░░░░░░░░░░ ░░░░░    ░░░░░
//####################################################################################
impl Debug for VoxelChildren {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match &self {
            VoxelChildren::NoChildren => write!(f, "NodeChildren::NoChildren"),
            VoxelChildren::Children(array) => {
                write!(f, "NodeChildren::Children({:?})", array)
            }
            VoxelChildren::OccupancyBitmap(mask) => {
                write!(f, "NodeChildren::OccupancyBitmap({:#10X})", mask)
            }
        }
    }
}
impl VoxelChildren {
    pub(crate) fn child(&self, sectant: u8) -> usize {
        match &self {
            VoxelChildren::Children(c) => c[sectant as usize] as usize,
            _ => empty_marker(),
        }
    }

    pub(crate) fn child_mut(&mut self, index: usize) -> Option<&mut u32> {
        if let VoxelChildren::NoChildren = self {
            *self = VoxelChildren::Children([empty_marker(); BOX_NODE_CHILDREN_COUNT]);
        }
        match self {
            VoxelChildren::Children(c) => Some(&mut c[index]),
            _ => panic!("Attempted to modify NodeChild[{:?}] of {:?}", index, self),
        }
    }

    /// Provides a slice for iteration, if there are children to iterate on
    pub(crate) fn iter(&self) -> Option<std::slice::Iter<u32>> {
        match &self {
            VoxelChildren::Children(c) => Some(c.iter()),
            _ => None,
        }
    }

    /// Erases content, if any
    pub(crate) fn clear(&mut self, child_index: usize) {
        debug_assert!(child_index < 8);
        if let VoxelChildren::Children(c) = self {
            c[child_index] = empty_marker();
            if 8 == c.iter().filter(|e| **e == empty_marker::<u32>()).count() {
                *self = VoxelChildren::NoChildren;
            }
        }
    }
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
//  ██████████     █████████   ███████████   █████████
// ░░███░░░░███   ███░░░░░███ ░█░░░███░░░█  ███░░░░░███
//  ░███   ░░███ ░███    ░███ ░   ░███  ░  ░███    ░███
//  ░███    ░███ ░███████████     ░███     ░███████████
//  ░███    ░███ ░███░░░░░███     ░███     ░███░░░░░███
//  ░███    ███  ░███    ░███     ░███     ░███    ░███
//  ██████████   █████   █████    █████    █████   █████
// ░░░░░░░░░░   ░░░░░   ░░░░░    ░░░░░    ░░░░░   ░░░░░
//####################################################################################
impl ChunkData {
    /// Calculates the Occupancy bitmap for the given Voxel chunk
    pub(crate) fn calculate_chunk_occupied_bits<V: VoxelData>(
        chunk: &[PaletteIndexValues],
        chunk_dimension: usize,
        color_palette: &[Albedo],
        data_palette: &[V],
    ) -> u64 {
        let mut bitmap = 0;
        for x in 0..chunk_dimension {
            for y in 0..chunk_dimension {
                for z in 0..chunk_dimension {
                    let flat_index = flat_projection(x, y, z, chunk_dimension);
                    if !VoxelContent::pix_points_to_empty(
                        &chunk[flat_index],
                        color_palette,
                        data_palette,
                    ) {
                        set_occupied_bitmap_value(
                            &V3c::new(x, y, z),
                            1,
                            chunk_dimension,
                            true,
                            &mut bitmap,
                        );
                    }
                }
            }
        }
        bitmap
    }

    /// Calculates the occupancy bitmap based on self
    pub(crate) fn calculate_occupied_bits<V: VoxelData>(
        &self,
        chunk_dimension: usize,
        color_palette: &[Albedo],
        data_palette: &[V],
    ) -> u64 {
        match self {
            ChunkData::Empty => 0,
            ChunkData::Solid(voxel) => {
                if VoxelContent::pix_points_to_empty(voxel, color_palette, data_palette) {
                    0
                } else {
                    u64::MAX
                }
            }
            ChunkData::Parted(chunk) => Self::calculate_chunk_occupied_bits(
                chunk,
                chunk_dimension,
                color_palette,
                data_palette,
            ),
        }
    }

    /// In case all contained voxels are the same, returns with a reference to the data
    pub(crate) fn get_homogeneous_data(&self) -> Option<&PaletteIndexValues> {
        match self {
            ChunkData::Empty => None,
            ChunkData::Solid(voxel) => Some(voxel),
            ChunkData::Parted(chunk) => {
                for voxel in chunk.iter() {
                    if *voxel != chunk[0] {
                        return None;
                    }
                }
                Some(&chunk[0])
            }
        }
    }

    pub(crate) fn contains_nothing<V: VoxelData>(
        &self,
        color_palette: &[Albedo],
        data_palette: &[V],
    ) -> bool {
        match self {
            ChunkData::Empty => true,
            ChunkData::Solid(voxel) => {
                VoxelContent::pix_points_to_empty(voxel, color_palette, data_palette)
            }
            ChunkData::Parted(chunk) => {
                for voxel in chunk.iter() {
                    if !VoxelContent::pix_points_to_empty(voxel, color_palette, data_palette) {
                        return false;
                    }
                }
                true
            }
        }
    }

    /// Tries to simplify chunk data, returns true if the view was simplified during function call
    pub(crate) fn simplify<V: VoxelData>(
        &mut self,
        color_palette: &[Albedo],
        data_palette: &[V],
    ) -> bool {
        if let Some(homogeneous_type) = self.get_homogeneous_data() {
            if VoxelContent::pix_points_to_empty(homogeneous_type, color_palette, data_palette) {
                *self = ChunkData::Empty;
            } else {
                *self = ChunkData::Solid(*homogeneous_type);
            }
            true
        } else {
            false
        }
    }
}

//####################################################################################
//  ██████   █████    ███████    ██████████   ██████████
// ░░██████ ░░███   ███░░░░░███ ░░███░░░░███ ░░███░░░░░█
//  ░███░███ ░███  ███     ░░███ ░███   ░░███ ░███  █ ░
//  ░███░░███░███ ░███      ░███ ░███    ░███ ░██████
//  ░███ ░░██████ ░███      ░███ ░███    ░███ ░███░░█
//  ░███  ░░█████ ░░███     ███  ░███    ███  ░███ ░   █
//  █████  ░░█████ ░░░███████░   ██████████   ██████████
// ░░░░░    ░░░░░    ░░░░░░░    ░░░░░░░░░░   ░░░░░░░░░░
//    █████████     ███████    ██████   █████ ███████████ ██████████ ██████   █████ ███████████
//   ███░░░░░███  ███░░░░░███ ░░██████ ░░███ ░█░░░███░░░█░░███░░░░░█░░██████ ░░███ ░█░░░███░░░█
//  ███     ░░░  ███     ░░███ ░███░███ ░███ ░   ░███  ░  ░███  █ ░  ░███░███ ░███ ░   ░███  ░
// ░███         ░███      ░███ ░███░░███░███     ░███     ░██████    ░███░░███░███     ░███
// ░███         ░███      ░███ ░███ ░░██████     ░███     ░███░░█    ░███ ░░██████     ░███
// ░░███     ███░░███     ███  ░███  ░░█████     ░███     ░███ ░   █ ░███  ░░█████     ░███
//  ░░█████████  ░░░███████░   █████  ░░█████    █████    ██████████ █████  ░░█████    █████
//   ░░░░░░░░░     ░░░░░░░    ░░░░░    ░░░░░    ░░░░░    ░░░░░░░░░░ ░░░░░    ░░░░░    ░░░░░
//####################################################################################

impl VoxelContent {
    pub(crate) fn pix_visual(color_index: u16) -> PaletteIndexValues {
        (color_index as u32) | ((empty_marker::<u16>() as u32) << 16)
    }

    pub(crate) fn pix_informal(data_index: u16) -> PaletteIndexValues {
        (empty_marker::<u16>() as u32) | ((data_index as u32) << 16)
    }

    pub(crate) fn pix_complex(color_index: u16, data_index: u16) -> PaletteIndexValues {
        (color_index as u32) | ((data_index as u32) << 16)
    }

    pub(crate) fn pix_color_index(index: &PaletteIndexValues) -> usize {
        (index & 0x0000FFFF) as usize
    }
    pub(crate) fn pix_data_index(index: &PaletteIndexValues) -> usize {
        ((index & 0xFFFF0000) >> 16) as usize
    }

    pub(crate) fn pix_overwrite_color(
        mut index: PaletteIndexValues,
        delta: &PaletteIndexValues,
    ) -> PaletteIndexValues {
        index = (index & 0xFFFF0000) | (delta & 0x0000FFFF);
        index
    }

    pub(crate) fn pix_overwrite_data(
        mut index: PaletteIndexValues,
        delta: &PaletteIndexValues,
    ) -> PaletteIndexValues {
        index = (index & 0x0000FFFF) | (delta & 0xFFFF0000);
        index
    }

    pub(crate) fn pix_color_is_some(index: &PaletteIndexValues) -> bool {
        Self::pix_color_index(index) < empty_marker::<u16>() as usize
    }

    pub(crate) fn pix_color_is_none(index: &PaletteIndexValues) -> bool {
        !Self::pix_color_is_some(index)
    }

    pub(crate) fn pix_data_is_none(index: &PaletteIndexValues) -> bool {
        Self::pix_data_index(index) == empty_marker::<u16>() as usize
    }

    pub(crate) fn pix_data_is_some(index: &PaletteIndexValues) -> bool {
        !Self::pix_data_is_none(index)
    }

    pub(crate) fn pix_points_to_empty<V: VoxelData>(
        index: &PaletteIndexValues,
        color_palette: &[Albedo],
        data_palette: &[V],
    ) -> bool {
        debug_assert!(
            Self::pix_color_index(index) < color_palette.len() || Self::pix_color_is_none(index),
            "Expected color index to be inside bounds: {} <> {}",
            Self::pix_color_index(index),
            color_palette.len()
        );
        debug_assert!(
            Self::pix_data_index(index) < data_palette.len() || Self::pix_data_is_none(index),
            "Expected data
             index to be inside bounds: {} <> {}",
            Self::pix_data_index(index),
            color_palette.len()
        );
        (Self::pix_color_is_none(index)
            || color_palette[Self::pix_color_index(index)].is_transparent())
            && (Self::pix_data_is_none(index)
                || data_palette[Self::pix_data_index(index)].is_empty())
    }

    pub(crate) fn pix_get_ref<'a, V: VoxelData>(
        index: &PaletteIndexValues,
        color_palette: &'a [Albedo],
        data_palette: &'a [V],
    ) -> ContreeEntry<'a, V> {
        if Self::pix_data_is_none(index) && Self::pix_color_is_none(index) {
            return ContreeEntry::Empty;
        }
        if Self::pix_data_is_none(index) {
            debug_assert!(Self::pix_color_is_some(index));
            debug_assert!(Self::pix_color_index(index) < color_palette.len());
            return ContreeEntry::Visual(&color_palette[Self::pix_color_index(index)]);
        }

        if Self::pix_color_is_none(index) {
            debug_assert!(Self::pix_data_is_some(index));
            debug_assert!(Self::pix_data_index(index) < data_palette.len());
            return ContreeEntry::Informative(&data_palette[Self::pix_data_index(index)]);
        }

        debug_assert!(
            Self::pix_color_index(index) < color_palette.len(),
            "Expected data
             index to be inside bounds: {} <> {}",
            Self::pix_color_index(index),
            color_palette.len()
        );
        debug_assert!(
            Self::pix_data_index(index) < data_palette.len(),
            "Expected data
             index to be inside bounds: {} <> {}",
            Self::pix_data_index(index),
            data_palette.len()
        );
        ContreeEntry::Complex(
            &color_palette[Self::pix_color_index(index)],
            &data_palette[Self::pix_data_index(index)],
        )
    }

    /// Returns with true if content doesn't have any data
    pub(crate) fn is_empty<V: VoxelData>(
        &self,
        color_palette: &[Albedo],
        data_palette: &[V],
    ) -> bool {
        match self {
            VoxelContent::UniformLeaf(chunk) => match chunk {
                ChunkData::Empty => true,
                ChunkData::Solid(voxel) => {
                    Self::pix_points_to_empty(voxel, color_palette, data_palette)
                }
                ChunkData::Parted(chunk) => {
                    for voxel in chunk.iter() {
                        if !Self::pix_points_to_empty(voxel, color_palette, data_palette) {
                            return false;
                        }
                    }
                    true
                }
            },
            VoxelContent::Leaf(chunks) => {
                for mat in chunks.iter() {
                    match mat {
                        ChunkData::Empty => {
                            continue;
                        }
                        ChunkData::Solid(voxel) => {
                            if !Self::pix_points_to_empty(voxel, color_palette, data_palette) {
                                return false;
                            }
                        }
                        ChunkData::Parted(chunk) => {
                            for voxel in chunk.iter() {
                                if !Self::pix_points_to_empty(voxel, color_palette, data_palette) {
                                    return false;
                                }
                            }
                        }
                    }
                }
                true
            }
            VoxelContent::Internal(_) => false,
            VoxelContent::Nothing => true,
        }
    }

    /// Returns with true if all contained elements equal the given data
    pub(crate) fn is_all(&self, data: &PaletteIndexValues) -> bool {
        match self {
            VoxelContent::UniformLeaf(chunk) => match chunk {
                ChunkData::Empty => false,
                ChunkData::Solid(voxel) => voxel == data,
                ChunkData::Parted(_chunk) => {
                    if let Some(homogeneous_type) = chunk.get_homogeneous_data() {
                        homogeneous_type == data
                    } else {
                        false
                    }
                }
            },
            VoxelContent::Leaf(chunks) => {
                for mat in chunks.iter() {
                    let chunk_is_all_data = match mat {
                        ChunkData::Empty => false,
                        ChunkData::Solid(voxel) => voxel == data,
                        ChunkData::Parted(_chunk) => {
                            if let Some(homogeneous_type) = mat.get_homogeneous_data() {
                                homogeneous_type == data
                            } else {
                                false
                            }
                        }
                    };
                    if !chunk_is_all_data {
                        return false;
                    }
                }
                true
            }
            VoxelContent::Internal(_) | VoxelContent::Nothing => false,
        }
    }

    pub(crate) fn compare(&self, other: &VoxelContent) -> bool {
        match self {
            VoxelContent::Nothing => matches!(other, VoxelContent::Nothing),
            VoxelContent::Internal(_) => false, // Internal nodes comparison doesn't make sense
            VoxelContent::UniformLeaf(chunk) => {
                if let VoxelContent::UniformLeaf(ochunk) = other {
                    chunk == ochunk
                } else {
                    false
                }
            }
            VoxelContent::Leaf(chunks) => {
                if let VoxelContent::Leaf(ochunks) = other {
                    chunks == ochunks
                } else {
                    false
                }
            }
        }
    }
}
