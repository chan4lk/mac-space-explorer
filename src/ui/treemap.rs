use iced::{
    widget::canvas::{self, Frame, Geometry, Path, Stroke, Event},
    Color, Point, Rectangle, Size, mouse,
};
use std::path::PathBuf;
use thousands::Separable;

use crate::core::scanner::FileEntry;

pub struct TreeMap {
    pub entries: Vec<FileEntry>,
    pub current_path: PathBuf,
    pub rects: Vec<ItemRect>,
}

#[derive(Debug, Clone)]
pub struct ItemRect {
    pub entry: FileEntry,
    pub bounds: Rectangle,
}

impl TreeMap {
    pub fn new(current_path: PathBuf) -> Self {
        Self {
            entries: Vec::new(),
            current_path,
            rects: Vec::new(),
        }
    }

    pub fn update_layout(&mut self, bounds: Rectangle) {
        if self.entries.is_empty() {
            return;
        }

        self.rects.clear();
        let total_size = self.entries.iter().map(|e| e.size).sum::<u64>() as f32;
        if total_size == 0.0 {
            return;
        }

        let mut remaining_area = bounds;
        let mut remaining_entries = self.entries.clone();
        remaining_entries.sort_by(|a, b| b.size.cmp(&a.size));

        while !remaining_entries.is_empty() && remaining_area.height > 0.0 && remaining_area.width > 0.0 {
            let remaining_size = remaining_entries.iter().map(|e| e.size).sum::<u64>() as f32;
            let (row, rest) = self.calculate_row(&remaining_entries, remaining_area, remaining_size);
            
            if !row.is_empty() {
                let row_size: u64 = row.iter().map(|e| e.size).sum();
                let row_height = ((row_size as f32 / total_size) * bounds.height).min(remaining_area.height);
                let mut x = remaining_area.x;
                
                for entry in row {
                    let width = ((entry.size as f32 / row_size as f32) * remaining_area.width).max(0.0);
                    if width > 0.0 {
                        self.rects.push(ItemRect {
                            entry,
                            bounds: Rectangle {
                                x,
                                y: remaining_area.y,
                                width,
                                height: row_height,
                            },
                        });
                        x += width;
                    }
                }
                
                remaining_area.y += row_height;
                remaining_area.height -= row_height;
            }
            
            remaining_entries = rest;
        }
    }

    fn calculate_row(&self, entries: &[FileEntry], bounds: Rectangle, total_size: f32) -> (Vec<FileEntry>, Vec<FileEntry>) {
        if entries.is_empty() {
            return (Vec::new(), Vec::new());
        }

        let mut row = Vec::new();
        let mut row_size = 0.0;
        let mut i = 0;

        while i < entries.len() {
            let size = entries[i].size as f32;
            let new_row_size = row_size + size;
            let aspect_ratio = bounds.width / (new_row_size / total_size * bounds.height);

            if !row.is_empty() && aspect_ratio < 1.0 {
                break;
            }

            row_size = new_row_size;
            row.push(entries[i].clone());
            i += 1;
        }

        (row, entries[i..].to_vec())
    }

    pub fn find_item_at(&self, position: Point) -> Option<&ItemRect> {
        self.rects.iter().find(|item| {
            item.bounds.contains(position)
        })
    }

    pub fn get_tooltip(&self, cursor: mouse::Cursor) -> Option<String> {
        if let Some(position) = cursor.position() {
            if let Some(item) = self.find_item_at(position) {
                let name = item.entry.path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                let size_text = format!("{} MB", 
                    (item.entry.size / 1024 / 1024).separate_with_commas()
                );
                let type_text = if item.entry.is_dir { "Directory" } else { "File" };
                let path_text = item.entry.path.to_string_lossy();
                
                return Some(format!(
                    "{}\nType: {}\nSize: {}\nPath: {}",
                    name, type_text, size_text, path_text
                ));
            }
        }
        None
    }
}

impl canvas::Program<crate::Message> for TreeMap {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let selected = crate::SELECTED_PATH.lock().unwrap().clone();

        for item in &self.rects {
            let rect = item.bounds;
            if rect.width <= 0.0 || rect.height <= 0.0 {
                continue;
            }

            // Calculate base color based on size and type
            let _intensity = ((item.entry.size as f32).log10() / 10.0).min(1.0).max(0.0);
            let base_color = if item.entry.is_dir {
                Color::from_rgb(0.2, 0.6, 0.6) // Teal for directories
            } else {
                Color::from_rgb(0.7, 0.2, 0.2) // Red for files
            };

            // Determine if this item is selected
            let is_selected = selected.as_ref().map_or(false, |p| p == &item.entry.path);

            // Apply selection effect
            let fill_color = if is_selected {
                Color::from_rgb(0.2, 0.4, 0.8) // Bright blue for selected items
            } else {
                base_color
            };

            // Draw rectangle
            frame.fill_rectangle(
                Point::new(rect.x, rect.y),
                Size::new(rect.width, rect.height),
                fill_color,
            );

            // Draw border (thicker for selected items)
            let stroke = if is_selected {
                Stroke {
                    width: 2.0,
                    style: canvas::Style::Solid(Color::WHITE),
                    line_cap: canvas::LineCap::Butt,
                    line_join: canvas::LineJoin::Miter,
                    line_dash: canvas::LineDash::default(),
                }
            } else {
                Stroke::default()
            };

            frame.stroke(&Path::rectangle(
                Point::new(rect.x, rect.y),
                Size::new(rect.width, rect.height),
            ), stroke);

            // Draw label if rectangle is big enough
            if rect.width > 60.0 && rect.height > 20.0 {
                let name = item.entry.path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                
                let size_text = format!("{} MB", 
                    (item.entry.size / 1024 / 1024).separate_with_commas()
                );

                let type_indicator = if item.entry.is_dir { "📁" } else { "📄" };
                let display_text = format!("{} {} ({})", type_indicator, name, size_text);

                frame.fill_text(canvas::Text {
                    content: display_text,
                    position: Point::new(rect.x + 5.0, rect.y + 15.0),
                    color: if is_selected { Color::WHITE } else { Color::from_rgb(0.9, 0.9, 0.9) },
                    size: 14.0,
                    ..Default::default()
                });
            }
        }

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if let Some(position) = cursor.position() {
            if bounds.contains(position) && self.find_item_at(position).is_some() {
                mouse::Interaction::Pointer
            } else {
                mouse::Interaction::default()
            }
        } else {
            mouse::Interaction::default()
        }
    }

    fn update(
        &self,
        _state: &mut Self::State,
        event: Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<crate::Message>) {
        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(position) = cursor.position() {
                    if bounds.contains(position) {
                        return (canvas::event::Status::Captured, None);
                    }
                }
                (canvas::event::Status::Ignored, None)
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(position) = cursor.position() {
                    if bounds.contains(position) {
                        if let Some(item) = self.find_item_at(position) {
                            println!("TreeMap: Selected item: {:?}", item.entry.path);
                            return (
                                canvas::event::Status::Captured,
                                Some(crate::Message::Select(Some(item.entry.path.clone())))
                            );
                        }
                    }
                }
                (canvas::event::Status::Ignored, None)
            }
            _ => (canvas::event::Status::Ignored, None),
        }
    }
}
