use core::next_tree::attributes::NodeAttributes;


/*
///To create a new callback do something like this:
///```
/// let call = CallbackContainer::new(|x: f32|{
///     println!("Got a callback after {}sec", x);
///});
/// ```
impl<T> CallbackContainer<T>{
    pub fn new(new: T) ->Self{
        CallbackContainer{
            callback: new,
        }
    }
}

///Can execute a closure of the type FnMut(f32).
pub trait DeltaCallback {
    fn execute(&mut self, delta: f32);
}

impl<T: FnMut(f32)> DeltaCallback for CallbackContainer<T>{
    fn execute(&mut self, delta: f32){
        (self.callback)(delta);
    }
}

///Same as `DeltaCallback` but supplies the nodes attributes as a mutable reference to the closure.
pub trait DeltaCallbackNode {
    fn execute(&mut self, delta: f32, attributes: &mut NodeAttributes);
}

impl<T: FnMut(f32, &mut NodeAttributes)> DeltaCallbackNode for CallbackContainer<T>{
    fn execute(&mut self, delta: f32, attributes: &mut NodeAttributes){
        (self.callback)(delta, attributes);
    }
}
*/
