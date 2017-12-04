extern crate jakar_engine;
extern crate jakar_tree;
extern crate collision;
extern crate cgmath;

use collision::*;
use cgmath::*;
use jakar_engine::*;
use jakar_tree::*;
use jakar_tree::node::Attribute;
use core::next_tree::SceneTree;

use std::time::*;

//a small test for the speed of the scene tree

//Sample rust file
fn main(){

	//create a nice tree
	let tree_atrib = core::next_tree::attributes::NodeAttributes::default();
	let node_empty = core::resources::empty::Empty::new("RootNode");
	let mut tree = tree::Tree::new(core::next_tree::content::ContentType::Empty(node_empty), tree_atrib);

	let names = vec!["Teddy", "Rolf", "Clair", "Eve", "Bob",
					"Alice", "Tedberg", "Fritz", "Ulf", "Romberg",
					"Gutsch", "Fiddelwut", "Nix", "Dix", "Lokus"
					];
	let mut w_x = 1.0;
	let mut w_y = 1.0;
	let mut w_z = 1.0;


	//start time
	let mut last_time = Instant::now();

	for x in names.iter(){
		//add a top node
		let mut top_node_attrib = core::next_tree::attributes::NodeAttributes::default();
		top_node_attrib.bound = collision::Aabb3::new(
			Point3::new(-1.0, -1.0, -1.0),
			Point3::new(1.0, 1.0, 1.0)
		);
		top_node_attrib.transform.disp = Vector3::new(w_x, w_y, w_z);
		let top_light = core::resources::light::LightPoint::new(x);
		//add it
		let _ = tree.add_at_root(core::next_tree::content::ContentType::PointLight(top_light), Some(top_node_attrib));

		w_x += 1.0;
		for y in names.iter(){

			//add a top node
			let mut sub_node_attrib = core::next_tree::attributes::NodeAttributes::default();
			sub_node_attrib.bound = collision::Aabb3::new(
				Point3::new(-1.0, -1.0, -1.0),
				Point3::new(1.0, 1.0, 1.0)
			);
			sub_node_attrib.transform.disp = Vector3::new(w_x, w_y, w_z);
			let sub_light = core::resources::light::LightPoint::new(&(x.to_string() + "_" + y));
			//add it
			let _ = tree.add(
				core::next_tree::content::ContentType::PointLight(sub_light),
				x.to_string(),
				Some(sub_node_attrib)
			);


			w_y += 1.0;
			for z in names.iter(){
				//add a top node
				let mut sub_sub_node_attrib = core::next_tree::attributes::NodeAttributes::default();
				sub_sub_node_attrib.bound = collision::Aabb3::new(
					Point3::new(-1.0, -1.0, -1.0),
					Point3::new(1.0, 1.0, 1.0)
				);
				sub_sub_node_attrib.transform.disp = Vector3::new(w_x, w_y, w_z);
				let sub_sub_light = core::resources::light::LightPoint::new(&(x.to_string() + "_" + y + "_" + z));
				//add it
				let _ = tree.add(
					core::next_tree::content::ContentType::PointLight(sub_sub_light),
					x.to_string() + "_" + y,
					Some(sub_sub_node_attrib)
				);

				w_z += 1.0;
			}
		}
	}

	//tree.print_tree();

	let mut time_needed_to_add = last_time.elapsed().subsec_nanos();
	println!("Needed {} sec to insert all meshes", time_needed_to_add as f32/1_000_000_000.0);
	last_time = Instant::now();

	let _ = tree.get_node("Ulf_Teddy_Rolf".to_string());
	time_needed_to_add = last_time.elapsed().subsec_nanos();
	println!("Needed {} sec to get this meshe", time_needed_to_add as f32/1_000_000_000.0);

	//now add a tree to another
	let tree_cpy = tree.clone();
	last_time = Instant::now();
	let _ = tree.join(&tree_cpy, "Ulf_Teddy_Nix".to_string());
	time_needed_to_add = last_time.elapsed().subsec_nanos();
	println!("Needed {} sec to join the trees", time_needed_to_add as f32/1_000_000_000.0);
	//tree.print_registry();

	last_time = Instant::now();
	let _ = tree.rebuild_bounds();
	time_needed_to_add = last_time.elapsed().subsec_nanos();
	println!("Needed {} sec to update bounds", time_needed_to_add as f32/1_000_000_000.0);

	tree.print_tree();

	println!("Hello World");
}
