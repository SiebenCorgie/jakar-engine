
use assimp;
use cgmath::*;
use std::sync::Arc;
use vulkano;

use rt_error;
use tools::Importer;
use core::ReturnBoundInfo;
use core::resources::mesh;
use core::resources::mesh::Vertex;
use tools;

pub struct AssimpImporter {}


impl Importer for AssimpImporter {
    ///Returns an importer object
    fn new() -> Self{
        AssimpImporter{}
    }

    ///Returns a full scene Graph from the data at `path`
    fn import(&mut self, path: &str, name: &str, device: Arc<vulkano::device::Device>,
        queue: Arc<vulkano::device::Queue>)
        -> Vec<mesh::Mesh>
    {

        let mut loc_assimp = assimp::Importer::new();
        //Setup import settings, later maybe from a dialog
        //TODO implement dialog in GUI stage of engine
        loc_assimp.triangulate(true);
        loc_assimp.calc_tangent_space(|x| x.enable = true);
        //Maybe more if needed
        //loc_assimp.generate_normals(|x| x.enable = true);
        //loc_assimp.flip_uvs(true);

        //Import scene with all meshes
        let scene = loc_assimp.read_file(path);

        //dummy root, gets no real mesh
        let mut mesh_collection = Vec::new();

        let mut mesh_append = 0;

        match scene{
            Ok(scene)=>{

                for mesh in scene.mesh_iter() {

                    //Create a dummy mesh to be stored
                    let tmp_name = String::from(name.clone()) + "_" + &mesh_append.to_string();
                    let mut tmp_mesh = mesh::Mesh::new(&tmp_name.clone(), device.clone(), queue.clone());
                    //println!("Adding {} to tmp_root", tmp_name);
                    mesh_append += 1;
                    //Create empty create_info for the mesh
                    let mut tmp_vertices: Vec<Vertex> = Vec::new();
                    let mut tmp_indices: Vec<u32> = Vec::new();
                    let mut bound_info: tools::BoundCreateInfo = tools::BoundCreateInfo::new();

                    //used to create first time actions;
                    let mut first_vertice = true;

                    //The vertices
                    for index in 0..mesh.num_vertices()
                    {
                        let mut pos: [f32; 3] = [0.0; 3];
                        let mut tex: [f32; 2] = [0.0; 2];
                        let mut norm: [f32; 3] = [0.0; 3];
                        let mut tang: [f32; 4] = [0.0; 4];
                        let mut col: [f32; 4] = [0.0; 4];

                        //TODO make save
                        //POSITION
                        //Set position (has to have positions)
                        match mesh.get_vertex(index){
                            Some(position) => pos = position.into(),
                            None => {
                                //println!("Failed to find position on index: {} of mesh: {}", index.clone(), tmp_name);
                                //fallback
                                pos = Vector3::new(1.0, 1.0, 1.0).into();
                            },
                        }
                        //if this was the first vertice, generate the bound from this starting location
                        if first_vertice{
                            bound_info.max_x = pos[0];
                            bound_info.max_y = pos[1];
                            bound_info.max_z = pos[2];

                            bound_info.min_x = pos[0];
                            bound_info.min_y = pos[1];
                            bound_info.min_z = pos[2];
                        }
                        first_vertice = false;

                        //UVs
                        //Search on channel 0 by default
                        match mesh.get_texture_coord(0, index){
                            Some(uv_coords) => {
                                    //Have to convert this from vec3 to Vec2,
                                    //TODO find out if we need 3d coordinates
                                    let vec3: [f32; 3] = uv_coords.into();
                                    tex = [vec3[0], vec3[1]];
                                },
                            None =>{
                                println!("Failed to find uv_coords on index: {} of mesh: {}", index.clone(), tmp_name);
                                //fallback
                                tex = Vector2::new(1.0, 1.0).into();
                            }
                        }

                        //NORMALS
                        match mesh.get_normal(index){
                            Some(normal) => norm = normal.into(),
                            None => {
                                println!("Failed to find normals on index: {} of mesh: {}", index.clone(), tmp_name);
                                //fallback
                                norm = Vector3::new(1.0, 1.0, 1.0).into();
                            },
                        }
                        //TANGENTS
                        match mesh.get_tangent(index){
                            Some(tangent) =>{
                                //convert to a vec4
                                let tang_3: [f32; 3] = tangent.into();
                                tang = [tang_3[0], tang_3[1], tang_3[2], -1.0];
                            }
                            None => {
                                println!("Failed to find tangent on index: {} of mesh: {}", index.clone(), tmp_name);
                                //fallback
                                tang = [1.0; 4];
                            },
                        }

                        //match mesh.
                        col = [0.0; 4];
                        /* THERE IS CURRENTLY NO VERTEX COLOR SUPPORT
                        if mesh.has_vertex_colors(index as usize){
                            ////println!("has color");
                            col = mesh.get_tangent(index).unwrap().into();
                        }
                        */
                        //Check bounds and update if needed
                        //max
                        //x
                        if pos[0] > bound_info.max_x{
                            bound_info.max_x = pos[0].clone();
                        }
                        //y
                        if pos[1] > bound_info.max_y{
                            bound_info.max_y = pos[1].clone();
                        }
                        //z
                        if pos[2] > bound_info.max_z{
                            bound_info.max_z = pos[2].clone();
                        }

                        //min
                        //x
                        if pos[0] < bound_info.min_x{
                            bound_info.min_x = pos[0].clone();
                        }
                        //y
                        if pos[1] < bound_info.min_y{
                            bound_info.min_y = pos[1].clone();
                        }
                        //z
                        if pos[2] < bound_info.min_z{
                            bound_info.min_z = pos[2].clone();
                        }

                        //Add the info to the vertex vector
                        tmp_vertices.push(Vertex::new(pos, tex, norm, tang, col));



                    }

                    // Safe to assume all faces are triangles due to import options
                    for face in mesh.face_iter() {
                        tmp_indices.push(face[0]);
                        tmp_indices.push(face[1]);
                        tmp_indices.push(face[2]);
                    }
                    {
                        tmp_mesh.set_vertices_and_indices(tmp_vertices, tmp_indices, device.clone(), queue.clone());
                        tmp_mesh.set_bound(bound_info.get_info_min(), bound_info.get_info_max());
                    } //Release mesh_ref for addind tmp_mesh to the scene tree
                    //Add the mesh to the collection
                    mesh_collection.push(tmp_mesh);
                }
            },
            Err(_)=> rt_error("ASSIMP_IMPORTER", "Loading scene failed"),

        }
        //return the imported scene
        mesh_collection
    }
}
