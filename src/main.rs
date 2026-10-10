use crate::{
    events::AppData,
    protocol::{river_node_v1::RiverNodeV1, river_xkb_bindings_v1::RiverXkbBindingsV1},
};

use wayland_client::Connection;

mod config;
mod events;
mod output;
mod protocol;
mod seat;
mod window;
mod window_manager;

wayland_client::delegate_noop!(AppData: ignore RiverXkbBindingsV1);
wayland_client::delegate_noop!(AppData: ignore RiverNodeV1);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Queue up a get_registry event.
    let conn = Connection::connect_to_env()?;
    let display = conn.display();
    let mut event_queue = conn.new_event_queue();
    let _registry = display.get_registry(&event_queue.handle(), ());

    // Initial state
    let mut app_data = AppData::default();

    // Roundtrip to process the get_registry event and bind interfaces.
    event_queue.roundtrip(&mut app_data)?;
    if app_data.river_wm.is_none() {
        eprintln!("river_window_manager_v1 global not found! Is river running?");
        std::process::exit(1);
    }
    if app_data.river_xkb.is_none() {
        eprintln!("river_xkb_bindings_v1 global not found! Is river running with xkb support?");
        std::process::exit(1);
    }

    loop {
        event_queue.blocking_dispatch(&mut app_data)?;
    }
}
