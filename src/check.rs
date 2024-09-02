use ratatui::crossterm::terminal;

use crate::graph::Graph;

pub fn term_size(graph: &Graph) -> std::io::Result<()> {
    let (w, h) = terminal::size()?;
    let image_cell_width = (graph.max_pos_x + 1) * 2;
    let required_width = image_cell_width + 2;
    if required_width > w as usize {
        panic!("Terminal size {w}x{h} is too small. Required width is {required_width}.");
    }
    Ok(())
}
