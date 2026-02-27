# TODO

## Larger TODO Tasks

- Uniforms
- Textures
- Windows
- Particles
- Compute Shaders
- Better more user friendly input systems for things like mesh data
- functions for caching compiled shaders or full pipelines
- Cameras, Projection
- push constants
- several frames in flight / several queues per type of task

## Smaller TODO Tasks

- make 'render_surface::image_surface::get_image_surface_data' not block
- add proper error types
  - The most important one here imo BETTER ERRORS FOR SHADER COMPILATION
- add more test cases for errors
- Add docs for things
- Maybe have engine defer creation of spir-v compiler / make it seperate class
- have transfer queue prioritize queue families that don't support non-transfer operations

## In Progress
