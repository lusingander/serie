use ratatui::crossterm::terminal;

use crate::graph::{CellWidthType, Graph};

pub fn decide_cell_width_type(
    graph: &Graph,
    cell_width_type: Option<CellWidthType>,
) -> std::io::Result<CellWidthType> {
    let (w, h) = terminal::size()?;
    let cell_width_type =
        decide_cell_width_type_from(graph.max_pos_x, w as usize, h as usize, cell_width_type);
    Ok(cell_width_type)
}

fn decide_cell_width_type_from(
    max_pos_x: usize,
    term_width: usize,
    term_height: usize,
    cell_width_type: Option<CellWidthType>,
) -> CellWidthType {
    let single_image_cell_width = max_pos_x + 1;
    let double_image_cell_width = single_image_cell_width * 2;

    match cell_width_type {
        Some(CellWidthType::Double) => {
            let required_width = double_image_cell_width + 2;
            if required_width > term_width {
                panic!("Terminal size {term_width}x{term_height} is too small. Required width is {required_width} (graph_width = double).");
            }
            CellWidthType::Double
        }
        Some(CellWidthType::Single) => {
            let required_width = single_image_cell_width + 2;
            if required_width > term_width {
                panic!("Terminal size {term_width}x{term_height} is too small. Required width is {required_width} (graph_width = single).");
            }
            CellWidthType::Single
        }
        None => {
            let double_required_width = double_image_cell_width + 2;
            if double_required_width <= term_width {
                return CellWidthType::Double;
            }
            let single_required_width = single_image_cell_width + 2;
            if single_required_width <= term_width {
                return CellWidthType::Single;
            }
            panic!(
                "Terminal size {term_width}x{term_height} is too small. Required width is {single_required_width} (graph_width = single) or {double_required_width} (graph_width = double)."
            );
        }
    }
}
