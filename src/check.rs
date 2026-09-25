use ratatui::crossterm::terminal;

use crate::{
    graph::{CellWidthType, Graph},
    padding::{clamp_uniform, shrink_to_fit_graph},
    GraphWidthType, Result,
};

pub fn decide_cell_width_type(
    graph: &Graph,
    cell_width_type: Option<GraphWidthType>,
    page_padding: u16,
) -> Result<CellWidthType> {
    let (w, h) = terminal::size()?;
    decide_cell_width_type_from(
        graph.max_pos_x,
        w as usize,
        h as usize,
        cell_width_type,
        page_padding,
    )
}

fn decide_cell_width_type_from(
    max_pos_x: usize,
    term_width: usize,
    term_height: usize,
    cell_width_type: Option<GraphWidthType>,
    page_padding: u16,
) -> Result<CellWidthType> {
    let single_image_cell_width = max_pos_x + 1;
    let double_image_cell_width = single_image_cell_width * 2;
    let single_required_width = single_image_cell_width + 2;
    let double_required_width = double_image_cell_width + 2;

    let term_w = u16::try_from(term_width).unwrap_or(u16::MAX);
    let term_h = u16::try_from(term_height).unwrap_or(u16::MAX);
    let mut pad = clamp_uniform(page_padding, term_w, term_h);
    let available_width = |pad: u16| term_w.saturating_sub(pad.saturating_mul(2)) as usize;
    let shrink_for = |pad: u16, required_width: usize| {
        shrink_to_fit_graph(
            pad,
            term_w,
            u16::try_from(required_width).unwrap_or(u16::MAX),
        )
    };

    match cell_width_type {
        Some(GraphWidthType::Double) => {
            pad = shrink_for(pad, double_required_width);
            if double_required_width <= available_width(pad) {
                Ok(CellWidthType::Double)
            } else {
                let msg = format!("Terminal too small ({term_width}x{term_height} characters). The current graph needs at least {double_required_width} columns to display properly.");
                Err(msg.into())
            }
        }
        Some(GraphWidthType::Single) => {
            pad = shrink_for(pad, single_required_width);
            if single_required_width <= available_width(pad) {
                Ok(CellWidthType::Single)
            } else {
                let msg = format!("Terminal too small ({term_width}x{term_height} characters). The current graph needs at least {single_required_width} columns to display properly.");
                Err(msg.into())
            }
        }
        Some(GraphWidthType::Auto) | None => {
            if double_required_width <= available_width(pad) {
                return Ok(CellWidthType::Double);
            }
            if single_required_width <= available_width(pad) {
                return Ok(CellWidthType::Single);
            }
            pad = shrink_for(pad, single_required_width);
            if single_required_width <= available_width(pad) {
                return Ok(CellWidthType::Single);
            }
            let msg = format!("Terminal too small ({term_width}x{term_height} characters). The current graph needs at least {single_required_width} columns to display properly.");
            Err(msg.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GraphWidthType;

    #[test]
    fn auto_extra_shrinks_padding_to_fit_single() {
        // max_pos_x=69 → single image width 70, required 72.
        // padding 20 on 80x24 clamps to 20 (remaining 40); extra-shrink until 72 fits.
        let result = decide_cell_width_type_from(69, 80, 24, Some(GraphWidthType::Auto), 20);
        assert_eq!(result.unwrap(), CellWidthType::Single);
    }

    #[test]
    fn auto_picks_double_when_it_fits_padded_area() {
        let result = decide_cell_width_type_from(10, 80, 24, Some(GraphWidthType::Auto), 2);
        assert_eq!(result.unwrap(), CellWidthType::Double);
    }

    #[test]
    fn explicit_double_errors_with_real_term_size_when_unpadded_too_small() {
        let err = decide_cell_width_type_from(50, 80, 24, Some(GraphWidthType::Double), 2)
            .unwrap_err()
            .to_string();
        assert!(err.contains("80x24"), "{err}");
        assert!(err.contains("104 columns"), "{err}");
    }

    #[test]
    fn error_mentions_full_terminal_width_not_remaining() {
        let err = decide_cell_width_type_from(50, 80, 24, Some(GraphWidthType::Double), 20)
            .unwrap_err()
            .to_string();
        assert!(err.contains("80x24"), "{err}");
        assert!(!err.contains("40x"), "{err}");
    }
}
