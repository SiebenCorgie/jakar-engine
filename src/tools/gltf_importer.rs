use core::resources::{texture, material, empty, mesh, light, camera};
use core::resources::camera::Camera;
//use core::simple_scene_system::node;
use jakar_tree::*;
use core::next_tree::*;
use jakar_tree::node::{Attribute, NodeContent};

use core::resource_management::{material_manager, mesh_manager, scene_manager, texture_manager};
use core;
use core::ReturnBoundInfo;
use render::pipeline_builder;
use render::pipeline_manager;
use render;
use render::render_passes::RenderPassConf;

use vulkano;
use vulkano::pipeline::blend::LogicOp;

use cgmath::*;

use gltf;
use gltf_importer;
use image;

use std::path::Path;
use std::sync::{Arc, Mutex};


///Imports a gltf texture
pub fn load_gltf_texture(
    texture: &gltf::Texture,
    name: String,
    buffers: &gltf_importer::Buffers,
    base: &Path,
    managers: &Arc<Mutex<core::resource_management::ManagerAndRenderInfo>>,
) -> Arc<texture::Texture>
{
    //The texture can be a buffer or an external file, depending on the case we load the texture
    //wrap it into an Arc<Texture>, then add it to the manager once and return the other one
    //first create a texture builder and configure it with the right sampler from the provided texture
    let mut texture_builder = {
        let texture_manager = {
            let managers_lck = managers.lock().expect("failed to lock managers struct");
            (*managers_lck).texture_manager.clone()
        };
        //lock the texture manager once to get some data
        let texture_manager_lck = texture_manager.lock().expect("failed to lock texture manager");
        //No create the textuer builder based on the type of data
        match texture.source().data(){

            gltf::image::Data::View{view, mime_type} => {
                //found a data buffer for the image
                println!("Image is Data", );
                let data = buffers.view(&view).expect("failed to load image data from gltf buffer");
                //we got the data, lets provide it to a TextureBuilder
                texture::TextureBuilder::from_data(
                    data.to_vec(),
                    (*texture_manager_lck).get_device(),
                    (*texture_manager_lck).get_queue(),
                    (*texture_manager_lck).get_settings(),
                )
            },
            gltf::image::Data::Uri{uri, mime_type} =>{
                println!("Image is file at: {}", uri);
                //prepare the path
                let path = base.join(uri);
                texture::TextureBuilder::from_image(
                    path.to_str().expect("failed to create string from path"),
                    (*texture_manager_lck).get_device(),
                    (*texture_manager_lck).get_queue(),
                    (*texture_manager_lck).get_settings(),
                )
            }
        }
    };

    //Now set all sampler settings on the builder
    let sampler = texture.sampler();
    match sampler.index(){
        Some(_) => {
            //This texture got an sampler, lets set the settings
            let mag_filter = {
                use gltf::texture::MagFilter;

                match sampler.mag_filter(){
                    Some(filter) => {
                        match filter{
                            //return the vulkano sampler based on the gltf sampler
                            MagFilter::Linear => vulkano::sampler::Filter::Linear,
                            MagFilter::Nearest => vulkano::sampler::Filter::Nearest,
                        }
                    },
                    None => {
                        //Use linear filtering if no filter is set
                        vulkano::sampler::Filter::Linear
                    },
                }
            };
            let min_filter = {
                use gltf::texture::MinFilter;

                match sampler.min_filter(){
                    Some(filter) => {
                        match filter{
                            //return the vulkano sampler based on the gltf sampler
                            MinFilter::Linear => vulkano::sampler::Filter::Linear,
                            MinFilter::Nearest => vulkano::sampler::Filter::Nearest,
                            _ => vulkano::sampler::Filter::Linear, //All other types are linear as well
                        }
                    },
                    None => {
                        //Use linear filtering if no filter is set
                        vulkano::sampler::Filter::Linear
                    },
                }
            };
            //Setup sampling
            texture_builder = texture_builder.with_sampling_filter(mag_filter, min_filter);
            //Setup wraping
            let wrap_u = {
                use gltf::texture::WrappingMode;
                match sampler.wrap_s(){
                    WrappingMode::ClampToEdge => vulkano::sampler::SamplerAddressMode::ClampToEdge,
                    WrappingMode::MirroredRepeat => vulkano::sampler::SamplerAddressMode::MirroredRepeat,
                    WrappingMode::Repeat => vulkano::sampler::SamplerAddressMode::Repeat,
                }
            };
            let wrap_v = {
                use gltf::texture::WrappingMode;
                match sampler.wrap_t(){
                    WrappingMode::ClampToEdge => vulkano::sampler::SamplerAddressMode::ClampToEdge,
                    WrappingMode::MirroredRepeat => vulkano::sampler::SamplerAddressMode::MirroredRepeat,
                    WrappingMode::Repeat => vulkano::sampler::SamplerAddressMode::Repeat,
                }
            };

            texture_builder = texture_builder.with_tiling_mode(wrap_u, wrap_v,wrap_v); //tilling on w will be same but unused because of 2D texture
        },
        None => {}, //this texture has no sampler => using the default one
    }

    //now set some flipping
    //texture_builder = texture_builder.with_rotation_180();


    //finally build the texture
    let new_texture = texture_builder.build_with_name(&name);
    //now add a copy to the manager and return the other one
    {
        let texture_manager = {
            let managers_lck = managers.lock().expect("failed to lock managers struct");
            (*managers_lck).texture_manager.clone()
        };
        let mut texture_manager_lck = texture_manager.lock().expect("failed to lock texture manager");
        let tex_error = (*texture_manager_lck).add_texture(new_texture.clone());
        match tex_error{
            Ok(_) => {}, //everything allright while adding
            Err(r) => println!("failed to add texture to manager while loading gltf: {}", r),
        }
    }
    //finally return the new texture
    new_texture
}

///Imports a gltf material, returns the loaded material from the manager
pub fn load_gltf_material(
    mat: &gltf::Material,
    material_name: String,
    buffers: &gltf_importer::Buffers,
    base: &Path,
    managers: &Arc<Mutex<core::resource_management::ManagerAndRenderInfo>>,
) -> Arc<Mutex<material::Material>>{
    //println!("Loading material with name: {}", material_name.clone());
    //first load the pbr info
    let pbr = mat.pbr_metallic_roughness();
    //now load all textures if there is none it returns none which will be respected at build time of the material
    let albedo = {
        match pbr.base_color_texture(){
            Some(t) => {
                Some(
                    load_gltf_texture(
                    &t.texture(), material_name.clone() + "_albedo", buffers, base, managers
                    )
                )
            },
            None => None,
        }
    };
    //normal
    let normal = {
        match mat.normal_texture(){
            Some(t) => {
                Some(
                    load_gltf_texture(
                    &t.texture(), material_name.clone() + "_normal", buffers, base, managers
                    )
                )
            },
            None => None,
        }
    };
    //metallic_roughness
    let metallic_roughness = {
        match pbr.metallic_roughness_texture(){
            Some(t) => {
                Some(
                    load_gltf_texture(
                    &t.texture(), material_name.clone() + "_met_rough", buffers, base, managers
                    )
                )
            },
            None => None,
        }
    };
    //occlusion
    let occlusion = {
        match mat.occlusion_texture(){
            Some(t) => {
                Some(
                    load_gltf_texture(
                    &t.texture(), material_name.clone() + "_occlu", buffers, base, managers
                    )
                )
            },
            None => None,
        }
    };
    //emissive
    let emissive = {
        match mat.emissive_texture(){
            Some(t) => {
                Some(
                    load_gltf_texture(
                    &t.texture(), material_name.clone() + "_emissive", buffers, base, managers
                    )
                )
            },
            None => None,
        }
    };

    //We also need the texture factors
    let texture_factors = {
        material::MaterialFactors::new()
        .with_factor_albedo(pbr.base_color_factor())
        .with_factor_normal(mat.normal_texture().map_or(1.0, |t| t.scale()))
        .with_factor_metal(pbr.metallic_factor())
        .with_factor_roughness(pbr.roughness_factor())
        .with_factor_occlusion(mat.occlusion_texture().map_or(1.0, |t| t.strength()))
        .with_factor_emissive(mat.emissive_factor())
    };
    /*
    println!("DEBUG: Factors:", );
    println!("\t Albedo: {:?}", pbr.base_color_factor());
    println!("\t Normal: {:?}", mat.normal_texture().map_or(1.0, |t| t.scale()));
    println!("\t Metal: {:?}", pbr.metallic_factor());
    println!("\t Roughness: {:?}", pbr.roughness_factor());
    println!("\t Occlusion: {:?}", mat.occlusion_texture().map_or(1.0, |t| t.strength()));
    println!("\t emmisive: {:?}", mat.emissive_factor());
    */
    //get the manager
    let texture_manager = {
        let managers_lck = managers.lock().expect("failed to lock managers struct");
        (*managers_lck).texture_manager.clone()
    };

    let fallback_texture = {
        let man_lck = texture_manager.lock().expect("failed to lock material manager");
        (*man_lck).get_none()
    };

    //Create a material builder from the info
    let material_builder = material::MaterialBuilder::new(
        albedo,
        normal,
        metallic_roughness,
        occlusion,
        emissive,
        fallback_texture,
    )

    //now configure the factors
    .with_factors(texture_factors);

    //To decide the pipeline of this material we need to know which attributes it has, we'll read
    // blending mode and culling from the material struct of the gltf model as well as the poly
    // mode from the parent polygone
    let blending_mode = {
        match mat.alpha_mode(){
            gltf::material::AlphaMode::Opaque =>{
                //println!("RENDING PASS THROUGH! ======================================================", );
                pipeline_builder::BlendTypes::BlendPassThrough
            },
            gltf::material::AlphaMode::Mask =>{
                //println!("RENDING ALPHA BLENDING! ======================================================", );
                pipeline_builder::BlendTypes::BlendAlphaBlending //TODO create a Shader for masking, this will come with the uber shading system
            },
            gltf::material::AlphaMode::Blend =>{
                //println!("RENDING ALPHA BLENDING! ======================================================", );
                pipeline_builder::BlendTypes::BlendAlphaBlending
            },

        }
    };

    let cull_mode = {
        if mat.double_sided(){
            //println!("RENDING DOUBLE SIDED! ======================================================", );
            pipeline_builder::CullMode::Back //Note I have to implement order independent transparency for this to work in complex models
        }else{
            //println!("RENDING SINGLE SIDED! ======================================================", );
            pipeline_builder::CullMode::Back
        }
    };

    //now create the requirements based on it
    let requirements = pipeline_manager::PipelineRequirements{
        blend_type: blending_mode,
        culling: cull_mode,
        render_pass: RenderPassConf::ObjectPass,
        shader_set: "Pbr".to_string(),
    };


    //Get the incredienses for building a material
    let (pipeline, uniform_manager, device) = {
        //get the device we are on
        let device = {
            let managers_lck = managers.lock().expect("failed to lock managers struct");
            (*managers_lck).device.clone()
        };

        //get the manager
        let pipeline_manager = {
            let managers_lck = managers.lock().expect("failed to lock managers struct");
            (*managers_lck).pipeline_manager.clone()
        };

        //Get the pipeline based on the needs of this material
        let mut pipeline_manager_lck = pipeline_manager.lock().expect("failed to lock pipe manager");
        //Build the pipeline by the requirements
        let pipeline = (*pipeline_manager_lck).get_pipeline_by_requirements(
            requirements, render::SubPassType::Forward.get_id() //need the forwad id
        );


        let uniform_manager = {
            let managers_lck = managers.lock().expect("failed to lock managers struct");
            (*managers_lck).uniform_manager.clone()
        };

        (pipeline, uniform_manager, device)
    };

    //build the final material
    let final_material = material_builder.build(&material_name, pipeline, uniform_manager, device);
    let material_manager = {
        let managers_lck = managers.lock().expect("failed to lock managers struct");
        (*managers_lck).material_manager.clone()
    };

    //now add a copy to the manager and return the name
    let mut material_manager_lck = material_manager.lock().expect("failed to lock material manager");
    //Add it and return its
    //println!("Finished loading material with name: {}", material_name);
    let material_in_manager_name = {
        match (*material_manager_lck).add_material(final_material){
            Ok(k) => k,
            Err(e) => e,
        }
    };

    (*material_manager_lck).get_material(&material_in_manager_name)
}

///Loads gltf primitves in an Vec<mesh::Mesh> and adds them to the managers as well as their textures
pub fn load_gltf_mesh(
    scene_name: String,
    mesh: &gltf::Mesh,
    buffers: &gltf_importer::Buffers,
    base: &Path,
    managers: &Arc<Mutex<core::resource_management::ManagerAndRenderInfo>>,
) -> Vec<Arc<Mutex<mesh::Mesh>>>{

    //this vec will be used to add new mesh nodes to the parent gltf node
    let mut return_vec = Vec::new();
    //the indices are used for nice naming
    let mut primitive_index = 0;
    //now cycle through all primitives, load the mesh and maybe the material
    for primitive in mesh.primitives(){
        use gltf_utils::PrimitiveIterators; //from the three crate
        let mut indices: Vec<u32> = Vec::new();
        //check for indices
        if let Some(mut iter) = primitive.indices_u32(buffers) {
            while let (Some(a), Some(b), Some(c)) = (iter.next(), iter.next(), iter.next()) {
                indices.push(a);
                indices.push(b);
                indices.push(c);
            }
        }
        //position
        let mut positions: Vec<[f32; 3]> = primitive
            .positions(buffers)
            .unwrap()
            .map(|x| x.into())
            .collect();
        //normal
        let mut normals: Vec<[f32; 3]> = if let Some(iter) = primitive.normals(buffers) {
            iter.map(|x| x.into()).collect()
        } else {
            Vec::new()
        };
        //tangents
        let mut tangents: Vec<[f32; 4]> = if let Some(iter) = primitive.tangents(buffers) {
            iter.map(|x| x.into()).collect()
        } else {
            Vec::new()
        };
        //tex_coors
        let mut tex_coords: Vec<[f32; 2]> = if let Some(iter) = primitive.tex_coords_f32(0, buffers) {
            iter.map(|x| x.into()).collect()
        } else {
            Vec::new()
        };
        //verte color
        let mut vertex_colors: Vec<[f32; 4]> = if let Some(iter) = primitive.colors_rgba_f32(0, 1.0, buffers) {
            iter.map(|x| x.into()).collect()
        } else {
            Vec::new()
        };

        let mesh_name = scene_name.clone() + "_mesh_" + &primitive_index.to_string();

        let (device, queue) = {
            let device = {
                let managers_lck = managers.lock().expect("failed to lock managers struct");
                (*managers_lck).device.clone()
            };
            let queue = {
                let managers_lck = managers.lock().expect("failed to lock managers struct");
                (*managers_lck).queue.clone()
            };

            (device, queue)
        };

        //get the fallback material for the mesh creation, if there is another materail set for
        // this mesh it will be created further down and be set.
        let fallback_material = {
            let material_manager = {
                let manager_lck = managers.lock().expect("failed to lock managers");
                (*manager_lck).material_manager.clone()
            };

            let mut material_manager_lck = material_manager.lock().expect("failed to lock material manager");
            (*material_manager_lck).get_default_material()
        };

        let mut add_mesh = mesh::Mesh::new(
            &mesh_name,
            device.clone(),
            queue.clone(),
            fallback_material
        );
        //create a dummy and fill it
        let mut vertices = Vec::new();

        //Have to update vectors to be as long as the positions
        if positions.len() != tex_coords.len(){
            tex_coords = vec![[0.0, 0.0]; positions.len()];
        }
        if positions.len() != normals.len(){
            normals = vec![[0.0, 0.0, 0.0]; positions.len()];
        }
        if positions.len() != tangents.len(){
            tangents = vec![[0.0, 0.0, 0.0, 0.0]; positions.len()];
        }
        if positions.len() != vertex_colors.len(){
            vertex_colors = vec![[0.0, 0.0, 0.0, 1.0]; positions.len()];
        }

        //after getting all the mesh informations, we have to find the mins and maxs of this mesh
        //to construct a static bound for it.
        let (mins, maxs) = {
            let mut min: [f32; 3] = [0.0; 3];
            let mut max: [f32; 3] = [0.0; 3];

            for position in positions.iter(){
                //X val
                //min
                if position[0] < min[0]{
                    min[0] = position[0];
                }
                //max
                if position[0] > max[0]{
                    max[0] = position[0];
                }


                //Y val
                //min
                if position[1] < min[1]{
                    min[1] = position[1];
                }
                //max
                if position[1] > max[1]{
                    max[1] = position[1];
                }
                //Z val
                //min
                if position[2] < min[2]{
                    min[2] = position[2];
                }
                //max
                if position[2] > max[2]{
                    max[2] = position[2];
                }
            }

            (min,max)
        };

        for i in 0..positions.len(){
            let vertex = mesh::Vertex::new(
                positions[i],
                tex_coords[i],
                normals[i],
                tangents[i],
                vertex_colors[i],
            );
            vertices.push(vertex);
        }
        //write new vertices as well as indices to mesh
        add_mesh.set_vertices_and_indices(vertices, indices);
        //TODO SETUP BOUNDS
        add_mesh.set_bound(
            Point3::new(mins[0], mins[1], mins[2]),
            Point3::new(maxs[0], maxs[1], maxs[2])
        );


        //look for materials
        let mesh_material = primitive.material();
        //test if its the default material if not, test if this material si alread in the scene
        println!("SORTING MATERIAL: ", );
        match mesh_material.index(){
            None => {
                //is the default material, we can leave the mesh material like it is
                //println!("\tIs using default material ... ", );
            },
            Some(material_index) =>{
                //create a String for the material name, then check for it, if it isn't in there
                //create a material from this name
                let material_name = String::from(scene_name.clone()) + "_material_" + &material_index.to_string();
                //println!("\tIs non default with name: {}", material_name.clone());
                //we need to lock the material manager twice seperatly because we otherwise get a memory lock
                let is_in_manager = {
                    //first check if there is already a material with this name, if not create one
                    let material_manager = {
                        let managers_lck = managers.lock().expect("failed to lock managers struct");
                        (*managers_lck).material_manager.clone()
                    };

                    //It has a material, check if its alread in the material manager by name
                    let mut material_manager_lck = material_manager
                    .lock()
                    .expect("could not look material manager");

                    (*material_manager_lck).is_available(&material_name)
                };
                //if it has already the material, search for it and set it a s the meshes material
                // else create a material with this name
                if is_in_manager {
                    //lock the material manager
                    let material_manager = {
                        let managers_lck = managers.lock().expect("failed to lock managers struct");
                        (*managers_lck).material_manager.clone()
                    };

                    let mut material_manager_lck = material_manager
                    .lock()
                    .expect("could not look material manager");
                    //now the the material
                    add_mesh.set_material((*material_manager_lck).get_material(&material_name));
                }else{

                    let new_material = load_gltf_material(
                            &mesh_material,
                            material_name,
                            &buffers,
                            &base,
                            managers,
                    );
                    add_mesh.set_material(new_material);
                }
            }
        }
        //We finished the mesh, time to put it in an Arc<Mutex<mesh::Mesh>>
        let arc_mesh = Arc::new(Mutex::new(add_mesh));
        //Now copy it to the manager and push the other one to the return vector
        let mesh_manager = {
            let managers_lck = managers.lock().expect("failed to lock managers struct");
            (*managers_lck).mesh_manager.clone()
        };

        let mut mesh_manager_lck = mesh_manager.lock().expect("failed to lock mesh manager in gltf loader");
        (*mesh_manager_lck).add_arc_mesh(arc_mesh.clone());
        //pushing to the return vector, continueing with the other meshes
        return_vec.push(arc_mesh);
        //adding one to the index for naming the new mesh
        primitive_index += 1;
    }

    return_vec
}

///Loads a gltf node into the right node::GenericNode
pub fn load_gltf_node(
    gltf_node: &gltf::Node, //used to reference gltf stuff
    scene_name: &String, //used to generate a short, but unique name from the node id
    parent_transform: Option<Decomposed<Vector3<f32>, Quaternion<f32>>>, //used to construct the initial location of self
    parent_node_name: &String, //used to add the node in the tree
    tree: &mut tree::Tree<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>, //the actual tree
    buffers: &gltf_importer::Buffers, //the buffers to read the gltf info from
    base: &Path, //the base path from the node
    managers: &Arc<Mutex<core::resource_management::ManagerAndRenderInfo>>, //teh manager to add textures etc to.
)
{
    //creates the new name, based on the indice
    let new_name = scene_name.clone() + "_node_" + &gltf_node.index().to_string();

    //let mut this_node = node::GenericNode::new_empty(&new_name);
    //println!("Created node: {}", new_name.clone());
    //get the transform of this node
    let node_transform = {
        let mut new_transform: Decomposed<Vector3<f32>, Quaternion<f32>> = Decomposed::one();

        //if we got a parent transform we can add it to the current transform, if not we have to leave it
        let offset_transform = {
            match parent_transform {
                Some(trans) => trans,
                //if there is no extra transform make all of them 0.0
                None => Decomposed{
                    scale: 1.0,
                    rot: Quaternion::zero(),
                    disp: Vector3::new(0.0, 0.0, 0.0),
                }

            }
        };

        let node_transform = gltf_node.transform().decomposed();

        //According to the gltf crate the decomposed is (translation, rotation, scale).
        //translation is the 0th field of decomposed with 3 elements
        let translation = Vector3::new(
            node_transform.0[0] + offset_transform.disp.x,
            node_transform.0[1] + offset_transform.disp.y,
            node_transform.0[2] + offset_transform.disp.z
        );
        //The 1th element is rotation and rotation is in the format of [w,x,y,z]
        //the rotation in gltf is saved as x,y,z,w while in cgmath its w,x,y,z, therefore we need to change
        let rotation = {
            let tmp_rot = Quaternion::new(
                node_transform.1[3],
                node_transform.1[0],
                node_transform.1[1],
                node_transform.1[2]
            );
            tmp_rot //+ offset_transform.rot
        };

        //NOTE: Scale is currently only linear in one direction, this might be changed in future to
        //be comformant to the gltf2.0 rules
        let scale = {
            node_transform.2[0]  * offset_transform.scale //is currently only the x value
        };
        /*
        println!("Node Transfrom:", );
        println!("\t Translation: {}, {}, {}", translation.x, translation.y, translation.z);
        println!("\t Rotation   : {}, {}, {}, {}", rotation.v.x, rotation.v.y, rotation.v.z, rotation.s);
        println!("\t Scale      : {}", scale);
        */
        //update the transform
        new_transform.scale = scale;
        new_transform.disp = translation;
        new_transform.rot = rotation;
        new_transform

    };

    //now create the node as an empty, this empty will be the parent to:
    // A: all meshs, cameras and later lights attached to it
    // B: all sub gltf nodes
    let empty = empty::Empty::new(&new_name);
    let empty_value = content::ContentType::Empty(empty);
    let mut node_attributes = attributes::NodeAttributes::default();
    node_attributes.transform = node_transform;
    //add the empty to the parent node in tree
    tree.add(empty_value, parent_node_name.clone(), Some(node_attributes));

    //check for a mesh in the gltf_node
    match gltf_node.mesh(){
        Some(mesh) =>{
            //println!("Found mesh in node: {}", new_name.clone());
            //load the primitves as an Vec<mesh::Mesh>
            let primitives = load_gltf_mesh(
                scene_name.clone(),
                &mesh,
                &buffers,
                base,
                managers,
            );
            //println!("Finished loading mesh from gltf, adding to node...", );
            //create a node from every mesh and add it to the own Node
            for prim in primitives{
                //create the mesh node
                let mesh_name = new_name.clone() + "_mesh_" + &mesh.index().to_string();
                //now we lock the mesh for a moment to decide:
                // transparency
                // bound
                // TODO shadow casting based on size?
                // transform
                let prim_attrib = {
                    //create a default set of values, first change the transform param to the same
                    // like the parent node
                    let mut attrib = attributes::NodeAttributes::default();
                    attrib.transform = node_transform;
                    //now we have to lock the mesh and then get its material, the the material
                    // transparency param and use it to set the transparency bool.
                    // After that we can read the bound info of the mesh and use it as the node bound
                    // because of the nature of this node (at least at import time) it is save to assume
                    // that the bound won't change till the next rebuild.
                    let transparent = {
                        let mesh_lck = prim.lock().expect("failed to lock the mesh while importing");

                        let material = (*mesh_lck)
                        .get_material();
                        let material_lck = material
                        .lock()
                        .expect("failed to lock mesh material while importing");
                        //well we have to get the pipeline now and match the transparency config.
                        // if its alpha belnding, set to transparent
                        // all other types (including masked operation) can drawn unordered
                        match (*material_lck).get_pipeline().pipeline_config.blending_operation {
                            pipeline_builder::BlendTypes::BlendAlphaBlending => {
                                true
                            },
                            _ => {
                                false
                            }
                        }
                    };

                    attrib.is_transparent = transparent;
                    attrib.bound = (*(prim.lock().expect("failed to lock mesh"))).get_bound();
                    attrib.value_bound = (*(prim.lock().expect("failed to lock mesh"))).get_bound();
                    //return the correct mesh attributes
                    attrib

                };
                //create a content struct from the mesh
                let mesh_node_value = content::ContentType::Mesh(prim);
                //now add this mesh node to the current tree together with its forged attributes :D
                let _ = tree.add(mesh_node_value, new_name.clone(), Some(prim_attrib));
            }
        }
        None => {}, //no mesh found for this node
    }

    //check for Camera
    //TODO

    //cycle to children based on own root node as parent
    //TODO

    //after adding everything to the current node, have a look for children, if there are any,
    //iterate through them, always create a node, load it and add it to the current parent
    if gltf_node.children().len() > 0{
        for child in gltf_node.children(){
            let new_child = load_gltf_node(
                &child,
                scene_name,
                Some(node_transform),
                &new_name,
                tree,
                buffers,
                base,
                managers,
            );
        }
    }
}

///Imports a scene from the file at `path`
pub fn import_gltf(
    path: &str, name: &str,
    managers: Arc<Mutex<core::resource_management::ManagerAndRenderInfo>>
){
    //load the gltf model into a gltf object
    let path = Path::new(path);
    //a default path if `path` doesn't exist, should load a default object in future
    let default = Path::new("");
    //go to the parent directory and load every gltf in this directory
    let base = path.parent().unwrap_or(default);
    //TODO don't panic, load a debug object
    let (gltf, buffers) = gltf_importer::import(path).expect("invalid model for gltf 2.0 loader");


    //build an empty root node
    let empty_object = empty::Empty::new(name);
    let empty_node = content::ContentType::Empty(empty_object);
    //create a tree from it
    //TODO we might have to read the roots node location, rotation and scale
    let mut scene_tree = tree::Tree::new(empty_node, attributes::NodeAttributes::default());

    for scene in gltf.scenes(){
        //create an empty scene node with the correct name
        let scene_name = String::from(name) + "_scene_" + &scene.index().to_string();

        let empty_scene_object = empty::Empty::new(&scene_name);
        let scene_node = content::ContentType::Empty(empty_scene_object);
        //now add the node to the tree
        let _ = scene_tree.add_at_root(scene_node, None); //TODO might need to read scene location/rotation/scale
        //now cycle through its nodes and add the correct meshes, lights whatever to it
        for node in scene.nodes(){

            //loading each node in this scene
            load_gltf_node(
                &node,
                &String::from(name), //This is the name of this gltf file used to reference global gltf file specific data like textures and materials
                None,       //Scenes dont have a transform
                &scene_name,
                &mut scene_tree,
                &buffers,
                base,
                &managers,
            );
        }
        //Now we added all nodes to the scene tree and can return
    }

    //Done with loading gltf
    let manager_lck = managers.lock().expect("failed to lock managers");
    let scene_manager = (*manager_lck).scene_manager.clone();
    let mut scene_manager_inst = scene_manager.lock().expect("failed to lock scene manager");

    (*scene_manager_inst).add_scene(scene_tree);
}
