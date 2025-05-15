#[cfg(test)]
#[cfg(feature = "bevy_wgpu")]
mod wgpu_tests {
    use crate::spatial::math::vector::V3cf32;
    use bevy::render::render_resource::encase::StorageBuffer;

    #[test]
    fn test_buffer_readback() {
        let original_value = V3cf32::new(0.666, 0.69, 420.0);
        let mut buffer = StorageBuffer::new(Vec::<u8>::new());
        buffer.write(&original_value).unwrap();
        let mut byte_buffer = buffer.into_inner();
        let buffer = StorageBuffer::new(&mut byte_buffer);
        let mut value = V3cf32::default();
        buffer.read(&mut value).unwrap();
        assert_eq!(value, original_value);
    }
}

#[cfg(test)]
mod bitmap_tests {
    use crate::contree::V3c;
    use crate::spatial::math::set_occupied_bitmap_value;

    #[test]
    fn test_occupancy_bitmap_aligned_dim() {
        let mut mask = 0;
        set_occupied_bitmap_value(&V3c::new(0, 0, 0), 1, 4, true, &mut mask);
        assert_eq!(0x0000000000000001, mask);

        set_occupied_bitmap_value(&V3c::new(3, 3, 3), 1, 4, true, &mut mask);
        assert_eq!(0x8000000000000001, mask);

        set_occupied_bitmap_value(&V3c::new(2, 2, 2), 1, 4, true, &mut mask);
        assert_eq!(0x8000040000000001, mask);
    }

    #[test]
    fn test_occupancy_bitmap_where_dim_is_1() {
        let mut mask = u64::MAX;

        set_occupied_bitmap_value(&V3c::new(0, 0, 0), 1, 1, false, &mut mask);
        assert_eq!(0, mask);

        set_occupied_bitmap_value(&V3c::new(0, 0, 0), 1, 1, true, &mut mask);
        assert_eq!(u64::MAX, mask);
    }

    #[test]
    fn test_occupancy_bitmap_where_dim_is_2() {
        let mut mask = 0;

        set_occupied_bitmap_value(&V3c::new(0, 0, 0), 1, 2, true, &mut mask);
        assert_eq!(0x0000000000330033, mask);

        set_occupied_bitmap_value(&V3c::new(1, 1, 1), 1, 2, true, &mut mask);
        assert_eq!(0xCC00CC0000330033, mask);
    }

    #[test]
    #[should_panic(expected = "Expected coordinate 5 < brick size(4)")]
    fn test_occupancy_bitmap_aligned_dim_pos_overflow() {
        let mut mask = 0;
        set_occupied_bitmap_value(&V3c::new(5, 5, 5), 1, 4, true, &mut mask);
        assert_eq!(0, mask);
    }

    #[test]
    #[should_panic(expected = "Expected coordinate 9 < brick size(4)")]
    fn test_occupancy_bitmap_aligned_dim_pos_partial_overflow() {
        let mut mask = 0;
        set_occupied_bitmap_value(&V3c::new(3, 1, 9), 1, 4, true, &mut mask);
        assert_eq!(0, mask);
    }

    #[test]
    #[should_panic(expected = "Expected coordinate 2 < brick size(1)")]
    fn test_occupancy_bitmap_where_dim_is_1_pos_overflow() {
        let mut mask = u64::MAX;
        set_occupied_bitmap_value(&V3c::new(2, 2, 3), 1, 1, true, &mut mask);
        assert_eq!(0, mask);
    }

    #[test]
    #[should_panic(expected = "Expected coordinate 4 < brick size(2)")]
    fn test_occupancy_bitmap_where_dim_is_2_pos_overflow() {
        let mut mask = 0;
        set_occupied_bitmap_value(&V3c::new(4, 4, 4), 1, 2, true, &mut mask);
        assert_eq!(0, mask);
    }

    #[test]
    fn test_occupancy_bitmap_sized_set_aligned_dim() {
        let mut mask = 0;
        set_occupied_bitmap_value(&V3c::new(0, 0, 0), 3, 4, true, &mut mask);
        assert_eq!(0x77707770777, mask);
    }

    #[test]
    fn test_occupancy_bitmap_sized_set_where_dim_is_2() {
        let mut mask = 0;
        set_occupied_bitmap_value(&V3c::new(0, 0, 0), 2, 2, true, &mut mask);
        assert_eq!(0xFFFFFFFFFFFFFFFF, mask);
    }

    #[test]
    fn test_occupancy_bitmap_sized_set_aligned_dim_overflow() {
        let mut mask = 0;
        set_occupied_bitmap_value(&V3c::new(0, 0, 0), 5, 4, true, &mut mask);
        assert_eq!(0xFFFFFFFFFFFFFFFF, mask);
    }

    #[test]
    fn test_occupancy_bitmap_sized_set_where_dim_is_2_overflow() {
        let mut mask = 0;
        set_occupied_bitmap_value(&V3c::new(0, 0, 0), 3, 2, true, &mut mask);
        assert_eq!(0xFFFFFFFFFFFFFFFF, mask);
    }
}

#[cfg(test)]
#[cfg(feature = "dot_vox_support")]
mod dot_vox_tests {

    use crate::contree::V3c;
    use crate::spatial::math::convert_coordinate;
    use crate::spatial::math::CoordinateSystemType;

    #[test]
    fn test_coordinate_conversion() {
        assert_eq!(
            V3c::new(1., 2., 3.),
            convert_coordinate(
                V3c::new(1., 2., 3.),
                CoordinateSystemType::Rzup,
                CoordinateSystemType::Rzup,
            ),
        );

        assert_eq!(
            V3c::new(1., 3., 2.),
            convert_coordinate(
                V3c::new(1., 2., 3.),
                CoordinateSystemType::Lzup,
                CoordinateSystemType::Ryup,
            ),
        );

        assert_eq!(
            V3c::new(1., 3., -2.),
            convert_coordinate(
                V3c::new(1., 2., 3.),
                CoordinateSystemType::Rzup,
                CoordinateSystemType::Ryup,
            ),
        );

        assert_eq!(
            V3c::new(1., 2., -3.),
            convert_coordinate(
                V3c::new(1., 2., 3.),
                CoordinateSystemType::Lyup,
                CoordinateSystemType::Ryup,
            ),
        );
    }
}
