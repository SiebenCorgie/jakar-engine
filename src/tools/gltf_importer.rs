use core::resources::{texture, material, mesh, light, camera};
use core::resources::camera::Camera;
use core::simple_scene_system::node;
use core::resource_management::{material_manager, mesh_manager, scene_manager, texture_manager};

use vulkano;

use cgmath::*;

use gltf;
use gltf_importer;
use image;

use std::path::Path;
use std::sync::{Arc, Mutex};
///An Import struct used to store information needed to create all meshes, textures etc.
pub struct GltfImporter {
    root_node: node::GenericNode,
    device: Arc<vulkano::device::Device>,
    queue: Arc<vulkano::device::Queue>
}

//TODO everything into struct which also holds a copy of the texture, material, mesh and scene
//manager for easy adding etc.

///Imports a gltf texture
pub fn load_gltf_texture(
    texture: &gltf::Texture,
    name: String,
    buffers: &gltf_importer::Buffers,
    base: &Path,
    material_manager: &Arc<Mutex<material_manager::MaterialManager>>,
    texture_manager: &Arc<Mutex<texture_manager::TextureManager>>,
) -> Arc<texture::Texture>
{
    //The texture can be a buffer or an external file, depending on the case we load the texture
    //wrap it into an Arc<Texture>, then add it to the manager once and return the other one

    //first create a texture builder and configure it with the right sampler from the provided texture
    let mut texture_builder = {
        //lock the texture manager once to get some data
        let texture_manager_lck = texture_manager.lock().expect("failed to lock texture manager");
        //No create the textuer builder based on the type of data
        match texture.source().data(){
            gltf::image::Data::View{view, mime_type} => {
                //found a data buffer for the image
                let data = buffers.view(&view).expect("failed to load image data from gltf buffer");
                //we got the data, lets provide it to a TextureBuilder
                texture::TextureBuilder::from_data(
                    data,
                    (*texture_manager_lck).get_device(),
                    (*texture_manager_lck).get_queue(),
                    (*texture_manager_lck).get_settings(),
                )
            },
            gltf::image::Data::Uri{uri, mime_type} =>{
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

    //finally build the texture
    let new_texture = texture_builder.build_with_name(&name);
    //now add a copy to the manager and return the other one
    {
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

///Imports a gltf material, returns the name of the loaded materials
pub fn load_gltf_material(
    mat: &gltf::Material,
    material_name: String,
    buffers: &gltf_importer::Buffers,
    base: &Path,
    material_manager: &Arc<Mutex<material_manager::MaterialManager>>,
    texture_manager: &Arc<Mutex<texture_manager::TextureManager>>,
) -> String{
    //first load the pbr info
    let pbr = mat.pbr_metallic_roughness();
    //now load all textures if there is none it returns none which will be respected at build time of the material
    let albedo = {
        match pbr.base_color_texture(){
            Some(t) => {
                Some(
                    load_gltf_texture(
                    &t.texture(), material_name.clone() + "_albedo", buffers, base, material_manager, texture_manager
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
                    &t.texture(), material_name.clone() + "_normal", buffers, base, material_manager, texture_manager
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
                    &t.texture(), material_name.clone() + "_met_rough", buffers, base, material_manager, texture_manager
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
                    &t.texture(), material_name.clone() + "_occlu", buffers, base, material_manager, texture_manager
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
                    &t.texture(), material_name.clone() + "_emissive", buffers, base, material_manager, texture_manager
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


    let fallback_texture = {
        let man_lck = texture_manager.lock().expect("failed to lock material manager");
        (*man_lck).get_none()
    };

    //Create a material builder from the info
    let mut material_builder = material::MaterialBuilder::new(
        albedo,
        normal,
        metallic_roughness,
        occlusion,
        emissive,
        fallback_texture,
    )
    //now configure the factors
    .with_factors(texture_factors);


    //TODO now, for every textuer do:
    // - load texture and its sampler info
    // - setup factors
    // - create material from textures
    // - add it to the texture manager
    // - return the name of it as reference for the mesh
    String::from("Teddy")
}


///Loads gltf primitves in an Vec<mesh::Mesh> and adds them to the managers as well as their textures
pub fn load_gltf_mesh(
    name: &String,
    scene_name: &str,
    mesh: &gltf::Mesh,
    buffers: &gltf_importer::Buffers,
    base: &Path,
    mesh_manager: &Arc<Mutex<mesh_manager::MeshManager>>,
    material_manager: &Arc<Mutex<material_manager::MaterialManager>>,
    texture_manager: &Arc<Mutex<texture_manager::TextureManager>>,
    device: &Arc<vulkano::device::Device>,
    queue: &Arc<vulkano::device::Queue>
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
        let positions: Vec<[f32; 3]> = primitive
            .positions(buffers)
            .unwrap()
            .map(|x| x.into())
            .collect();
        //normal
        let normals: Vec<[f32; 3]> = if let Some(iter) = primitive.normals(buffers) {
            iter.map(|x| x.into()).collect()
        } else {
            Vec::new()
        };
        //tangents
        let tangents: Vec<[f32; 4]> = if let Some(iter) = primitive.tangents(buffers) {
            iter.map(|x| x.into()).collect()
        } else {
            Vec::new()
        };
        //tex_coors
        let tex_coords: Vec<[f32; 2]> = if let Some(iter) = primitive.tex_coords_f32(0, buffers) {
            iter.map(|x| x.into()).collect()
        } else {
            Vec::new()
        };
        //verte color
        let vertex_colors: Vec<[f32; 4]> = if let Some(iter) = primitive.colors_rgba_f32(0, 1.0, buffers) {
            iter.map(|x| x.into()).collect()
        } else {
            Vec::new()
        };

        //TODO create mesh, as Arc, store it in the mesh manager, look for materials, if
        let mesh_name = name.clone() + "_mesh_" + &primitive_index.to_string();
        let mut add_mesh = mesh::Mesh::new(&mesh_name, device.clone(), queue.clone());
        //create a dummy and fill it
        let mut vertices = Vec::new();
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
        add_mesh.set_vertices_and_indices(vertices, indices, device.clone(), queue.clone());
        //look for materials
        let mesh_material = primitive.material();
        //test if its the default material if not, test if this material si alread in the scene
        match mesh_material.index(){
            None => {
                //is the default material, we can leave the mesh material like it is
            },
            Some(material_index) =>{

                //create a String for the material name, then check for it, if it isn't in there
                //create a material from this name
                let material_name = String::from(scene_name) + &material_index.to_string();

                let has_material ={
                    //It has a material, check if its alread in the material manager by name
                    let material_manager_lck = material_manager
                    .lock()
                    .expect("could not look material manager");
                    (*material_manager_lck).is_available(&material_name)
                };
                //if the material is already there we can change the mesh mateiral to this name
                //iof not we have to create it first and change then
                if has_material{
                    add_mesh.set_material(&material_name);
                }else{
                    //Damn has no such material will create one
                    add_mesh.set_material(&load_gltf_material(
                        &mesh_material,
                        material_name,
                        &buffers,
                        &base,
                        &material_manager,
                        &texture_manager)
                    );
                }
            }
        }
        //We finished the mesh, time to put it in an Arc<Mutex<mesh::Mesh>>
        let arc_mesh = Arc::new(Mutex::new(add_mesh));
        //Now copy it to the manager and push the other one to the return vector
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
    node: &gltf::Node,
    parent: &mut node::GenericNode,
    parent_name: String,
    scene_name: &str,
    buffers: &gltf_importer::Buffers,
    base: &Path,
    mesh_manager: &Arc<Mutex<mesh_manager::MeshManager>>,
    material_manager: &Arc<Mutex<material_manager::MaterialManager>>,
    texture_manager: &Arc<Mutex<texture_manager::TextureManager>>,
    device: &Arc<vulkano::device::Device>,
    queue: &Arc<vulkano::device::Queue>
)
{
    //creates the new name, based on the indice
    let new_name = parent_name + "_node_" + &node.index().to_string();
    let mut node_object: Option<node::ContentType> = None;

    //get the transform of this node
    let node_transform = {
        let mut new_transform: Decomposed<Vector3<f32>, Quaternion<f32>> = Decomposed::one();
        let node_transform = node.transform().decomposed();
        //According to the gltf crate the decomposed is (translation, rotation, scale).
        //translation is the 0th field of decomposed with 3 elements
        let translation = Vector3::new(
            node_transform.0[0], node_transform.0[1], node_transform.0[2]
        );
        //The 1th element is rotation and rotation is in the format of [w,x,y,z]
        let rotation = Quaternion::new(
            node_transform.1[0], node_transform.1[1], node_transform.1[2], node_transform.1[3]
        );
        //NOTE: Scale is currently only linear in one direction, this might be changed in future to
        //be comformant to the gltf2.0 rules
        let scale = {
            node_transform.2[0] //is currently only the x value
        };

        //update the transform
        new_transform.scale = scale;
        new_transform.disp = translation;
        new_transform.rot = rotation;
        new_transform
    };

    //check for a mesh in the node
    match node.mesh(){
        Some(mesh) =>{
            //load the primitves as an Vec<mesh::Mesh>
            let primitives = load_gltf_mesh(
                &new_name,
                scene_name,
                &mesh,
                &buffers,
                base,
                &mesh_manager,
                &material_manager,
                &texture_manager,
                &device,
                &queue
            );
            //create a node from every mesh and add it to the own Node
            //TODO
            for mesh in primitives{
                let mesh_node = node::ContentType::Renderable(node::RenderableContent::Mesh(mesh));
                parent.add_child(mesh_node);
            }
        }
        None => {}, //no mesh found for this node
    }
    //check for Camera
    //TODO

    //cycle to children based on own root node as parent
    //TODO

    //Done!
}

///Imports a scene from the file at `path`
pub fn import_gltf(
    path: &str, name: &str,
    mesh_manager: &Arc<Mutex<mesh_manager::MeshManager>>,
    material_manager: &Arc<Mutex<material_manager::MaterialManager>>,
    texture_manager: &Arc<Mutex<texture_manager::TextureManager>>,
    device: &Arc<vulkano::device::Device>,
    queue: &Arc<vulkano::device::Queue>
){
    //load the gltf model into a gltf object
    let path = Path::new(path);
    //a default path if `path` doesn't exist, should load a default object in future
    let default = Path::new("");
    //go to the parent directory and load every gltf in this directory
    let base = path.parent().unwrap_or(default);
    //TODO don't panic, load a debug object
    let (gltf, buffers) = gltf_importer::import(path).expect("invalid model for gltf 2.0 loader");


    let mut scene_tree = node::GenericNode::new_empty(name);


    for scene in gltf.scenes(){
        //create an empty scene node with the correct name
        let scene_name = String::from(name) + "_scene_" + &scene.index().to_string();
        let mut scene_node = node::GenericNode::new_empty(&scene_name.to_string());
        //now cycle through its nodes and add the correct meshes, lights whatever to it
        for node in scene.nodes(){
            //loading ech node in this scene
            load_gltf_node(
                &node,
                &mut scene_node,
                scene_name.clone(), //The node name is now the scene name because a gltf file can have many
                            //scene which are in the node::GenericNode view also nodes
                name,       //This is the name of this gltf file used to reference global gltf file specific data like textures and materials
                &buffers,
                base,
                mesh_manager,
                material_manager,
                texture_manager,
                &device,
                &queue
            );
        }
        //now add the new scene node to the root empty
        scene_tree.add_node(scene_node);
    }
}
