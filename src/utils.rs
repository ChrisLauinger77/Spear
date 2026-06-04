use gdk4::prelude::*;

pub fn copy_to_clipboard(text: &str) {
    // 1. Try GTK4's native GDK clipboard (preferred if it works)
    if let Some(display) = gdk4::Display::default() {
        let clipboard = display.clipboard();
        clipboard.set_text(text);
    }

    // 2. Fallback using standard CLI utilities to bypass Wayland focus limits
    let text_string = text.to_string();
    std::thread::spawn(move || {
        // Try wl-copy (Wayland)
        if let Ok(mut child) = std::process::Command::new("wl-copy")
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            use std::io::Write;
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(text_string.as_bytes());
            }
            let _ = child.wait();
        } else {
            // Try xclip (X11)
            if let Ok(mut child) = std::process::Command::new("xclip")
                .args(["-selection", "clipboard"])
                .stdin(std::process::Stdio::piped())
                .spawn()
            {
                use std::io::Write;
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(text_string.as_bytes());
                }
                let _ = child.wait();
            }
        }
    });
}

pub fn launch_url(url: &str) -> Result<(), glib::Error> {
    gio::AppInfo::launch_default_for_uri(url, gio::AppLaunchContext::NONE)
}

pub fn evaluate_math(expr: &str) -> Option<f64> {
    let mut parser = MathParser::new(expr);
    let val = parser.parse_expression()?;
    // If there are leftover characters (other than whitespace), it's not a fully valid expression
    parser.consume_whitespace();
    if parser.chars.peek().is_some() {
        None
    } else {
        Some(val)
    }
}

struct MathParser<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
}

impl<'a> MathParser<'a> {
    fn new(expr: &'a str) -> Self {
        Self {
            chars: expr.chars().peekable(),
        }
    }

    fn consume_whitespace(&mut self) {
        while let Some(&c) = self.chars.peek() {
            if c.is_whitespace() {
                self.chars.next();
            } else {
                break;
            }
        }
    }

    fn parse_expression(&mut self) -> Option<f64> {
        let mut val = self.parse_term()?;
        loop {
            self.consume_whitespace();
            match self.chars.peek() {
                Some('+') => {
                    self.chars.next();
                    let next_val = self.parse_term()?;
                    val += next_val;
                }
                Some('-') => {
                    self.chars.next();
                    let next_val = self.parse_term()?;
                    val -= next_val;
                }
                _ => break,
            }
        }
        Some(val)
    }

    fn parse_term(&mut self) -> Option<f64> {
        let mut val = self.parse_factor()?;
        loop {
            self.consume_whitespace();
            match self.chars.peek() {
                Some('*') => {
                    self.chars.next();
                    let next_val = self.parse_factor()?;
                    val *= next_val;
                }
                Some('/') => {
                    self.chars.next();
                    let next_val = self.parse_factor()?;
                    if next_val == 0.0 {
                        return None;
                    }
                    val /= next_val;
                }
                Some('%') => {
                    self.chars.next();
                    let next_val = self.parse_factor()?;
                    if next_val == 0.0 {
                        return None;
                    }
                    val %= next_val;
                }
                _ => break,
            }
        }
        Some(val)
    }

    fn parse_factor(&mut self) -> Option<f64> {
        self.consume_whitespace();
        let sign = if let Some(&'-') = self.chars.peek() {
            self.chars.next();
            -1.0
        } else if let Some(&'+') = self.chars.peek() {
            self.chars.next();
            1.0
        } else {
            1.0
        };

        let mut val = if let Some(&'(') = self.chars.peek() {
            self.chars.next();
            let sub_val = self.parse_expression()?;
            self.consume_whitespace();
            if self.chars.next()? != ')' {
                return None;
            }
            sub_val
        } else if let Some(&c) = self.chars.peek() {
            if c.is_ascii_digit() || c == '.' {
                self.parse_number()?
            } else if c.is_alphabetic() {
                self.parse_identifier()?
            } else {
                return None;
            }
        } else {
            return None;
        };

        self.consume_whitespace();
        if let Some(&'^') = self.chars.peek() {
            self.chars.next();
            let exponent = self.parse_factor()?;
            val = val.powf(exponent);
        }

        Some(val * sign)
    }

    fn parse_number(&mut self) -> Option<f64> {
        let mut s = String::new();
        let mut has_dot = false;
        while let Some(&c) = self.chars.peek() {
            if c.is_ascii_digit() {
                s.push(self.chars.next().unwrap());
            } else if c == '.' && !has_dot {
                has_dot = true;
                s.push(self.chars.next().unwrap());
            } else {
                break;
            }
        }
        s.parse::<f64>().ok()
    }

    fn parse_identifier(&mut self) -> Option<f64> {
        let mut s = String::new();
        while let Some(&c) = self.chars.peek() {
            if c.is_alphabetic() {
                s.push(self.chars.next().unwrap());
            } else {
                break;
            }
        }

        match s.to_lowercase().as_str() {
            "pi" => Some(std::f64::consts::PI),
            "e" => Some(std::f64::consts::E),
            "sqrt" => {
                let arg = self.parse_argument()?;
                Some(arg.sqrt())
            }
            "sin" => {
                let arg = self.parse_argument()?;
                Some(arg.sin())
            }
            "cos" => {
                let arg = self.parse_argument()?;
                Some(arg.cos())
            }
            "tan" => {
                let arg = self.parse_argument()?;
                Some(arg.tan())
            }
            "abs" => {
                let arg = self.parse_argument()?;
                Some(arg.abs())
            }
            _ => None,
        }
    }

    fn parse_argument(&mut self) -> Option<f64> {
        self.consume_whitespace();
        if self.chars.peek()? != &'(' {
            return None;
        }
        self.chars.next();
        let arg = self.parse_expression()?;
        self.consume_whitespace();
        if self.chars.next()? != ')' {
            return None;
        }
        Some(arg)
    }
}
