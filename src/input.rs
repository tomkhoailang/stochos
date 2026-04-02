use crate::{config::config, runtime::options};

pub fn hints() -> &'static [char] {
    &config().grid.hints
}
pub fn cols() -> u32 {
    config().cols()
}
pub fn rows() -> u32 {
    config().rows()
}

pub fn sub_hints() -> &'static [char] {
    let hints = &config().grid.sub_hints;
    if let Some(size) = options().subgrid_size {
        let len = (size as usize)
            .saturating_mul(size as usize)
            .min(hints.len());
        &hints[..len]
    } else {
        hints
    }
}
pub fn sub_cols() -> u32 {
    options().subgrid_size.unwrap_or(config().grid.sub_cols)
}
pub fn sub_rows() -> u32 {
    if let Some(size) = options().subgrid_size {
        size
    } else {
        config().sub_rows()
    }
}

#[derive(Clone, Copy)]
pub enum InputState {
    /// Waiting for the first main-grid character
    First,
    /// First main-grid character pressed; waiting for second
    Second(char),
    /// Main cell chosen; waiting for a single sub-grid character
    SubFirst { col: u32, row: u32 },
    /// Sub-cell chosen; mouse positioned, waiting for Space/Enter
    Ready {
        col: u32,
        row: u32,
        sub_col: u32,
        sub_row: u32,
    },
}

#[cfg(test)]
mod tests {
    fn clipped_subgrid_len(size: u32, available: usize) -> usize {
        (size as usize).saturating_mul(size as usize).min(available)
    }

    #[test]
    fn clips_override_to_available_hints() {
        assert_eq!(clipped_subgrid_len(4, 25), 16);
        assert_eq!(clipped_subgrid_len(5, 25), 25);
        assert_eq!(clipped_subgrid_len(5, 17), 17);
    }
}

impl InputState {
    /// Returns the key string encoding the current navigation position.
    /// Returns an empty string for states that haven't reached a target yet.
    pub fn keys(&self) -> String {
        match self {
            InputState::SubFirst { col, row } => {
                format!("{}{}", hints()[*col as usize], hints()[*row as usize])
            }
            InputState::Ready {
                col,
                row,
                sub_col,
                sub_row,
            } => {
                format!(
                    "{}{}{}",
                    hints()[*col as usize],
                    hints()[*row as usize],
                    sub_hints()[(*sub_row * sub_cols() + *sub_col) as usize]
                )
            }
            _ => String::new(),
        }
    }
}

/// Converts a 2- or 3-character key string to a pixel position.
pub fn keys_to_pos(keys: &str, w: u32, h: u32) -> Option<(u32, u32)> {
    let hints = hints();
    let mut chars = keys.chars();
    let c0 = chars.next()?;
    let c1 = chars.next()?;
    let col = hints.iter().position(|&c| c == c0)? as u32;
    let row = hints.iter().position(|&c| c == c1)? as u32;
    let cell_w = w / cols();
    let cell_h = h / rows();
    match chars.next() {
        None => Some((col * cell_w + cell_w / 2, row * cell_h + cell_h / 2)),
        Some(c2) => {
            let idx = sub_hints().iter().position(|&c| c == c2)? as u32;
            let sub_col = idx % sub_cols();
            let sub_row = idx / sub_cols();
            let sub_cell_w = cell_w / sub_cols();
            let sub_cell_h = cell_h / sub_rows();
            Some((
                col * cell_w + sub_col * sub_cell_w + sub_cell_w / 2,
                row * cell_h + sub_row * sub_cell_h + sub_cell_h / 2,
            ))
        }
    }
}
