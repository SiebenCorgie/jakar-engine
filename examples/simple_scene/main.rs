
extern crate vulkano;
extern crate jakar_engine;
extern crate cgmath;
extern crate jakar_tree;

use cgmath::*;

use jakar_engine::*;
use jakar_engine::core::next_tree::*;
use jakar_engine::core::resources::*;
use jakar_engine::core::resources::camera::Camera;
use jakar_engine::core::resources::light;
use jakar_engine::core::next_tree::*;
use jakar_tree::node::*;
use jakar_engine::core::render_settings::*;
use jakar_engine::core::next_tree::JakarNode;

use std::thread;
use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};




extern crate winit;

fn main() {
    let light_settings = LightSettings::new(DirectionalLightSettings::new(
        8, //pcf samples
        4096, //lightmap resolution
        0.95, //cascade lambda
        [0.1, 0.1, 0.1, 0.1], //occupy bias
        1500.0) //poisson spread
    );

    let graphics_settings = core::render_settings::RenderSettings::default()
    .with_msaa_factor(8)
    .with_gamma(1.0)
    .with_exposure(jakar_engine::core::render_settings::ExposureSettings::new(
        0.2, 4.0, 0.005, 0.003, 0.5, true
    ))
    .with_anisotropical_filtering(16)
    .with_light_settings(light_settings)
    .with_bloom(
        8, //levles
        2, //initial blured level
        0.5, //size
        0.01, //brightness
    );

    let settings = core::engine_settings::EngineSettings::default()
    .with_dimensions(1600, 900)
    .with_name("Jakar Instance")
    .in_release_mode()
    .with_max_input_polling_speed(200)
    .with_fullscreen_mode(false)
    //.with_cursor_state(winit::CursorState::Normal)
    .with_cursor_state(winit::CursorState::Grab)
    //.with_cursor_visibility(winit::MouseCursor::Default)
    .with_cursor_visibility(winit::MouseCursor::NoneCursor)
    .with_render_settings(graphics_settings)
    .with_camera_settings(core::engine_settings::CameraSettings{
        far_plane: 1000.0,
        near_plane: 0.2,
    })
    ;


    //Start the engine
    let mut engine = match jakar_engine::JakarEngine::build(Some(settings)){
        Ok(eng) => eng,
        Err(er) => {
            println!("Failed to create engine!");
            return;
        }
    };


    engine.get_asset_manager().import_gltf("TestScene", "examples/simple_scene/TestScenes/Cube_Plane.gltf");
    //engine.get_asset_manager().import_gltf("TestScene", "examples/simple_scene/Gate/Gate.gltf");
    //engine.get_asset_manager().import_gltf("TestScene", "examples/simple_scene/Sponza/Sponza.gltf");
    //engine.get_asset_manager().import_gltf("TestScene", "examples/simple_scene/Helmet/Helmet.gltf");


    let mut light_tree =jakar_tree::tree::Tree::new(
        jakar_engine::core::next_tree::content::ContentType::Empty(core::resources::empty::Empty::new("LightsRoot")),
        jakar_engine::core::next_tree::attributes::NodeAttributes::default()
    );


    //SUN========================================================================
    //add a matrix of lights

    let mut matrix_size = 0;
    matrix_size = matrix_size - (matrix_size / 2);
    let spacing = 2.0;

    for x in -(matrix_size)..matrix_size{
        for y in -(matrix_size)..matrix_size{
            let mut point = light::LightPoint::new("LightPoint");
            point.set_intensity(
                5.0
            );
            point.set_color(
                Vector3::new(
                    (x + matrix_size) as f32 / matrix_size as f32,
                    (y + matrix_size) as f32 / matrix_size as f32,
                    (1-y + matrix_size) as f32 / matrix_size as f32
                )
            );
            point.set_radius(5.0);

            let node_name = light_tree
            .add_at_root(content::ContentType::PointLight(point), None);

            //Set the location
            match light_tree.get_node(&node_name.unwrap()){
                Some(scene) => {
                    scene.add_job(
                        jobs::SceneJobs::Move(
                            Vector3::new(
                                x as f32 * spacing,
                                (spacing / 2.0),
                                y as f32 * spacing,
                            )
                        )
                    );

                }
                None => {println!("Could not find Light", );}, //get on with it
            }
        }
    }

    //Now add a sun
    let mut sun = light::LightDirectional::new("Sunny");
    sun.set_intensity(200.0);
    sun.set_color(Vector3::new(1.0, 0.85, 0.9));
    let sun_node = light_tree.add_at_root(content::ContentType::DirectionalLight(sun), None).expect("fail");
    //Now rotate it a bit on x
    match light_tree.get_node(&sun_node){
        Some(sun)=> {
            sun.add_job(jobs::SceneJobs::Rotate(Vector3::new(0.0, 10.0, -60.0)));

        },
        None => {println!("Could not find sun", );}
    }


    light_tree.update();
    engine.get_asset_manager().get_active_scene().join_at_root(&light_tree);
    println!("LightreeJoined!", );
    engine.get_asset_manager().get_active_scene().update();
    //println!("THE SCENE ==================================================", );
    //engine.get_asset_manager().get_active_scene().print_tree();
    //println!("END ========================================================", );
    let mut scene_added = false;
    let mut arrow_added = false;
    //Its time to actually start the engine
    engine.start();

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
        }

        //try to get the TestScene and move it if a key is pressed


        //test if a is pressed
        if engine.get_asset_manager().get_keymap().h{

            match engine.get_asset_manager().get_active_scene().get_node("TestScene"){
                Some(scene) => {
                    scene.add_job(jobs::SceneJobs::Rotate(Vector3::new(0.0, 1.0, 0.0)));
                }
                None => {println!("Could not find TestScene", );}, //get on with it
            }
        }

        if engine.get_current_keymap().p{
            engine.get_engine_settings_unlocked().capture_next_frame();
        }

        if engine.get_current_keymap().r{
            engine.get_asset_manager().get_active_scene().print_tree();
        }

        if engine.get_current_keymap().up{
            engine.get_engine_settings_unlocked().
            get_render_settings_mut().get_debug_settings_mut().ldr_debug_view_level += 1;
        }


        if engine.get_current_keymap().down{
            engine.get_engine_settings_unlocked()
            .get_render_settings_mut().get_debug_settings_mut().ldr_debug_view_level -= 1;
        }

        if engine.get_current_keymap().f1{
            engine.get_engine_settings_unlocked()
            .get_render_settings_mut().get_debug_settings_mut().ldr_debug_view_level = 0;
        }
        if engine.get_current_keymap().f2{
            engine.get_engine_settings_unlocked()
            .get_render_settings_mut().get_debug_settings_mut().ldr_debug_view_level = 1;
        }
        if engine.get_current_keymap().f3{
            engine.get_engine_settings_unlocked()
            .get_render_settings_mut().get_debug_settings_mut().ldr_debug_view_level = 2;
        }
        if engine.get_current_keymap().f4{
            engine.get_engine_settings_unlocked()
            .get_render_settings_mut().get_debug_settings_mut().ldr_debug_view_level = 3;
        }
        if engine.get_current_keymap().f5{
            engine.get_engine_settings_unlocked()
            .get_render_settings_mut().get_debug_settings_mut().ldr_debug_view_level = 4;
        }
        if engine.get_current_keymap().f6{
            engine.get_engine_settings_unlocked()
            .get_render_settings_mut().get_debug_settings_mut().ldr_debug_view_level = 5;
        }
        if engine.get_current_keymap().f7{
            engine.get_engine_settings_unlocked()
            .get_render_settings_mut().get_debug_settings_mut().ldr_debug_view_level = 6;
        }
        if engine.get_current_keymap().f8{
            engine.get_engine_settings_unlocked()
            .get_render_settings_mut().get_debug_settings_mut().ldr_debug_view_level = 7;
        }
        if engine.get_current_keymap().f9{
            engine.get_engine_settings_unlocked()
            .get_render_settings_mut().get_debug_settings_mut().ldr_debug_view_level = 8;
        }
        if engine.get_current_keymap().f10{
            engine.get_engine_settings_unlocked()
            .get_render_settings_mut().get_debug_settings_mut().ldr_debug_view_level = 9;
        }
        if engine.get_current_keymap().f11{
            engine.get_engine_settings_unlocked()
            .get_render_settings_mut().get_debug_settings_mut().ldr_debug_view_level = 10;
        }
        if engine.get_current_keymap().f12{
            engine.get_engine_settings_unlocked()
            .get_render_settings_mut().get_debug_settings_mut().ldr_debug_view_level = 11;
        }
        //Set the debug settings
        if engine.get_current_keymap().b{
            engine.get_engine_settings_unlocked().get_render_settings_mut()
            .get_bloom_mut().brightness += 0.001;
            println!("Changed stuff", );
        }
        //Set the debug settings
        if engine.get_current_keymap().n{
            engine.get_engine_settings_unlocked().get_render_settings_mut()
            .get_bloom_mut().brightness -= 0.001;
            println!("Changed stuff", );
        }

        if engine.get_current_keymap().t_1{
            engine.get_engine_settings_unlocked().get_render_settings_mut()
            .get_debug_settings_mut().debug_view = jakar_engine::core::render_settings::DebugView::MainDepth;
        }

        if engine.get_current_keymap().t_2{
            engine.get_engine_settings_unlocked().get_render_settings_mut()
            .get_debug_settings_mut().debug_view = jakar_engine::core::render_settings::DebugView::DebugGrid;
        }

        if engine.get_current_keymap().t_3{
            engine.get_engine_settings_unlocked().get_render_settings_mut()
            .get_debug_settings_mut().debug_view = jakar_engine::core::render_settings::DebugView::ShadowMaps;
        }

        if engine.get_current_keymap().t_4{
            engine.get_engine_settings_unlocked().get_render_settings_mut()
            .get_debug_settings_mut().debug_view = jakar_engine::core::render_settings::DebugView::DirectionalDepth;
        }

        if engine.get_current_keymap().t_5{
            engine.get_engine_settings_unlocked().get_render_settings_mut()
            .get_debug_settings_mut().debug_view = jakar_engine::core::render_settings::DebugView::Shaded;
        }

        //test if a is pressed
        if engine.get_current_keymap().escape{
            engine.end();
            break;
        }

        thread::sleep(Duration::from_millis(10));

    }

}
