///Descibes a struct which can be used to sort in a tree
#[derive(Clone)]
pub struct SortAttributes {
    ///How to look for the shadow attribute of the node
    pub casts_shadow: AttributeState,
    ///How to look for the translucency attribute of the node
    pub is_translucent: AttributeState,
    ///How to look for the hide attribute of the node
    pub hide_in_game: AttributeState,
}

impl SortAttributes {
    ///All values are set to `Ignore`
    #[inline]
    pub fn new() -> Self{
        SortAttributes{
            casts_shadow: AttributeState::Ignore,
            is_translucent: AttributeState::Ignore,
            hide_in_game: AttributeState::Ignore,
        }
    }

    ///Sets the cast_shadow attribute to `yes`
    #[inline]
    pub fn casts_shadow(mut self) -> Self{
        self.casts_shadow = AttributeState::Yes;
        self
    }

    ///Sets the cast_shadow attribute to `no`
    #[inline]
    pub fn casts_no_shadow(mut self) -> Self{
        self.casts_shadow = AttributeState::No;
        self
    }

    ///Sets the cast_shadow attribute to `yes`
    #[inline]
    pub fn is_translucent(mut self) -> Self{
        self.is_translucent = AttributeState::Yes;
        self
    }

    ///Sets the cast_shadow attribute to `no`
    #[inline]
    pub fn is_not_translucent(mut self) -> Self{
        self.is_translucent = AttributeState::No;
        self
    }

    ///Sets the hide_in_game attribute to `yes`
    #[inline]
    pub fn hidden_in_game(mut self) -> Self{
        self.hide_in_game = AttributeState::Yes;
        self
    }

    ///Sets the hide_in_game attribute to `no`
    #[inline]
    pub fn not_hidden_in_game(mut self) -> Self{
        self.hide_in_game = AttributeState::No;
        self
    }
}

///Describes the state a sort attribute can have.
/// - **Yes**: The node has to have this attribute set to `true`
/// - **No**: The node has to have this attribute set to `false`#
/// - **Ignore**: Id doesn't matter what the attribute is set to.
#[derive(Clone)]
pub enum AttributeState {
    Yes,
    No,
    Ignore
}
