mod app;
mod backend;
mod config;
mod input;
mod macro_store;
mod mode;
mod render;
mod runtime;

fn main() -> anyhow::Result<()> {
    match runtime::parse_args(std::env::args_os())? {
        runtime::ArgsAction::Run(options) => runtime::set_options(options),
        runtime::ArgsAction::Help => {
            print!("{}", runtime::usage());
            return Ok(());
        }
    }

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
