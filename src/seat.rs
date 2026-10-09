use std::collections::{HashMap, VecDeque};

use crate::{
    events::{Action, AppData},
    protocol::{
        river_seat_v1::{Modifiers, RiverSeatV1},
        river_window_manager_v1::RiverWindowManagerV1,
        river_window_v1::RiverWindowV1,
        river_xkb_binding_v1::RiverXkbBindingV1,
        river_xkb_bindings_v1::RiverXkbBindingsV1,
    },
    window::Window,
};
use wayland_backend::client::ObjectId;
use wayland_client::{Proxy, QueueHandle};

#[derive(Debug)]
pub struct XkbBinding {
    pub proxy: RiverXkbBindingV1,
    pub action: Action,
}

#[derive(Debug)]
pub struct Seat {
    pub proxy: RiverSeatV1,
    pub new: bool,
    pub removed: bool,
    focused: Option<RiverWindowV1>,
    pub interacted: Option<RiverWindowV1>,
    pub xkb_bindings: HashMap<ObjectId, XkbBinding>,
    pub pending_action: Action,
}

impl Seat {
    pub fn new(proxy: RiverSeatV1) -> Self {
        Self {
            proxy,
            new: true,
            removed: false,
            focused: None,
            interacted: None,
            xkb_bindings: HashMap::new(),
            pending_action: Action::None,
        }
    }

    pub fn create_xkb_binding(
        &mut self,
        river_xkb: &RiverXkbBindingsV1,
        qh: &QueueHandle<AppData>,
        mods: Modifiers,
        keysym: u32,
        action: Action,
    ) {
        let proxy = river_xkb.get_xkb_binding(&self.proxy, keysym, mods, qh, self.proxy.id());
        proxy.enable();
        let binding = XkbBinding { proxy, action };
        self.xkb_bindings.insert(binding.proxy.id(), binding);
    }

    pub fn do_action(&mut self, windows: &mut VecDeque<Window>, wm_proxy: &RiverWindowManagerV1) {
        match self.pending_action {
            Action::None => {}
            // Don't pass WAYLAND_DEBUG on to children, the added noise makes
            // debugging the window manager itself impractical.
            Action::SpawnKitty => match std::process::Command::new("kitty")
                .env_remove("WAYLAND_DEBUG")
                .spawn()
            {
                Ok(_) => {}
                Err(e) => eprintln!("Failed to spawn kitty: {e}"),
            },
            Action::Close => {
                if let Some(window_proxy) = self.focused.as_ref() {
                    window_proxy.close();
                }
            }
            Action::FocusNext => {
                if !windows.is_empty() {
                    windows.rotate_left(1);
                    self.focus_top(windows);
                }
            }
            Action::Exit => wm_proxy.exit_session(),
        }
        self.pending_action = Action::None;
    }

    pub fn focus_top(&mut self, windows: &VecDeque<Window>) {
        if let Some(window) = windows.back() {
            self.proxy.focus_window(&window.proxy);
            window.node.place_top();
            self.focused = Some(window.proxy.clone());
        } else {
            self.proxy.clear_focus();
            self.focused = None;
        }
    }
}
