pub mod color;
pub mod config;
pub mod git;
pub mod graph;
pub mod protocol;

mod app;
mod check;
mod event;
mod external;
mod keybind;
mod view;
mod widget;

use std::path::Path;

use app::App;
use clap::{Parser, ValueEnum};
use graph::GraphImageManager;

/// Serie - A rich git commit graph in your terminal, like magic 📚
#[derive(Parser)]
#[command(version)]
struct Args {
    /// Image protocol to render graph
    #[arg(short, long, value_name = "TYPE", default_value = "auto")]
    protocol: ImageProtocolType,

    /// Commit ordering algorithm
    #[arg(short, long, value_name = "TYPE", default_value = "chrono")]
    order: CommitOrderType,

    /// Commit graph image cell width
    #[arg(short, long, value_name = "TYPE")]
    graph_width: Option<GraphWidthType>,

    /// Preload all graph images
    #[arg(long, default_value = "false")]
    preload: bool,
}

#[derive(Debug, Clone, ValueEnum)]
enum ImageProtocolType {
    Auto,
    Iterm,
    Kitty,
}

impl From<ImageProtocolType> for protocol::ImageProtocol {
    fn from(protocol: ImageProtocolType) -> Self {
        match protocol {
            ImageProtocolType::Auto => protocol::auto_detect(),
            ImageProtocolType::Iterm => protocol::ImageProtocol::Iterm2,
            ImageProtocolType::Kitty => protocol::ImageProtocol::Kitty,
        }
    }
}

#[derive(Debug, Clone, ValueEnum)]
enum CommitOrderType {
    Chrono,
    Topo,
}

impl From<CommitOrderType> for git::SortCommit {
    fn from(order: CommitOrderType) -> Self {
        match order {
            CommitOrderType::Chrono => git::SortCommit::Chronological,
            CommitOrderType::Topo => git::SortCommit::Topological,
        }
    }
}

#[derive(Debug, Clone, ValueEnum)]
enum GraphWidthType {
    Double,
    Single,
}

impl From<GraphWidthType> for graph::CellWidthType {
    fn from(width: GraphWidthType) -> Self {
        match width {
            GraphWidthType::Double => graph::CellWidthType::Double,
            GraphWidthType::Single => graph::CellWidthType::Single,
        }
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    color_eyre::install().unwrap();
    let args = Args::parse();
    let (ui_config, graph_config, key_bind_patch) = config::load();
    let key_bind = keybind::KeyBind::new(key_bind_patch);

    let color_theme = color::ColorTheme::default();
    let graph_color_set = color::GraphColorSet::new(&graph_config.color);
    let image_protocol = args.protocol.into();

    let repository = git::Repository::load(Path::new("."), args.order.into());

    let graph = graph::calc_graph(&repository);

    let cell_width_type =
        check::decide_cell_width_type(&graph, args.graph_width.map(|w| w.into()))?;

    let graph_image_manager = GraphImageManager::new(
        &graph,
        &graph_color_set,
        cell_width_type,
        image_protocol,
        args.preload,
    );

    let mut terminal = ratatui::init();

    let (tx, rx) = event::init();

    let mut app = App::new(
        &repository,
        graph_image_manager,
        &graph,
        &key_bind,
        &ui_config,
        &color_theme,
        &graph_color_set,
        cell_width_type,
        image_protocol,
        tx,
    );
    let ret = app.run(&mut terminal, rx);

    ratatui::restore();
    ret.map_err(Into::into)
}
