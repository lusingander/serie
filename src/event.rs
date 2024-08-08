use std::{
    fmt::{self, Debug, Formatter},
    sync::mpsc,
    thread,
};

use ratatui::crossterm::event::KeyEvent;
use serde::Deserialize;
use strum::{EnumIter, EnumMessage};

pub enum AppEvent {
    Key(KeyEvent),
    Resize(usize, usize),
    Quit,
    OpenDetail,
    CloseDetail,
    ClearDetail,
    OpenRefs,
    CloseRefs,
    OpenHelp,
    CloseHelp,
    ClearHelp,
    CopyToClipboard { name: String, value: String },
    ClearStatusLine,
    UpdateStatusInput(String, Option<u16>),
    NotifyInfo(String),
    NotifySuccess(String),
    NotifyWarn(String),
    NotifyError(String),
}

#[derive(Clone)]
pub struct Sender {
    tx: mpsc::Sender<AppEvent>,
}

impl Sender {
    pub fn send(&self, event: AppEvent) {
        self.tx.send(event).unwrap();
    }
}

impl Debug for Sender {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Sender")
    }
}

pub struct Receiver {
    rx: mpsc::Receiver<AppEvent>,
}

impl Receiver {
    pub fn recv(&self) -> AppEvent {
        self.rx.recv().unwrap()
    }
}

pub fn init() -> (Sender, Receiver) {
    let (tx, rx) = mpsc::channel();
    let tx = Sender { tx };
    let rx = Receiver { rx };

    let event_tx = tx.clone();
    thread::spawn(move || loop {
        match ratatui::crossterm::event::read() {
            Ok(e) => match e {
                ratatui::crossterm::event::Event::Key(key) => {
                    event_tx.send(AppEvent::Key(key));
                }
                ratatui::crossterm::event::Event::Resize(w, h) => {
                    event_tx.send(AppEvent::Resize(w as usize, h as usize));
                }
                _ => {}
            },
            Err(e) => {
                panic!("Failed to read event: {}", e);
            }
        }
    });

    (tx, rx)
}

/// The event triggered by user's key input
#[derive(Clone, Debug, strum::Display, Deserialize, EnumIter, Eq, EnumMessage, Hash, PartialEq)]
pub enum UserEvent {
    // NOTE User Event should have document, else the enum item will be hidden in the help page
    /// Navigate up
    NavigateUp,
    /// Navigate down
    NavigateDown,
    /// Navigate right
    NavigateRight,
    /// Navigate left
    NavigateLeft,
    /// Force Quit serie without passing input into widges or views
    ForceQuit,
    /// Quit serie
    Quit,
    /// Close widget or cancel current progress
    CloseOrCancel,
    /// Toggle Help page
    HelpToggle,
    /// Go to top
    GoToTop,
    /// Go to bottom
    GoToBottom,
    /// Go to next item
    GoToNext,
    /// Go to previous item
    GoToPrevious,
    /// Scroll one line up
    ScrollUp,
    /// Scroll one line down
    ScrollDown,
    /// Scroll one page up
    PageUp,
    /// Scroll one page down
    PageDown,
    /// Scroll half page up
    HalfPageUp,
    /// Scroll half page down
    HalfPageDown,
    /// Select top part
    SelectTop,
    /// Select middle part
    SelectMiddle,
    /// Select bottom part
    SelectBottom,
    /// Show details
    ShowDetails,
    /// Search
    Search,
    /// Copy part of content
    ShortCopy,
    /// Copy
    FullCopy,
    /// Toggle for Reference List
    RefListToggle,
    Unknown,
}
