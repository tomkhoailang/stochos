use std::time::{SystemTime, UNIX_EPOCH};

use wayland_client::{
    globals::{registry_queue_init, GlobalListContents},
    protocol::{wl_output, wl_pointer::ButtonState, wl_registry},
    Connection, Dispatch, QueueHandle,
};
use wayland_protocols_wlr::virtual_pointer::v1::client::{
    zwlr_virtual_pointer_manager_v1, zwlr_virtual_pointer_v1,
};

struct State;

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for State {
    fn event(
        _state: &mut Self,
        _: &wl_registry::WlRegistry,
        _: wl_registry::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<State>,
    ) {
    }
}

impl Dispatch<zwlr_virtual_pointer_manager_v1::ZwlrVirtualPointerManagerV1, ()> for State {
    fn event(
        _state: &mut Self,
        _: &zwlr_virtual_pointer_manager_v1::ZwlrVirtualPointerManagerV1,
        _: zwlr_virtual_pointer_manager_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<State>,
    ) {
    }
}

impl Dispatch<zwlr_virtual_pointer_v1::ZwlrVirtualPointerV1, ()> for State {
    fn event(
        _state: &mut Self,
        _: &zwlr_virtual_pointer_v1::ZwlrVirtualPointerV1,
        _: zwlr_virtual_pointer_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<State>,
    ) {
    }
}

impl Dispatch<wl_output::WlOutput, ()> for State {
    fn event(
        _state: &mut Self,
        _: &wl_output::WlOutput,
        event: <wl_output::WlOutput as wayland_client::Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_output::Event::Mode { width, height, .. } = event {
            println!("Found output mode: {}x{}", width, height);
        }
    }
}

static BTN_LEFT: u32 = 0x110;

fn main() -> anyhow::Result<()> {
    let conn = Connection::connect_to_env()?;
    let (globals, mut event_queue) = registry_queue_init::<State>(&conn)?;
    let qh = event_queue.handle();

    globals.bind::<wl_output::WlOutput, _, _>(&qh, 1..=2, ())?;

    let manager: zwlr_virtual_pointer_manager_v1::ZwlrVirtualPointerManagerV1 =
        globals.bind(&qh, 1..=2, ())?;

    let v_pointer = manager.create_virtual_pointer(None, &qh, ());

    v_pointer.motion_absolute(timestamp(), 100, 100, 1920, 1080);
    v_pointer.frame();

    v_pointer.button(timestamp(), BTN_LEFT, ButtonState::Pressed);
    v_pointer.frame();

    v_pointer.button(timestamp(), BTN_LEFT, ButtonState::Released);
    v_pointer.frame();

    conn.flush()?;

    event_queue.roundtrip(&mut State)?;

    v_pointer.destroy();
    manager.destroy();

    Ok(())
}

fn timestamp() -> u32 {
    return SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u32;
}
