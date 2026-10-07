// No console window in release builds on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // The installer runs `BeamLink --setup` after copying files and
    // `--remove-mods` before uninstalling; neither opens a window.
    if let Some(arg) = std::env::args().nth(1)
        && let Some(code) = beamlink_lib::headless(&arg)
    {
        std::process::exit(code);
    }
    beamlink_lib::run();
}
