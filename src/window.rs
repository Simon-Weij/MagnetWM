use crate::{
    AppData,
    protocol::{river_node_v1::RiverNodeV1, river_window_v1::RiverWindowV1},
};
use wayland_client::QueueHandle;

#[derive(Debug)]
pub struct Window {
    pub proxy: RiverWindowV1,
    pub node: RiverNodeV1,
    pub new: bool,
    pub closed: bool,
}

impl Window {
    pub fn new(proxy: RiverWindowV1, qh: &QueueHandle<AppData>) -> Self {
        let node = proxy.get_node(qh, ());
        Self {
            proxy,
            node,
            new: true,
            closed: false,
        }
    }
}
