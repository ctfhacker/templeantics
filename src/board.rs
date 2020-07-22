use crate::*;
use std::collections::HashMap;

/// Game board storing game state
pub struct Board {
    /// Current state of the game
    state: BoardState,

    /// Background image for this board
    texture: Texture2D,

    /// Clickable regions on the board
    clickables: Vec<Button>,

    /// Which square is currently selected
    selected: Option<usize>,

    /// Value of die 1
    die1: Option<usize>,

    /// Value of die 2
    die2: Option<usize>,

    /// Currently selected die
    selected_die: Option<usize>,

    /// Current die on Next Tile
    next_tile: Option<usize>,

    /// Current die on Encounter
    encounter: Option<usize>,

    /// Images of dice
    dice_textures: [Texture2D; 7],

    /// Current location of the player. Location is the ID of the clickable as shown in 
    /// print_debug()
    player_location: usize,

    /// Current turn
    current_turn: usize,

    /// Walls on the board
    walls: HashMap<usize, Vec<Wall>>,

    /// Current orientation of walls to place
    wall_orientation: usize,

    /// Current walls selected
    curr_walls: Vec<Wall>,

    /// Local rng
    rng: Rng,
}

/// Wall positions on the board
#[derive(Debug, Copy, Clone)]
enum Wall {
    Top,
    Right,
    Bottom,
    Left
}

/// The actions of a given turn
#[derive(Debug, Copy, Clone)]
enum BoardState {
    AssignDice,
    DrawWalls,
    Movement,
    TileEffect,
    Encounter
}

impl Board {
    pub fn new(texture: Texture2D, dice_textures: [Texture2D; 7]) -> Self {
        // Parse Clickables 
        let clickables_str = include_str!("../static/clickables.txt");
        let mut clickables = Vec::new();

        // Create clickables
        for line in clickables_str.split("\n") {
            if line.len() == 0 {
                continue;
            }

            let mut coords: Vec<f32> = line.split(" ").map(|x| x.parse().unwrap()).collect();
            let h = coords.pop().expect("No h");
            let w = coords.pop().expect("No w");
            let y = coords.pop().expect("No y");
            let x = coords.pop().expect("No x");
            clickables.push(Button::new(x, y, w, h));
        }

        // Re-write the clickables for the
        // Whole board
        let x = 0.155;
        let y = 0.155;
        let w = 0.6938 / 8.;
        let h = 0.4886 / 4.;

        let mut raw_walls = HashMap::new();

        let starting_id = 36;
        let mut wall_id = 0;

        for curr_y in 0..4 {
            for curr_x in 0..8 {
                let curr_id = 36 + curr_x + (8 * curr_y);

                let curr_x = x + (curr_x as f32 * w);
                let curr_y = y + (curr_y as f32 * h);
                let new_button = Button::new(curr_x, curr_y, w, h);
                clickables.push(new_button);

                let wall_translate = HashMap::new();
                let wall_width = 0.010;


                // Top wall
                let top = Button::new(curr_x, curr_y - wall_width / 2., w, wall_width);
                let key = format!("{:?}", top).to_string();
                let mut curr_wall = wall_id;
                if raw_walls.contains_key(&key) {
                    info!("Wall already exists! {:?}", top);
                    curr_wall = *raw_walls.get(&key).unwrap();
                } else {
                    raw_walls.insert(key.clone(), wall_id);
                    wall_id += 1;
                    info!("{:?}", &top);
                    clickables.push(top);
                }

                // Insert the mapping of board square ID to wall ID
                wall_translate.insert((curr_id, Wall::Top), curr_wall);


                // Bottom wall
                let bottom = Button::new(curr_x, curr_y - wall_width / 2. + h, w, wall_width);
                let key = format!("{:?}", bottom).to_string();
                let mut curr_wall = wall_id;
                if raw_walls.contains_key(&key) {
                    info!("Wall already exists! {:?}", bottom);
                    curr_wall = *raw_walls.get(&key).unwrap();
                } else {
                    raw_walls.insert(key.clone(), wall_id);
                    wall_id += 1;
                    info!("{:?}", &bottom);
                    clickables.push(bottom);
                }

                // Left wall
                let left = Button::new(curr_x - wall_width / 2., curr_y, wall_width, h);
                let key = format!("{:?}", left).to_string();
                let mut curr_wall = wall_id;
                if raw_walls.contains_key(&key) {
                    info!("Wall already exists! {:?}", left);
                    curr_wall = *raw_walls.get(&key).unwrap();
                } else {
                    raw_walls.insert(key.clone(), wall_id);
                    wall_id += 1;
                    info!("{:?}", &left);
                    clickables.push(left);
                }

                // Right wall
                let right = Button::new(curr_x + w - wall_width / 2., curr_y, wall_width, h);
                let key = format!("{:?}", right).to_string();
                let mut curr_wall = wall_id;
                if raw_walls.contains_key(&key) {
                    info!("Wall already exists! {:?}", right);
                    curr_wall = *raw_walls.get(&key).unwrap();
                } else {
                    raw_walls.insert(key.clone(), wall_id);
                    wall_id += 1;
                    info!("{:?}", &right);
                    clickables.push(right);
                }

                /*
                let key = format!("{:?}", wall).to_string();
                let mut curr_wall = wall_id;
                if raw_walls.contains_key(&key) {
                    info!("Wall already exists! {:?}", wall);
                    curr_wall = *raw_walls.get(&key).unwrap();
                } else {
                    raw_walls.insert(key.clone(), wall_id);
                    wall_id += 1;
                    info!("{:?}", &wall);
                    clickables.push(wall);
                }
                */

            }
        }



        // get_wall(36, Wall::Top)

        info!("clickables: {}", clickables.len());

        // Setup the Next action button
        clickables[ButtonId::TurnNextAction as usize].change_text("Next action".to_string());
        clickables[ButtonId::TurnNextAction as usize].change_text_color(RED);

        let mut rng = Rng::new();

        let die1 = Some(rng.roll_d6());
        let die2 = Some(rng.roll_d6());
        info!("rand die1 die2: {:?} {:?}", die1, die2);

        let mut die1_button = &mut clickables[ButtonId::Die1 as usize];
        die1_button.texture = Some(dice_textures[die1.unwrap()]);

        let mut die2_button = &mut clickables[ButtonId::Die2 as usize]; 
        die2_button.texture = Some(dice_textures[die2.unwrap()]);

        Board { 
            state: BoardState::AssignDice,
            texture, 
            clickables, 
            selected: None, 
            die1, 
            die2, 
            selected_die: None,
            next_tile: None, 
            encounter: None, 
            dice_textures,
            player_location: 39,
            current_turn: 1,
            walls: HashMap::new(),
            wall_orientation: 0,
            curr_walls: Vec::new(),
            rng
        }
    } 

    /// Set the current wall orientation at the current location
    pub fn insert_walls(&mut self) {
        /*
        let wall_index = match (self.player_location, self.curr_walls[0]) {
            (10, Wall::Top)   => 0,
            (11, Wall::Top)   => 1,
            (12, Wall::Top)   => 2,
            (13, Wall::Top)   => 3,
            (14, Wall::Top)   => 4,
            (15, Wall::Top)   => 5,
            (16, Wall::Top)   => 6,
            (17, Wall::Top)   => 7,
            (10, Wall::Left)  => 8,
            (10, Wall::Right) => 9
        };
        */
    }

    /// Sets the value and the texture of die 1
    pub fn set_die1(&mut self, val: Option<usize>) {
        if let Some(new_val) = val {
            let die1_button = &mut self.clickables[ButtonId::Die1 as usize];
            die1_button.change_texture(self.dice_textures[new_val]);
        }
        self.die1 = val;
    }

    /// Sets the value and the texture of die 2
    pub fn set_die2(&mut self, val: Option<usize>) {
        if let Some(new_val) = val {
            let die2_button = &mut self.clickables[ButtonId::Die2 as usize];
            die2_button.change_texture(self.dice_textures[new_val]);
        }
        self.die2 = val;
    }
}

#[derive(Debug, Copy, Clone)]
/// Messages that are triggered by mouse clicks
pub enum BoardMessage {
    NextState,
    Select(usize),
    ChooseDie1,
    ChooseDie2,
    ChooseNextTile,
    ChooseEncounter,
    ChangeWallOrientation
}

#[derive(Debug, Copy, Clone)]
#[repr(usize)]
pub enum ButtonId {
    TurnNextAction = 30,
    TurnAssignDice = 31,
    TurnDrawWalls = 32,
    TurnMovement = 33,
    TurnTileEffect = 34,
    TurnEncounter = 35,
    Die1 = 28,
    Die2 = 29,
    NextTile = 8,
    Encounter = 9,
}

impl Clickable for Board { 
    type Message = BoardMessage;

    fn handle_click(&mut self, pos: (f32, f32)) -> Option<Self::Message> {
        info!("Handle click: {:?}", pos);

        for (i, rect) in self.clickables.iter().enumerate() {
            if rect.contains(pos) {
                info!("Selecting: {}", i);
                self.selected = Some(i);
            }
        }

        // Check if we clicked Next Action
        info!("{:?} {:?}", pos, self.clickables[ButtonId::TurnNextAction as usize].to_screen());
        if self.clickables[ButtonId::TurnNextAction as usize].contains(pos) {
            info!("Sending NextState");
            return Some(BoardMessage::NextState);
        }

        // Assign Dice clickable checks
        if matches!(self.state, BoardState::AssignDice) {
            // For assign dice, the only available options are to click one of the 2 dice, 
            // and then click the next tile or encounter spaces
            let mut msg = None;

            if self.clickables[ButtonId::Die1 as usize].contains(pos) {
                msg = Some(BoardMessage::ChooseDie1);
            }

            if self.clickables[ButtonId::Die2 as usize].contains(pos) {
                msg = Some(BoardMessage::ChooseDie2);
            }

            if self.clickables[ButtonId::NextTile as usize].contains(pos) {
                msg = Some(BoardMessage::ChooseNextTile);
            }

            if self.clickables[ButtonId::Encounter as usize].contains(pos) {
                msg = Some(BoardMessage::ChooseEncounter);
            }

            if msg.is_some() {
                info!("{:?}: Sending msg: {:?}", self.state, msg);
                return msg;
            }
        }

        if matches!(self.state, BoardState::DrawWalls) {
            // For draw walls, the only available clickable location is the current location
            // to switch the orientation of the walls
            if self.clickables[self.player_location].contains(pos) {
                return Some(BoardMessage::ChangeWallOrientation);
            }
        }

        None
    }

    fn handle_message(&mut self, message: Self::Message) { 
        info!("Handling message: {:?}", message);
        match message {
            BoardMessage::NextState => {
                match self.state {
                    BoardState::AssignDice => {
                        if self.die1.is_none() && self.die2.is_none() {
                            info!("NextState -> AssignDice");
                            self.state = BoardState::DrawWalls;

                            // Rerolling 6 on Next Tile
                            loop {
                                if let Some(val) = self.next_tile {
                                    if val != 6 { 
                                        // Ensure we have the right texture in the button window
                                        let next_tile_button = &mut self.clickables[
                                            ButtonId::NextTile as usize];

                                        // Set the enounter button to the selected die
                                        next_tile_button.change_texture(
                                            self.dice_textures[self.next_tile.unwrap()]);
                                        
                                        break; 
                                    }
                                    self.next_tile = Some(self.rng.roll_d6());
                                }
                            }


                            // Init the walls
                            self.wall_orientation = 0;
                            self.walls.clear();
                            match self.next_tile {
                                Some(1) => [Wall::Left, Wall::Top, Wall::Right].iter()
                                    .for_each(|x| self.curr_walls.push(*x)),
                                Some(2) => [Wall::Left, Wall::Bottom].iter()
                                    .for_each(|x| self.curr_walls.push(*x)),
                                Some(3) => [Wall::Left, Wall::Right].iter()
                                    .for_each(|x| self.curr_walls.push(*x)),
                                Some(4) => [Wall::Left].iter().for_each(|x| self.curr_walls.push(*x)),
                                Some(5) => {},
                                _ => unreachable!()
                            }
                        }
                    }
                    BoardState::DrawWalls  => {
                        info!("Setting walls: {:?}", self.curr_walls);
                        self.state = BoardState::Movement;
                        self.insert_walls();
                    }
                    BoardState::Movement   => self.state = BoardState::TileEffect,
                    BoardState::TileEffect => self.state = BoardState::Encounter,
                    BoardState::Encounter  => self.state = BoardState::AssignDice
                }
            }
            BoardMessage::Select(x)  => self.selected = Some(x),
            BoardMessage::ChooseDie1 => {
                self.selected_die = Some(1);
            }
            BoardMessage::ChooseDie2 => {
                self.selected_die = Some(2);
            }
            BoardMessage::ChooseNextTile => {
                match self.selected_die {
                    Some(which_die) => {
                        // Save the old next_tile in case we need to swap die
                        let old_next_tile = self.next_tile;

                        info!("Old next_tile: {:?}", old_next_tile);

                        // Get the next_tile button
                        let next_tile_button = &mut self.clickables[ButtonId::NextTile as usize];

                        // Get the face of the die corresponding to the selected die
                        let new_die_face = match which_die {
                            1 => self.die1.expect("No die1?"),
                            2 => self.die2.expect("No die2?"),
                            _ => unreachable!()
                        };

                        // Set the next_tile to the value in the selected die
                        self.next_tile = Some(new_die_face);

                        // Set the enounter button to the selected die
                        next_tile_button.change_texture(
                            self.dice_textures[new_die_face]);


                        // Reset selected die
                        self.selected_die = None;

                        // If the next_tile was selected already before, swap the dice. Otherwise,
                        // clear the selected die
                        match old_next_tile {
                            None => {
                                // Reset die
                                match which_die {
                                    1 => self.set_die1(None),
                                    2 => self.set_die2(None),
                                    _ => unreachable!()
                                }
                            }
                            Some(old_die) => {
                                info!("Setting die {} to {}", which_die, old_die);

                                // Swap die
                                match which_die {
                                    1 => self.set_die1(Some(old_die)),
                                    2 => self.set_die2(Some(old_die)),
                                    _ => unreachable!()
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            BoardMessage::ChooseEncounter => {
                match self.selected_die {
                    Some(which_die) => {
                        // Save the old encounter in case we need to swap die
                        let old_encounter = self.encounter;

                        info!("Old encounter: {:?}", old_encounter);

                        // Get the encounter button
                        let encounter_button = &mut self.clickables[ButtonId::Encounter as usize];

                        // Get the face of the die corresponding to the selected die
                        let new_die_face = match which_die {
                            1 => self.die1.expect("No die1?"),
                            2 => self.die2.expect("No die2?"),
                            _ => unreachable!()
                        };

                        // Set the encounter to the value in the selected die
                        self.encounter = Some(new_die_face);

                        // Set the enounter button to the selected die
                        encounter_button.change_texture(
                            self.dice_textures[new_die_face]);


                        // Reset selected die
                        self.selected_die = None;

                        // If the encounter was selected already before, swap the dice. Otherwise,
                        // clear the selected die
                        match old_encounter {
                            None => {
                                // Reset die
                                match which_die {
                                    1 => self.set_die1(None),
                                    2 => self.set_die2(None),
                                    _ => unreachable!()
                                }
                            }
                            Some(old_die) => {
                                info!("Setting die {} to {}", which_die, old_die);

                                // Swap die
                                match which_die {
                                    1 => self.set_die1(Some(old_die)),
                                    2 => self.set_die2(Some(old_die)),
                                    _ => unreachable!()
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            BoardMessage::ChangeWallOrientation => {
                // Rotate the wall orientation by one
                self.wall_orientation = (self.wall_orientation + 1) % 4;

                // Clear the current walls to avoid another allocation
                self.curr_walls.clear();

                // Set the current walls based on the wall orientation
                match (self.next_tile, self.wall_orientation) {
                    (Some(1), 0) => [Wall::Left, Wall::Top, Wall::Right]
                        .iter().for_each(|x| self.curr_walls.push(*x)),
                    (Some(1), 1) => [Wall::Top, Wall::Right, Wall::Bottom]
                        .iter().for_each(|x| self.curr_walls.push(*x)),
                    (Some(1), 2) => [Wall::Right, Wall::Bottom, Wall::Left]
                        .iter().for_each(|x| self.curr_walls.push(*x)),
                    (Some(1), 3) => [Wall::Bottom, Wall::Left, Wall::Top]
                        .iter().for_each(|x| self.curr_walls.push(*x)),
                    (Some(2), 0) => [Wall::Left, Wall::Bottom]
                        .iter().for_each(|x| self.curr_walls.push(*x)),
                    (Some(2), 1) => [Wall::Top, Wall::Left]
                        .iter().for_each(|x| self.curr_walls.push(*x)),
                    (Some(2), 2) => [Wall::Right, Wall::Top]
                        .iter().for_each(|x| self.curr_walls.push(*x)),
                    (Some(2), 3) => [Wall::Bottom, Wall::Right]
                        .iter().for_each(|x| self.curr_walls.push(*x)),
                    (Some(3), 0|2) => [Wall::Left, Wall::Right]
                        .iter().for_each(|x| self.curr_walls.push(*x)),
                    (Some(3), 1|3) => [Wall::Top, Wall::Bottom]
                        .iter().for_each(|x| self.curr_walls.push(*x)),
                    (Some(4), 0) => [Wall::Left].iter().for_each(|x| self.curr_walls.push(*x)),
                    (Some(4), 1) => [Wall::Top].iter().for_each(|x| self.curr_walls.push(*x)),
                    (Some(4), 2) => [Wall::Right].iter().for_each(|x| self.curr_walls.push(*x)),
                    (Some(4), 3) => [Wall::Bottom].iter().for_each(|x| self.curr_walls.push(*x)),
                    _ => {}
                }
            }
        }

        // State machine of the turn order
        match self.state {
            BoardState::AssignDice => {
                if self.die1.is_none() && self.die2.is_none() {
                }
            }
            _ => {}
        }
    }
}

impl Drawable for Board {
    fn texture(&self) -> Texture2D { self.texture }
    fn clickables(&self) -> Option<&Vec<Button>> {
        Some(&self.clickables)
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

        // Draw the NEXT button
        let (x, y, w, h) = self.clickables[ButtonId::TurnNextAction as usize].to_screen();
        draw_rectangle(x, y, w, h, WHITE);
        self.clickables[ButtonId::TurnNextAction as usize].draw();

        // Get the current board state button ID
        let state_button_id = match self.state {
            BoardState::AssignDice => 63,
            BoardState::DrawWalls => 64,
            BoardState::Movement => 65,
            BoardState::TileEffect => 66,
            BoardState::Encounter => 67,
        };

        // Draw a rectangle around the current state
        let (x, y, w, h) = self.clickables[state_button_id].to_screen();
        draw_rectangle_lines(x, y, w, h, 8., BLACK);

        let die1_button = &self.clickables[ButtonId::Die1 as usize];
        let die2_button = &self.clickables[ButtonId::Die2 as usize];

        if self.die1.is_some() {
            die1_button.draw();
        }

        if self.die2.is_some() {
            die2_button.draw();
        }

        if let Some(chosen_die) = self.selected_die {
            match chosen_die {
                1 => {
                    let (x, y, w, h) = die1_button.to_screen();
                    draw_rectangle_lines(x, y, w, h, 5., GREEN);
                }
                2 => {
                    let (x, y, w, h) = die2_button.to_screen();
                    draw_rectangle_lines(x, y, w, h, 5., GREEN);
                }
                _ => unreachable!()
            }
        }

       if let Some(next_tile) = self.next_tile {
            let mut next_tile_button = &self.clickables[ButtonId::NextTile as usize];
            next_tile_button.draw();
        }

        if let Some(encounter) = self.encounter {
            let mut encounter_button = &self.clickables[ButtonId::Encounter as usize];
            encounter_button.draw();
        }

        // Mark the turns
        for turn_box in 10..10 + self.current_turn {
            let (x, y, w, h) = self.clickables[turn_box].to_screen();
            draw_rectangle(x, y, w, h, RED);
        }

        // Display the current player location
        let (x, y, w, h) = self.clickables[self.player_location].to_screen();
        draw_rectangle(x, y, w, h, color_from_usize(139, 69, 19, 240));

        if matches!(self.state, BoardState::DrawWalls) {
            let (x, y, w, h) = self.clickables[self.player_location].to_screen();
            for wall in &self.curr_walls {
                match wall {
                    Wall::Top    => draw_line(x,     y,     x + w, y,     6., BLACK),
                    Wall::Right  => draw_line(x + w, y,     x + w, y + h, 6., BLACK),
                    Wall::Bottom => draw_line(x,     y + h, x + w, y + h, 6., BLACK),
                    Wall::Left   => draw_line(x,     y,     x,     y + h, 6., BLACK),
                }
            }
            
        }

        if let Some(select) = self.selected {
            info!("Drawing {}", select);
            let selected_rect = &self.clickables[select];
            selected_rect.draw();
        }
    }
}
