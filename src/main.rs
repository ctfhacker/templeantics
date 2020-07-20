use macroquad::*;

mod button;
use button::Button;

mod traits;
use traits::*;

mod board;
use board::Board;


struct Rules {
    texture: Texture2D
}

impl Rules {
    pub fn new(texture: Texture2D) -> Self {
        Rules { texture }
    }
}

impl Drawable for Rules {
    fn texture(&self) -> Texture2D { self.texture }
}

enum State {
    Rules,
    Board
}

#[macroquad::main("TempleAntics")]
async fn main() {
    // Current state state
    let mut state = State::Board;

    // Various board states
    let board_tex: Texture2D = load_texture("./static/board.png").await;
    let dice_textures = [
        load_texture("./static/die_1.png").await, // 0th die texture does not matter
        load_texture("./static/die_1.png").await,
        load_texture("./static/die_2.png").await,
        load_texture("./static/die_3.png").await,
        load_texture("./static/die_4.png").await,
        load_texture("./static/die_5.png").await,
        load_texture("./static/die_6.png").await,
    ];

    let mut board = Board::new(board_tex, dice_textures);

    let rules_tex: Texture2D = load_texture("./static/rules.png").await;
    let rules = Rules::new(rules_tex);

    let mut rules_button = Button { x: 0.5, y: 0.95, w: 0.09, h: 0.04, 
        text: Some(("To Board".to_string(), RED)), texture: None };

    // 0.0312407 0.57166123 0.051741533 0.06194806
    // 0.089999534 0.57197994 0.055775665 0.06312579
    // 0.07401181 0.52855253 0.07110771 0.035393357

    let mut mouse_repeat = false;
    let mut mouse_click  = false;
    let mut last_click   = (0., 0.);

    loop {
        /* Naive mouse click event */
        mouse_click = false;
        if is_mouse_button_down(MouseButton::Left) && !mouse_repeat {
            info!("{:?}", mouse_position());
            mouse_repeat = true;
        }
        if !is_mouse_button_down(MouseButton::Left) && mouse_repeat {
            last_click = mouse_position();
            mouse_click = true;
            mouse_repeat = false;
        }

        if mouse_click {
            if rules_button.contains(last_click) {
                info!("Click rules button");
                state = match state {
                    State::Rules => State::Board,
                    State::Board => State::Rules,
                };

            }

            if matches!(state, State::Board) {
                if let Some(message) = board.handle_click(last_click) {
                    board.handle_message(message);
                }
            }
        }

        // Draw the current state
        match state {
            State::Board => {
                board.draw();
                board.debug_draw();
                rules_button.change_text("To Rules".to_string());
            }
            State::Rules => {
                rules.draw();
                rules.debug_draw();
                rules_button.change_text("To Board".to_string());
            }
        }

        draw_circle(last_click.0, last_click.1, 10., GREEN);

        // Draw the switch board/rules button
        rules_button.draw();

        next_frame().await
    }
}
