use tagged_box::{tagged_box, TaggableContainer, TaggableInner};
use smallvec::SmallVec;

tagged_box! {
    #[derive(Debug, Clone, PartialEq)]
    struct Container, enum Item {
        Integer(i32),
        Decimal(f32),
        Boolean(bool),
        String(String),
    }
}
