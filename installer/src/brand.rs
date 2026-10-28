//! The Entropy Fox name and logo (ADR 0011: not MIT; see assets/brand/LICENSE).
//! Display only: nothing else in the installer depends on them, so a fork can
//! replace this file and assets/brand/.

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};

pub const NAME: &str = "Entropy Fox";
// The brand palette as xterm 256-colour indices, so it shows the same in
// terminals without truecolor. Mid tones: each shows on dark and on light
// backgrounds. The logo and the UI's colour scheme both use it.
/// Fox orange (#ff6d05).
pub const ORANGE: Color = Color::Indexed(202);
pub const CREAM: Color = Color::Indexed(180);
pub const LIGHT_ORANGE: Color = Color::Indexed(214);
pub const RUST: Color = Color::Indexed(166);
pub const BROWN: Color = Color::Indexed(94);

/// The logo's cells: the size of assets/brand/fox.txt (ADR 0012).
pub const LOGO_WIDTH: u16 = 41;
pub const LOGO_HEIGHT: u16 = 12;

const FOX: &str = include_str!("../../assets/brand/fox.txt");
/// One letter per character of FOX, naming its shade; see `shade`.
const FOX_COLORS: &str = include_str!("../../assets/brand/fox.colors");

/// Room for the logo next to the rest of the screen: 80×24 or larger.
pub fn logo_fits(area: Rect) -> bool {
    area.width >= 80 && area.height >= 24
}

/// The brand shade a letter in fox.colors names.
fn shade(key: u8) -> Option<Color> {
    match key {
        b'c' => Some(CREAM),
        b'l' => Some(LIGHT_ORANGE),
        b'o' => Some(ORANGE),
        b'r' => Some(RUST),
        b'b' => Some(BROWN),
        _ => None,
    }
}

/// The logo as ASCII; in its shades when `colored`.
pub fn logo(colored: bool) -> Text<'static> {
    FOX.lines()
        .zip(FOX_COLORS.lines())
        .map(|(chars, keys)| {
            if !colored {
                return Line::raw(chars);
            }
            let keys = keys.as_bytes();
            let mut spans = Vec::new();
            let mut start = 0;
            for end in 1..=chars.len() {
                if end == chars.len() || keys.get(end) != keys.get(start) {
                    let style = keys
                        .get(start)
                        .copied()
                        .and_then(shade)
                        .map_or_else(Style::default, |c| Style::default().fg(c));
                    spans.push(Span::styled(&chars[start..end], style));
                    start = end;
                }
            }
            Line::from(spans)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// fox.txt and fox.colors are edited by hand; a character without a
    /// shade, or a logo that outgrows its cells, would show only on screen.
    #[test]
    fn every_character_of_the_logo_has_a_shade_and_fits() {
        let chars: Vec<&str> = FOX.lines().collect();
        let keys: Vec<&str> = FOX_COLORS.lines().collect();
        assert_eq!(chars.len(), usize::from(LOGO_HEIGHT));
        assert_eq!(keys.len(), chars.len());
        for (n, (line, key)) in chars.iter().zip(&keys).enumerate() {
            assert!(line.len() <= usize::from(LOGO_WIDTH), "line {n} too wide");
            assert_eq!(line.len(), key.len(), "line {n}: lengths differ");
            for (c, k) in line.bytes().zip(key.bytes()) {
                assert!(c.is_ascii_graphic() || c == b' ', "line {n}: not ASCII");
                if c == b' ' {
                    assert_eq!(k, b' ', "line {n}: shade on a space");
                } else {
                    assert!(shade(k).is_some(), "line {n}: no shade for {:?}", c as char);
                }
            }
        }
        assert!(chars.iter().any(|l| l.len() == usize::from(LOGO_WIDTH)));
    }

    /// Both modes draw every character of fox.txt; coloured, each character
    /// has the shade fox.colors names for it, and plain, none.
    #[test]
    fn the_logo_keeps_every_character_in_its_shade() {
        for colored in [false, true] {
            let text = logo(colored);
            assert_eq!(text.lines.len(), FOX.lines().count());
            for ((line, chars), keys) in text.lines.iter().zip(FOX.lines()).zip(FOX_COLORS.lines())
            {
                let drawn: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
                assert_eq!(drawn, chars);
                let mut at = 0;
                for span in &line.spans {
                    for (i, key) in keys.bytes().enumerate().skip(at).take(span.content.len()) {
                        let want = if colored { shade(key) } else { None };
                        assert_eq!(span.style.fg, want, "{chars:?} at {i}");
                    }
                    at += span.content.len();
                }
            }
        }
    }
}
