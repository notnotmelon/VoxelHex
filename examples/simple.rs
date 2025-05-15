use voxelhex::{
    contree::{Albedo, Contree, ContreeEntry, V3c, VoxelData},
    voxel_data,
};

fn main() {
    // To create an empty contree the size and brick dimension needs to be set
    const TREE_SIZE: u32 = 128; // The length of the edges of the cube the BoxTree covers ( number of voxels )
    const BRICK_DIMENSION: u32 = 8; // How big should one "group of voxels" should be refer to docs @Octree::new
                                    // If you have no idea what it should be, 32 is a good reference
    let mut tree: Contree = Contree::new(TREE_SIZE, BRICK_DIMENSION).ok().unwrap();

    // The visual data the contree contains are provided through the ALbedo type
    let voxel_color_red: Albedo = 0xFF0000FF.into(); // RGBA hex codes can be used like this
    let voxel_color_green: Albedo = 0x00FF00FF.into();
    let voxel_color_blue: Albedo = 0x0000FFFF.into();

    // Data can be inserted through a reference to a position inside bounds of the contree
    tree.insert(&V3c::new(0, 0, 0), &voxel_color_red)
        .ok()
        .unwrap();
    tree.insert(&V3c::new(0, 0, 1), &voxel_color_green)
        .ok()
        .unwrap();
    assert_eq!(tree.get(&V3c::new(0, 0, 0)), (&voxel_color_red).into());
    assert_eq!(tree.get(&V3c::new(0, 0, 1)), (&voxel_color_green).into());

    // Don't try to insert fully transparent colors though, it won't work!
    tree.insert(
        &V3c::new(0, 1, 0),
        &Albedo::default()
            .with_red(69)
            .with_green(69)
            .with_blue(69)
            .with_alpha(0),
    )
    .ok()
    .unwrap();
    assert_eq!(tree.get(&V3c::new(0, 1, 0)), ContreeEntry::Empty);

    // To overwrite data, just insert it to the same position
    tree.insert(&V3c::new(0, 0, 0), &voxel_color_green)
        .ok()
        .unwrap();
    assert_eq!(tree.get(&V3c::new(0, 0, 0)), (&voxel_color_green).into());
    assert_eq!(tree.get(&V3c::new(0, 0, 1)), (&voxel_color_green).into());

    // custom data can also be stored inside the contree, e.g. u32 ( most number types by default )
    tree.insert(&V3c::new(0, 1, 1), voxel_data!(&0xBEEF))
        .ok()
        .unwrap();
    assert_eq!(tree.get(&V3c::new(0, 1, 1)), voxel_data!(&0xBEEF).into());

    // it can also be stored next to visual information
    tree.insert(
        &V3c::new(1, 0, 0),
        (&voxel_color_green, &0xBEEF), //BEWARE: not fresh, do not eat
    )
    .ok()
    .unwrap();
    assert_eq!(
        tree.get(&V3c::new(1, 0, 0)),
        (&voxel_color_green, &0xBEEF).into()
    );

    // updating only one component of the voxel is also possible
    tree.update(&V3c::new(1, 0, 0), &voxel_color_red)
        .ok()
        .unwrap();
    assert_eq!(
        tree.get(&V3c::new(1, 0, 0)),
        (&voxel_color_red, &0xBEEF).into()
    );
    tree.update(&V3c::new(1, 0, 0), voxel_data!(&0xFACEFEED))
        .ok()
        .unwrap();
    assert_eq!(
        tree.get(&V3c::new(1, 0, 0)),
        (&voxel_color_red, &0xFACEFEED).into()
    );

    // The below will do nothing
    tree.insert(&V3c::new(1, 0, 0), voxel_data!()).ok().unwrap();
    tree.insert(&V3c::new(1, 0, 0), &Albedo::default())
        .ok()
        .unwrap();

    // use clear instead!
    // There is no way to only clear one component of a voxel,
    // both color and data information will be erased through clear
    tree.clear(&V3c::new(1, 0, 0)).ok().unwrap();
    assert_eq!(tree.get(&V3c::new(1, 0, 0)), voxel_data!());

    // data can also be inserted in bulk!
    tree.insert_at_lod(&V3c::new(0, 0, 0), 16, &voxel_color_blue)
        .ok()
        .unwrap();
    for x in 0..16 {
        for y in 0..16 {
            for z in 0..16 {
                assert_eq!(tree.get(&V3c::new(x, y, z)), (&voxel_color_blue).into());
            }
        }
    }

    // ..or cleared in bulk!
    // Both insert and clear bulk operations update the data until the end of one target node
    // In the below example, most voxels from 5,5,5 until 32,32,32 will be cleared
    // instead of 5,5,5 -/-> 69,69,69
    // But in these cases not every voxel in the area will be updated...
    // This is to be improved in #33
    // If interested, a simple explanation:
    // Bulk updates work on (relatively) simple logic, so in every node
    // An update size(in x,y,z) is calculated inside the node, and the actual update
    // in that particular node is based on min(x,min(y,z))
    // In the below example:
    // - node boundaries: (4,4,6), node size: 2
    // - update position: (5,5,5), update size: 64
    // - update inside the node starts from (5,5,6)
    // - update size in the node would be: (1,1,2)
    // - actual update size is going to be 1, so some parts of the update in Z are left out
    // This is a simple set function, other, more sophisticated tools will be implemented
    // to edit data in the future, so forgive me for letting this slide for now '^^
    tree.clear_at_lod(&V3c::new(5, 5, 5), 64).ok().unwrap();

    // The update size in a bulk operation aligns to node boundaries
    // It sounds a bit tricky at first:
    // - One node contains 64 other nodes
    // - Nodes are packed together into 4x4x4 cubes
    // - A leaf node is the size of 64 voxel bricks, strucutre as above
    // - Any node might be a leaf node
    // - Each node starts at a multiple of its size, which is the smallest corner of it
    // - e.g. a node of size 16 might start at (0,0,0), (16,0,0), (0,16,0), (0,0,32), ... etc..
    tree.clear_at_lod(&V3c::new(0, 0, 0), 32).ok().unwrap();
    for x in 0..32 {
        for y in 0..32 {
            for z in 0..32 {
                assert_eq!(tree.get(&V3c::new(x, y, z)), voxel_data!());
            }
        }
    }

    // You can also use your own data types to be stored in an contree
    // You have to implement some traits(e.g. VoxelData) for it though. See below!
    let _custom_contree: Contree<MyAwesomeData> = Contree::new(8, 2).ok().unwrap();
}

// The trait VoxelData is required in order to differentiate between empty and non-empty contents of a voxel
impl VoxelData for MyAwesomeData {
    fn is_empty(&self) -> bool {
        self.data_field == 69420
    }
}

// To serialize the tree the serde traits are needed
#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

// ..And to be able to save and load the data in the contree, the bendy crate is used.
// The traits below need to be implemented for bytecode serialization
// This is used instead of serde, as the contents are much more thightly packed,
// and there a significant difference in performance
#[cfg(feature = "bytecode")]
use bendy::{
    decoding::{FromBencode, Object},
    encoding::{SingleItemEncoder, ToBencode},
};

#[cfg(feature = "bytecode")]
impl ToBencode for MyAwesomeData {
    const MAX_DEPTH: usize = 1;
    fn encode(&self, encoder: SingleItemEncoder<'_>) -> Result<(), bendy::encoding::Error> {
        encoder.emit_int(self.data_field)
    }
}

#[cfg(feature = "bytecode")]
impl FromBencode for MyAwesomeData {
    fn decode_bencode_object(object: Object<'_, '_>) -> Result<Self, bendy::decoding::Error> {
        match object {
            Object::Integer(i) => Ok(MyAwesomeData {
                data_field: i.parse()?,
            }),
            _ => Err(bendy::decoding::Error::unexpected_field(
                "Expected a single integer from bytestream",
            )),
        }
    }
}

#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
#[derive(Default, Clone, Eq, PartialEq, Hash)]
struct MyAwesomeData {
    data_field: i64,
}
