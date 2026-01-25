use crate::domain::terminal::{TerminalBuffer, TerminalAttribute, TerminalColor};

pub struct AnsiParser {
    incomplete_sequence: String,
}

impl AnsiParser {
    pub fn new() -> Self {
        Self {
            incomplete_sequence: String::new(),
        }
    }

    pub fn parse(&mut self, s: &str, buffer: &mut TerminalBuffer) {
        let input = if !self.incomplete_sequence.is_empty() {
            let mut combined = std::mem::take(&mut self.incomplete_sequence);
            combined.push_str(s);
            combined
        } else {
            s.to_string()
        };

        let char_vec: Vec<char> = input.chars().collect();
        let mut i = 0;

        while i < char_vec.len() {
            let c = char_vec[i];
            if c == '\x1b' {
                if i + 1 >= char_vec.len() {
                    self.incomplete_sequence.push(c);
                    break;
                }

                let next_c = char_vec[i + 1];
                if next_c == '[' {
                    let start_idx = i;
                    let mut current_idx = i + 2;
                    let mut complete = false;

                    while current_idx < char_vec.len() {
                        let ch = char_vec[current_idx];
                        let code = ch as u32 as u8;
                        if (0x30..=0x3F).contains(&code) || (0x20..=0x2F).contains(&code) {
                            current_idx += 1;
                        } else {
                            complete = true;
                            break;
                        }
                    }

                    if complete {
                        let end_idx = current_idx;
                        let cmd = char_vec[end_idx];
                        let inner = &char_vec[(i + 2)..end_idx];
                        let param_str: String = inner
                            .iter()
                            .take_while(|&&c| (0x30..=0x3F).contains(&(c as u32 as u8)))
                            .collect();

                        let intermediate_str: String = inner.iter().skip(param_str.len()).collect();

                        self.handle_csi(buffer, cmd, &param_str, &intermediate_str);
                        i = end_idx + 1;
                    } else {
                        self.incomplete_sequence = char_vec[start_idx..].iter().collect();
                        break;
                    }
                } else if next_c == ']' {
                    let start_idx = i;
                    let mut current_idx = i + 2;
                    let mut found_terminator = false;
                    let mut terminator_len = 0;

                    while current_idx < char_vec.len() {
                        let ch = char_vec[current_idx];
                        if ch == '\x07' {
                            found_terminator = true;
                            terminator_len = 1;
                            break;
                        } else if ch == '\x1b' {
                            if current_idx + 1 < char_vec.len() {
                                if char_vec[current_idx + 1] == '\\' {
                                    found_terminator = true;
                                    terminator_len = 2;
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                        current_idx += 1;
                    }

                    if found_terminator {
                        i = current_idx + terminator_len;
                    } else {
                        self.incomplete_sequence = char_vec[start_idx..].iter().collect();
                        break;
                    }
                } else {
                    match next_c {
                        '7' => buffer.save_cursor(),
                        '8' => buffer.restore_cursor(),
                        _ => {} // Ignore other control chars
                    }
                    i += 2;
                }
            } else {
                buffer.process_normal_char(c);
                i += 1;
            }
        }
    }

    fn handle_csi(&mut self, buffer: &mut TerminalBuffer, command: char, params: &str, _intermediates: &str) {
        match command {
            'm' => self.handle_sgr(buffer, params),
            'A' => {
                let n = self.parse_csi_param(params, 1);
                let current_col = buffer.get_display_width_up_to(buffer.cursor.y, buffer.cursor.x);
                if buffer.cursor.y >= n {
                    buffer.cursor.y -= n;
                } else {
                    buffer.cursor.y = 0;
                }
                buffer.cursor.x = buffer.display_col_to_char_index(buffer.cursor.y, current_col);
            }
            'B' => {
                let n = self.parse_csi_param(params, 1);
                let current_col = buffer.get_display_width_up_to(buffer.cursor.y, buffer.cursor.x);
                buffer.cursor.y = std::cmp::min(buffer.height - 1, buffer.cursor.y + n);
                buffer.cursor.x = buffer.display_col_to_char_index(buffer.cursor.y, current_col);
            }
            'C' => {
                let n = self.parse_csi_param(params, 1);
                let current_col = buffer.get_display_width_up_to(buffer.cursor.y, buffer.cursor.x);
                let target_col = std::cmp::min(buffer.width - 1, current_col + n);
                buffer.cursor.x = buffer.display_col_to_char_index(buffer.cursor.y, target_col);
            }
            'D' => {
                let n = self.parse_csi_param(params, 1);
                let current_col = buffer.get_display_width_up_to(buffer.cursor.y, buffer.cursor.x);
                let target_col = current_col.saturating_sub(n);
                buffer.cursor.x = buffer.display_col_to_char_index(buffer.cursor.y, target_col);
            }
            'K' => {
                let mode = params.parse::<usize>().unwrap_or(0);
                if let Some(line) = buffer.lines.get_mut(buffer.cursor.y) {
                    while line.len() < buffer.width {
                        line.push(crate::domain::terminal::Cell::default());
                    }
                    match mode {
                        0 => {
                            if buffer.cursor.x < line.len() {
                                for cell in line.iter_mut().skip(buffer.cursor.x) {
                                    *cell = crate::domain::terminal::Cell::default();
                                }
                            }
                        }
                        1 => {
                            let end = std::cmp::min(buffer.cursor.x + 1, line.len());
                            for cell in line.iter_mut().take(end) {
                                *cell = crate::domain::terminal::Cell::default();
                            }
                        }
                        2 => {
                            line.fill(crate::domain::terminal::Cell::default());
                        }
                        _ => {} 
                    }
                }
            }
            'P' => {
                let n = self.parse_csi_param(params, 1);
                if let Some(line) = buffer.lines.get_mut(buffer.cursor.y) {
                    if buffer.cursor.x < line.len() {
                        let end_idx = std::cmp::min(buffer.cursor.x + n, line.len());
                        let removed_count = end_idx - buffer.cursor.x;
                        line.drain(buffer.cursor.x..end_idx);
                        line.extend(std::iter::repeat_n(crate::domain::terminal::Cell::default(), removed_count));
                    }
                }
            }
            'X' => {
                let n = self.parse_csi_param(params, 1);
                if let Some(line) = buffer.lines.get_mut(buffer.cursor.y) {
                    while line.len() < buffer.width {
                        line.push(crate::domain::terminal::Cell::default());
                    }
                    if buffer.cursor.x < line.len() {
                        let end_idx = std::cmp::min(buffer.cursor.x + n, line.len());
                        for cell in line.iter_mut().take(end_idx).skip(buffer.cursor.x) {
                            *cell = crate::domain::terminal::Cell::default();
                        }
                    }
                }
            }
            'H' | 'f' => {
                let parts: Vec<&str> = params.split(';').collect();
                let row = if parts.is_empty() || parts[0].is_empty() {
                    1
                } else {
                    parts[0].parse::<usize>().unwrap_or(1)
                };
                let col = if parts.len() < 2 || parts[1].is_empty() {
                    1
                } else {
                    parts[1].parse::<usize>().unwrap_or(1)
                };
                buffer.cursor.y = if row > 0 { row - 1 } else { 0 };
                if buffer.cursor.y >= buffer.height {
                    buffer.cursor.y = buffer.height - 1;
                }
                let target_display_col = if col > 0 { col - 1 } else { 0 };
                buffer.cursor.x = buffer.display_col_to_char_index(buffer.cursor.y, target_display_col);
            }
            'J' => {
                let mode = params.parse::<usize>().unwrap_or(0);
                match mode {
                    0 => {
                        if let Some(line) = buffer.lines.get_mut(buffer.cursor.y) {
                            while line.len() < buffer.width {
                                line.push(crate::domain::terminal::Cell::default());
                            }
                            for cell in line.iter_mut().skip(buffer.cursor.x) {
                                *cell = crate::domain::terminal::Cell::default();
                            }
                        }
                        for y in (buffer.cursor.y + 1)..buffer.lines.len() {
                            if let Some(line) = buffer.lines.get_mut(y) {
                                line.fill(crate::domain::terminal::Cell::default());
                            }
                        }
                    }
                    1 => {
                        for y in 0..buffer.cursor.y {
                            if let Some(line) = buffer.lines.get_mut(y) {
                                line.fill(crate::domain::terminal::Cell::default());
                            }
                        }
                        if let Some(line) = buffer.lines.get_mut(buffer.cursor.y) {
                            while line.len() < buffer.width {
                                line.push(crate::domain::terminal::Cell::default());
                            }
                            for cell in line.iter_mut().take(buffer.cursor.x + 1) {
                                *cell = crate::domain::terminal::Cell::default();
                            }
                        }
                    }
                    2 | 3 => {
                        for line in buffer.lines.iter_mut() {
                            line.fill(crate::domain::terminal::Cell::default());
                        }
                    }
                    _ => {} 
                }
            }
            'G' => {
                let col = self.parse_csi_param(params, 1);
                let target_display_col = if col > 0 { col - 1 } else { 0 };
                let target_display_col = 
                    std::cmp::min(target_display_col, buffer.width.saturating_sub(1));
                buffer.cursor.x = buffer.display_col_to_char_index(buffer.cursor.y, target_display_col);
            }
            'd' => {
                let row = self.parse_csi_param(params, 1);
                let current_display_col = 
                    buffer.get_display_width_up_to(buffer.cursor.y, buffer.cursor.x);
                buffer.cursor.y = if row > 0 { row - 1 } else { 0 };
                if buffer.cursor.y >= buffer.height {
                    buffer.cursor.y = buffer.height - 1;
                }
                buffer.cursor.x = buffer.display_col_to_char_index(buffer.cursor.y, current_display_col);
            }
            'E' => {
                let n = self.parse_csi_param(params, 1);
                buffer.cursor.y = std::cmp::min(buffer.height - 1, buffer.cursor.y + n);
                buffer.cursor.x = 0;
            }
            'F' => {
                let n = self.parse_csi_param(params, 1);
                if buffer.cursor.y >= n {
                    buffer.cursor.y -= n;
                } else {
                    buffer.cursor.y = 0;
                }
                buffer.cursor.x = 0;
            }
            'h' => {
                if params == "?25" {
                    buffer.cursor.visible = true;
                }
            }
            'l' => {
                if params == "?25" {
                    buffer.cursor.visible = false;
                }
            }
            'r' => {
                let parts: Vec<&str> = params.split(';').collect();
                let top = if parts.is_empty() || parts[0].is_empty() {
                    1
                } else {
                    parts[0].parse::<usize>().unwrap_or(1)
                };
                let bottom = if parts.len() < 2 || parts[1].is_empty() {
                    buffer.height
                } else {
                    parts[1].parse::<usize>().unwrap_or(buffer.height)
                };
                let top_idx = if top > 0 { top - 1 } else { 0 };
                let bottom_idx = if bottom > 0 { bottom - 1 } else { 0 };
                if top_idx < bottom_idx && bottom_idx < buffer.height {
                    buffer.scroll_top = top_idx;
                    buffer.scroll_bottom = bottom_idx;
                    buffer.cursor.x = 0;
                    buffer.cursor.y = 0;
                } else {
                    buffer.scroll_top = 0;
                    buffer.scroll_bottom = buffer.height.saturating_sub(1);
                    buffer.cursor.x = 0;
                    buffer.cursor.y = 0;
                }
            }
            _ => {} 
        }
    }

    fn handle_sgr(&mut self, buffer: &mut TerminalBuffer, params: &str) {
        if params.is_empty() {
            buffer.current_attribute = TerminalAttribute::default();
            return;
        }

        let parts: Vec<&str> = params.split(|c| c == ';' || c == ':').collect();
        let mut i = 0;
        while i < parts.len() {
            let p = parts[i].parse::<u8>().unwrap_or(0);
            match p {
                0 => buffer.current_attribute = TerminalAttribute::default(),
                1 => buffer.current_attribute.bold = true,
                2 => buffer.current_attribute.dim = true,
                3 => buffer.current_attribute.italic = true,
                4 => buffer.current_attribute.underline = true,
                7 => buffer.current_attribute.inverse = true,
                9 => buffer.current_attribute.strikethrough = true,
                22 => {
                    buffer.current_attribute.bold = false;
                    buffer.current_attribute.dim = false;
                }
                23 => buffer.current_attribute.italic = false,
                24 => buffer.current_attribute.underline = false,
                27 => buffer.current_attribute.inverse = false,
                29 => buffer.current_attribute.strikethrough = false,
                30..=37 => buffer.current_attribute.fg = TerminalColor::Ansi(p - 30),
                38 => {
                    i += 1;
                    if i < parts.len() {
                        let type_p = parts[i].parse::<u8>().unwrap_or(0);
                        match type_p {
                            5 => {
                                i += 1;
                                if i < parts.len() {
                                    let color_idx = parts[i].parse::<u8>().unwrap_or(0);
                                    buffer.current_attribute.fg = TerminalColor::Xterm(color_idx);
                                    i += 1;
                                }
                            }
                            2 => {
                                i += 1;
                                if i + 2 < parts.len() {
                                    let r = parts[i].parse::<u8>().unwrap_or(0);
                                    let g = parts[i + 1].parse::<u8>().unwrap_or(0);
                                    let b = parts[i + 2].parse::<u8>().unwrap_or(0);
                                    buffer.current_attribute.fg = TerminalColor::Rgb(r, g, b);
                                    i += 3;
                                } else {
                                    i = parts.len();
                                }
                            }
                            _ => {
                                i += 1;
                            }
                        }
                    }
                    continue;
                }
                39 => buffer.current_attribute.fg = TerminalColor::Default,
                40..=47 => buffer.current_attribute.bg = TerminalColor::Ansi(p - 40),
                48 => {
                    i += 1;
                    if i < parts.len() {
                        let type_p = parts[i].parse::<u8>().unwrap_or(0);
                        match type_p {
                            5 => {
                                i += 1;
                                if i < parts.len() {
                                    let color_idx = parts[i].parse::<u8>().unwrap_or(0);
                                    buffer.current_attribute.bg = TerminalColor::Xterm(color_idx);
                                    i += 1;
                                }
                            }
                            2 => {
                                i += 1;
                                if i + 2 < parts.len() {
                                    let r = parts[i].parse::<u8>().unwrap_or(0);
                                    let g = parts[i + 1].parse::<u8>().unwrap_or(0);
                                    let b = parts[i + 2].parse::<u8>().unwrap_or(0);
                                    buffer.current_attribute.bg = TerminalColor::Rgb(r, g, b);
                                    i += 3;
                                } else {
                                    i = parts.len();
                                }
                            }
                            _ => {
                                i += 1;
                            }
                        }
                    }
                    continue;
                }
                49 => buffer.current_attribute.bg = TerminalColor::Default,
                90..=97 => buffer.current_attribute.fg = TerminalColor::Ansi(p - 90 + 8),
                100..=107 => buffer.current_attribute.bg = TerminalColor::Ansi(p - 100 + 8),
                _ => {} 
            }
            i += 1;
        }
    }

    fn parse_csi_param(&self, params: &str, default: usize) -> usize {
        let n = params.parse::<usize>().unwrap_or(default);
        if n == 0 {
            1
        } else {
            n
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::terminal::{TerminalBuffer, Cell};

    fn line_to_string(line: &[Cell]) -> String {
        line.iter().map(|cell| cell.c).collect()
    }

    #[test]
    fn test_parser_basic() {
        let mut buffer = TerminalBuffer::new(80, 25);
        let mut parser = AnsiParser::new();
        parser.parse("Hello", &mut buffer);
        let first_line = line_to_string(&buffer.get_lines()[0]);
        assert!(first_line.starts_with("Hello"));
    }

    #[test]
    fn test_sgr_colors() {
        let mut buffer = TerminalBuffer::new(80, 25);
        let mut parser = AnsiParser::new();
        parser.parse("\x1b[31mRed\x1b[39mDefault", &mut buffer);
        assert_eq!(buffer.get_lines()[0][0].attribute.fg, TerminalColor::Ansi(1));
        assert_eq!(buffer.get_lines()[0][3].attribute.fg, TerminalColor::Default);
    }

    #[test]
    fn test_terminal_resize() {
        let mut buffer = TerminalBuffer::new(10, 5);
        let mut parser = AnsiParser::new();
        parser.parse("Hello CJKあいう", &mut buffer);
        // "Hello CJK" is 9 chars. "あ" is 2 width. Total 11.
        // With width 10, "CJK" might be wrapped or truncated.
        // Current logic wraps at char boundary.
        assert_eq!(buffer.get_cursor_pos().1, 1); 
        buffer.resize(20, 10);
        assert_eq!(line_to_string(&buffer.get_lines()[0]).trim(), "Hello CJK");
    }
}
