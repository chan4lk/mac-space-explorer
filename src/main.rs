mod core;
mod ui;

use iced::{
    widget::{
        button, canvas, container, text,
        column, row,
    },
    Application, Command, Element, Length, Rectangle, Settings,
    Color, Theme, theme, Subscription, time,
};

use native_dialog::{FileDialog, MessageDialog, MessageType};
use thousands::Separable;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

use crate::core::scanner::{FileEntry, scan_directory, ScanProgress};
use crate::ui::treemap::TreeMap;

lazy_static::lazy_static! {
    pub static ref SELECTED_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);
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
    OpenInExplorer,
    Delete,
    DeleteConfirmed(PathBuf),
    Tick,
    CanvasEvent(canvas::Event),
}

pub struct SpaceExplorer {
    root_path: PathBuf,
    initial_root_path: PathBuf,
    treemap: TreeMap,
    total_size: u64,
    filter_age: Option<u64>,
    filter_size: Option<u64>,
    scan_progress: Option<ScanProgress>,
    scanning: bool,
    largest_files: Vec<FileEntry>,
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
                initial_root_path: home.clone(),
                treemap: TreeMap::new(home),
                total_size: 0,
                filter_age: None,
                filter_size: None,
                scan_progress: None,
                scanning: false,
                largest_files: Vec::new(),
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
                    self.root_path = path.clone();
                    self.initial_root_path = path;
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
                    
                    // Find largest files (only regular files, not directories)
                    let mut all_files: Vec<_> = entries.iter()
                        .filter(|e| !e.is_dir)
                        .cloned()
                        .collect();
                    all_files.sort_by(|a, b| b.size.cmp(&a.size));
                    self.largest_files = all_files.into_iter().take(10).collect();
                    println!("Found {} largest files in {}", self.largest_files.len(), self.root_path.display());
                    
                    for (i, file) in self.largest_files.iter().enumerate() {
                        println!("{}. {} ({} MB)", 
                            i + 1,
                            file.path.display(),
                            file.size / 1024 / 1024
                        );
                    }
                    
                    self.treemap = TreeMap::new(self.root_path.clone());
                    self.treemap.entries = entries;
                    self.treemap.update_layout(Rectangle {
                        x: 0.0,
                        y: 0.0,
                        width: 1000.0,
                        height: 800.0,
                    });
                    self.total_size = progress.total_size;
                    self.scanning = false;
                }
                Command::none()
            }
            Message::ScanProgress(_) => Command::none(),
            Message::ScanComplete(_) => Command::none(),
            Message::Select(path) => {
                println!("Select message received with path: {:?}", path);
                *SELECTED_PATH.lock().unwrap() = path;
                Command::none()
            }
            Message::DrillDown => {
                let path_to_drill = SELECTED_PATH.lock()
                    .unwrap()
                    .clone()
                    .filter(|p| p.is_dir());

                if let Some(path) = path_to_drill {
                    println!("Drilling down to: {:?}", path);
                    self.root_path = path.clone();
                    self.treemap = TreeMap::new(self.root_path.clone());
                    return Command::perform(async {}, |_| Message::Scan);
                }
                Command::none()
            }
            Message::DrillUp => {
                // Release any existing selection
                *SELECTED_PATH.lock().unwrap() = None;
                
                if let Some(parent) = self.root_path.parent() {
                    // Only drill up if we're not at the initial root path
                    if self.root_path != self.initial_root_path {
                        println!("Drilling up to: {:?}", parent);
                        self.root_path = parent.to_path_buf();
                        self.treemap = TreeMap::new(self.root_path.clone());
                        return Command::perform(async {}, |_| Message::Scan);
                    }
                }
                Command::none()
            }
            Message::OpenInFinder => {
                // Get the path and release the lock immediately
                let path_to_open = SELECTED_PATH.lock()
                    .unwrap()
                    .clone();

                if let Some(path) = path_to_open {
                    let _ = open::that(path);
                }
                Command::none()
            }
            Message::OpenInExplorer => {
                self.open_in_explorer();
                Command::none()
            }
            Message::Delete => {
                // Get the path and release the lock immediately
                let path_to_delete = SELECTED_PATH.lock()
                    .unwrap()
                    .clone();

                if let Some(path) = path_to_delete {
                    if let Ok(true) = MessageDialog::new()
                        .set_title("Move to Trash")
                        .set_text(&format!("Are you sure you want to move {} to trash?", path.display()))
                        .set_type(MessageType::Warning)
                        .show_confirm()
                    {
                        if let Err(e) = trash::delete(&path) {
                            MessageDialog::new()
                                .set_title("Error")
                                .set_text(&format!("Failed to move to trash: {}", e))
                                .set_type(MessageType::Error)
                                .show_alert()
                                .unwrap_or(());
                        } else {
                            *SELECTED_PATH.lock().unwrap() = None;
                            return Command::perform(async {}, |_| Message::Scan);
                        }
                    }
                }
                Command::none()
            }
            Message::DeleteConfirmed(_) => Command::none(),
            Message::Tick => Command::none(),
            Message::CanvasEvent(_) => Command::none(),
        }
    }

    fn view(&self) -> Element<Message> {
        let title = text("Mac Space Explorer")
            .size(40)
            .style(Color::from_rgb(0.4, 0.4, 1.0));

        let path_text = text(format!("Path: {}", self.root_path.display()))
            .size(16)
            .style(Color::from_rgb(0.7, 0.7, 0.7));

        let total_size_text = text(format!(
            "Total Size: {} MB",
            (self.total_size / 1024 / 1024).separate_with_commas()
        ))
        .size(16)
        .style(Color::from_rgb(0.7, 0.7, 0.7));

        let button_row = {
            // Get selected path once to avoid multiple locks
            let selected = SELECTED_PATH.lock().unwrap().clone();
            
            row![
                button("Select Folder").on_press(Message::SelectFolder),
                button("Scan").on_press(Message::Scan),
                button("Drill Up").on_press(Message::DrillUp),
                if selected.as_ref().map_or(false, |p| p.is_dir()) {
                    button("Drill Down").on_press(Message::DrillDown)
                } else {
                    button("Drill Down").style(theme::Button::Secondary)
                },
                if selected.is_some() {
                    button("Open").on_press(Message::OpenInFinder)
                } else {
                    button("Open").style(theme::Button::Secondary)
                },
                if selected.is_some() {
                    button("Explore").on_press(Message::OpenInExplorer)
                } else {
                    button("Explore").style(theme::Button::Secondary)
                },
                if selected.is_some() {
                    button("Delete")
                        .style(theme::Button::Destructive)
                        .on_press(Message::Delete)
                } else {
                    button("Delete").style(theme::Button::Secondary)
                }
            ]
            .spacing(10)
            .padding(10)
        };

        let content: Element<Message> = if self.scanning {
            column![
                title,
                path_text,
                button_row,
                text("Scanning...").size(20),
            ]
            .spacing(20)
            .padding(20)
            .into()
        } else {
            let legend = row![
                text("📁 Folders").style(Color::from_rgb(0.2, 0.6, 0.6)),
                text("📄 Files").style(Color::from_rgb(0.7, 0.2, 0.2))
            ]
            .spacing(10);

            let treemap = canvas::Canvas::new(&self.treemap)
                .width(Length::Fill)
                .height(Length::Fill);

            // Create the largest files panel
            let largest_files_panel = {
                let mut files_list = self.largest_files.clone();
                files_list.sort_by(|a, b| b.size.cmp(&a.size));
                
                if !files_list.is_empty() {
                    let selected = SELECTED_PATH.lock().unwrap().clone();
                    let items: Element<_> = column(
                        files_list
                            .iter()
                            .enumerate()
                            .map(|(i, entry)| {
                                let is_selected = selected.as_ref().map_or(false, |p| p == &entry.path);
                                let name = entry.path.file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .into_owned();
                                
                                let row = row![
                                    text(format!("{}. ", i + 1))
                                        .size(14)
                                        .width(Length::Fixed(30.0)),
                                    text(&name)
                                        .size(14)
                                        .width(Length::Fill),
                                    text(format!(
                                        "{} MB",
                                        (entry.size / 1024 / 1024).separate_with_commas()
                                    ))
                                    .size(14)
                                    .width(Length::Fixed(100.0)),
                                ]
                                .spacing(5)
                                .width(Length::Fill)
                                .align_items(iced::Alignment::Center);

                                let container = container(row)
                                    .padding(5)
                                    .width(Length::Fill);

                                if is_selected {
                                    container
                                        .style(theme::Container::Custom(Box::new(SelectedStyle)))
                                        .into()
                                } else {
                                    let entry_path = entry.path.clone();
                                    button(container)
                                        .on_press(Message::Select(Some(entry_path)))
                                        .style(theme::Button::Text)
                                        .into()
                                }
                            })
                            .collect(),
                    )
                    .spacing(2)
                    .width(Length::Fill)
                    .into();

                    container(
                        column![
                            text("Largest Files").size(20),
                            items,
                        ]
                        .spacing(10)
                        .width(Length::Fill)
                    )
                    .width(Length::Fixed(400.0))
                    .padding(10)
                    .style(theme::Container::Box)
                } else {
                    container(
                        text("No files found")
                            .size(16)
                            .style(Color::from_rgb(0.7, 0.7, 0.7))
                    )
                    .width(Length::Fixed(400.0))
                    .padding(10)
                    .style(theme::Container::Box)
                }
            };

            row![
                column![
                    title,
                    path_text,
                    total_size_text,
                    button_row,
                    legend,
                    treemap,
                ]
                .spacing(20)
                .padding(20)
                .width(Length::Fill),
                largest_files_panel,
            ]
            .width(Length::Fill)
            .into()
        };

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl SpaceExplorer {
    fn open_in_explorer(&self) {
        if let Some(path) = SELECTED_PATH.lock().unwrap().as_ref() {
            let parent = if path.is_file() {
                path.parent().unwrap_or(path)
            } else {
                path
            };
            
            if let Err(e) = opener::open(parent) {
                MessageDialog::new()
                    .set_title("Error")
                    .set_text(&format!("Failed to open in explorer: {}", e))
                    .set_type(MessageType::Error)
                    .show_alert()
                    .unwrap_or(());
            }
        }
    }
}

struct SelectedStyle;

impl container::StyleSheet for SelectedStyle {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Color::from_rgb(0.4, 0.4, 1.0).into()),
            text_color: Some(Color::WHITE),
            ..Default::default()
        }
    }
}

pub fn main() -> iced::Result {
    SpaceExplorer::run(Settings::default())
}
