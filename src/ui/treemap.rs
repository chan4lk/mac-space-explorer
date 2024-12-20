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
            let bounds = item.bounds;
            position.x >= bounds.x && position.x <= bounds.x + bounds.width &&
            position.y >= bounds.y && position.y <= bounds.y + bounds.height
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

        // First draw all rectangles
        for item in &self.rects {
            let is_selected = selected.as_ref().map_or(false, |p| p == &item.entry.path);

            // Calculate base color based on size and type
            let intensity = ((item.entry.size as f32).log10() / 10.0).min(1.0).max(0.0);
            let base_color = if item.entry.is_dir {
                Color::from_rgb(0.2, 0.6, 0.6) // Teal for directories
            } else {
                Color::from_rgb(0.7, 0.2, 0.2) // Red for files
            };

            let color = if is_selected {
                Color::from_rgb(0.2, 0.4, 0.8) // Bright blue for selected
            } else {
                base_color
            };

            // Draw rectangle
            frame.fill_rectangle(
                Point::new(item.bounds.x, item.bounds.y),
                Size::new(item.bounds.width, item.bounds.height),
                color,
            );

            // Draw border using stroke
            let stroke = if is_selected {
                Stroke {
                    width: 2.0,
                    style: canvas::Style::Solid(Color::WHITE),
                    line_cap: canvas::LineCap::Butt,
                    line_join: canvas::LineJoin::Miter,
                    line_dash: canvas::LineDash::default(),
                }
            } else {
                Stroke {
                    width: 1.0,
                    style: canvas::Style::Solid(Color::from_rgb(0.3, 0.3, 0.3)),
                    line_cap: canvas::LineCap::Butt,
                    line_join: canvas::LineJoin::Miter,
                    line_dash: canvas::LineDash::default(),
                }
            };

            frame.stroke(
                &Path::rectangle(
                    Point::new(item.bounds.x, item.bounds.y),
                    Size::new(item.bounds.width, item.bounds.height),
                ),
                stroke,
            );
        }

        // Then draw tooltip if mouse is over any item
        if let Some(cursor_position) = cursor.position() {
            // Only show tooltip if cursor is within treemap bounds
            if bounds.contains(cursor_position) {
                // Convert cursor position to be relative to treemap bounds
                let relative_position = Point::new(
                    cursor_position.x - bounds.x,
                    cursor_position.y - bounds.y
                );

                if let Some(item) = self.find_item_at(relative_position) {
                    let name = item.entry.path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown");
                    let size_text = format!("{} MB", 
                        (item.entry.size / 1024 / 1024).separate_with_commas()
                    );
                    let type_text = if item.entry.is_dir { "Directory" } else { "File" };

                    // Draw tooltip background
                    let tooltip_text = format!("{}\n{}\n{}", name, type_text, size_text);
                    
                    let padding = 5.0;
                    let line_height = 16.0;
                    let tooltip_height = line_height * 3.0 + padding * 2.0;
                    let tooltip_width = 200.0;

                    let mut tooltip_x = cursor_position.x + 10.0;
                    let mut tooltip_y = cursor_position.y + 10.0;

                    // Adjust position to keep tooltip within bounds
                    if tooltip_x + tooltip_width > bounds.width + bounds.x {
                        tooltip_x = cursor_position.x - tooltip_width - 10.0;
                    }
                    if tooltip_y + tooltip_height > bounds.height + bounds.y {
                        tooltip_y = cursor_position.y - tooltip_height - 10.0;
                    }

                    // Draw tooltip background
                    frame.fill_rectangle(
                        Point::new(tooltip_x, tooltip_y),
                        Size::new(tooltip_width, tooltip_height),
                        Color::from_rgba(0.0, 0.0, 0.0, 0.8),
                    );

                    // Draw tooltip text
                    let lines = tooltip_text.lines();
                    for (i, line) in lines.enumerate() {
                        frame.fill_text(canvas::Text {
                            content: line.to_string(),
                            position: Point::new(
                                tooltip_x + padding,
                                tooltip_y + padding + line_height * i as f32
                            ),
                            color: Color::WHITE,
                            size: 14.0,
                            ..canvas::Text::default()
                        });
                    }
                }
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
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(cursor_position) = cursor.position() {
                    if bounds.contains(cursor_position) {
                        // Convert cursor position to be relative to treemap bounds
                        let relative_position = Point::new(
                            cursor_position.x - bounds.x,
                            cursor_position.y - bounds.y
                        );

                        if let Some(item) = self.find_item_at(relative_position) {
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
