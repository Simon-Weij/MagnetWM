use crate::{
    events::{Action, AppData},
    output::Output,
    protocol::{
        river_seat_v1::Modifiers, river_window_manager_v1::RiverWindowManagerV1,
        river_xkb_bindings_v1::RiverXkbBindingsV1,
    },
    seat::Seat,
    window::Window,
};

use std::collections::{HashMap, VecDeque};
use wayland_backend::client::ObjectId;
use wayland_client::QueueHandle;

#[derive(Debug, Default)]
pub struct WindowManager {
    pub windows: VecDeque<Window>,
    pub outputs: HashMap<ObjectId, Output>,
    pub seats: HashMap<ObjectId, Seat>,
}

impl WindowManager {
    pub fn handle_manage_start(
        &mut self,
        proxy: &RiverWindowManagerV1,
        river_xkb: &RiverXkbBindingsV1,
        qh: &QueueHandle<AppData>,
    ) {
        self.remove_outputs();
        self.remove_windows();
        self.remove_seats();
        self.init_new_windows();
        self.init_new_seats(river_xkb, qh);
        self.manage_seats(proxy);
        proxy.manage_finish();
    }

    pub fn handle_render_start(proxy: &RiverWindowManagerV1) {
        proxy.render_finish();
    }

    fn remove_outputs(&mut self) {
        self.outputs.retain(|_, output| {
            if output.removed {
                output.proxy.destroy();
                return false;
            }
            true
        });
    }

    fn remove_windows(&mut self) {
        self.windows.retain(|window| !window.closed);
    }

    fn remove_seats(&mut self) {
        self.seats.retain(|_, seat| {
            if seat.removed {
                seat.xkb_bindings
                    .values_mut()
                    .for_each(|binding| binding.proxy.destroy());
                seat.proxy.destroy();
                return false;
            }
            true
        });
    }

    fn init_new_windows(&mut self) {
        let Some(output) = self.outputs.values().next() else {
            return;
        };
        for window in self.windows.iter_mut().filter(|w| w.new) {
            window.proxy.fullscreen(&output.proxy);
            window.proxy.inform_fullscreen();
            window.new = false;
        }
    }

    fn init_new_seats(&mut self, river_xkb: &RiverXkbBindingsV1, qh: &QueueHandle<AppData>) {
        // See xkbcommon/xkbcommon-keysyms.h
        const SPACE: u32 = 0x20;
        const N: u32 = 0x6e;
        const Q: u32 = 0x71;
        const ESC: u32 = 0xff1b;
        let mods = Modifiers::Mod1;

        for seat in self.seats.values_mut() {
            if seat.new {
                seat.create_xkb_binding(river_xkb, qh, mods, SPACE, Action::SpawnKitty);
                seat.create_xkb_binding(river_xkb, qh, mods, Q, Action::Close);
                seat.create_xkb_binding(river_xkb, qh, mods, N, Action::FocusNext);
                seat.create_xkb_binding(river_xkb, qh, mods, ESC, Action::Exit);
                seat.new = false;
            }
        }
    }

    fn manage_seats(&mut self, wm_proxy: &RiverWindowManagerV1) {
        for seat in self.seats.values_mut() {
            if let Some(window_proxy) = seat.interacted.take() {
                let i = self
                    .windows
                    .iter()
                    .position(|window| window.proxy == window_proxy)
                    .expect("Interacted window not found");

                let Some(window) = self.windows.remove(i) else {
                    return;
                };

                self.windows.push_back(window);
            }
            seat.focus_top(&self.windows);
            seat.do_action(&mut self.windows, wm_proxy);
        }
    }
}
