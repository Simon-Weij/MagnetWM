use crate::{
    AppData,
    protocol::{
        river_node_v1::RiverNodeV1,
        river_seat_v1::RiverSeatV1,
        river_window_v1::{Edges, RiverWindowV1},
    },
};
use wayland_client::QueueHandle;

#[derive(Debug)]
pub struct Window {
    pub proxy: RiverWindowV1,
    pub node: RiverNodeV1,
    pub new: bool,
    pub closed: bool,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub pointer_move_requested: Option<RiverSeatV1>,
    pub pointer_resize_requested: Option<RiverSeatV1>,
    pub pointer_resize_requested_edges: Edges,
}

impl Window {
    pub fn new(proxy: RiverWindowV1, qh: &QueueHandle<AppData>) -> Self {
        let node = proxy.get_node(qh, ());
        Self {
            proxy,
            node,
            new: true,
            closed: false,
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            pointer_move_requested: None,
            pointer_resize_requested: None,
            pointer_resize_requested_edges: Edges::None,
        }
    }

    pub fn set_position(&mut self, x: i32, y: i32) {
        self.node.set_position(x, y);
        self.x = x;
        self.y = y;
    }
}
