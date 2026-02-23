use crate::domain::model::terminal_buffer_entity::{
    TerminalAttribute, TerminalBufferEntity, TerminalColor,
};

pub(crate) struct AnsiParserDomainService {
    incomplete_sequence: String,
    byte_buffer: Vec<u8>,
}

impl AnsiParserDomainService {
    pub(crate) fn new() -> Self {
        Self {
            incomplete_sequence: String::new(),
            byte_buffer: Vec::new(),
        }
    }
}

impl Default for AnsiParserDomainService {
    fn default() -> Self {
        Self::new()
    }
}

impl AnsiParserDomainService {
    pub(crate) fn parse(&mut self, bytes: &[u8], buffer: &mut TerminalBufferEntity) {
        self.byte_buffer.extend_from_slice(bytes);

        let mut processed_len = 0;
        let mut to_parse = Vec::new();

        {
            let bytes_ref = &self.byte_buffer;
            loop {
                match std::str::from_utf8(&bytes_ref[processed_len..]) {
                    Ok(s) => {
                        if !s.is_empty() {
                            to_parse.push(s.to_string());
                        }
                        processed_len = bytes_ref.len();
                        break;
                    }
                    Err(e) => {
                        let valid_up_to = e.valid_up_to();
                        if valid_up_to > 0 {
                            let s = std::str::from_utf8(&bytes_ref[processed_len..processed_len + valid_up_to])
                                .unwrap_or("")
                                .to_string();
                            to_parse.push(s);
                            processed_len += valid_up_to;
                        }

                        if let Some(error_len) = e.error_len() {
                            processed_len += error_len;
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        if processed_len > 0 {
            self.byte_buffer.drain(0..processed_len);
        }

        for s in to_parse {
            self.parse_string(&s, buffer);
        }
    }

    fn parse_string(&mut self, valid_str: &str, buffer: &mut TerminalBufferEntity) {
        let input = if !self.incomplete_sequence.is_empty() {
            let mut combined = std::mem::take(&mut self.incomplete_sequence);
            combined.push_str(valid_str);
            combined
        } else {
            valid_str.to_string()
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
                        'M' => buffer.reverse_index(),
                        'D' => buffer.index(),
                        _ => {}
                    }
                    i += 2;
                }
            } else {
                buffer.process_normal_char(c);
                i += 1;
            }
        }
    }

    fn handle_csi(
        &mut self,
        buffer: &mut TerminalBufferEntity,
        command: char,
        params: &str,
        _intermediates: &str,
    ) {
        let empty_cell = buffer.get_empty_cell();
        match command {
            'm' => self.handle_sgr(buffer, params),
            'A' => {
                let n = self.parse_csi_param(params, 1);
                buffer.cursor.y = buffer.cursor.y.saturating_sub(n);
            }
            'B' => {
                let n = self.parse_csi_param(params, 1);
                buffer.cursor.y = std::cmp::min(buffer.height.saturating_sub(1), buffer.cursor.y + n);
            }
            '@' => {
                let n = self.parse_csi_param(params, 1);
                buffer.insert_cells(n);
            }
            'C' => {
                let n = self.parse_csi_param(params, 1);
                buffer.cursor.x = std::cmp::min(buffer.width.saturating_sub(1), buffer.cursor.x + n);
            }
            'D' => {
                let n = self.parse_csi_param(params, 1);
                buffer.cursor.x = buffer.cursor.x.saturating_sub(n);
            }
            'K' => {
                let mode = params.parse::<usize>().unwrap_or(0);
                if let Some(line) = buffer.lines.get_mut(buffer.cursor.y) {
                    match mode {
                        0 => { for cell in line.iter_mut().skip(buffer.cursor.x) { *cell = empty_cell; } }
                        1 => {
                            let end = std::cmp::min(buffer.cursor.x + 1, buffer.width);
                            for cell in line.iter_mut().take(end) { *cell = empty_cell; }
                        }
                        2 => { line.fill(empty_cell); }
                        _ => {}
                    }
                }
            }
            'P' => {
                let n = self.parse_csi_param(params, 1);
                if let Some(line) = buffer.lines.get_mut(buffer.cursor.y) {
                    let x = buffer.cursor.x;
                    if x < buffer.width {
                        let end_idx = std::cmp::min(x + n, buffer.width);
                        let removed_count = end_idx - x;
                        line.drain(x..end_idx);
                        line.extend(std::iter::repeat_n(empty_cell, removed_count));
                    }
                }
            }
            'X' => {
                let n = self.parse_csi_param(params, 1);
                if let Some(line) = buffer.lines.get_mut(buffer.cursor.y) {
                    let end_idx = std::cmp::min(buffer.cursor.x + n, buffer.width);
                    for cell in line.iter_mut().take(end_idx).skip(buffer.cursor.x) { *cell = empty_cell; }
                }
            }
            'H' | 'f' => {
                let parts: Vec<&str> = params.split(';').collect();
                let row = parts[0].parse::<usize>().unwrap_or(1);
                let col = if parts.len() > 1 { parts[1].parse::<usize>().unwrap_or(1) } else { 1 };

                let target_row = if buffer.is_origin_mode { (buffer.scroll_top + row).saturating_sub(1) } else { row.saturating_sub(1) };
                buffer.cursor.y = std::cmp::min(buffer.height.saturating_sub(1), target_row);
                buffer.cursor.x = std::cmp::min(buffer.width.saturating_sub(1), col.saturating_sub(1));
            }
            'J' => {
                let mode = params.parse::<usize>().unwrap_or(0);
                match mode {
                    0 => {
                        if let Some(line) = buffer.lines.get_mut(buffer.cursor.y) {
                            for cell in line.iter_mut().skip(buffer.cursor.x) { *cell = empty_cell; }
                        }
                        for y in (buffer.cursor.y + 1)..buffer.height {
                            if let Some(line) = buffer.lines.get_mut(y) { line.fill(empty_cell); }
                        }
                    }
                    1 => {
                        for y in 0..buffer.cursor.y {
                            if let Some(line) = buffer.lines.get_mut(y) { line.fill(empty_cell); }
                        }
                        if let Some(line) = buffer.lines.get_mut(buffer.cursor.y) {
                            let end = std::cmp::min(buffer.cursor.x + 1, buffer.width);
                            for cell in line.iter_mut().take(end) { *cell = empty_cell; }
                        }
                    }
                    2 | 3 => { for line in buffer.lines.iter_mut() { line.fill(empty_cell); } }
                    _ => {}
                }
            }
            'G' => {
                let col = self.parse_csi_param(params, 1);
                buffer.cursor.x = std::cmp::min(buffer.width.saturating_sub(1), col.saturating_sub(1));
            }
            'd' => {
                let row = self.parse_csi_param(params, 1);
                buffer.cursor.y = std::cmp::min(buffer.height.saturating_sub(1), row.saturating_sub(1));
            }
            'E' => {
                let n = self.parse_csi_param(params, 1);
                buffer.cursor.y = std::cmp::min(buffer.height.saturating_sub(1), buffer.cursor.y + n);
                buffer.cursor.x = 0;
            }
            'F' => {
                let n = self.parse_csi_param(params, 1);
                buffer.cursor.y = buffer.cursor.y.saturating_sub(n);
                buffer.cursor.x = 0;
            }
            'h' => {
                if params == "?25" { buffer.cursor.is_visible = true; }
                else if params == "?6" {
                    buffer.is_origin_mode = true;
                    buffer.cursor.y = buffer.scroll_top;
                    buffer.cursor.x = 0;
                }
            }
            'l' => {
                if params == "?25" { buffer.cursor.is_visible = false; }
                else if params == "?6" {
                    buffer.is_origin_mode = false;
                    buffer.cursor.y = 0;
                    buffer.cursor.x = 0;
                }
            }
            'r' => {
                let parts: Vec<&str> = params.split(';').collect();
                let top = parts[0].parse::<usize>().unwrap_or(1);
                let bottom = if parts.len() > 1 { parts[1].parse::<usize>().unwrap_or(buffer.height) } else { buffer.height };
                let top_idx = top.saturating_sub(1);
                let bottom_idx = bottom.saturating_sub(1);
                if top_idx < bottom_idx && bottom_idx < buffer.height {
                    buffer.scroll_top = top_idx;
                    buffer.scroll_bottom = bottom_idx;
                } else {
                    buffer.scroll_top = 0;
                    buffer.scroll_bottom = buffer.height.saturating_sub(1);
                }
                buffer.cursor.y = if buffer.is_origin_mode { buffer.scroll_top } else { 0 };
                buffer.cursor.x = 0;
            }
            'S' => { let n = self.parse_csi_param(params, 1); for _ in 0..n { buffer.scroll_up(); } }
            'T' => { let n = self.parse_csi_param(params, 1); for _ in 0..n { buffer.scroll_down(); } }
            'L' => { let n = self.parse_csi_param(params, 1); buffer.insert_lines(n); }
            'M' => { let n = self.parse_csi_param(params, 1); buffer.delete_lines(n); }
            _ => {}
        }
    }

    fn handle_sgr(&mut self, buffer: &mut TerminalBufferEntity, params: &str) {
        if params.is_empty() { buffer.current_attribute = TerminalAttribute::default(); return; }
        let parts: Vec<&str> = params.split([';', ':']).collect();
        let mut i = 0;
        while i < parts.len() {
            let p = parts[i].parse::<u8>().unwrap_or(0);
            match p {
                0 => buffer.current_attribute = TerminalAttribute::default(),
                1 => buffer.current_attribute.is_bold = true,
                2 => buffer.current_attribute.is_dim = true,
                3 => buffer.current_attribute.is_italic = true,
                4 => buffer.current_attribute.is_underline = true,
                7 => buffer.current_attribute.is_inverse = true,
                9 => buffer.current_attribute.is_strikethrough = true,
                22 => { buffer.current_attribute.is_bold = false; buffer.current_attribute.is_dim = false; }
                23 => buffer.current_attribute.is_italic = false,
                24 => buffer.current_attribute.is_underline = false,
                27 => buffer.current_attribute.is_inverse = false,
                29 => buffer.current_attribute.is_strikethrough = false,
                30..=37 => buffer.current_attribute.fg = TerminalColor::Ansi(p - 30),
                38 => {
                    i += 1;
                    if i < parts.len() {
                        let type_p = parts[i].parse::<u8>().unwrap_or(0);
                        match type_p {
                            5 => { i += 1; if i < parts.len() { buffer.current_attribute.fg = TerminalColor::Xterm(parts[i].parse::<u8>().unwrap_or(0)); i += 1; } }
                            2 => { i += 1; if i + 2 < parts.len() {
                                let r = parts[i].parse::<u8>().unwrap_or(0);
                                let g = parts[i + 1].parse::<u8>().unwrap_or(0);
                                let b = parts[i + 2].parse::<u8>().unwrap_or(0);
                                buffer.current_attribute.fg = TerminalColor::Rgb(r, g, b);
                                i += 3;
                            } else { i = parts.len(); } }
                            _ => { i += 1; }
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
                            5 => { i += 1; if i < parts.len() { buffer.current_attribute.bg = TerminalColor::Xterm(parts[i].parse::<u8>().unwrap_or(0)); i += 1; } }
                            2 => { i += 1; if i + 2 < parts.len() {
                                let r = parts[i].parse::<u8>().unwrap_or(0);
                                let g = parts[i + 1].parse::<u8>().unwrap_or(0);
                                let b = parts[i + 2].parse::<u8>().unwrap_or(0);
                                buffer.current_attribute.bg = TerminalColor::Rgb(r, g, b);
                                i += 3;
                            } else { i = parts.len(); } }
                            _ => { i += 1; }
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
        if n == 0 { 1 } else { n }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::terminal_buffer_entity::{Cell, TerminalBufferEntity};

    fn line_to_string(line: &[Cell]) -> String {
        line.iter().filter(|c| !c.is_wide_continuation).map(|cell| cell.c).collect()
    }

    #[test]
    fn test_parser_basic() {
        let mut buffer = TerminalBufferEntity::new(80, 25);
        let mut parser = AnsiParserDomainService::new();
        parser.parse(b"Hello", &mut buffer);
        let first_line = line_to_string(&buffer.get_lines()[0]);
        assert!(first_line.starts_with("Hello"));
    }

    #[test]
    fn test_utf8_fragmentation() {
        let mut buffer = TerminalBufferEntity::new(10, 5);
        let mut parser = AnsiParserDomainService::new();
        parser.parse(&[0xE3, 0x81], &mut buffer);
        assert_eq!(buffer.cursor.x, 0);
        parser.parse(&[0x82], &mut buffer);
        assert_eq!(buffer.cursor.x, 2);
        assert_eq!(buffer.lines[0][0].c, 'あ');
    }
}
