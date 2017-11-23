extern crate vulkano;
extern crate jakar_engine;
extern crate cgmath;
extern crate jakar_tree;

use cgmath::*;

use jakar_engine::*;
use jakar_engine::core::resources::*;
use jakar_engine::core::resources::camera::Camera;
use jakar_engine::core::resources::light;
use jakar_engine::core::next_tree::*;

use std::thread;
use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};




extern crate winit;

fn main() {

    let settings = core::engine_settings::EngineSettings::new()
    .with_dimensions(1600, 900)
    .with_name("jakar Instance")
    .in_release_mode()
    .with_fullscreen_mode(false)
    .with_cursor_state(winit::CursorState::Grab)
    .with_cursor_visibility(winit::MouseCursor::NoneCursor)
    .with_msaa_factor(4)
    ;

    //Start the engine
    let mut engine = match jakar_engine::JakarEngine::start(Some(settings)){
        Ok(eng) => eng,
        Err(er) => {
            println!("Failed to create engine!");
            return;
        }
    };


    engine.get_asset_manager().import_gltf("TestScene", "examples/simple_scene/TestScene/TestScene.gltf");


    //SUN========================================================================
    let mut sun = light::LightDirectional::new("Sun");
    //looking down in vulkan space
    sun.set_direction(Vector3::new(1.0, -0.5, 0.0));
    sun.set_color(Vector3::new(1.0, 0.75, 0.75));
    sun.set_intensity(25.0);


    engine.get_asset_manager().get_active_scene().add_at_root(content::ContentType::DirectionalLight(sun), None);
    //SUN========================================================================




    engine.get_asset_manager().get_active_scene().print_tree();

    let mut scene_added = false;

    'game_loop: loop{

        //try adding by brute force to the main scene, could be handled nice :D
        if !scene_added{
            println!("Adding Test Scene to main scnene", );
            match engine.get_asset_manager().add_scene_to_main_scene("TestScene"){
                Ok(_) => scene_added = true,
                Err(_) => {
                    println!("Could not find TestScene", );
                }
            }
        }

        //try to get the TestScene and move it if a key is pressed

        //test if a is pressed
        if engine.get_asset_manager().get_keymap().a{

            match engine.get_asset_manager().get_active_scene().get_node("TestScene".to_string()){
                Some(scene) => {
                    scene.add_job(jobs::SceneJobs::Move(Vector3::new(1.0, 0.0, 0.0)));
                }
                None => {println!("Could not find TestScene :( !0!0!0!=!=!=!0!=!=!=!=!=!0!0!0", );}, //get on with it
            }
        }

        if engine.get_asset_manager().get_keymap().q{
            let mut asset_manager = engine.get_asset_manager();
            asset_manager.get_scene_manager().print_all_scenes();
        }

        //test if a is pressed
        if engine.get_asset_manager().get_keymap().escape{
            println!("Scene: ", );
            engine.get_asset_manager().get_active_scene().print_tree();
            engine.end();
            break;
        }

        thread::sleep(Duration::from_millis(100));

    }

}
