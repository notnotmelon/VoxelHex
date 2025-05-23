use voxelhex::spatial::math::vector::{V3c, V3cf32};

pub(crate) const OOB_SECTANT: u8 = 64;
pub(crate) const BOX_NODE_DIMENSION: usize = 4;
pub(crate) const BOX_NODE_CHILDREN_COUNT: usize = 64;

pub(crate) fn flat_projection(x: usize, y: usize, z: usize, size: usize) -> usize {
    x + (y * size) + (z * size * size)
}

pub(crate) fn hash_region(offset: &V3c<f32>, size: f32) -> u8 {
    // Scale to 0..BOX_NODE_DIMENSION, then project to an unique index
    debug_assert!(
        offset.x <= size
            && offset.y <= size
            && offset.z <= size
            && offset.x >= 0.
            && offset.y >= 0.
            && offset.z >= 0.,
        "Expected relative offset {:?} to be inside {size}^3",
        offset
    );
    let index: V3c<usize> = (*offset * BOX_NODE_DIMENSION as f32 / size).floor().into();
    flat_projection(index.x, index.y, index.z, BOX_NODE_DIMENSION) as u8
}

#[rustfmt::skip]
pub(crate) const SECTANT_OFFSET_REGION_LUT: [V3cf32; 64] = [
    V3c { x: 0.0, y: 0.0, z: 0.0 }, V3c { x: 0.25, y: 0.0, z: 0.0 }, V3c { x: 0.5, y: 0.0, z: 0.0 }, V3c { x: 0.75, y: 0.0, z: 0.0 },
    V3c { x: 0.0, y: 0.25, z: 0.0 }, V3c { x: 0.25, y: 0.25, z: 0.0 }, V3c { x: 0.5, y: 0.25, z: 0.0 }, V3c { x: 0.75, y: 0.25, z: 0.0 },
    V3c { x: 0.0, y: 0.5, z: 0.0 }, V3c { x: 0.25, y: 0.5, z: 0.0 }, V3c { x: 0.5, y: 0.5, z: 0.0 }, V3c { x: 0.75, y: 0.5, z: 0.0 },
    V3c { x: 0.0, y: 0.75, z: 0.0 }, V3c { x: 0.25, y: 0.75, z: 0.0 }, V3c { x: 0.5, y: 0.75, z: 0.0 }, V3c { x: 0.75, y: 0.75, z: 0.0 },

    V3c { x: 0.0, y: 0.0, z: 0.25 }, V3c { x: 0.25, y: 0.0, z: 0.25 }, V3c { x: 0.5, y: 0.0, z: 0.25 }, V3c { x: 0.75, y: 0.0, z: 0.25 },
    V3c { x: 0.0, y: 0.25, z: 0.25 }, V3c { x: 0.25, y: 0.25, z: 0.25 }, V3c { x: 0.5, y: 0.25, z: 0.25 }, V3c { x: 0.75, y: 0.25, z: 0.25 },
    V3c { x: 0.0, y: 0.5, z: 0.25 }, V3c { x: 0.25, y: 0.5, z: 0.25 }, V3c { x: 0.5, y: 0.5, z: 0.25 }, V3c { x: 0.75, y: 0.5, z: 0.25 },
    V3c { x: 0.0, y: 0.75, z: 0.25 }, V3c { x: 0.25, y: 0.75, z: 0.25 }, V3c { x: 0.5, y: 0.75, z: 0.25 }, V3c { x: 0.75, y: 0.75, z: 0.25 },

    V3c { x: 0.0, y: 0.0, z: 0.5 }, V3c { x: 0.25, y: 0.0, z: 0.5 }, V3c { x: 0.5, y: 0.0, z: 0.5 }, V3c { x: 0.75, y: 0.0, z: 0.5 },
    V3c { x: 0.0, y: 0.25, z: 0.5 }, V3c { x: 0.25, y: 0.25, z: 0.5 }, V3c { x: 0.5, y: 0.25, z: 0.5 }, V3c { x: 0.75, y: 0.25, z: 0.5 },
    V3c { x: 0.0, y: 0.5, z: 0.5 }, V3c { x: 0.25, y: 0.5, z: 0.5 }, V3c { x: 0.5, y: 0.5, z: 0.5 }, V3c { x: 0.75, y: 0.5, z: 0.5 },
    V3c { x: 0.0, y: 0.75, z: 0.5 }, V3c { x: 0.25, y: 0.75, z: 0.5 }, V3c { x: 0.5, y: 0.75, z: 0.5 }, V3c { x: 0.75, y: 0.75, z: 0.5 },

    V3c { x: 0.0, y: 0.0, z: 0.75 }, V3c { x: 0.25, y: 0.0, z: 0.75 }, V3c { x: 0.5, y: 0.0, z: 0.75 }, V3c { x: 0.75, y: 0.0, z: 0.75 },
    V3c { x: 0.0, y: 0.25, z: 0.75 }, V3c { x: 0.25, y: 0.25, z: 0.75 }, V3c { x: 0.5, y: 0.25, z: 0.75 }, V3c { x: 0.75, y: 0.25, z: 0.75 },
    V3c { x: 0.0, y: 0.5, z: 0.75 }, V3c { x: 0.25, y: 0.5, z: 0.75 }, V3c { x: 0.5, y: 0.5, z: 0.75 }, V3c { x: 0.75, y: 0.5, z: 0.75 },
    V3c { x: 0.0, y: 0.75, z: 0.75 }, V3c { x: 0.25, y: 0.75, z: 0.75 }, V3c { x: 0.5, y: 0.75, z: 0.75 }, V3c { x: 0.75, y: 0.75, z: 0.75 }
];

fn sectant_after_step(step_vector: &V3c<i32>, sectant: usize) -> u8 {
    let step_signum = V3c::new(
        step_vector.x.signum() as f32,
        step_vector.y.signum() as f32,
        step_vector.z.signum() as f32,
    );
    let sectant_offset = SECTANT_OFFSET_REGION_LUT[sectant];
    let sectant_size = 1. / BOX_NODE_DIMENSION as f32;
    let sectant_center = sectant_offset + V3c::unit(sectant_size / 2.);
    let center_after_step = sectant_center + V3c::unit(sectant_size) * step_signum;

    if 0 == sectant {
        println!(
            "{:?} -{:?}-> {:?} ==> {:?}",
            sectant_center,
            step_signum,
            center_after_step,
            if center_after_step.x < 0.
                || center_after_step.x > 1.
                || center_after_step.y < 0.
                || center_after_step.y > 1.
                || center_after_step.z < 0.
                || center_after_step.z > 1.
            {
                OOB_SECTANT
            } else {
                hash_region(&center_after_step, 1.)
            }
        );
    }

    if center_after_step.x < 0.
        || center_after_step.x > 1.
        || center_after_step.y < 0.
        || center_after_step.y > 1.
        || center_after_step.z < 0.
        || center_after_step.z > 1.
    {
        OOB_SECTANT
    } else {
        hash_region(&center_after_step, 1.)
    }
}

/// Internal utility for generating Lookup tables
/// Generates the sectant result of a step in any direction
fn main() {
    // LUT to be generated for every sectant and for every direction
    // --> usage: sectant_step_result[sectant][direction_x_signum][direction_y_signum][direction_z_signum];
    let mut sectant_step_result = [[[[0u8; 3]; 3]; 3]; BOX_NODE_CHILDREN_COUNT];
    for (sectant, step_result) in sectant_step_result
        .iter_mut()
        .enumerate()
        .take(BOX_NODE_CHILDREN_COUNT)
    {
        for z in -1i32..=1 {
            for y in -1i32..=1 {
                for x in -1i32..=1 {
                    step_result[(x + 1) as usize][(y + 1) as usize][(z + 1) as usize] =
                        sectant_after_step(&V3c::new(x, y, z), sectant);
                }
            }
        }
    }

    println!("CPU LUT:{:?}", sectant_step_result);
    println!("WGSL LUT:");
    println!(
        "//const\nvar<private> SECTANT_STEP_RESULT_LUT: array<array<array<array<vec3f, 3>, 3>, 3>,{}> = array<array<array<array<vec3f, 3>, 3>, 3>,{}>(",
        BOX_NODE_CHILDREN_COUNT, BOX_NODE_CHILDREN_COUNT
    );

    for (sectant, step_lut) in sectant_step_result.iter().enumerate() {
        print!("\tarray<array<array<u32, 3>, 3>, 3>(");
        for (x, xarr) in step_lut.iter().enumerate() {
            print!("array<array<u32, 3>, 3>(");
            for (y, yarr) in xarr.iter().enumerate() {
                print!("array<u32, 3>(");
                for (idx, step_result) in yarr.iter().enumerate() {
                    print!("{:?}", step_result);
                    if idx < 2 {
                        print!(",");
                    }
                }
                print!(")");
                if y < 2 {
                    print!(",");
                }
            }
            print!(")");
            if x < 2 {
                print!(",");
            }
        }
        print!(")");
        if sectant < (OOB_SECTANT as usize - 1) {
            print!(",");
        }
        println!();
    }
    println!("\n);");
    #[rustfmt::skip]
    let _sectant_step_result_lut = [
        [[[64, 64, 64], [64, 64, 64], [64, 64, 64]], [[64, 64, 64], [64, 0, 16],  [64, 4, 20]],  [[64, 64, 64], [64, 1, 17], [64, 5, 21]]],
        [[[64, 64, 64], [64, 0, 16],  [64, 4, 20]],  [[64, 64, 64], [64, 1, 17],  [64, 5, 21]],  [[64, 64, 64], [64, 2, 18], [64, 6, 22]]],
        [[[64, 64, 64], [64, 1, 17],  [64, 5, 21]],  [[64, 64, 64], [64, 2, 18],  [64, 6, 22]],  [[64, 64, 64], [64, 3, 19], [64, 7, 23]]],
        [[[64, 64, 64], [64, 2, 18],  [64, 6, 22]],  [[64, 64, 64], [64, 3, 19],  [64, 7, 23]],  [[64, 64, 64], [64, 64, 64], [64, 64, 64]]],
        [[[64, 64, 64], [64, 64, 64], [64, 64, 64]], [[64, 0, 16],  [64, 4, 20],  [64, 8, 24]],  [[64, 1, 17], [64, 5, 21], [64, 9, 25]]],
        [[[64, 0, 16],  [64, 4, 20],  [64, 8, 24]],  [[64, 1, 17],  [64, 5, 21],  [64, 9, 25]],  [[64, 2, 18], [64, 6, 22], [64, 10, 26]]],
        [[[64, 1, 17],  [64, 5, 21],  [64, 9, 25]],  [[64, 2, 18],  [64, 6, 22],  [64, 10, 26]], [[64, 3, 19], [64, 7, 23], [64, 11, 27]]],
        [[[64, 2, 18],  [64, 6, 22],  [64, 10, 26]], [[64, 3, 19],  [64, 7, 23],  [64, 11, 27]], [[64, 64, 64], [64, 64, 64], [64, 64, 64]]],
        [[[64, 64, 64], [64, 64, 64], [64, 64, 64]], [[64, 4, 20],  [64, 8, 24],  [64, 12, 28]], [[64, 5, 21], [64, 9, 25], [64, 13, 29]]],
        [[[64, 4, 20],  [64, 8, 24],  [64, 12, 28]], [[64, 5, 21],  [64, 9, 25],  [64, 13, 29]], [[64, 6, 22], [64, 10, 26], [64, 14, 30]]],
        [[[64, 5, 21],  [64, 9, 25],  [64, 13, 29]], [[64, 6, 22],  [64, 10, 26], [64, 14, 30]], [[64, 7, 23], [64, 11, 27], [64, 15, 31]]],
        [[[64, 6, 22],  [64, 10, 26], [64, 14, 30]], [[64, 7, 23],  [64, 11, 27], [64, 15, 31]], [[64, 64, 64], [64, 64, 64], [64, 64, 64]]],
        [[[64, 64, 64], [64, 64, 64], [64, 64, 64]], [[64, 8, 24],  [64, 12, 28], [64, 64, 64]], [[64, 9, 25], [64, 13, 29], [64, 64, 64]]],
        [[[64, 8, 24],  [64, 12, 28], [64, 64, 64]], [[64, 9, 25],  [64, 13, 29], [64, 64, 64]], [[64, 10, 26], [64, 14, 30], [64, 64, 64]]],
        [[[64, 9, 25],  [64, 13, 29], [64, 64, 64]], [[64, 10, 26], [64, 14, 30], [64, 64, 64]], [[64, 11, 27], [64, 15, 31], [64, 64, 64]]],
        [[[64, 10, 26], [64, 14, 30], [64, 64, 64]], [[64, 11, 27], [64, 15, 31], [64, 64, 64]], [[64, 64, 64], [64, 64, 64], [64, 64, 64]]],
        [[[64, 64, 64], [64, 64, 64], [64, 64, 64]], [[64, 64, 64], [0, 16, 32],  [4, 20, 36]],  [[64, 64, 64], [1, 17, 33], [5, 21, 37]]],
        [[[64, 64, 64], [0, 16, 32],  [4, 20, 36]],  [[64, 64, 64], [1, 17, 33],  [5, 21, 37]],  [[64, 64, 64], [2, 18, 34], [6, 22, 38]]],
        [[[64, 64, 64], [1, 17, 33],  [5, 21, 37]],  [[64, 64, 64], [2, 18, 34],  [6, 22, 38]],  [[64, 64, 64], [3, 19, 35], [7, 23, 39]]],
        [[[64, 64, 64], [2, 18, 34],  [6, 22, 38]],  [[64, 64, 64], [3, 19, 35],  [7, 23, 39]],  [[64, 64, 64], [64, 64, 64], [64, 64, 64]]],
        [[[64, 64, 64], [64, 64, 64], [64, 64, 64]], [[0, 16, 32],  [4, 20, 36],  [8, 24, 40]],  [[1, 17, 33], [5, 21, 37], [9, 25, 41]]],
        [[[0, 16, 32],  [4, 20, 36],  [8, 24, 40]],  [[1, 17, 33],  [5, 21, 37],  [9, 25, 41]],  [[2, 18, 34], [6, 22, 38], [10, 26, 42]]],
        [[[1, 17, 33],  [5, 21, 37],  [9, 25, 41]],  [[2, 18, 34],  [6, 22, 38],  [10, 26, 42]], [[3, 19, 35], [7, 23, 39], [11, 27, 43]]],
        [[[2, 18, 34],  [6, 22, 38],  [10, 26, 42]], [[3, 19, 35],  [7, 23, 39],  [11, 27, 43]], [[64, 64, 64], [64, 64, 64], [64, 64, 64]]],
        [[[64, 64, 64], [64, 64, 64], [64, 64, 64]], [[4, 20, 36],  [8, 24, 40],  [12, 28, 44]], [[5, 21, 37], [9, 25, 41], [13, 29, 45]]],
        [[[4, 20, 36],  [8, 24, 40],  [12, 28, 44]], [[5, 21, 37],  [9, 25, 41],  [13, 29, 45]], [[6, 22, 38], [10, 26, 42], [14, 30, 46]]],
        [[[5, 21, 37],  [9, 25, 41],  [13, 29, 45]], [[6, 22, 38],  [10, 26, 42], [14, 30, 46]], [[7, 23, 39], [11, 27, 43], [15, 31, 47]]],
        [[[6, 22, 38],  [10, 26, 42], [14, 30, 46]], [[7, 23, 39],  [11, 27, 43], [15, 31, 47]], [[64, 64, 64], [64, 64, 64], [64, 64, 64]]],
        [[[64, 64, 64], [64, 64, 64], [64, 64, 64]], [[8, 24, 40],  [12, 28, 44], [64, 64, 64]], [[9, 25, 41], [13, 29, 45], [64, 64, 64]]],
        [[[8, 24, 40],  [12, 28, 44], [64, 64, 64]], [[9, 25, 41],  [13, 29, 45], [64, 64, 64]], [[10, 26, 42], [14, 30, 46], [64, 64, 64]]],
        [[[9, 25, 41],  [13, 29, 45], [64, 64, 64]], [[10, 26, 42], [14, 30, 46], [64, 64, 64]], [[11, 27, 43], [15, 31, 47], [64, 64, 64]]],
        [[[10, 26, 42], [14, 30, 46], [64, 64, 64]], [[11, 27, 43], [15, 31, 47], [64, 64, 64]], [[64, 64, 64], [64, 64, 64], [64, 64, 64]]],
        [[[64, 64, 64], [64, 64, 64], [64, 64, 64]], [[64, 64, 64], [16, 32, 48], [20, 36, 52]], [[64, 64, 64], [17, 33, 49], [21, 37, 53]]],
        [[[64, 64, 64], [16, 32, 48], [20, 36, 52]], [[64, 64, 64], [17, 33, 49], [21, 37, 53]], [[64, 64, 64], [18, 34, 50], [22, 38, 54]]],
        [[[64, 64, 64], [17, 33, 49], [21, 37, 53]], [[64, 64, 64], [18, 34, 50], [22, 38, 54]], [[64, 64, 64], [19, 35, 51], [23, 39, 55]]],
        [[[64, 64, 64], [18, 34, 50], [22, 38, 54]], [[64, 64, 64], [19, 35, 51], [23, 39, 55]], [[64, 64, 64], [64, 64, 64], [64, 64, 64]]],
        [[[64, 64, 64], [64, 64, 64], [64, 64, 64]], [[16, 32, 48], [20, 36, 52], [24, 40, 56]], [[17, 33, 49], [21, 37, 53], [25, 41, 57]]],
        [[[16, 32, 48], [20, 36, 52], [24, 40, 56]], [[17, 33, 49], [21, 37, 53], [25, 41, 57]], [[18, 34, 50], [22, 38, 54], [26, 42, 58]]],
        [[[17, 33, 49], [21, 37, 53], [25, 41, 57]], [[18, 34, 50], [22, 38, 54], [26, 42, 58]], [[19, 35, 51], [23, 39, 55], [27, 43, 59]]],
        [[[18, 34, 50], [22, 38, 54], [26, 42, 58]], [[19, 35, 51], [23, 39, 55], [27, 43, 59]], [[64, 64, 64], [64, 64, 64], [64, 64, 64]]],
        [[[64, 64, 64], [64, 64, 64], [64, 64, 64]], [[20, 36, 52], [24, 40, 56], [28, 44, 60]], [[21, 37, 53], [25, 41, 57], [29, 45, 61]]],
        [[[20, 36, 52], [24, 40, 56], [28, 44, 60]], [[21, 37, 53], [25, 41, 57], [29, 45, 61]], [[22, 38, 54], [26, 42, 58], [30, 46, 62]]],
        [[[21, 37, 53], [25, 41, 57], [29, 45, 61]], [[22, 38, 54], [26, 42, 58], [30, 46, 62]], [[23, 39, 55], [27, 43, 59], [31, 47, 63]]],
        [[[22, 38, 54], [26, 42, 58], [30, 46, 62]], [[23, 39, 55], [27, 43, 59], [31, 47, 63]], [[64, 64, 64], [64, 64, 64], [64, 64, 64]]],
        [[[64, 64, 64], [64, 64, 64], [64, 64, 64]], [[24, 40, 56], [28, 44, 60], [64, 64, 64]], [[25, 41, 57], [29, 45, 61], [64, 64, 64]]],
        [[[24, 40, 56], [28, 44, 60], [64, 64, 64]], [[25, 41, 57], [29, 45, 61], [64, 64, 64]], [[26, 42, 58], [30, 46, 62], [64, 64, 64]]],
        [[[25, 41, 57], [29, 45, 61], [64, 64, 64]], [[26, 42, 58], [30, 46, 62], [64, 64, 64]], [[27, 43, 59], [31, 47, 63], [64, 64, 64]]],
        [[[26, 42, 58], [30, 46, 62], [64, 64, 64]], [[27, 43, 59], [31, 47, 63], [64, 64, 64]], [[64, 64, 64], [64, 64, 64], [64, 64, 64]]],
        [[[64, 64, 64], [64, 64, 64], [64, 64, 64]], [[64, 64, 64], [32, 48, 64], [36, 52, 64]], [[64, 64, 64], [33, 49, 64], [37, 53, 64]]],
        [[[64, 64, 64], [32, 48, 64], [36, 52, 64]], [[64, 64, 64], [33, 49, 64], [37, 53, 64]], [[64, 64, 64], [34, 50, 64], [38, 54, 64]]],
        [[[64, 64, 64], [33, 49, 64], [37, 53, 64]], [[64, 64, 64], [34, 50, 64], [38, 54, 64]], [[64, 64, 64], [35, 51, 64], [39, 55, 64]]],
        [[[64, 64, 64], [34, 50, 64], [38, 54, 64]], [[64, 64, 64], [35, 51, 64], [39, 55, 64]], [[64, 64, 64], [64, 64, 64], [64, 64, 64]]],
        [[[64, 64, 64], [64, 64, 64], [64, 64, 64]], [[32, 48, 64], [36, 52, 64], [40, 56, 64]], [[33, 49, 64], [37, 53, 64], [41, 57, 64]]],
        [[[32, 48, 64], [36, 52, 64], [40, 56, 64]], [[33, 49, 64], [37, 53, 64], [41, 57, 64]], [[34, 50, 64], [38, 54, 64], [42, 58, 64]]],
        [[[33, 49, 64], [37, 53, 64], [41, 57, 64]], [[34, 50, 64], [38, 54, 64], [42, 58, 64]], [[35, 51, 64], [39, 55, 64], [43, 59, 64]]],
        [[[34, 50, 64], [38, 54, 64], [42, 58, 64]], [[35, 51, 64], [39, 55, 64], [43, 59, 64]], [[64, 64, 64], [64, 64, 64], [64, 64, 64]]],
        [[[64, 64, 64], [64, 64, 64], [64, 64, 64]], [[36, 52, 64], [40, 56, 64], [44, 60, 64]], [[37, 53, 64], [41, 57, 64], [45, 61, 64]]],
        [[[36, 52, 64], [40, 56, 64], [44, 60, 64]], [[37, 53, 64], [41, 57, 64], [45, 61, 64]], [[38, 54, 64], [42, 58, 64], [46, 62, 64]]],
        [[[37, 53, 64], [41, 57, 64], [45, 61, 64]], [[38, 54, 64], [42, 58, 64], [46, 62, 64]], [[39, 55, 64], [43, 59, 64], [47, 63, 64]]],
        [[[38, 54, 64], [42, 58, 64], [46, 62, 64]], [[39, 55, 64], [43, 59, 64], [47, 63, 64]], [[64, 64, 64], [64, 64, 64], [64, 64, 64]]],
        [[[64, 64, 64], [64, 64, 64], [64, 64, 64]], [[40, 56, 64], [44, 60, 64], [64, 64, 64]], [[41, 57, 64], [45, 61, 64], [64, 64, 64]]],
        [[[40, 56, 64], [44, 60, 64], [64, 64, 64]], [[41, 57, 64], [45, 61, 64], [64, 64, 64]], [[42, 58, 64], [46, 62, 64], [64, 64, 64]]],
        [[[41, 57, 64], [45, 61, 64], [64, 64, 64]], [[42, 58, 64], [46, 62, 64], [64, 64, 64]], [[43, 59, 64], [47, 63, 64], [64, 64, 64]]],
        [[[42, 58, 64], [46, 62, 64], [64, 64, 64]], [[43, 59, 64], [47, 63, 64], [64, 64, 64]], [[64, 64, 64], [64, 64, 64], [64, 64, 64]]]
    ];
}
