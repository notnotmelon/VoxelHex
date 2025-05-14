use crate::boxtree::types::PaletteIndexValues;
use crate::boxtree::BOX_NODE_CHILDREN_COUNT;
use crate::boxtree::{
    types::{BrickData, VoxelChildren, VoxelContent},
    Color, Contree,
};
use crate::object_pool::ObjectPool;
use bendy::{
    decoding::{FromBencode, Object},
    encoding::{Error as BencodeError, SingleItemEncoder, ToBencode},
};
use std::{collections::HashMap, hash::Hash};

//####################################################################################
//  █████   █████    ███████    █████ █████ ██████████ █████
// ░░███   ░░███   ███░░░░░███ ░░███ ░░███ ░░███░░░░░█░░███
//  ░███    ░███  ███     ░░███ ░░███ ███   ░███  █ ░  ░███
//  ░███    ░███ ░███      ░███  ░░█████    ░██████    ░███
//  ░░███   ███  ░███      ░███   ███░███   ░███░░█    ░███
//   ░░░█████░   ░░███     ███   ███ ░░███  ░███ ░   █ ░███      █
//     ░░███      ░░░███████░   █████ █████ ██████████ ███████████
//      ░░░         ░░░░░░░    ░░░░░ ░░░░░ ░░░░░░░░░░ ░░░░░░░░░░░
//    █████████     ███████    ██████   █████ ███████████ ██████████ ██████   █████ ███████████
//   ███░░░░░███  ███░░░░░███ ░░██████ ░░███ ░█░░░███░░░█░░███░░░░░█░░██████ ░░███ ░█░░░███░░░█
//  ███     ░░░  ███     ░░███ ░███░███ ░███ ░   ░███  ░  ░███  █ ░  ░███░███ ░███ ░   ░███  ░
// ░███         ░███      ░███ ░███░░███░███     ░███     ░██████    ░███░░███░███     ░███
// ░███         ░███      ░███ ░███ ░░██████     ░███     ░███░░█    ░███ ░░██████     ░███
// ░░███     ███░░███     ███  ░███  ░░█████     ░███     ░███ ░   █ ░███  ░░█████     ░███
//  ░░█████████  ░░░███████░   █████  ░░█████    █████    ██████████ █████  ░░█████    █████
//   ░░░░░░░░░     ░░░░░░░    ░░░░░    ░░░░░    ░░░░░    ░░░░░░░░░░ ░░░░░    ░░░░░    ░░░░░
//####################################################################################
impl ToBencode for Color {
    const MAX_DEPTH: usize = 2;
    fn encode(&self, encoder: SingleItemEncoder) -> Result<(), BencodeError> {
        encoder.emit_list(|e| {
            e.emit(self.r)?;
            e.emit(self.g)?;
            e.emit(self.b)?;
            e.emit(self.a)
        })
    }
}

impl FromBencode for Color {
    fn decode_bencode_object(data: Object) -> Result<Self, bendy::decoding::Error> {
        match data {
            Object::List(mut list) => {
                let r = match list.next_object()?.unwrap() {
                    Object::Integer(i) => Ok(i.parse()?),
                    _ => Err(bendy::decoding::Error::unexpected_token(
                        "int field red color component",
                        "Something else",
                    )),
                }?;
                let g = match list.next_object()?.unwrap() {
                    Object::Integer(i) => Ok(i.parse()?),
                    _ => Err(bendy::decoding::Error::unexpected_token(
                        "int field green color component",
                        "Something else",
                    )),
                }?;
                let b = match list.next_object()?.unwrap() {
                    Object::Integer(i) => Ok(i.parse()?),
                    _ => Err(bendy::decoding::Error::unexpected_token(
                        "int field blue color component",
                        "Something else",
                    )),
                }?;
                let a = match list.next_object()?.unwrap() {
                    Object::Integer(i) => Ok(i.parse()?),
                    _ => Err(bendy::decoding::Error::unexpected_token(
                        "int field alpha color component",
                        "Something else",
                    )),
                }?;
                Ok(Self { r, g, b, a })
            }
            _ => Err(bendy::decoding::Error::unexpected_token("List", "not List")),
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
//####################################################################################
impl ToBencode for BrickData {
    const MAX_DEPTH: usize = 3;

    fn encode(&self, encoder: SingleItemEncoder) -> Result<(), BencodeError> {
        match self {
            BrickData::Empty => encoder.emit_str("#b"),
            BrickData::Solid(voxel) => encoder.emit_list(|e| {
                e.emit_str("#b#")?;
                e.emit(voxel)
            }),
            BrickData::Parted(brick) => encoder.emit_list(|e| {
                e.emit_str("##b#")?;
                e.emit_int(brick.len())?;
                for voxel in brick.iter() {
                    e.emit(voxel)?;
                }
                e.emit_str("#")?;
                Ok(())
            }),
        }
    }
}

impl FromBencode for BrickData {
    fn decode_bencode_object(data: Object) -> Result<Self, bendy::decoding::Error> {
        match data {
            Object::Bytes(b) => {
                debug_assert_eq!(
                    String::from_utf8(b.to_vec())
                        .unwrap_or("".to_string())
                        .as_str(),
                    "#b"
                );
                Ok(BrickData::Empty)
            }
            Object::List(mut list) => {
                let is_solid = match list.next_object()?.unwrap() {
                    Object::Bytes(b) => {
                        match String::from_utf8(b.to_vec())
                            .unwrap_or("".to_string())
                            .as_str()
                        {
                            "#b#" => Ok(true),   // The content is a single voxel
                            "##b#" => Ok(false), // The content is a brick of voxels
                            misc => Err(bendy::decoding::Error::unexpected_token(
                                "A NodeContent Identifier string, which is either # or ##",
                                "The string ".to_owned() + misc,
                            )),
                        }
                    }
                    _ => Err(bendy::decoding::Error::unexpected_token(
                        "BrickData string identifier",
                        "Something else",
                    )),
                }?;
                if is_solid {
                    Ok(BrickData::Solid(PaletteIndexValues::decode_bencode_object(
                        list.next_object()?.unwrap(),
                    )?))
                } else {
                    let len = match list.next_object()?.unwrap() {
                        Object::Integer(i) => Ok(i.parse()?),
                        _ => Err(bendy::decoding::Error::unexpected_token(
                            "int field brick length",
                            "Something else",
                        )),
                    }?;
                    debug_assert!(0 < len, "Expected brick to be of non-zero length!");
                    let mut brick_data = Vec::with_capacity(len as usize);
                    for _ in 0..len {
                        brick_data.push(PaletteIndexValues::decode_bencode_object(list.next_object()?.unwrap())?);
                    }
                    Ok(BrickData::Parted(brick_data))
                }
            }
            _ => Err(bendy::decoding::Error::unexpected_token(
                "A NodeContent Object, either a List or a ByteString",
                "Something else",
            )),
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
impl ToBencode for VoxelContent where {
    const MAX_DEPTH: usize = 8;
    fn encode(&self, encoder: SingleItemEncoder) -> Result<(), BencodeError> {
        match self {
            VoxelContent::Nothing => encoder.emit_str("#"),
            VoxelContent::Internal(occupied_bits) => encoder.emit_list(|e| {
                e.emit_str("##")?;
                e.emit_int(*occupied_bits)
            }),
            VoxelContent::Leaf(bricks) => encoder.emit_list(|e| {
                e.emit_str("###")?;
                for brick in bricks.iter().take(BOX_NODE_CHILDREN_COUNT) {
                    e.emit(brick.clone())?;
                }
                Ok(())
            }),
            VoxelContent::UniformLeaf(brick) => encoder.emit_list(|e| {
                e.emit_str("##u#")?;
                e.emit(brick.clone())
            }),
        }
    }
}

impl FromBencode for VoxelContent {
    fn decode_bencode_object(data: Object) -> Result<Self, bendy::decoding::Error> {
        match data {
            Object::List(mut list) => {
                let (is_leaf, is_uniform) = match list.next_object()?.unwrap() {
                    Object::Bytes(b) => {
                        match String::from_utf8(b.to_vec())
                            .unwrap_or("".to_string())
                            .as_str()
                        {
                            "##" => {
                                // The content is an internal Node
                                Ok((false, false))
                            }
                            "###" => {
                                // The content is a leaf
                                Ok((true, false))
                            }
                            "##u#" => {
                                // The content is a uniform leaf
                                Ok((true, true))
                            }
                            misc => Err(bendy::decoding::Error::unexpected_token(
                                "A NodeContent Identifier string, which is either # or ##",
                                "The string ".to_owned() + misc,
                            )),
                        }
                    }
                    _ => Err(bendy::decoding::Error::unexpected_token(
                        "A NodeContent Identifier, which is a string",
                        "Something else",
                    )),
                }?;

                if !is_leaf && !is_uniform {
                    let occupied_bits;
                    match list.next_object()?.unwrap() {
                        Object::Integer(i) => occupied_bits = i.parse()?,
                        _ => {
                            return Err(bendy::decoding::Error::unexpected_token(
                                "int field for Internal Node Occupancy bitmap",
                                "Something else",
                            ))
                        }
                    };
                    return Ok(VoxelContent::Internal(occupied_bits));
                }

                if is_leaf && !is_uniform {
                    let leaf_data: [BrickData; BOX_NODE_CHILDREN_COUNT] = (0
                        ..BOX_NODE_CHILDREN_COUNT)
                        .map(|_sectant| {
                            BrickData::decode_bencode_object(
                                list.next_object()
                                    .expect("Expected BrickData object:")
                                    .unwrap(),
                            )
                            .expect("Expected to decode BrickData:")
                        })
                        .collect::<Vec<_>>()
                        .try_into()
                        .unwrap();

                    return Ok(VoxelContent::Leaf(leaf_data));
                }

                if is_leaf && is_uniform {
                    return Ok(VoxelContent::UniformLeaf(BrickData::decode_bencode_object(
                        list.next_object()?.unwrap(),
                    )?));
                }
                panic!(
                    "The logical combination of !is_leaf and is_uniform should never be reached"
                );
            }
            Object::Bytes(b) => {
                assert!(String::from_utf8(b.to_vec()).unwrap_or("".to_string()) == "#");
                Ok(VoxelContent::Nothing)
            }
            _ => Err(bendy::decoding::Error::unexpected_token(
                "A NodeContent Object, either a List or a ByteString",
                "Something else",
            )),
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
//    █████████  █████   █████ █████ █████       ██████████   ███████████   ██████████ ██████   █████
//   ███░░░░░███░░███   ░░███ ░░███ ░░███       ░░███░░░░███ ░░███░░░░░███ ░░███░░░░░█░░██████ ░░███
//  ███     ░░░  ░███    ░███  ░███  ░███        ░███   ░░███ ░███    ░███  ░███  █ ░  ░███░███ ░███
// ░███          ░███████████  ░███  ░███        ░███    ░███ ░██████████   ░██████    ░███░░███░███
// ░███          ░███░░░░░███  ░███  ░███        ░███    ░███ ░███░░░░░███  ░███░░█    ░███ ░░██████
// ░░███     ███ ░███    ░███  ░███  ░███      █ ░███    ███  ░███    ░███  ░███ ░   █ ░███  ░░█████
//  ░░█████████  █████   █████ █████ ███████████ ██████████   █████   █████ ██████████ █████  ░░█████
//   ░░░░░░░░░  ░░░░░   ░░░░░ ░░░░░ ░░░░░░░░░░░ ░░░░░░░░░░   ░░░░░   ░░░░░ ░░░░░░░░░░ ░░░░░    ░░░░░
//####################################################################################
// using generic arguments means the default key needs to be serialzied along with the data, which means a lot of wasted space..
// so serialization for the current ObjectPool key is adequate; The engineering hour cost of implementing new serialization logic
// every time the ObjectPool::Itemkey type changes is acepted.
impl ToBencode for VoxelChildren {
    const MAX_DEPTH: usize = 2;
    fn encode(&self, encoder: SingleItemEncoder) -> Result<(), BencodeError> {
        match &self {
            VoxelChildren::Children(c) => encoder.emit_list(|e| {
                e.emit_str("##c##")?;
                for child in c.iter().take(BOX_NODE_CHILDREN_COUNT) {
                    e.emit(child)?;
                }
                Ok(())
            }),
            VoxelChildren::NoChildren => encoder.emit_str("##x##"),
            VoxelChildren::OccupancyBitmap(map) => encoder.emit_list(|e| {
                e.emit_str("##b##")?;
                e.emit(map)
            }),
        }
    }
}

impl FromBencode for VoxelChildren {
    fn decode_bencode_object(data: Object) -> Result<Self, bendy::decoding::Error> {
        match data {
            Object::List(mut list) => {
                let marker = String::decode_bencode_object(list.next_object()?.unwrap())?;
                match marker.as_str() {
                    "##c##" => {
                        let mut c = Vec::new();
                        for _ in 0..BOX_NODE_CHILDREN_COUNT {
                            c.push(
                                u32::decode_bencode_object(list.next_object()?.unwrap())
                                    .ok()
                                    .unwrap(),
                            );
                        }
                        Ok(VoxelChildren::Children(c.try_into().ok().unwrap()))
                    }
                    "##b##" => Ok(VoxelChildren::OccupancyBitmap(u64::decode_bencode_object(
                        list.next_object()?.unwrap(),
                    )?)),
                    s => Err(bendy::decoding::Error::unexpected_token(
                        "A NodeChildren marker, either ##b## or ##c##",
                        s,
                    )),
                }
            }
            Object::Bytes(b) => {
                debug_assert_eq!(
                    String::from_utf8(b.to_vec())
                        .unwrap_or("".to_string())
                        .as_str(),
                    "##x##"
                );
                Ok(VoxelChildren::default())
            }
            _ => Err(bendy::decoding::Error::unexpected_token(
                "A NodeChildren Object, Either a List or a ByteString",
                "Something else",
            )),
        }
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
impl<T> ToBencode for Contree<T>
where
    T: ToBencode + Default + Clone + Eq + Hash,
{
    const MAX_DEPTH: usize = 10;
    fn encode(&self, encoder: SingleItemEncoder) -> Result<(), BencodeError> {
        encoder.emit_list(|e| {
            e.emit_int(self.auto_simplify as u8)?;
            e.emit_int(self.contree_size)?;
            e.emit_int(self.brick_dim)?;
            e.emit(&self.nodes)?;
            e.emit(&self.node_children)?;
            e.emit(&self.voxel_color_palette)?;
            e.emit(&self.voxel_data_palette)?;
            Ok(())
        })
    }
}

impl<T> FromBencode for Contree<T>
where
    T: FromBencode + Default + Clone + Eq + Hash,
{
    fn decode_bencode_object(data: Object) -> Result<Self, bendy::decoding::Error> {
        match data {
            Object::List(mut list) => {
                let auto_simplify = match list.next_object()?.unwrap() {
                    Object::Integer("0") => Ok(false),
                    Object::Integer("1") => Ok(true),
                    Object::Integer(i) => Err(bendy::decoding::Error::unexpected_token(
                        "boolean field auto_simplify",
                        format!("the number: {}", i),
                    )),
                    _ => Err(bendy::decoding::Error::unexpected_token(
                        "boolean field auto_simplify",
                        "Something else",
                    )),
                }?;

                let boxtree_size = match list.next_object()?.unwrap() {
                    Object::Integer(i) => Ok(i.parse()?),
                    _ => Err(bendy::decoding::Error::unexpected_token(
                        "int field boxtree_size",
                        "Something else",
                    )),
                }?;

                let brick_dim = match list.next_object()?.unwrap() {
                    Object::Integer(i) => Ok(i.parse()?),
                    _ => Err(bendy::decoding::Error::unexpected_token(
                        "int field boxtree_size",
                        "Something else",
                    )),
                }?;

                let nodes = ObjectPool::decode_bencode_object(list.next_object()?.unwrap())?;
                let node_children = Vec::decode_bencode_object(list.next_object()?.unwrap())?;

                let voxel_color_palette =
                    Vec::<Color>::decode_bencode_object(list.next_object()?.unwrap())?;
                let mut map_to_color_index_in_palette = HashMap::new();
                for (i, voxel_color) in voxel_color_palette.iter().enumerate() {
                    map_to_color_index_in_palette.insert(*voxel_color, i);
                }

                let voxel_data_palette =
                    Vec::<T>::decode_bencode_object(list.next_object()?.unwrap())?;
                let mut map_to_data_index_in_palette = HashMap::new();
                for (i, voxel_data) in voxel_data_palette.iter().enumerate() {
                    map_to_data_index_in_palette.insert(voxel_data.clone(), i);
                }

                Ok(Self {
                    auto_simplify,
                    contree_size: boxtree_size,
                    brick_dim,
                    nodes,
                    node_children,
                    voxel_color_palette,
                    voxel_data_palette,
                    map_to_color_index_in_palette,
                    map_to_data_index_in_palette,
                })
            }
            _ => Err(bendy::decoding::Error::unexpected_token("List", "not List")),
        }
    }
}
