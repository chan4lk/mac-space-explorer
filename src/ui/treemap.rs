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
    rects: Vec<ItemRect>,
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

        while !remaining_entries.is_empty() {
            let remaining_size = remaining_entries.iter().map(|e| e.size).sum::<u64>() as f32;
            let (row, rest) = self.calculate_row(&remaining_entries, remaining_area, remaining_size);
            
            if !row.is_empty() {
                let row_size: u64 = row.iter().map(|e| e.size).sum();
                let row_height = (row_size as f32 / total_size) * bounds.height;
                let mut x = remaining_area.x;
                
                for entry in row {
                    let width = (entry.size as f32 / row_size as f32) * remaining_area.width;
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
                
                remaining_area.y += row_height;
                remaining_area.height -= row_height;
            }
            
            remaining_entries = rest;
        }
    }

    fn calculate_row(&self, entries: &[FileEntry], bounds: Rectangle, total_size: f32) 
        -> (Vec<FileEntry>, Vec<FileEntry>) {
        let mut row = Vec::new();
        let mut rest = Vec::new();
        let mut row_size = 0.0;
        let target_ratio = bounds.width / bounds.height;

        for entry in entries {
            let new_row_size = row_size + entry.size as f32;
            let width = (new_row_size / total_size) * bounds.width;
            let height = bounds.height * (new_row_size / total_size);
            let ratio = width / height;

            if !row.is_empty() && (ratio - target_ratio).abs() > 
               ((row_size / total_size * bounds.width) / (bounds.height * row_size / total_size) - target_ratio).abs() {
                rest.push(entry.clone());
            } else {
                row_size = new_row_size;
                row.push(entry.clone());
            }
        }

        (row, rest)
    }

    pub fn find_item_at(&self, point: Point) -> Option<&ItemRect> {
        self.rects.iter().find(|rect| rect.bounds.contains(point))
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
        cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        // Draw rectangles
        for item in &self.rects {
            let rect = item.bounds;
            
            // Create rectangle path
            let mut builder = canvas::path::Builder::new();
            builder.move_to(Point::new(rect.x, rect.y));
            builder.line_to(Point::new(rect.x + rect.width, rect.y));
            builder.line_to(Point::new(rect.x + rect.width, rect.y + rect.height));
            builder.line_to(Point::new(rect.x, rect.y + rect.height));
            builder.close();
            
            let path = builder.build();

            // Calculate color based on size and type
            let intensity = (item.entry.size as f32).log10() / 10.0;
            let color = if item.entry.is_dir {
                Color::from_rgb(0.3, intensity, intensity)
            } else {
                Color::from_rgb(intensity, 0.3, 0.3)
            };

            // Highlight if under cursor
            let is_hovered = if let Some(position) = cursor.position() {
                rect.contains(position)
            } else {
                false
            };

            frame.fill(&path, color);
            frame.stroke(
                &path,
                Stroke::default()
                    .with_color(if is_hovered {
                        Color::WHITE
                    } else {
                        Color::from_rgb(0.1, 0.1, 0.1)
                    })
                    .with_width(if is_hovered { 2.0 } else { 1.0 }),
            );

            // Draw label if rectangle is big enough
            if rect.width > 60.0 && rect.height > 20.0 {
                let name = item.entry.path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                
                let size_text = format!("{} MB", 
                    (item.entry.size / 1024 / 1024).separate_with_commas()
                );

                frame.fill_text(canvas::Text {
                    content: name.to_string(),
                    position: Point::new(rect.x + 5.0, rect.y + 15.0),
                    color: Color::WHITE,
                    size: 14.0,
                    ..Default::default()
                });

                frame.fill_text(canvas::Text {
                    content: size_text,
                    position: Point::new(rect.x + 5.0, rect.y + 30.0),
                    color: Color::WHITE,
                    size: 12.0,
                    ..Default::default()
                });
            }
        }

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        _state: &mut Self::State,
        event: Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<crate::Message>) {
        match event {
            Event::Mouse(mouse_event) => {
                if let mouse::Event::ButtonPressed(mouse::Button::Left) = mouse_event {
                    if let Some(position) = cursor.position_in(bounds) {
                        if let Some(item) = self.find_item_at(position) {
                            return (
                                canvas::event::Status::Captured,
                                Some(crate::Message::Select(Some(item.entry.path.clone())))
                            );
                        }
                    }
                }
                (canvas::event::Status::Ignored, None)
            }
            Event::Touch(_) | Event::Keyboard(_) => (canvas::event::Status::Ignored, None),
        }
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        _bounds: Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> mouse::Interaction {
        if let Some(position) = cursor.position() {
            if self.find_item_at(position).is_some() {
                return mouse::Interaction::Pointer;
            }
        }
        mouse::Interaction::default()
    }
}
