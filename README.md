# VoxelHex(v0x7H3X) 
![Repository logo](https://github.com/Ministry-of-Voxel-Affairs/VoxelHex/blob/61cc0cc36becdc93a63ab7b7ca3dc3b65a3e54cd/new_logo.png)
A Sparse voxel-brick tree implementation in Rust/WGPU.
The leaf nodes of the tree contain voxel bricks instead of a single Voxel. This makes it possible to have a unique compression system, where Voxels of different resolutions can be mixed together.
An implementation for raytracing is available with GPU support!
The library uses Left handed Y up coordinate system.

Videos I made about the tech: 
https://www.youtube.com/watch?v=pVmUQUhrfjg&list=PL_3Xjx_NV4tw6vhcij03fZFTpt0eaO_-b

Roadmap:
- Shadows, Lighting, Illumination: https://github.com/Ministry-of-Voxel-Affairs/VoxelHex/milestone/2
- Data Compression, Load time minimizations: https://github.com/Ministry-of-Voxel-Affairs/VoxelHex/milestone/3
- Displaying Vast Voxel landscapes: https://github.com/Ministry-of-Voxel-Affairs/VoxelHex/milestone/1

Issue spotlight: 
- Performance upgrade: https://github.com/Ministry-of-Voxel-Affairs/VoxelHex/issues/10
- Examples Quality of Life Updates: https://github.com/Ministry-of-Voxel-Affairs/VoxelHex/issues/9
- Improved normal handling: https://github.com/Ministry-of-Voxel-Affairs/VoxelHex/issues/11
- Performance improvement (allegedly) : https://github.com/Ministry-of-Voxel-Affairs/VoxelHex/issues/13

Special thanks to contributors and supporters!
-

[@mogambro](https://github.com/mogambro) For the Albedo type and amazing support!

[@DouglasDwyer](https://github.com/DouglasDwyer) My nemesis; Check out [his project](https://github.com/DouglasDwyer/octo-release) it's amazing! ( I hate him )

[@Neo-Zhixing](https://github.com/Neo-Zhixing) For [his amazing project](https://github.com/dust-engine) and awesome idea about how to utilize hardware RT for Voxel rendering!
