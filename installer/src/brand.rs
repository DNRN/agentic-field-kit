//! The Entropy Fox name and logo (ADR 0011: not MIT; see assets/brand/LICENSE).
//! Display only: nothing else in the installer depends on them, so a fork can
//! replace this file and assets/brand/.

use std::env;
use std::time::Duration;

use image::{DynamicImage, ImageFormat};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui_image::StatefulImage;
use ratatui_image::picker::cap_parser::QueryStdioOptions;
use ratatui_image::picker::{Picker, ProtocolType};
use ratatui_image::protocol::StatefulProtocol;

pub const NAME: &str = "Entropy Fox";
/// Fox orange from the brand palette.
pub const ORANGE: Color = Color::Rgb(0xff, 0x6d, 0x05);

/// The logo's cells. Half-blocks below 24 columns lose the fox's head
/// (checked 2026-10-28 at 16, 20, and 24 columns).
pub const LOGO_WIDTH: u16 = 24;
pub const LOGO_HEIGHT: u16 = 8;

const FOX_PNG: &[u8] = include_bytes!("../../assets/brand/fox.png");

/// Room for the logo next to the rest of the screen: 80×24 or larger.
pub fn logo_fits(area: Rect) -> bool {
    area.width >= 80 && area.height >= 24
}

fn truecolor() -> bool {
    env::var("COLORTERM").is_ok_and(|v| v == "truecolor" || v == "24bit")
        || env::var_os("WT_SESSION").is_some()
}

/// Terminals known to answer the image-protocol query. ratatui-image's query
/// leaves a thread reading stdin until the terminal answers, which would take
/// keystrokes away from the UI in a terminal that never does.
fn answers_graphics_query() -> bool {
    let var = |k: &str| env::var(k).unwrap_or_default();
    !var("KITTY_WINDOW_ID").is_empty()
        || matches!(
            var("TERM").as_str(),
            "xterm-kitty" | "xterm-ghostty" | "foot" | "foot-extra"
        )
        || matches!(
            var("TERM_PROGRAM").as_str(),
            "iTerm.app" | "WezTerm" | "ghostty"
        )
}

pub struct Logo {
    protocol: StatefulProtocol,
}

impl Logo {
    /// The logo for this terminal: its image protocol (Kitty, Sixel, iTerm2)
    /// in terminals known to have one, half-blocks on a truecolor terminal,
    /// otherwise none. Never under `NO_COLOR`. Call after entering the
    /// alternate screen.
    pub fn detect() -> Option<Logo> {
        if env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty()) {
            return None;
        }
        let queried = if answers_graphics_query() {
            let options = QueryStdioOptions {
                timeout: Duration::from_millis(1000),
                ..QueryStdioOptions::default()
            };
            Picker::from_query_stdio_with_options(options).ok()
        } else {
            None
        };
        let picker = match queried {
            Some(p) if p.protocol_type() != ProtocolType::Halfblocks => p,
            _ if truecolor() => Picker::halfblocks(),
            _ => return None,
        };
        let image: DynamicImage =
            image::load_from_memory_with_format(FOX_PNG, ImageFormat::Png).ok()?;
        Some(Logo {
            protocol: picker.new_resize_protocol(image),
        })
    }

    /// The logo drawn with half-block characters, for any truecolor terminal.
    pub fn halfblocks() -> Option<Logo> {
        let image = image::load_from_memory_with_format(FOX_PNG, ImageFormat::Png).ok()?;
        Some(Logo {
            protocol: Picker::halfblocks().new_resize_protocol(image),
        })
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_stateful_widget(StatefulImage::default(), area, &mut self.protocol);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_logo_decodes() {
        let image = image::load_from_memory_with_format(FOX_PNG, ImageFormat::Png).unwrap();
        assert!(image.width() > image.height());
    }

    #[test]
    fn the_logo_needs_80_by_24() {
        assert!(logo_fits(Rect::new(0, 0, 80, 24)));
        assert!(!logo_fits(Rect::new(0, 0, 79, 24)));
        assert!(!logo_fits(Rect::new(0, 0, 120, 23)));
    }
}
