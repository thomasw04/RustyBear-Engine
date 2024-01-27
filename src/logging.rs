use indicatif::{MultiProgress, ProgressBar};
use indicatif_log_bridge::LogWrapper;
use once_cell::sync::OnceCell;
#[cfg(not(target_arch = "wasm32"))]
use simplelog::{Color, ColorChoice, Level, LevelFilter, TerminalMode};
#[cfg(not(target_arch = "wasm32"))]
use simplelog::{ConfigBuilder, TermLogger};

static PROGRESS_BARS: OnceCell<MultiProgress> = OnceCell::new();

pub fn install_bar(bar: ProgressBar) -> Option<ProgressBar> {
    if let Some(bars) = PROGRESS_BARS.get() {
        Some(bars.add(bar))
    } else {
        log::error!("Failed to install progress bar.");
        None
    }
}

pub fn remove_bar(bar: &ProgressBar) {
    if let Some(bars) = PROGRESS_BARS.get() {
        bars.remove(bar);
    } else {
        log::error!("Failed to remove progress bar.");
    }
}

pub fn init() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")]
        {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Failed to init logger.");
        }
        else
        {
            let config = ConfigBuilder::new()
            .set_level_color(Level::Trace, Some(Color::White))
            .set_level_color(Level::Info, Some(Color::Green))
            .set_level_color(Level::Warn, Some(Color::Yellow))
            .set_level_color(Level::Error, Some(Color::Red))
            .build();



            let logger = TermLogger::new(LevelFilter::Info, config, TerminalMode::Mixed, ColorChoice::Auto);
            let _ = PROGRESS_BARS.set(MultiProgress::new());

            if let Some(bar) = PROGRESS_BARS.get() {
                if let Err(err) = LogWrapper::new(bar.clone(), logger).try_init() {
                    eprintln!("Failed to init logger: {} Abort.", err);
                }
            } else {
                eprintln!("Failed to install progress bar. Abort.");
            }
        }
    }
}
