pub mod color;
pub mod git;
pub mod graph;
pub mod protocol;

mod app;
mod config;
mod event;
mod external;
mod keybind;
mod macros;
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

    /// Do not use graph image cache
    #[arg(long, default_value = "false")]
    no_cache: bool,
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

impl From<CommitOrderType> for graph::SortCommit {
    fn from(order: CommitOrderType) -> Self {
        match order {
            CommitOrderType::Chrono => graph::SortCommit::Chronological,
            CommitOrderType::Topo => graph::SortCommit::Topological,
        }
    }
}

pub fn run() -> std::io::Result<()> {
    color_eyre::install().unwrap();
    let args = Args::parse();
    let (ui_config, key_bind_patch) = config::load();
    let key_bind = keybind::KeyBind::new(key_bind_patch);

    let color_set = color::ColorSet::default();
    let image_protocol = args.protocol.into();

    let repository = git::Repository::load(Path::new("."), args.order.into());

    let graph = graph::calc_graph(&repository);

    let graph_image_options = graph::GraphImageOptions::new(color_set.clone(), args.no_cache);
    let graph_image = graph::build_graph_image(&graph, graph_image_options);

    let graph_image_options = graph::GraphImageOptions::new(color_set.clone(), args.no_cache);
    let graph_image_manager = GraphImageManager::new(&graph, graph_image_options, image_protocol);

    let mut terminal = ratatui::init();

    let (tx, rx) = event::init();

    let mut app = App::new(
        &repository,
        graph_image_manager,
        &graph,
        &graph_image,
        &key_bind,
        &ui_config,
        &color_set,
        image_protocol,
        tx,
    );
    app.run(&mut terminal, rx)?;

    ratatui::restore();
    Ok(())
}
