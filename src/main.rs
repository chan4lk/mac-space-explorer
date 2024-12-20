mod core;
mod ui;

use core::scanner::{FileEntry, ScanProgress, scan_directory};
use iced::{
    widget::{button, canvas, container, Column, progress_bar, text},
    Application, Command, Element, Length, Rectangle, Settings, Theme, Subscription,
    event, mouse, time,
};
use iced::widget::canvas::Event;
use humansize::{format_size, BINARY};
use native_dialog::{FileDialog, MessageDialog, MessageType};
use std::{path::PathBuf, time::Duration};
use ui::treemap::TreeMap;

pub struct SpaceExplorer {
    root_path: PathBuf,
    treemap: TreeMap,
    total_size: u64,
    filter_age: Option<u64>,
    filter_size: Option<u64>,
    selected_path: Option<PathBuf>,
    scan_progress: Option<ScanProgress>,
    scanning: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    SelectFolder,
    FolderSelected(Option<PathBuf>),
    Scan,
    ScanProgress(ScanProgress),
    ScanComplete(u64),
    Select(Option<PathBuf>),
    DrillDown,
    DrillUp,
    OpenInFinder,
    Delete,
    DeleteConfirmed,
    Tick,
    CanvasEvent(Event),
}

impl Application for SpaceExplorer {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        (
            SpaceExplorer {
                root_path: home.clone(),
                treemap: TreeMap::new(home),
                total_size: 0,
                filter_age: None,
                filter_size: None,
                selected_path: None,
                scan_progress: None,
                scanning: false,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        format!("Mac Space Explorer - {}", self.root_path.display())
    }

    fn subscription(&self) -> Subscription<Message> {
        if self.scanning {
            time::every(Duration::from_millis(100)).map(|_| Message::Tick)
        } else {
            Subscription::none()
        }
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SelectFolder => {
                if let Ok(Some(path)) = FileDialog::new()
                    .set_location(&self.root_path)
                    .show_open_single_dir()
                {
                    self.root_path = path;
                    self.treemap = TreeMap::new(self.root_path.clone());
                    return Command::perform(async {}, |_| Message::Scan);
                }
                Command::none()
            }
            Message::FolderSelected(_) => Command::none(),
            Message::Scan => {
                if self.root_path.exists() {
                    self.scanning = true;
                    self.scan_progress = Some(ScanProgress::default());
                    let mut progress = ScanProgress::default();
                    let entries = scan_directory(&self.root_path, &mut progress);
                    self.treemap.entries = entries;
                    self.treemap.update_layout(Rectangle {
                        x: 0.0,
                        y: 0.0,
                        width: 800.0,  // Default width
                        height: 600.0, // Default height
                    });
                    self.total_size = progress.total_size;
                    self.scanning = false;
                }
                Command::none()
            }
            Message::ScanProgress(_) => Command::none(),
            Message::ScanComplete(_) => Command::none(),
            Message::Select(path) => {
                self.selected_path = path;
                Command::none()
            }
            Message::OpenInFinder => {
                if let Some(path) = &self.selected_path {
                    let _ = open::that(path);
                }
                Command::none()
            }
            Message::Delete => {
                if let Some(path) = &self.selected_path {
                    let confirm = MessageDialog::new()
                        .set_title("Confirm Delete")
                        .set_text(&format!("Are you sure you want to delete {}?", path.display()))
                        .set_type(MessageType::Warning)
                        .show_confirm()
                        .unwrap_or(false);

                    if confirm {
                        if path.is_dir() {
                            let _ = std::fs::remove_dir_all(path);
                        } else {
                            let _ = std::fs::remove_file(path);
                        }
                        self.selected_path = None;
                        return Command::perform(async {}, |_| Message::Scan);
                    }
                }
                Command::none()
            }
            Message::DeleteConfirmed => Command::none(),
            Message::DrillDown => {
                if let Some(path) = &self.selected_path {
                    if path.is_dir() {
                        self.root_path = path.clone();
                        self.treemap = TreeMap::new(self.root_path.clone());
                        return Command::perform(async {}, |_| Message::Scan);
                    }
                }
                Command::none()
            }
            Message::DrillUp => {
                if let Some(parent) = self.root_path.parent() {
                    self.root_path = parent.to_path_buf();
                    self.treemap = TreeMap::new(self.root_path.clone());
                    return Command::perform(async {}, |_| Message::Scan);
                }
                Command::none()
            }
            Message::Tick => Command::none(),
            Message::CanvasEvent(_) => Command::none(),
        }
    }

    fn view(&self) -> Element<Message> {
        let controls = Column::new()
            .push(button("Select Folder").on_press(Message::SelectFolder))
            .push(button("Scan").on_press(Message::Scan))
            .push(button("↑ Up").on_press(Message::DrillUp))
            .push(button(if self.selected_path.is_some() { "Open in Finder" } else { "Select to Open" })
                .on_press(Message::OpenInFinder))
            .push(button(if self.selected_path.is_some() { "Delete" } else { "Select to Delete" })
                .on_press(Message::Delete))
            .push(button(if self.selected_path.is_some() { "Drill Down" } else { "Select to Drill" })
                .on_press(Message::DrillDown))
            .spacing(10);

        let progress = if let Some(progress) = &self.scan_progress {
            let progress_value = if progress.total_files > 0 {
                progress.scanned_files as f32 / progress.total_files as f32
            } else {
                0.0
            };

            Column::new()
                .push(progress_bar(0.0..=1.0, progress_value))
                .push(text(format!(
                    "Scanning: {}/{} files",
                    progress.scanned_files, progress.total_files
                )))
                .spacing(10)
        } else {
            Column::new().spacing(10)
        };

        let treemap = canvas::Canvas::new(&self.treemap)
            .width(Length::Fill)
            .height(Length::Fill);

        let content = Column::new()
            .push(controls)
            .push(progress)
            .push(text(format!(
                "Total Size: {}",
                format_size(self.total_size, BINARY)
            )))
            .push(treemap)
            .spacing(20);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .into()
    }
}

fn main() -> iced::Result {
    SpaceExplorer::run(Settings::default())
}
