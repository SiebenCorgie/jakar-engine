use cgmath::*;
use collision::Aabb3;
use jakar_tree::node::Attribute;
use super::jobs::SceneJobs;

///A node can have this attributes
#[derive(Clone)]
pub struct NodeAttributes {

    ///Transform of this node in local space
    pub transform: Decomposed<Vector3<f32>, Quaternion<f32>>,
    ///The bounds of this note, takes the `content` bound as well as the max and min values of
    ///all its children into consideration.
    pub bound: Aabb3<f32>,


    /// Can be turned off to disable shadow casting, usefull for many small objects
    pub cast_shadow: bool,
    /// Is used to determin at which point this object is rendered.
    /// There is the first pass for opaque objects, as wella s msked objects, and the second one for
    /// transparent ones.
    pub is_transparent: bool,
    /// If true the object won't be rendered if the engine is in gmae mode.
    pub hide_in_game: bool,
}

///A custom implementation
impl NodeAttributes{
    /// Returns a reference to the transform component. Can be used to determin 3d relation between
    /// nodes.
    pub fn get_transform(&self) -> &Decomposed<Vector3<f32>, Quaternion<f32>>{
        &self.transform
    }

    ///Returns the model matrix of this node
    pub fn get_matrix(&self) -> Matrix4<f32>{
        Matrix4::from(self.transform)
        /*
        Matrix4::from_translation(self.transform.disp)
        // * Matrix4::from_scale(self.transform.scale)
         * Matrix4::from(self.transform.rot)
        */

    }

    /// Returns bound information of this node (**NOT THE MESH BOUND**)
    pub fn get_bound(&self) -> &Aabb3<f32>{
        &self.bound
    }


}


///The jakar tree impementation
impl Attribute<SceneJobs> for NodeAttributes{
    ///The type used to comapre nodes which a a `comaprer`
    type Comparer = super::SceneComparer;

    ///Creates a default set of attribtues.
    ///with:
    /// - transform: [position[0.0, 0.0, 0.0], rotation(euler)[0.0, 0.0, 0.0], scale[1.0, 1.0, 1.0]]
    /// - bound: from [0.0, 0.0, 0.0] to [0.0, 0.0, 0.0]
    /// - cast_shadow: true
    /// - is_transparent: false
    /// - hide_in_game: false
    fn default() -> Self{
        NodeAttributes{
            transform: Decomposed{
                    scale: 1.0,
                    rot: Quaternion::from(Euler::new(Deg(0.0), Deg(0.0), Deg(0.0))),
                    disp: Vector3::new(0.0, 0.0, 0.0),
            },
            bound: Aabb3::new(Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 0.0, 0.0)),
            cast_shadow: true,
            is_transparent: false,
            hide_in_game: false,
        }
    }

    ///Exectues a `job` on this set of attributes. Returns job to be passed down to the children.
    fn execute(&mut self, job: &SceneJobs) -> SceneJobs{
        match job{
            &SceneJobs::Move(t) =>{
                self.transform.disp = self.transform.disp + t;
                //Return the same because we want to move each vector the same
                SceneJobs::Move(t)
            } ,
            &SceneJobs::Rotate(r) => {
                self.transform.rot += Quaternion::from(Euler::new(Deg(r.x), Deg(r.y), Deg(r.z)));
                //if we rotate self, we want to rotate the children around self's location
                SceneJobs::RotateAroundPoint(r, self.transform.disp)
            }

            &SceneJobs::RotateAroundPoint(rotation, point) => {
                //FIXME reimplemt from https://gamedev.stackexchange.com/questions/16719/what-is-the-correct-order-to-multiply-scale-rotation-and-translation-matrices-f
                //move to point
                //create a rotation Quaternion from the angles in rotation.xyz
                let q_rotation = Quaternion::from(Euler {
                    x: Deg(rotation.x),
                    y: Deg(rotation.y),
                    z: Deg(rotation.z),
                });

                //go to the point
                self.transform.disp -= point;
                //do rotation
                self.transform.rot = self.transform.rot * q_rotation;
                //rotate selfs disp to match the rotation at the point
                self.transform.disp = q_rotation.rotate_vector(self.transform.disp);
                //move back to the new origin
                self.transform.disp += point;
                //All other children should be rotated the same
                SceneJobs::RotateAroundPoint(rotation, point)
            }
            &SceneJobs::Scale(s) => {
                self.transform.scale += s.x;
                SceneJobs::Scale(s)
            },
        }
    }

    ///prints a readable representation of `self` with `lvl` indends in front of each line
    fn print_atr(&self, lvl: i32){
        //print attribtues
        for _ in 0..lvl{
            print!("\t");
        }
        println!("Attributes:", );
        //print location
        for _ in 0..lvl + 1{
            print!("\t");
        }
        println!("\tposition: {:?}", self.transform.disp);
        //print rotation
        for _ in 0..lvl + 1{
            print!("\t");
        }
        println!("\trotation: {:?}", Euler::from(self.transform.rot));
        //print scale
        for _ in 0..lvl + 1{
            print!("\t");
        }
        println!("\tscale: {}", self.transform.scale);

        //print bound
        for _ in 0..lvl + 1{
            print!("\t");
        }
        println!("\tbound: from: {:?} to: {:?}", self.bound.min, self.bound.max);

        //print shadow flag
        for _ in 0..lvl + 1{
            print!("\t");
        }
        println!("\tcasts shadow?: {}", self.cast_shadow);

        //print is_transparent flag
        for _ in 0..lvl + 1{
            print!("\t");
        }
        println!("\tis transparent?: {}", self.is_transparent);

        //print hide_in_game flag
        for _ in 0..lvl + 1{
            print!("\t");
        }
        println!("\thide in game?: {}", self.hide_in_game);
    }

    ///Compares this node with a `comp` and returns true if all requierments are met,
    /// otherwise it returns false.
    fn compare(&self, comp: &Self::Comparer) -> bool{
        //This will compare all "Some"s in the comparer with the actual value, if one is
        //wrong it will early returna false, else the true is returned at the end of this funtion.

        //transform
        match comp.transform{
            Some(transform) => {
                if transform.disp != self.transform.disp{
                    return false;
                }
                if transform.rot != self.transform.rot{
                    return false;
                }
                if transform.scale != self.transform.scale{
                    return false;
                }
            },
            None => {},
        }

        //bound
        match comp.bound{
            Some(bnd) => {
                if bnd != self.bound{
                    return false;
                }
            },
            None => {},
        }

        //shadow
        match comp.cast_shadow{
            Some(cast) => {
                if cast != self.cast_shadow{
                    return false;
                }
            }
            None => {},
        }

        //transparency
        match comp.is_transparent{
            Some(trans) => {
                if trans != self.is_transparent{
                    return false;
                }
            }
            None => {},
        }

        //hide in game
        match comp.hide_in_game{
            Some(hide) => {
                if hide != self.hide_in_game{
                    return false;
                }
            }
            None => {},
        }

        //all test where sucessful, returning true
        true
    }

}
