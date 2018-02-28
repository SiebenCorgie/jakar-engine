
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
use jakar_tree::node::Attribute;

use std::thread;
use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};




extern crate winit;

fn main() {

    let graphics_settings = core::render_settings::RenderSettings::default()
    .with_msaa_factor(8)
    .with_gamma(1.0)
    .with_exposure(1.0)
    .with_anisotropical_filtering(16);

    let settings = core::engine_settings::EngineSettings::default()
    .with_dimensions(1600, 900)
    .with_name("Jakar Instance")
    .in_release_mode()
    .with_input_poll_speed(400)
    .with_fullscreen_mode(false)
    .with_cursor_state(winit::CursorState::Normal)
    .with_cursor_visibility(winit::MouseCursor::Default)
    .with_render_settings(graphics_settings)
    .with_asset_update_speed(100)
    .with_max_fps(200)
    .with_camera_settings(core::engine_settings::CameraSettings{
        far_plane: 100.0,
        near_plane: 0.1,
    })
    ;


    //Start the engine
    let mut engine = match jakar_engine::JakarEngine::start(Some(settings)){
        Ok(eng) => eng,
        Err(er) => {
            println!("Failed to create engine!");
            return;
        }
    };


    engine.get_asset_manager().import_gltf("TestScene", "examples/simple_scene/TestScenes/Cube_Plane.gltf");


    let mut light_tree =jakar_tree::tree::Tree::new(
        jakar_engine::core::next_tree::content::ContentType::Empty(core::resources::empty::Empty::new("LightsRoot")),
        jakar_engine::core::next_tree::attributes::NodeAttributes::default()
    );


    //SUN========================================================================
    //add a matrix of lights

    for x in -10..10{
        for y in -10..10{
            let mut point = light::LightPoint::new("LightPoint");
            point.set_intensity(
                5.0
            );
            point.set_color(
                Vector3::new(
                    (x + 5) as f32 / 15.0, (x + 10) as f32 / 100.0, (y + 5) as f32 / 15.0
                )
            );
            point.set_radius(2.0);

            let node_name = light_tree
            .add_at_root(content::ContentType::PointLight(point), None);

            //Set the location
            match light_tree.get_node(&node_name.unwrap()){
                Some(scene) => {
                    scene.add_job(
                        jobs::SceneJobs::Move(
                            Vector3::new(
                                x as f32 * 2.0,
                                (x + y) as f32 * 2.0,
                                y as f32 * 2.0
                            )
                        )
                    );
                }
                None => {println!("Could not find Light", );}, //get on with it
            }
        }
    }
    /*
    //Now add a sun
    let mut sun = light::LightDirectional::new("Sunny");
    sun.set_intensity(25.0);
    sun.set_color(Vector3::new(1.0, 0.85, 0.9));
    let sun_node = light_tree.add_at_root(content::ContentType::DirectionalLight(sun), None);
    //Now rotate it a bit on x
    match light_tree.get_node("Sunny"){
        Some(sun)=> {
            sun.add_job(jobs::SceneJobs::Rotate(Vector3::new(0.0, 0.0, -60.0)));
        },
        None => {println!("Could not find sun", );}
    }
    */

    light_tree.update();
    engine.get_asset_manager().get_active_scene().join_at_root(&light_tree);
    engine.get_asset_manager().get_active_scene().update();
    //println!("THE SCENE ==================================================", );
    //engine.get_asset_manager().get_active_scene().print_tree();
    //println!("END ========================================================", );
    let mut scene_added = false;

    'game_loop: loop{

        //try adding by brute force to the main scene, could be handled nice :D
        if !scene_added{
            //println!("Adding Test Scene to main scnene", );
            match engine.get_asset_manager().add_scene_to_main_scene("TestScene"){
                Ok(_) => scene_added = true,
                Err(_) => {
                    //println!("Could not find TestScene", );
                }
            }

            //Scale by .1
            match engine.get_asset_manager().get_active_scene().get_node("TestScene"){
                Some(scene) => {
                    //println!("Scaling!", );
                    //scene.add_job(jobs::SceneJobs::Scale(Vector3::new(0.01, 0.01, 0.01)));
                }
                None => {
                    //println!("Could not find TestScene", );
                }, //get on with it
            }

        }

        //try to get the TestScene and move it if a key is pressed

        //test if a is pressed
        if engine.get_asset_manager().get_keymap().h{

            match engine.get_asset_manager().get_active_scene().get_node("TestScene"){
                Some(scene) => {
                    scene.add_job(jobs::SceneJobs::Rotate(Vector3::new(1.0, 0.0, 0.0)));
                }
                None => {println!("Could not find TestScene", );}, //get on with it
            }
        }

        //test if a is pressed
        if engine.get_asset_manager().get_keymap().h{

            match engine.get_asset_manager().get_active_scene().get_node("TestScene"){
                Some(scene) => {
                    scene.add_job(jobs::SceneJobs::Rotate(Vector3::new(1.0, 0.0, 0.0)));
                }
                None => {println!("Could not find TestScene", );}, //get on with it
            }
        }


        let light_names = engine.get_asset_manager().get_active_scene().all_point_light_names(&None);
        for i in light_names.into_iter(){
            //Get the light (unwarp is save)
            let mut engine_lock = engine.get_asset_manager();
            let mut light = engine_lock.get_active_scene().get_node(&i).unwrap();

            let add_vec = Vector3::new(
                0.01,
                0.02,
                0.005
            );

            light.add_job(jobs::SceneJobs::Move(
                add_vec)
            );

            if light.attributes.transform.disp.x > 20.0{
                light.attributes.transform.disp.x = -20.0;
            }

            if light.attributes.transform.disp.y > 5.0{
                light.attributes.transform.disp.y = -5.0;
            }

            if light.attributes.transform.disp.z > 20.0{
                light.attributes.transform.disp.z = -20.0;
            }

            //light.add_job(jobs::SceneJobs::Move(Vector3::new(10.0, 10.0, 10.0)));

        }


        /*
        if engine.get_asset_manager().get_keymap().t{
            let mut asset_manager = engine.get_asset_manager();
            asset_manager.get_scene_manager().print_all_scenes();
        }
        */
        if engine.get_asset_manager().get_keymap().p{
            let settings = engine.get_settings();
            settings.lock().expect("fail up").capture_next_frame();
        }

        if engine.get_asset_manager().get_keymap().up{
            let settings = engine.get_settings();
            settings.lock().expect("fail up").get_render_settings().add_exposure(0.01);
        }

        if engine.get_asset_manager().get_keymap().down{
            let settings = engine.get_settings();
            settings.lock().expect("fail down").get_render_settings().add_exposure(-0.01);
        }
        //Set the debug settings
        if engine.get_asset_manager().get_keymap().b{
            let settings = engine.get_settings();
            settings.lock().expect("fail debug true").get_render_settings().set_debug_bound(true);
        }
        //Set the debug settings
        if engine.get_asset_manager().get_keymap().n{
            let settings = engine.get_settings();
            settings.lock().expect("fail debug false").get_render_settings().set_debug_bound(false);
        }

        if engine.get_asset_manager().get_keymap().t_1{
            let settings = engine.get_settings();
            settings.lock().expect("fail debug false").get_render_settings().set_debug_view(jakar_engine::core::render_settings::DebugView::ClusterId);
        }

        if engine.get_asset_manager().get_keymap().t_2{
            let settings = engine.get_settings();
            settings.lock().expect("fail debug false").get_render_settings().set_debug_view(jakar_engine::core::render_settings::DebugView::HeatMap);
        }

        if engine.get_asset_manager().get_keymap().t_3{
            let settings = engine.get_settings();
            settings.lock().expect("fail debug false").get_render_settings().set_debug_view(jakar_engine::core::render_settings::DebugView::MainDepth);
        }

        if engine.get_asset_manager().get_keymap().t_4{
            let settings = engine.get_settings();
            settings.lock().expect("fail debug false").get_render_settings().set_debug_view(jakar_engine::core::render_settings::DebugView::Shaded);
        }

        if engine.get_asset_manager().get_keymap().p{
            let settings = engine.get_settings();
            let current_strength = settings.lock().expect("fail debug false").get_render_settings().get_blur().strength;
            let current_scale = settings.lock().expect("fail debug false").get_render_settings().get_blur().scale;
            settings.lock().expect("fail debug false").get_render_settings().set_blur(current_scale + 0.05, current_strength + 0.05);
        }

        if engine.get_asset_manager().get_keymap().o{
            let settings = engine.get_settings();
            let current_strength = settings.lock().expect("fail debug false").get_render_settings().get_blur().strength;
            let current_scale = settings.lock().expect("fail debug false").get_render_settings().get_blur().scale;
            settings.lock().expect("fail debug false").get_render_settings().set_blur(current_scale - 0.05, current_strength - 0.05);        }




        //test if a is pressed
        if engine.get_asset_manager().get_keymap().escape{
            //println!("Scene: ", );
            //engine.get_asset_manager().get_active_scene().print_tree();
            engine.end();
            break;
        }

        thread::sleep(Duration::from_millis(10));

    }

}
