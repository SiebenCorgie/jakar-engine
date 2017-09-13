///The engines top level

extern crate cgmath;
extern crate collision;
//extern crate assimp; Put the assimp crate out because of legacy and compile problems

//All thrid party crates
extern crate winit;
#[macro_use]
extern crate vulkano;
#[macro_use]
extern crate vulkano_shader_derive;
extern crate vulkano_win;
extern crate time;
extern crate image;
extern crate gltf;
extern crate gltf_importer;
extern crate gltf_utils;


///The engine core defines most functions and
///traits needed to feed the renderer and communicate with the physics.
///It also mamanges the scene tree and how to get specific information out of it
pub mod core;

///The engines renderer currently WIP
pub mod render;

///A collection of helpfull tools for integration of data with the engine
pub mod tools;

///A small thread who will run and administrate the winit window, as well as its input
///processing
pub mod input;


//Some Helper functions
///Returns an runtime error
pub fn rt_error(location: &str, content: &str){
    println!("ERROR AT: {} FOR: {}", location, content);
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}


/*TODO
3rd Render on main thread, manage materials on event on different thread,
manage objects on secondary thread, manage loading on n-threads (per object?)
4th then continue

4th create get_*_in_frustum functions for all types in ContentTypes done, needs to be tested
5th create a high level fn collection for adding and removing things from the scene tree
6th build a simple forward renderer with vulkano and test the scene tree //NOTE Done in 3.1-3.4 ?
7th make core, render and later physics independend threads //NOTE Done in 3.1-3.4 ?
8th multithread asset import //NOTE Done in 3.1-3.4 ?
9th add lights to the renderer
10th shadow generation?

9th CREATE A FLUFFY TILED RENDERER WITH ALL THE STUFF
10th PBR ?
11th Editor and templates https://www.youtube.com/watch?v=UWacQrEcMHk , https://www.youtube.com/watch?annotation_id=annotation_661107683&feature=iv&src_vid=UWacQrEcMHk&v=xYiiD-p2q80 , https://www.youtube.com/watch?annotation_id=annotation_2106536565&feature=iv&src_vid=UWacQrEcMHk&v=yIedljapuz0
*/


/*TODO
*/
