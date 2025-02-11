use std::{
    fmt::{self, Debug, Formatter},
    sync::mpsc,
    thread,
};

use ratatui::crossterm::event::KeyEvent;
use serde::Deserialize;

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

// The event triggered by user's key input
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserEvent {
    ForceQuit,
    Quit,
    HelpToggle,
    Cancel,
    Close,
    NavigateUp,
    NavigateDown,
    NavigateRight,
    NavigateLeft,
    GoToTop,
    GoToBottom,
    GoToParent,
    ScrollUp,
    ScrollDown,
    PageUp,
    PageDown,
    HalfPageUp,
    HalfPageDown,
    SelectTop,
    SelectMiddle,
    SelectBottom,
    GoToNext,
    GoToPrevious,
    Confirm,
    RefListToggle,
    Search,
    ShortCopy,
    FullCopy,
    Unknown,
}
