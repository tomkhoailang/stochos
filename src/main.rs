mod app;
mod backend;
mod config;
mod input;
mod macro_store;
mod mode;
mod render;

fn main() -> anyhow::Result<()> {
    config::init();

    #[cfg(feature = "wayland")]
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        if let Ok(mut b) = backend::wayland::WaylandBackend::new() {
            return app::run(&mut b);
        }
    }

    #[cfg(feature = "x11")]
    if std::env::var_os("DISPLAY").is_some() {
        let mut b = backend::x11::X11Backend::new()?;
        return app::run(&mut b);
    }

    anyhow::bail!("no display server found (need WAYLAND_DISPLAY or DISPLAY)")
}
