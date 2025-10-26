use ratatui::crossterm::terminal;

use crate::{
    graph::{CellWidthType, Graph},
    GraphWidthType, Result,
};

pub fn decide_cell_width_type(
    graph: &Graph,
    cell_width_type: Option<GraphWidthType>,
) -> Result<CellWidthType> {
    let (w, h) = terminal::size()?;
    decide_cell_width_type_from(graph.max_pos_x, w as usize, h as usize, cell_width_type)
}

fn decide_cell_width_type_from(
    max_pos_x: usize,
    term_width: usize,
    term_height: usize,
    cell_width_type: Option<GraphWidthType>,
) -> Result<CellWidthType> {
    let single_image_cell_width = max_pos_x + 1;
    let double_image_cell_width = single_image_cell_width * 2;

    match cell_width_type {
        Some(GraphWidthType::Double) => {
            let required_width = double_image_cell_width + 2;
            if required_width > term_width {
                let msg = format!("Terminal too small ({term_width}x{term_height} characters). The current graph needs at least {required_width} columns to display properly.");
                return Err(msg.into());
            }
            Ok(CellWidthType::Double)
        }
        Some(GraphWidthType::Single) => {
            let required_width = single_image_cell_width + 2;
            if required_width > term_width {
                let msg = format!("Terminal too small ({term_width}x{term_height} characters). The current graph needs at least {required_width} columns to display properly.");
                return Err(msg.into());
            }
            Ok(CellWidthType::Single)
        }
        Some(GraphWidthType::Auto) | None => {
            let double_required_width = double_image_cell_width + 2;
            if double_required_width <= term_width {
                return Ok(CellWidthType::Double);
            }
            let single_required_width = single_image_cell_width + 2;
            if single_required_width <= term_width {
                return Ok(CellWidthType::Single);
            }
            let msg = format!("Terminal too small ({term_width}x{term_height} characters). The current graph needs at least {single_required_width} columns to display properly.");
            Err(msg.into())
        }
    }
}
