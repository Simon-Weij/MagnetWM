use crate::protocol::river_output_v1::RiverOutputV1;

#[derive(Debug)]
pub struct Output {
    pub proxy: RiverOutputV1,
    pub removed: bool,
}

impl Output {
    pub fn new(proxy: RiverOutputV1) -> Self {
        Self {
            proxy,
            removed: false,
        }
    }
}
