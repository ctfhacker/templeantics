use crate::*;

/// Provides an ability to draw dynamically with the screen size
pub trait Drawable {
    fn texture(&self) -> Texture2D;
    fn clickables(&self) -> Option<&Vec<Button>> {
        None 
    }

    fn draw(&self) {
        // Draw the current image
        draw_texture_ex(
            self.texture(),
            0.,
            0.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            }
        );
    }

    fn debug_draw(&self) {
        if let Some(clickables) = self.clickables() {
            for (i, rect) in clickables.iter().enumerate() {
                let (x, y, w, h) = rect.to_screen();

                // Draw debug rectangle
                draw_rectangle_lines(x, y, w, h, 2., RED);

                // Draw text
                draw_text(&format!("{}", i), x + 2., y + 2., 12., RED);
            }
        }
    }
}

/// Provides basic mouse event handler mechanism
pub trait Clickable {
    /// Type of messages that can be triggered by a click
    type Message;

    /// Return a Message corresponding to that click
    fn handle_click(&mut self, pos: (f32, f32)) -> Option<Self::Message> {
        None
    }

    /// Update the state of Self, based on a given message
    fn handle_message(&mut self, message: Self::Message) { }
}

