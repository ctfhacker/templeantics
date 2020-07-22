use crate::*;

/// A drawable or clickable rectangle
pub struct Button {
    /// X coordinate (in percentage of screen width)
    pub x: f32,

    /// Y coordinate (in percentage of screen height)
    pub y: f32,

    /// Width of the rectangle (in percentage of screen width)
    pub w: f32,

    /// Height of the rectangle (in percentage of screen height)
    pub h: f32,

    /// Optional text on the Button
    ///
    /// (Text string, color)
    pub text: Option<(String, Color)>,

    pub texture: Option<Texture2D>,
}

impl std::fmt::Debug for Button {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Button")
            .field("x", &self.x)
            .field("y", &self.y)
            .field("w", &self.w)
            .field("h", &self.h)
            .finish()
    }
}

impl Button {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Button { x , y, w, h, text: None, texture: None }
    }

    pub fn new_with_text(x: f32, y: f32, w: f32, h: f32, text: (String, Color)) -> Self {
        Button { x , y, w, h, text: Some(text), texture: None }
    }

    pub fn new_with_texture(x: f32, y: f32, w: f32, h: f32, texture: Texture2D) -> Self {
        Button { x , y, w, h, text: None, texture: Some(texture) }
    }
    
    /// Converts the percentage coordinates to actual pixels on the current screen
    pub fn to_screen(&self) -> (f32, f32, f32, f32) {
        // Convert percentages to actual coords
        let x = self.x * screen_width();
        let y = self.y * screen_height();
        let w = self.w * screen_width();
        let h = self.h * screen_height();
        (x, y, w, h)
    }

    /// Returns true if the given mouse position is in the Button
    pub fn contains(&self, mouse: (f32, f32)) -> bool {
        let (x, y) = mouse;
        let min_x = self.x * screen_width();
        let max_x = (self.x + self.w) * screen_width();
        let min_y = self.y * screen_height();
        let max_y = (self.y + self.h) * screen_height();
        min_x <= x && x <= max_x && min_y <= y && y <= max_y
    }

    /// Draws the button to the screen
    pub fn draw(&self) {
        // Convert percentages to actual coords
        let x = self.x * screen_width();
        let y = self.y * screen_height();
        let w = self.w * screen_width();
        let h = self.h * screen_height();

        draw_rectangle(x, y, w, h, BLACK);

        if let Some(tex) = self.texture {
            draw_texture_ex(tex, x, y, WHITE, DrawTextureParams {
                dest_size: Some(vec2(w, h)),
                ..Default::default()
            });
        }

        if let Some((text, color)) = &self.text {
            let mut best_size = 0.;
            for size in 1..100 {
                let (text_width, text_height) = measure_text(text, size as f32);
                if text_width < w * 0.9 && text_height < h * 0.9 {
                    best_size = size as f32;
                    continue;
                }
                break;
            }

            draw_text(&text, 
                    x,
                    y, // - (h / 4.),
                    best_size,
                    *color);
        }
    }

    /// Modifies the text of the current button
    pub fn change_text(&mut self, new_text: String) {
        let text = self.text.take();
        let val = match text {
            None => (new_text, BLACK),
            Some((_, color)) => (new_text, color)
        };

        self.text = Some(val);
    }

    /// Modifies the color of text of the current button
    pub fn change_text_color(&mut self, color: Color) {
        let text = self.text.take();
        let val = match text {
            None => ("NA".to_string(), color),
            Some((text, _)) => (text, color)
        };

        self.text = Some(val);
    }

    /// Modifies the texture of the current button
    pub fn change_texture(&mut self, texture: Texture2D) {
        self.texture = Some(texture);
    }
}
