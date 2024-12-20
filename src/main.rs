mod core;
mod ui;

use core::scanner::{FileEntry, ScanProgress, scan_directory};
use iced::{
    widget::{button, canvas::Canvas, column, container, progress_bar, row, text, text_input},
    Application, Command, Element, Length, Settings, Subscription, Theme, time,
};
use humansize::{format_size, BINARY};
use std::{path::PathBuf, time::Duration};
use ui::heat_map::HeatMap;

pub struct SpaceExplorer {
    path_input: String,
    root_path: PathBuf,
    heat_map: HeatMap,
    total_size: u64,
    filter_age: Option<u64>,
    filter_size: Option<u64>,
    selected_path: Option<PathBuf>,
    scan_progress: Option<ScanProgress>,
    scanning: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    PathInputChanged(String),
    Scan,
    Select(PathBuf),
    OpenInFinder,
    Delete,
    FilterByAge(u64),
    FilterBySize(u64),
    Refresh,
    Tick,
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
                path_input: home.to_string_lossy().into_owned(),
                root_path: home,
                heat_map: HeatMap::new(),
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
        String::from("Mac Space Explorer")
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
            Message::PathInputChanged(path) => {
                self.path_input = path;
            }
            Message::Scan => {
                self.root_path = PathBuf::from(&self.path_input);
                if self.root_path.exists() {
                    self.scanning = true;
                    self.scan_progress = Some(ScanProgress::default());
                    let mut progress = ScanProgress::default();
                    let entries = scan_directory(&self.root_path, &mut progress);
                    self.heat_map.entries = entries;
                    self.total_size = progress.total_size;
                    self.scanning = false;
                }
            }
            Message::Select(path) => {
                self.selected_path = Some(path);
            }
            Message::OpenInFinder => {
                if let Some(path) = &self.selected_path {
                    let _ = open::that(path);
                }
            }
            Message::Delete => {
                if let Some(path) = &self.selected_path {
                    if path.is_dir() {
                        let _ = std::fs::remove_dir_all(path);
                    } else {
                        let _ = std::fs::remove_file(path);
                    }
                    self.selected_path = None;
                }
            }
            Message::FilterByAge(days) => {
                self.filter_age = Some(days);
            }
            Message::FilterBySize(size) => {
                self.filter_size = Some(size);
            }
            Message::Refresh => {
                // Implement refresh logic
            }
            Message::Tick => {
                // Update progress if needed
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let path_input = text_input("Enter path to scan", &self.path_input)
            .on_input(Message::PathInputChanged)
            .padding(10);

        let controls = row![
            path_input,
            button("Scan").on_press(Message::Scan),
            button(if self.selected_path.is_some() { "Open in Finder" } else { "Select to Open" })
                .on_press(Message::OpenInFinder),
            button(if self.selected_path.is_some() { "Delete" } else { "Select to Delete" })
                .on_press(Message::Delete),
        ]
        .spacing(10);

        let progress = if let Some(progress) = &self.scan_progress {
            let progress_value = if progress.total_files > 0 {
                progress.scanned_files as f32 / progress.total_files as f32
            } else {
                0.0
            };

            column![
                progress_bar(0.0..=1.0, progress_value),
                text(format!(
                    "Scanning: {}/{} files",
                    progress.scanned_files, progress.total_files
                )),
            ]
            .spacing(10)
        } else {
            column![].spacing(10)
        };

        let heat_map = Canvas::new(&self.heat_map)
            .width(Length::Fill)
            .height(Length::Fill);

        let content = column![
            controls,
            progress,
            text(format!(
                "Total Size: {}",
                format_size(self.total_size, BINARY)
            )),
            heat_map,
        ]
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
