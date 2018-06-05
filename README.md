
![alt text](https://github.com/SiebenCorgie/jakar-engine/blob/master/media/banner.jpg)
# Jakar-Engine
A small engine written in rust + Vulkan.

## MOVED TO GITLAB

Since I am kind of a idealist I moved the whole project including the sub crates to Gitlab:
https://gitlab.com/Siebencorgie/jakar-engine

## Target


### Overview

The target of this engine is, to have a safe and fast, but still visually
beautiful engine which works on most modern PC systems.
The speed should be supplied by the parallel nature of the engine.
It will have at least 4 different loops which will manage different
aspects of the engine and its systems.
 1. rendering loop
 2. asset management
 3. physics (TODO)
 4. input

In addition the system will spawn threads for workloads like config-loading
or mesh importing / streaming.


### Safety

The engine should be as safe as possible. Which mostly means it shouldn't crash.
The safety is supplied mostly through two design decisions:
 1. it uses Rust which is "safe" by design
 2. it uses Vulkano for interaction with Vulkan, which will be safe as well
    in the future

My target is to handle most of the non fatal errors and at least end the system controlled if something happens.

### Graphical target

The graphical target is defined by these key points:

- [x] PBR shading (no IBL yet)
- [x] normal mapping
- [ ] parallax occlusion mapping
- [x] HDR rendering with dynamic eye adaption
- [x] dynamic lighting (point, spot and directional lights for now)
- [x] clustered light culling for spot and point lights (currently in world space)
- [x] bloom
- [ ] DOF
- [x] translucency
- [x] masked materials
- [x] cascaded shadow maps for unlimited dynamic lights
- [ ] single cascade shadows for point and spot lights
- [ ] dynamic ambient cube-map for run time IBL and reflections (like GTA V)

This will be accomplished by static Shaders + different material definition for
now. In later development this system could be changed to a UE4 type
material system with different Shader components.

### Graphics Showcase
#### Videos
[![Kinda latest video](https://img.youtube.com/vi/FhV0eGdSGFY/0.jpg)](https://www.youtube.com/watch?v=FhV0eGdSGFY)


### Asset management

#### Meshes, cameras and lights
All of the assets will be grouped in different managers as `Arc<Mutex<T>>`
components, for instance all meshes.
There is a scene manager who saves different hierarchical "scenes" of those
assets. For example a light which is parent to a mesh and a camera which is
saved as a scene "wall_lamp".
This has the advantage of being able to modify one of the meshes in the mesh
manager and simultaneously changing all of its references as well.

This scene trees are 1-n trees for now. So every parent can have n children.
The Node types are hard coded for now but will be changed to a Arc<NodeType> in
the future. Have a look at the `JakarTree` repository if you want to know more.
I took inspiration from the Godot-engine for the scene system.

#### Other assets
The meshes usually have a material attached which consists of several textures.
Those are manged by a `TextureManager`. Each material can request one of the stored textures as a `Arc<Texture>` and then use it in one of the slots.
This way no texture needs to be loaded twice.
The `MaterialManager` works similar, it takes the created materials and provides `Arc<Mutex<Material>>` copies upon request.

## Documentation
There is currently no documentation hosted, but you can do
```
cargo doc --open
```
to build the documentation yourself. The index.html will be saved to
```
target/doc/jakar_engine/index.html
```

The documentation is not complete but at least describes most of the functions and
systems. If you need any clarification, open a pull request.

## Building

Pull the git repository via
```
git clone https://github.com/SiebenCorgie/jakar-engine.git
```
then do
```
cargo build
```
to compile.

If you want to start the engine with a scene, first download a gltf 2.0 files
somewhere and provide its path in the main function of the `simple` example (around
  line 70). Then do:

```
cargo run --examples simple --release
```
to run an example application.

*Note: You'll need to have Vulkan capable drivers installed*

## License

You can decide if you want to use MIT or Apache License, Version 2.0.
