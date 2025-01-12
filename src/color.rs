/// Represents possible terminal colors.
#[derive(Clone, Copy)]
pub enum Color {
    End,
    Blue,
    Green,
    LightGreen,
    Orange,
    Red,
    BrightRed,
    Yellow,
    White,
}

/// Color implementation.
impl Color {
    /// Maps the color to the corresponding ANSI escape sequence.
    fn as_ansi(&self) -> &str {
        match self {
            Color::End => "\x1b[0m",
            Color::Blue => "\x1b[38;5;81m",
            Color::Green => "\x1b[38;5;47m",
            Color::LightGreen => "\x1b[38;5;156m",
            Color::Orange => "\x1b[38;5;214m",
            Color::Red => "\x1b[38;5;203m",
            Color::BrightRed => "\x1b[1;38;5;203m",
            Color::Yellow => "\x1b[38;5;227m",
            Color::White => "\x1b[38;5;231m",
        }
    }
}

/// Responsible for coloring the terminal foreground.
pub struct Foreground;

/// Foreground implementation.
impl Foreground {
    pub fn color(what: &String, color: Color) -> String {
        format!("{}{}{}", color.as_ansi(), what, Color::End.as_ansi())
    }
}
