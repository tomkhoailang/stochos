use anyhow::{bail, Result};
use std::{ffi::OsString, sync::OnceLock};

static OPTIONS: OnceLock<Options> = OnceLock::new();

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Options {
    pub single_click: bool,
}

#[derive(Debug)]
pub enum ArgsAction {
    Run(Options),
    Help,
}

pub fn parse_args<I, T>(args: I) -> Result<ArgsAction>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString>,
{
    let mut options = Options::default();

    for arg in args.into_iter().skip(1) {
        match arg.into().to_string_lossy().as_ref() {
            "--single-click" => options.single_click = true,
            "-h" | "--help" => return Ok(ArgsAction::Help),
            other => bail!("unknown argument: {other}\n\n{}", usage()),
        }
    }

    Ok(ArgsAction::Run(options))
}

pub fn set_options(options: Options) {
    OPTIONS
        .set(options)
        .expect("runtime options already initialized");
}

pub fn options() -> &'static Options {
    OPTIONS.get_or_init(Options::default)
}

pub const fn usage() -> &'static str {
    "Usage: stochos [--single-click]\n\
     \n\
     Options:\n\
       --single-click  Click immediately after the third hint key\n\
       -h, --help      Show this help message\n"
}

#[cfg(test)]
mod tests {
    use super::{parse_args, ArgsAction, Options};

    #[test]
    fn parses_single_click_flag() {
        let args = ["stochos", "--single-click"];
        let parsed = parse_args(args).unwrap();
        match parsed {
            ArgsAction::Run(options) => assert_eq!(options, Options { single_click: true }),
            ArgsAction::Help => panic!("unexpected help action"),
        }
    }

    #[test]
    fn parses_help_flag() {
        let args = ["stochos", "--help"];
        assert!(matches!(parse_args(args).unwrap(), ArgsAction::Help));
    }

    #[test]
    fn rejects_unknown_flags() {
        let args = ["stochos", "--wat"];
        let err = parse_args(args).unwrap_err();
        assert!(err.to_string().contains("unknown argument: --wat"));
    }
}
