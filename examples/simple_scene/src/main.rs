extern crate vulkano;
extern crate jakar_engine;
extern crate cgmath;

use cgmath::*;

use jakar_engine::*;
use jakar_engine::core::simple_scene_system::node;
use jakar_engine::core::resources::camera::Camera;
use jakar_engine::core::resources::light;
use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};




extern crate winit;

fn main() {
    //Start
    //Settings
    let settings = Arc::new(Mutex::new(core::engine_settings::EngineSettings::new()
    .with_dimensions(1600, 900)
    .with_name("jakar Instance")
    .set_vulkan_silent()
    .with_fullscreen_mode(false)
    .with_cursor_state(winit::CursorState::Grab)
    .with_cursor_visibility(winit::MouseCursor::NoneCursor)
    .with_msaa_factor(4)
    ));

    //Input
    let mut input_handler = input::Input::new(settings.clone()).with_polling_speed(60);

    //Create a renderer with the input system
    let render = Arc::new(
        Mutex::new(
            render::renderer::Renderer::new(
                input_handler.get_events_loop(),
                settings.clone(),
                input_handler.get_key_map(),
            )
        )
    );
    //Create a asset manager for the renderer
    let mut asset_manager = core::resource_management::asset_manager::AssetManager::new(
        render.clone(),
        settings.clone(),
        input_handler.key_map.clone()
    );

    asset_manager.import_gltf("PickleRick", "pickle_rick/scene.gltf");


    //Start the input thread
    input_handler.start();
    /*
    //Import the ball_01
    asset_manager.import_scene("ball_01", "Ball_01.obj");
    asset_manager.import_scene("ball_02", "Ball_02.obj");
    asset_manager.import_scene("plane", "Plane.obj");
    //asset_manager.import_scene("ball_01_02", "ball_01s.fbx");
    //asset_manager.import_scene("ball_01_03", "ball_01s.fbx");
    {
        println!("metal", );
        //Albedo
        let mut tex_builder_01 = asset_manager.create_texture("/share/3DFiles/TextureLibary/Metal/RustSteal/lowRes/rustediron2_basecolor.png");
        tex_builder_01 = tex_builder_01.with_flipped_v();
        asset_manager.add_texture_to_manager(tex_builder_01, "metal_albedo").expect("failed to add new_texture");
        //Normal
        let mut tex_builder_02 = asset_manager.create_texture("/share/3DFiles/TextureLibary/Metal/RustSteal/lowRes/rustediron2_normal.png");
        tex_builder_02 = tex_builder_02.with_flipped_v();
        asset_manager.add_texture_to_manager(tex_builder_02, "metal_normal").expect("failed to add new_texture");
        //Physical
        let mut tex_builder_03 = asset_manager.create_texture("/share/3DFiles/TextureLibary/Metal/RustSteal/lowRes/rustediron2_physical.png");
        tex_builder_03 = tex_builder_03.with_flipped_v();
        asset_manager.add_texture_to_manager(tex_builder_03, "metal_physical").expect("failed to add new_texture");
        //Creating a new material, currently a bit ugly
        {

            let albedo_in_manager = asset_manager.get_texture_manager().get_texture("metal_albedo");
            let nrm_in_manager = asset_manager.get_texture_manager().get_texture("metal_normal");
            let physical_in_manager = asset_manager.get_texture_manager().get_texture("metal_physical");

            let new_material = core::resources::material::MaterialBuilder::new(
                Some(albedo_in_manager),
                Some(nrm_in_manager),
                Some(physical_in_manager.clone()),
                Some(physical_in_manager),
                None,
                asset_manager.get_texture_manager().get_none()
            ).with_factors(
                core::resources::material::MaterialFactors::new()
                .with_factor_albedo([1.0, 1.0, 1.0, 1.0])
            );

            asset_manager.add_material_to_manager(new_material, "new_material").expect("failed to add new_material");
        }
        println!("Metal end", );
    }

    //Black Material
    {
        let mut tex_builder_01 = asset_manager.create_texture("brickwall128.jpg");
        //tex_builder_01 = tex_builder_01.with_flipped_v();
        //tex_builder_01 = tex_builder_01.with_flipped_h();
        asset_manager.add_texture_to_manager(tex_builder_01, "cube_albedo").expect("failed to add new_texture");
        //Normal
        let mut tex_builder_02 = asset_manager.create_texture("brickwall_normal128.jpg");
        //tex_builder_02 = tex_builder_02.with_flipped_v();
        asset_manager.add_texture_to_manager(tex_builder_02, "cube_normal").expect("failed to add new_texture");

        let albedo_in_manager = asset_manager.get_texture_manager().get_texture("cube_albedo");
        let nrm_in_manager = asset_manager.get_texture_manager().get_texture("cube_normal");

        let new_material = core::resources::material::MaterialBuilder::new(
            Some(albedo_in_manager),
            Some(nrm_in_manager),
            None,
            None,
            None,
            asset_manager.get_texture_manager().get_none()
        ).with_factors(
            core::resources::material::MaterialFactors::new()
            //.with_factor_albedo([0.4, 0.2, 0.0, 1.0])
            .with_factor_metal(0.0)
            .with_factor_roughness(0.1)
        );

        asset_manager.add_material_to_manager(new_material, "metalBlack").expect("failed to add new_material");

    }

    */

    //SUN========================================================================
    let mut sun = light::LightDirectional::new("Sun");
    sun.set_direction(Vector3::new(1.0, -0.5, 0.0));
    sun.set_color(Vector3::new(1.0, 0.75, 0.75));
    sun.set_intensity(200.0);

    let sun_node = node::ContentType::Light(node::LightsContent::DirectionalLight(sun));
    asset_manager.get_active_scene().add_child(sun_node);
    //SUN========================================================================

/*
    //SPOT 01 ===================================================================
    let mut spot_01 = light::LightSpot::new("Spot_01");
    spot_01.set_color(Vector3::new(1.0, 1.0, 1.0));
    spot_01.set_intensity(100.0);
    spot_01.set_location(Vector3::new(0.0, 1.0, 5.0));
    spot_01.set_direction(Vector3::new(0.5, 0.0, -1.0));
    spot_01.set_outer_radius(15.0);
    spot_01.set_inner_radius(10.0);

    let spot_node_01 = node::ContentType::Light(node::LightsContent::SpotLight(spot_01));
    asset_manager.get_active_scene().add_child(spot_node_01);
    //SPOT 01 ===================================================================


    //POINT 00 ==================================================================
    let mut point_00 = light::LightPoint::new("Point_00");
    point_00.set_color(Vector3::new(1.0, 1.0, 1.0));
    point_00.set_intensity(150.0);
    point_00.set_location(Vector3::new(0.0, 0.0, 0.0));

    let point_node_00 = node::ContentType::Light(node::LightsContent::PointLight(point_00));
    asset_manager.get_active_scene().add_child(point_node_00);
    //POINT 00 ==================================================================

    //POINT 01 ==================================================================
    let mut point_01 = light::LightPoint::new("Point_01");
    point_01.set_color(Vector3::new(150.0, 150.0, 150.0));
    point_01.set_location(Vector3::new(-3.0, 0.0, -3.0));

    let point_node_01 = node::ContentType::Light(node::LightsContent::PointLight(point_01));
    asset_manager.get_active_scene().add_child(point_node_01);
    //POINT 01 ==================================================================

    //POINT 02 ==================================================================
    let mut point_02 = light::LightPoint::new("Point_02");
    point_02.set_color(Vector3::new(150.0, 150.0, 150.0));
    point_02.set_location(Vector3::new(-3.0, 0.0, 3.0));

    let point_node_02 = node::ContentType::Light(node::LightsContent::PointLight(point_02));
    asset_manager.get_active_scene().add_child(point_node_02);
    //POINT 02 ==================================================================

    //POINT 03 ==================================================================
    let mut point_03 = light::LightPoint::new("Point_03");
    point_03.set_color(Vector3::new(150.0, 150.0, 150.0));
    point_03.set_location(Vector3::new(3.0, 0.0, -3.0));

    let point_node_03 = node::ContentType::Light(node::LightsContent::PointLight(point_03));
    asset_manager.get_active_scene().add_child(point_node_03);
    //POINT 03 ==================================================================

    //POINT 04 ==================================================================
    let mut point_04 = light::LightPoint::new("Point_04");
    point_04.set_color(Vector3::new(150.0, 150.0, 150.0));
    point_04.set_location(Vector3::new(3.0, 0.0, 3.0));

    let point_node_04 = node::ContentType::Light(node::LightsContent::PointLight(point_04));
    asset_manager.get_active_scene().add_child(point_node_04);
    //POINT 04 ==================================================================
*/



    asset_manager.get_active_scene().print_member(0);

    let mut adding_status_plane = false;
    let mut adding_status = false;

    let mut start_time = Instant::now();

    let mut avg_fps = 60.0;

    let mut min_fps = 100.0;
    let mut max_fps = 0.0;

    loop {
        //Add the ball_01 scene if finished loading. This will be managed by a defined loader later
        if adding_status == false && asset_manager.has_scene("ball_01") && asset_manager.has_scene("ball_02"){
            println!("Adding ball_01", );
            let mut ball_01_scene ={
                //let scene_manager = asset_manager.get_scene_manager();
                asset_manager.get_scene_manager().get_scene_arc("ball_01").expect("no ball_01s :(")
            };

            for i in (*ball_01_scene).lock().unwrap().get_all_meshes().iter(){
                //Unwrap the mesh from the tubel
                let mesh = i.0.clone();

                let mesh_inst = mesh.clone();
                let mut mesh_lck = mesh_inst.lock().expect("failed to change material");
                (*mesh_lck).set_material("new_material");
            }
            asset_manager.add_scene_to_main_scene("ball_01");
            asset_manager.add_scene_to_main_scene("ball_02");

            adding_status = true;
            println!("STATUS: GAME: added all ball_01s=============================================", );

        }

        if !adding_status_plane && asset_manager.has_scene("plane"){
            println!("Adding plane", );

            let plane_scene = asset_manager.get_scene_manager().get_scene_arc("plane").expect("no plane :(");

            println!("Set plane lock", );
            for i in (*plane_scene).lock().unwrap().get_all_meshes().iter(){
                let mesh = i.0.clone();
                println!("Cloned mesh", );
                let mut mesh_lck = mesh.lock().expect("failed to lock plane");
                println!("Locked mesh", );
                (*mesh_lck).set_material("metalBlack");
                println!("SetMaterial", );
            }

            asset_manager.add_scene_to_main_scene("plane");

            adding_status_plane = true;
            println!("Finished plane", );
        }

        if !adding_status_plane && asset_manager.has_scene("PickleRick"){

            {
                let mut man = asset_manager.get_scene_manager();
                let mut boom_scene = man.get_scene("PickleRick");
                boom_scene.unwrap().scale(1.0);
            }

            asset_manager.add_scene_to_main_scene("PickleRick");
            println!("Adding PickleRick", );
            adding_status_plane = true;
        }

        //println!("STATUS: GAME: Starting loop in game", );
        //Update the content of the render_manager
/*
        //Updating the light based on the camera position
        let camera_inst = asset_manager.get_camera().clone();
        {
            let light_inst = asset_manager.get_active_scene().get_light_spot("Spot_01").unwrap();
            light_inst.set_location(camera_inst.get_position());
            light_inst.set_direction(- camera_inst.get_direction());

        }
*/

        asset_manager.update();
        //println!("STATUS: GAME: Updated all assets", );
        (*render).lock().expect("Failed to lock renderer for rendeball_02").render(&mut asset_manager);
        //Check if loop should close
        if input_handler.get_key_map_copy().closed{
            println!("STATUS: GAME: Shuting down", );
            input_handler.end();
            break;
        }

        if input_handler.get_key_map_copy().escape{
            input_handler.end();
            println!("Max FPS: {}", max_fps);
            println!("Min FPS: {}", min_fps);

            break;
        }

        if input_handler.get_key_map_copy().t{
            //Get the ball_02 scene and translate it by 10,10,0
            let ball_01_scene ={
                //Get the reference in the current active scene
                match asset_manager.get_active_scene().get_node("PickleRick"){
                    Some(scene) => scene,
                    None => continue,
                }
            };
            //Set the translation on this node
            ball_01_scene.set_location(Vector3::new(0.0, 0.0, 0.0));
            //println!("Translated", );
        }

        if input_handler.get_key_map_copy().z{
            //Get the ball_02 scene and translate it by 10,10,0
            let mut plane_scene ={
                //Get the reference in the current active scene
                match asset_manager.get_active_scene().get_node("PickleRick"){
                    Some(scene) => scene,
                    None => continue,
                }
            };
            //Set the translation on this node
            plane_scene.rotate(Vector3::new(0.0, 1.0, 0.0));
        }

        if input_handler.get_key_map_copy().u{
            println!("Translating test #########!", );
            //Get the ball_02 scene and translate it by 10,10,0
            let mut tree_scene ={
                //Get the reference in the current active scene
                match asset_manager.get_active_scene().get_node("PickleRick"){
                    Some(scene) => scene,
                    None => continue,
                }
            };
            println!("Translating!", );
            //Set the translation on this node
            tree_scene.translate(Vector3::new(0.0, 1.0, 0.0));
        }

        asset_manager.get_material_manager().print_all_materials();
        asset_manager.get_scene_manager().print_all_scenes();
        //Prints all materials and the scene tree
        asset_manager.get_active_scene().print_member(0);

        let fps_time = start_time.elapsed().subsec_nanos();

        let fps = 1.0/ (fps_time as f32 / 1_000_000_000.0);
        avg_fps = (avg_fps + fps) / 2.0;
        //println!("STATUS: RENDER: AVG FPS IN GAME: {}", avg_fps);
        //println!("This Frame: {}", fps);

        if fps < min_fps{
            min_fps = fps;
        }

        if fps > max_fps{
            max_fps = fps;
        }


        start_time = Instant::now();
    }
}
