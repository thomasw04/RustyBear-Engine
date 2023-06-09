use simplelog::{ConfigBuilder, TermLogger};
use simplelog::{Level, Color, LevelFilter, TerminalMode, ColorChoice};

pub fn init()
{
    let config = ConfigBuilder::new()
        .set_level_color(Level::Trace, Some(Color::White))
        .set_level_color(Level::Info, Some(Color::Green))
        .set_level_color(Level::Warn, Some(Color::Yellow))
        .set_level_color(Level::Error, Some(Color::Red))
        .build();

    let _ = TermLogger::init(LevelFilter::Info, config, TerminalMode::Mixed, ColorChoice::Auto);
}