use std::io::{self, Write};

const ANSI_BOLD: &str = "\x1b[1m";
const ANSI_GREEN: &str = "\x1b[92m";
const ANSI_RESET: &str = "\x1b[0m";

pub struct ProgressBar {
    total: usize,
    current: usize,
    width: usize,
    prefix: String,
    suffix: String,
    fill: char,
    empty: char,
}

impl ProgressBar {
    pub fn new(prefix: String, total: usize) -> Self {
        Self {
            total,
            current: 0,
            width: 50,
            prefix,
            suffix: "Complete".to_string(),
            fill: '█',
            empty: '░',
        }
    }

    pub fn increment(&mut self) {
        self.current = (self.current + 1).min(self.total);
        self.render();
    }

    fn render(&self) {
        let percent = if self.total > 0 {
            self.current as f64 / self.total as f64
        } else {
            0.0
        };

        let filled_width = (self.width as f64 * percent) as usize;

        let bar = format!(
            "{}{}",
            self.fill.to_string().repeat(filled_width),
            self.empty.to_string().repeat(self.width - filled_width)
        );

        let mut output = format!(
            "\r{}{}{} {}{}{}",
            ANSI_BOLD, self.prefix, ANSI_RESET, ANSI_GREEN, bar, ANSI_RESET
        );

        // Percent
        output.push_str(&format!(" {:.1}", percent * 100.0));

        // Count
        output.push_str(&format!(" ({}/{})", self.current, self.total));

        if percent >= 1.0 {
            output.push_str(&format!(
                " {}{}{}{}",
                ANSI_GREEN, ANSI_BOLD, self.suffix, ANSI_RESET
            ));
        }

        // Write it all out to stderr
        let _ = io::stderr().write_all(output.as_bytes());
        let _ = io::stderr().flush();
    }

    pub fn finish(&self) {
        let _ = io::stderr().write_all(b"\n");
        let _ = io::stderr().flush();
    }
}
