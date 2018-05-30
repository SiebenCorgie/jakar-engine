use vulkano::command_buffer::AutoCommandBufferBuilder;


///A Handy trait to call functions of a boxed `FnOnce`
pub trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)()
    }
}

///A Handy trait to call a `FnOnce(AutoCommandBufferBuilder) -> AutoCommandBufferBuilder`
pub trait FnCbBox {
    fn call_box(self: Box<Self>, command_buffer: AutoCommandBufferBuilder) -> AutoCommandBufferBuilder;
}

impl<F: FnOnce(AutoCommandBufferBuilder) -> AutoCommandBufferBuilder> FnCbBox for F {
    fn call_box(self: Box<F>, command_buffer: AutoCommandBufferBuilder) -> AutoCommandBufferBuilder {
        (*self)(command_buffer)
    }
}
