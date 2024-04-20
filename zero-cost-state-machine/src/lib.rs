pub trait Switch<P> {
    type Target;
    fn transition(self, path: P) -> Self::Target;
}

pub struct NoEdge;

pub struct NoState;
