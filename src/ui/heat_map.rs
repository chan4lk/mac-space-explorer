use iced::{
    widget::canvas::{self, Frame, Geometry},
    Color, Point, Rectangle, Size,
};

use crate::core::scanner::FileEntry;

pub struct HeatMap {
    pub entries: Vec<FileEntry>,
}

impl HeatMap {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl canvas::Program<crate::Message> for HeatMap {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        if !self.entries.is_empty() {
            let max_size = self.entries.iter().map(|e| e.size).max().unwrap_or(1) as f32;
            let width = bounds.width / self.entries.len() as f32;

            for (i, entry) in self.entries.iter().enumerate() {
                let height = (entry.size as f32 / max_size) * bounds.height;
                let x = i as f32 * width;
                let y = bounds.height - height;

                // Calculate color based on size (red for larger files)
                let intensity = entry.size as f32 / max_size;
                frame.fill_rectangle(
                    Point::new(x, y),
                    Size::new(width, height),
                    Color::from_rgb(intensity, 0.3, 0.3),
                );
            }
        }

        vec![frame.into_geometry()]
    }
}
