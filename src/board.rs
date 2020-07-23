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

    /// Current health of the player
    health: usize,

    /// Value of die 1
    die1: Option<usize>,

    /// Value of die 2
    die2: Option<usize>,

    /// Currently selected die
    selected_die: Option<usize>,

    /// Currently selected movement location and the resulting health if the location
    /// is chosen
    /// (Next location, next health, Wall to break)
    selected_move: Option<(usize, usize, Option<usize>)>,

    /// Currently selected teleport location
    selected_teleport: Option<usize>,

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

    /// Currently built walls on the board
    built_walls: Vec<usize>,

    /// All visited locations on the board
    visited_locations: Vec<usize>,

    /// Walls on the board. This is to dedup shared walls between neighboring spaces
    raw_walls: HashMap<String, usize>,

    /// Translation of board space and Wall to the clickable index. 
    walls_translation: HashMap<(usize, Wall), usize>,

    /// Current orientation of walls to place
    wall_orientation: usize,

    /// Current walls selected
    selected_walls: Vec<Wall>,

    /// Local rng
    rng: Rng,

    /// Does the player have the idol
    idol: bool,

    /// Does the player have the elixir
    elixir: bool,

    /// Does the player have the machete
    machete: bool,

    /// Number of uses for the charm
    charm: usize,

    /// Number of uses for the pickaxe
    pickaxe: usize,

    /// Number of uses for the shotgun
    shotgun: usize,

    /// Number of uses for the bandage
    bandage: usize,
}

/// Wall positions on the board
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Wall {
    Top,
    Right,
    Bottom,
    Left
}

/// The actions of a given turn
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum BoardState {
    AssignDice,
    DrawWalls,
    Movement,
    TileEffect,
    Encounter,
    ChooseTeleport,
    ShortcutDrawWalls,
    ShortcutMovement,
    ShortcutChooseTeleport,
    ShortcutTileEffect,
    EndTurn
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
        let mut walls_translation = HashMap::new();

        for curr_y in 0..4 {
            for curr_x in 0..8 {
                // Get the ID of the current button. Since they are ID'd based on 
                // their index in clickables, the current ID is the length of
                // the clickables
                let curr_id = clickables.len();

                // Get the coordinates for the current button.
                let curr_x = x + (curr_x as f32 * w);
                let curr_y = y + (curr_y as f32 * h);
                let new_button = Button::new(curr_x, curr_y, w, h);
                clickables.push(new_button);
            }
        }

        for curr_y in 0..4 {
            for curr_x in 0..8 {
                // Get the ID of the current button. 
                let curr_id = 36 + (curr_x + (8 * curr_y));

                let curr_x = x + (curr_x as f32 * w);
                let curr_y = y + (curr_y as f32 * h);

                // Top wall
                let top = Button::new(curr_x, curr_y - WALL_WIDTH / 2., w, 
                    WALL_WIDTH);
                let key = format!("{:?}", top).to_string();

                // Assume the wall has NOT been added to the `raw_walls`. If it 
                // had already been added, then the `curr_wall` ID is updated
                let mut curr_wall = clickables.len();
                if raw_walls.contains_key(&key) {
                    info!("Wall already exists! {:?}", top);
                    curr_wall = *raw_walls.get(&key).unwrap();
                } else {
                    raw_walls.insert(key.clone(), curr_wall);
                    info!("{:?}", &top);
                    clickables.push(top);
                }

                // Insert the mapping of board square ID to wall ID
                walls_translation.insert((curr_id, Wall::Top), curr_wall);

                // Bottom wall
                let bottom = Button::new(curr_x, curr_y - WALL_WIDTH / 2. + h, w, 
                    WALL_WIDTH);
                let key = format!("{:?}", bottom).to_string();

                // Assume the wall has NOT been added to the `raw_walls`. If it 
                // had already been added, then the `curr_wall` ID is updated
                let mut curr_wall = clickables.len();
                if raw_walls.contains_key(&key) {
                    info!("Wall already exists! {:?}", bottom);
                    curr_wall = *raw_walls.get(&key).unwrap();
                } else {
                    raw_walls.insert(key.clone(), curr_wall);
                    info!("{:?}", &bottom);
                    clickables.push(bottom);
                }

                // Insert the mapping of board square ID to wall ID
                walls_translation.insert((curr_id, Wall::Bottom), curr_wall);

                // Left wall
                let left = Button::new(curr_x - WALL_WIDTH / 2., curr_y, 
                    WALL_WIDTH, h);
                let key = format!("{:?}", left).to_string();

                // assume the wall has not been added to the `raw_walls`. if it 
                // had already been added, then the `curr_wall` id is updated
                let mut curr_wall = clickables.len();
                if raw_walls.contains_key(&key) {
                    info!("Wall already exists! {:?}", left);
                    curr_wall = *raw_walls.get(&key).unwrap();
                } else {
                    raw_walls.insert(key.clone(), curr_wall);
                    info!("{:?}", &left);
                    clickables.push(left);
                }

                // Insert the mapping of board square ID to wall ID
                walls_translation.insert((curr_id, Wall::Left), curr_wall);

                // Right wall
                let right = Button::new(curr_x + w - WALL_WIDTH / 2., curr_y, 
                    WALL_WIDTH, h);
                let key = format!("{:?}", right).to_string();

                // assume the wall has not been added to the `raw_walls`. if it 
                // had already been added, then the `curr_wall` id is updated
                let mut curr_wall = clickables.len();
                if raw_walls.contains_key(&key) {
                    info!("Wall already exists! {:?}", right);
                    curr_wall = *raw_walls.get(&key).unwrap();
                } else {
                    raw_walls.insert(key.clone(), curr_wall);
                    info!("{:?}", &right);
                    clickables.push(right);
                }

                // Insert the mapping of board square ID to wall ID
                walls_translation.insert((curr_id, Wall::Right), curr_wall);
            }
        }

        info!("60, right: {:?}", walls_translation.get(&(60, Wall::Right)));
        info!("61, left: {:?}", walls_translation.get(&(61, Wall::Left)));

        // info!("clickables: {}", clickables.len());

        // Setup the Next action button
        clickables[ButtonId::TurnNextAction as usize]
            .change_text("Next action".to_string());
        clickables[ButtonId::TurnNextAction as usize].change_text_color(RED);

        // Draw the items
        for &item in [ButtonId::Charm, ButtonId::Machete, 
                     ButtonId::Pickaxe, ButtonId::Shotgun, 
                     ButtonId::Bandage, ButtonId::Elixir, ButtonId::Idol].iter() {
            let mut button = &mut clickables[item as usize];
            button.change_background_color(Color::new(0., 0., 0., 0.,));
        }


        let mut rng = Rng::new();

        let die1 = Some(rng.roll_d6());
        let die2 = Some(rng.roll_d6());
        info!("rand die1 die2: {:?} {:?}", die1, die2);

        // Roll the initial die 1
        let mut die1_button = &mut clickables[ButtonId::Die1 as usize];
        die1_button.texture = Some(dice_textures[die1.unwrap()]);

        // Roll the initial die 2
        let mut die2_button = &mut clickables[ButtonId::Die2 as usize]; 
        die2_button.texture = Some(dice_textures[die2.unwrap()]);

        // Set the starting health
        let mut health_button = &mut clickables[ButtonId::Health as usize];
        health_button.texture = Some(dice_textures[6]);

        let starting_location = 60;

        Board { 
            state: BoardState::AssignDice,
            texture, 
            clickables, 
            selected: None, 
            health: 6,
            die1, 
            die2, 
            selected_die: None,
            selected_move: None,
            selected_teleport: None,
            next_tile: None, 
            encounter: None, 
            dice_textures,
            player_location: starting_location,
            current_turn: 1,
            wall_orientation: 0,
            selected_walls: Vec::new(),
            rng,
            built_walls: Vec::new(),
            visited_locations: vec![starting_location],
            raw_walls,
            walls_translation,
            idol: false,
            elixir: false,
            machete: false,
            charm: 0,
            pickaxe: 0,
            shotgun: 0,
            bandage: 0,
        }
    } 

    /// Set the current wall orientation at the current location
    pub fn insert_walls(&mut self) {
        for &wall in &self.selected_walls {
            info!("Setting wall: {:?} {:?}", self.player_location, wall);
            // Get the clickable ID for the selected walls
            let wall_key = (self.player_location, wall);
            if let Some(wall_index) = self.walls_translation.get(&wall_key) {
                // Insert the wall into the built walls list
                self.built_walls.push(*wall_index);
            } else {
                panic!("Did not find wall: {:?}", wall_key);
            }
        }
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

    /// Sets the value and the texture of die 2
    pub fn set_next_tile(&mut self, val: Option<usize>) {
        if let Some(new_val) = val {
            let button = &mut self.clickables[ButtonId::NextTile as usize];
            button.change_texture(self.dice_textures[new_val]);
        }
        self.next_tile = val;
    }

    /// Sets the value and the texture of health
    pub fn set_health(&mut self, val: usize) {
        assert!(val <= 6);
        let button = &mut self.clickables[ButtonId::Health as usize];
        if val == 0 {
            button.delete_texture();
        } else {
            button.change_texture(self.dice_textures[val]);
        }
        self.health = val;
    }

    /// Sets the value and the texture of charm
    pub fn set_charm(&mut self, val: usize) {
        assert!(val <= 2);
        let button = &mut self.clickables[ButtonId::Charm as usize];
        if val == 0 {
            button.delete_texture();
        } else {
            button.change_texture(self.dice_textures[val]);
        }
        self.charm = val;
    }

    /// Enable the machete
    pub fn find_machete(&mut self) {
        let button = &mut self.clickables[ButtonId::Machete as usize];
        button.change_texture(self.dice_textures[6]);
        self.machete = true;
    }

    /// Enable the idol
    pub fn find_idol(&mut self) {
        let button = &mut self.clickables[ButtonId::Idol as usize];
        button.change_texture(self.dice_textures[6]);
        self.idol = true;
    }

    /// Set the value and texture for pickaxe 
    pub fn set_pickaxe(&mut self, val: usize) {
        assert!(val <= 2);
        let button = &mut self.clickables[ButtonId::Pickaxe as usize];
        if val == 0 {
            button.delete_texture();
        } else {
            button.change_texture(self.dice_textures[val]);
        }
        self.pickaxe = val;
    }

    /// Set the value and texture for shotgun 
    pub fn set_shotgun(&mut self, val: usize) {
        assert!(val <= 2);
        let button = &mut self.clickables[ButtonId::Shotgun as usize];
        if val == 0 {
            button.delete_texture();
        } else {
            button.change_texture(self.dice_textures[val]);
        }
        self.shotgun = val;
    }

    /// Set the value and texture for shotgun 
    pub fn set_bandage(&mut self, val: usize) {
        assert!(val <= 2);
        let button = &mut self.clickables[ButtonId::Bandage as usize];
        if val == 0 {
            button.delete_texture();
        } else {
            button.change_texture(self.dice_textures[val]);
        }
        self.bandage = val;
    }

    /// Set the value and texture for elixir 
    pub fn find_elixir(&mut self) {
        let button = &mut self.clickables[ButtonId::Elixir as usize];
        button.change_texture(self.dice_textures[1]);
        self.elixir = true;
    }

    /// Set the value and texture for elixir 
    pub fn use_elixir(&mut self) {
        let button = &mut self.clickables[ButtonId::Elixir as usize];
        button.delete_texture();
        self.elixir = false;
        self.set_health(self.health + 4);
    }

    /// Returns the neighbors of the current location
    pub fn get_neighbors(&self) -> Vec<(Wall, usize)> {
        let loc = self.player_location;
        let mut neighbors = Vec::new();
        // Locations that don't have left neighbors
        if loc - 1 >= 36 && ![36, 44, 52, 60].contains(&loc) { 
            neighbors.push((Wall::Left, loc - 1)); 
        }

        // Locations that don't have right neighbors
        if loc + 1 <= 67 && ![43, 51, 59, 67].contains(&loc) {
            neighbors.push((Wall::Right, loc + 1)); 
        }

        // Check for top neighbors
        if loc - 8 >= 36 { 
            neighbors.push((Wall::Top, loc - 8)); 
        }

        // Check for top neighbors
        if loc + 8 <= 67 {
            neighbors.push((Wall::Bottom, loc + 8)); 
        }
        neighbors
    }

    /// Check if the given wall is built for the current player location
    pub fn is_wall_built(&self, wall: Wall) -> bool {
        let checked_wall = (self.player_location, wall);
        let curr_wall = self.walls_translation.get(&checked_wall)
            .expect(&format!("Wall not in translation: {:?}", checked_wall));

        self.built_walls.contains(&curr_wall)
    }
}

/// Messages that are triggered by mouse clicks
#[derive(Debug, Copy, Clone)]
pub enum BoardMessage {
    NextState,
    Select(usize),
    ChooseDie1,
    ChooseDie2,
    ChooseNextTile,
    ChooseEncounter,
    ChangeWallOrientation,
    MoveToLocation((Wall, usize)),
    ChooseTeleport(usize)
}

#[derive(Debug, Copy, Clone)]
#[repr(usize)]
pub enum ButtonId {
    Health = 0,
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
    Charm = 1,
    Machete = 2,
    Pickaxe = 3,
    Shotgun = 4,
    Bandage = 5,
    Elixir = 6,
    Idol = 7,
}

impl Clickable for Board { 
    type Message = BoardMessage;

    fn handle_click(&mut self, pos: (f32, f32)) -> Option<Self::Message> {
        info!("Handle click: {:?}", pos);

        /*
        for (i, rect) in self.clickables.iter().enumerate() {
            if rect.contains(pos) {
                self.selected = Some(i);
            }
        }
        */

        // Check if we clicked Next Action
        // info!("{:?} {:?}", pos, self.clickables[ButtonId::TurnNextAction as usize]
            // .to_screen());
        if self.clickables[ButtonId::TurnNextAction as usize].contains(pos) {
            info!("Sending NextState");
            return Some(BoardMessage::NextState);
        }

        // Assign Dice clickable checks
        if matches!(self.state, BoardState::AssignDice) {
            // For assign dice, the only available options are to click one of the 
            // 2 dice, and then click the next tile or encounter spaces
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

        if matches!(self.state, BoardState::DrawWalls) ||
                matches!(self.state, BoardState::ShortcutDrawWalls) {
            info!("[{:?}] Clicked on change wall orientation", self.state);
            // For draw walls, the only available clickable location is the current 
            // location to switch the orientation of the walls
            if self.clickables[self.player_location].contains(pos) {
                return Some(BoardMessage::ChangeWallOrientation);
            }
        }

        if matches!(self.state, BoardState::Movement) ||
                matches!(self.state, BoardState::ShortcutMovement) {
            // For movement, only accept a click in one of the neighbors
            for (wall, index) in self.get_neighbors() {
                if self.clickables[index].contains(pos) {
                    return Some(BoardMessage::MoveToLocation((wall, index)));
                }
            }
        }

        if matches!(self.state, BoardState::ChooseTeleport) {
            let tele_locations = [36, 43];

            for &index in &tele_locations {
                if self.clickables[index].contains(pos) {
                    return Some(BoardMessage::ChooseTeleport(index));
                }
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
                            self.clickables[ButtonId::TurnNextAction as usize]
                                .change_text("Draw walls".to_string());
                            self.state = BoardState::DrawWalls;

                            // Rerolling 6 on Next Tile
                            loop {
                                if let Some(val) = self.next_tile {
                                    if val != 6 { 
                                        // Found a non-6 for the next tile, success!
                                        self.set_next_tile(Some(val));
                                        break; 
                                    }


                                    let val = self.rng.roll_d6();
                                    // A 6 causes a reroll until not a 6 is found
                                    self.set_next_tile(Some(val));
                                }
                            }


                            // Init the walls
                            self.wall_orientation = 0;
                            match self.next_tile {
                                Some(1) => [Wall::Left, Wall::Top, Wall::Right]
                                    .iter()
                                    .for_each(|x| self.selected_walls.push(*x)),
                                Some(2) => [Wall::Left, Wall::Bottom].iter()
                                    .for_each(|x| self.selected_walls.push(*x)),
                                Some(3) => [Wall::Left, Wall::Right].iter()
                                    .for_each(|x| self.selected_walls.push(*x)),
                                Some(4) => [Wall::Left].iter()
                                    .for_each(|x| self.selected_walls.push(*x)),
                                Some(5) => {},
                                _ => unreachable!()
                            }
                        }
                    }
                    BoardState::DrawWalls => {
                        info!("DrawWalls -> Movement");
                        self.clickables[ButtonId::TurnNextAction as usize]
                            .change_text("Move player".to_string());
                        self.state = BoardState::Movement;
                        self.insert_walls();

                        // Clear the selected walls
                        self.selected_walls.clear();
                    }
                    BoardState::Movement => {
                        if let Some((next_location, next_health, wall)) = 
                                self.selected_move {
                            // Duplicates in this vec are fine. Won't be crazy large
                            self.visited_locations.push(next_location);

                            // Update player location
                            self.player_location = next_location;

                            // Check if the player dies if moved here
                            if next_health == 0 {
                                panic!("DIED!");
                            }

                            if let Some(broken_wall) = wall {
                                self.built_walls.remove_item(&broken_wall);
                            }


                            // Set the new health value and texture
                            self.set_health(next_health);

                            // Reset the selected move
                            self.selected_move = None;

                            // If we are on a teleport location, go to the Teleport 
                            // selection location
                            if [36, 43].contains(&self.player_location) {
                                info!("Movement -> ChooseTeleport");
                                self.clickables[ButtonId::TurnNextAction as usize]
                                    .change_text("Choose teleport".to_string());
                                self.state = BoardState::ChooseTeleport;
                            } else { 
                                // Go to the next state
                                info!("Movement -> TileEffect");
                                self.clickables[ButtonId::TurnNextAction as usize]
                                    .change_text("Check tile effect".to_string());
                                self.state = BoardState::TileEffect;
                            }
                        }
                    }
                    BoardState::ShortcutDrawWalls => {
                        info!("ShortcutDrawWalls -> ShortcutMovement");
                        self.clickables[ButtonId::TurnNextAction as usize]
                            .change_text("Shortcut move".to_string());
                        self.state = BoardState::ShortcutMovement;
                        self.insert_walls();

                        // Clear the selected walls
                        self.selected_walls.clear();
                    }
                    BoardState::ShortcutMovement => {
                        if let Some((next_location, next_health, wall)) 
                                = self.selected_move {
                            // Duplicates in this vec are fine. Won't be crazy large
                            self.visited_locations.push(next_location);

                            // Update player location
                            self.player_location = next_location;

                            // Check if the player dies if moved here
                            if next_health == 0 {
                                panic!("DIED!");
                            }

                            if let Some(broken_wall) = wall {
                                self.built_walls.remove_item(&broken_wall);
                            }

                            // Set the new health value and texture
                            self.set_health(next_health);

                            // Reset the selected move
                            self.selected_move = None;

                            // If we are on a teleport location, go to the Teleport 
                            // selection location
                            if [36, 43].contains(&self.player_location) {
                                info!("ShortcutMovement -> ChooseTeleport");
                                self.clickables[ButtonId::TurnNextAction as usize]
                                    .change_text("Choose shortcut tele".to_string());
                                self.state = BoardState::ShortcutChooseTeleport;
                            } else { 
                                // Go to the next state
                                info!("ShortcutMovement -> AssignDice");
                                self.clickables[ButtonId::TurnNextAction as usize]
                                    .change_text("End Turn".to_string());
                                self.state = BoardState::ShortcutTileEffect;
                            }
                        }
                    }
                    BoardState::TileEffect => {
                        match self.player_location {
                            40 => self.find_idol(),
                            45 => self.set_pickaxe(2),
                            57 => self.find_elixir(),
                            63 => self.find_machete(),
                            _ => {}
                        }

                        info!("TileEffect -> Encounter");
                        self.clickables[ButtonId::TurnNextAction as usize]
                            .change_text("Check encounter".to_string());
                        self.state = BoardState::Encounter;
                    }
                    BoardState::ShortcutTileEffect => {
                        match self.player_location {
                            40 => self.find_idol(),
                            45 => self.set_pickaxe(2),
                            57 => self.find_elixir(),
                            63 => self.find_machete(),
                            _ => {}
                        }

                        info!("ShortcutTileEffect -> EndTurn");
                        self.clickables[ButtonId::TurnNextAction as usize]
                            .change_text("End turn".to_string());
                        self.state = BoardState::EndTurn;
                    }
                    BoardState::ChooseTeleport|BoardState::ShortcutChooseTeleport => {
                        // This is reached if the player is on a teleport location
                        if let Some(tele_loc) = self.selected_teleport {
                            self.visited_locations.push(tele_loc);
                            self.player_location = tele_loc;
                        }

                        // Reset the selected teleport
                        self.selected_teleport = None;

                        if self.state == BoardState::ChooseTeleport {
                            // Move to the next state
                            info!("ChooseTeleport -> Encounter");
                            self.clickables[ButtonId::TurnNextAction as usize]
                                .change_text("Check encounter".to_string());
                            self.state = BoardState::Encounter;
                        } else {
                            // Taking a teleport due to a shortcut will lead 
                            // back to the assign dice
                            info!("ShortcutChooseTeleport -> AssignDice");
                            self.clickables[ButtonId::TurnNextAction as usize]
                                .change_text("End turn".to_string());
                            self.state = BoardState::EndTurn;
                        }
                    }
                    BoardState::Encounter => {
                        info!("Trying encoutner: {:?}", self.encounter);
                        match self.encounter {
                            Some(1) => {
                                // Sneak beast attack
                                let rand_roll = self.rng.roll_d6();
                                info!("Sneak beast roll: {}", rand_roll);

                                match (rand_roll, self.current_turn) {
                                   (1..=2,  1..=6)   => {
                                       self.set_health(self.health.saturating_sub(2));
                                   }
                                   (1..=2,  7..=12)  => {
                                       self.set_health(self.health.saturating_sub(3));
                                   }
                                   (1..=2, 13..=18)  => {
                                       self.set_health(self.health.saturating_sub(4));
                                   }
                                   (3..=4,  1..=6)   => {
                                       self.set_health(self.health.saturating_sub(3));
                                   }
                                   (3..=4,  7..=12)  => {
                                       self.set_health(self.health.saturating_sub(4));
                                   }
                                   (3..=4, 13..=18)  => {
                                       self.set_health(self.health.saturating_sub(5));
                                   }
                                   (5..=6,  1..=6)   => {
                                       self.set_health(self.health.saturating_sub(4));
                                   }
                                   (5..=6,  7..=12)  => {
                                       self.set_health(self.health.saturating_sub(5));
                                   }
                                   (5..=6, 13..=18)  => {
                                       self.set_health(self.health.saturating_sub(6));
                                   }
                                   (_, _) => unreachable!()
                                }
                            }
                            Some(2) => {
                                // Rest at a Campfire
                                // +1 health
                                info!("Rest: +1 health");
                                if self.health < 6 {
                                    self.set_health(self.health + 1);
                                }
                            }
                            Some(3) => {
                                // Beast attack
                                let rand_roll = self.rng.roll_d6();
                                info!("Beast attack roll: {}", rand_roll);

                                match (rand_roll, self.current_turn) {
                                   (1..=2,  1..=6) => {
                                       self.set_health(self.health.saturating_sub(1));
                                   }
                                   (1..=2,  7..=12) => {
                                       self.set_health(self.health.saturating_sub(2));
                                   }
                                   (1..=2, 13..=18) => {
                                       self.set_health(self.health.saturating_sub(3));
                                   }
                                   (3..=4,  1..=6)  => {
                                       self.set_health(self.health.saturating_sub(2));
                                   }
                                   (3..=4,  7..=12) => {
                                       self.set_health(self.health.saturating_sub(3));
                                   }
                                   (3..=4, 13..=18) => {
                                       self.set_health(self.health.saturating_sub(4));
                                   }
                                   (5..=6,  1..=6)  => {
                                       self.set_health(self.health.saturating_sub(3));
                                   }
                                   (5..=6,  7..=12) => {
                                       self.set_health(self.health.saturating_sub(4));
                                   }
                                   (5..=6, 13..=18) => {
                                       self.set_health(self.health.saturating_sub(5));
                                   }
                                   (_, _) => unreachable!()
                                }
                            }
                            Some(4) => {
                                let val = self.rng.roll_d6();
                                self.set_next_tile(Some(val));
                                while self.next_tile == Some(6) {
                                    let val = self.rng.roll_d6();
                                    self.set_next_tile(Some(val));
                                }

                                info!("Shortcut next tile: {:?}", self.next_tile);

                                // Bail early in order to draw a new tile
                                info!("Encounter -> ShortcutDrawWalls");
                                self.clickables[ButtonId::TurnNextAction as usize]
                                    .change_text("Draw shortcut walls".to_string());
                                self.state = BoardState::ShortcutDrawWalls;

                                // Init the walls
                                self.wall_orientation = 0;
                                match self.next_tile {
                                    Some(1) => [Wall::Left, Wall::Top, Wall::Right]
                                        .iter()
                                        .for_each(|x| self.selected_walls.push(*x)),
                                    Some(2) => [Wall::Left, Wall::Bottom].iter()
                                        .for_each(|x| self.selected_walls.push(*x)),
                                    Some(3) => [Wall::Left, Wall::Right].iter()
                                        .for_each(|x| self.selected_walls.push(*x)),
                                    Some(4) => [Wall::Left].iter()
                                        .for_each(|x| self.selected_walls.push(*x)),
                                    Some(5) => {},
                                    _ => unreachable!()
                                }
                            }
                            Some(5) => {
                                // We only pick up an item if we didn't pick up a 
                                // tile item this turn
                                if ![40, 45, 57, 63].contains(&self.player_location) {
                                    match self.rng.roll_d6() {
                                        1 => {
                                            info!("New item: Charm");
                                            self.set_charm(2);
                                        }
                                        2 => {
                                            info!("New item: Machete");
                                            self.find_machete();
                                        }
                                        3 => {
                                            info!("New item: Pickaxe");
                                            self.set_pickaxe(2);
                                        }
                                        4 => {
                                            info!("New item: Shotgun");
                                            self.set_shotgun(2);
                                        }
                                        5 => {
                                            info!("New item: Bandage");
                                            self.set_bandage(2);
                                        }
                                        6 => {
                                            info!("New item: Elixir");
                                            self.find_elixir();
                                        }
                                        _ => unreachable!()
                                    }
                                }
                            }
                            Some(6) => {
                                info!("Fall into a trap!");
                                match self.current_turn {
                                    ( 1..=6)  => {
                                        self.set_health(self.health.saturating_sub(1));
                                    }
                                    ( 7..=12) => {
                                        self.set_health(self.health.saturating_sub(2));
                                    }
                                    (13..=18) => {
                                        self.set_health(self.health.saturating_sub(3));
                                    }
                                    _ => unreachable!()
                                }
                            }
                            _ => unreachable!()
                        }

                        if self.health == 0 {
                            info!("GAME OVER");
                            panic!("GAME OVER");
                        }

                        if self.state != BoardState::ShortcutMovement &&
                            self.state != BoardState::ShortcutDrawWalls {
                            // Normal execution
                            // Jump back to the beginning of the turn
                            info!("Encounter -> AssignDice");
                            self.clickables[ButtonId::TurnNextAction as usize]
                                .change_text("End turn".to_string());
                            self.state = BoardState::EndTurn;
                        } 
                    }
                    BoardState::EndTurn => {
                        // Reroll the two dice
                        let val1 = self.rng.roll_d6();
                        self.set_die1(Some(val1));

                        let val2 = self.rng.roll_d6();
                        self.set_die2(Some(val2));

                        // Reset the board for the next round 
                        self.next_tile = None;
                        self.encounter = None;

                        // Increment the turn
                        self.current_turn += 1;

                        // Set the next state
                        self.clickables[ButtonId::TurnNextAction as usize]
                            .change_text("Assign dice".to_string());
                        self.state = BoardState::AssignDice;
                    }
                }
            }
            BoardMessage::Select(x)  => self.selected = Some(x),
            BoardMessage::ChooseDie1 => {
                info!("Selected die = 1");
                self.selected_die = Some(1);
            }
            BoardMessage::ChooseDie2 => {
                info!("Selected die = 1");
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
                self.selected_walls.clear();

                // Set the current walls based on the wall orientation
                match (self.next_tile, self.wall_orientation) {
                    (Some(1), 0) => [Wall::Left, Wall::Top, Wall::Right]
                        .iter().for_each(|x| self.selected_walls.push(*x)),
                    (Some(1), 1) => [Wall::Top, Wall::Right, Wall::Bottom]
                        .iter().for_each(|x| self.selected_walls.push(*x)),
                    (Some(1), 2) => [Wall::Right, Wall::Bottom, Wall::Left]
                        .iter().for_each(|x| self.selected_walls.push(*x)),
                    (Some(1), 3) => [Wall::Bottom, Wall::Left, Wall::Top]
                        .iter().for_each(|x| self.selected_walls.push(*x)),
                    (Some(2), 0) => [Wall::Left, Wall::Bottom]
                        .iter().for_each(|x| self.selected_walls.push(*x)),
                    (Some(2), 1) => [Wall::Top, Wall::Left]
                        .iter().for_each(|x| self.selected_walls.push(*x)),
                    (Some(2), 2) => [Wall::Right, Wall::Top]
                        .iter().for_each(|x| self.selected_walls.push(*x)),
                    (Some(2), 3) => [Wall::Bottom, Wall::Right]
                        .iter().for_each(|x| self.selected_walls.push(*x)),
                    (Some(3), 0|2) => [Wall::Left, Wall::Right]
                        .iter().for_each(|x| self.selected_walls.push(*x)),
                    (Some(3), 1|3) => [Wall::Top, Wall::Bottom]
                        .iter().for_each(|x| self.selected_walls.push(*x)),
                    (Some(4), 0) => [Wall::Left].iter()
                        .for_each(|x| self.selected_walls.push(*x)),
                    (Some(4), 1) => [Wall::Top].iter()
                        .for_each(|x| self.selected_walls.push(*x)),
                    (Some(4), 2) => [Wall::Right].iter()
                        .for_each(|x| self.selected_walls.push(*x)),
                    (Some(4), 3) => [Wall::Bottom].iter()
                        .for_each(|x| self.selected_walls.push(*x)),
                    _ => panic!("ChangeWallOrientation: {:?} {:?}", 
                        self.next_tile, self.wall_orientation), 
                }
            }
            BoardMessage::MoveToLocation((through_wall, next_index)) => {
                let mut health = self.health;
                let mut wall = None;
                if self.is_wall_built(through_wall) {
                    // Set the selected movement as well as the resulting health
                    // if this move is selected
                    let checked_wall = (self.player_location, through_wall);
                    let curr_wall = self.walls_translation.get(&checked_wall)
                        .expect(&format!("Wall not in translation: {:?}", 
                                         checked_wall));

                    health = self.health.saturating_sub(4);
                    wall = Some(*curr_wall);
                }

                self.selected_move = Some((next_index, health, wall));
                info!("Selecting move: {:?}", self.selected_move);
            }
            BoardMessage::ChooseTeleport(index) => {
                self.selected_teleport = Some(index);
                info!("Selecting teleport location: {:?}", self.selected_move);
            }
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

        // Draw the health die
        let health_button = &self.clickables[ButtonId::Health as usize];
        health_button.draw();



        // Draw the items
        for &item in [ButtonId::Charm, ButtonId::Machete, 
                     ButtonId::Pickaxe, ButtonId::Shotgun, 
                     ButtonId::Bandage, ButtonId::Elixir, ButtonId::Idol].iter() {
            let button = &self.clickables[item as usize];
            button.draw();
        }

        // Draw the NEXT button
        let (x, y, w, h) = self.clickables[ButtonId::TurnNextAction as usize].to_screen();
        draw_rectangle(x, y, w, h, WHITE);
        self.clickables[ButtonId::TurnNextAction as usize].draw();

        // Get the current board state button ID
        let state_button_id = match self.state {
            BoardState::AssignDice => 31,
            BoardState::DrawWalls => 32,
            BoardState::ShortcutDrawWalls => 32,
            BoardState::Movement => 33,
            BoardState::ShortcutMovement => 33,
            BoardState::TileEffect => 34,
            BoardState::ShortcutTileEffect => 34,
            BoardState::ChooseTeleport => 34,
            BoardState::ShortcutChooseTeleport => 34,
            BoardState::Encounter => 35,
            BoardState::EndTurn => 30,
        };

        // Draw a rectangle around the current state
        let (x, y, w, h) = self.clickables[state_button_id].to_screen();
        draw_rectangle_lines(x, y, w, h, 8., BLACK);


        // Draw the next_tile die
        if let Some(next_tile) = self.next_tile {
            let mut next_tile_button = &self.clickables[ButtonId::NextTile as usize];
            next_tile_button.draw();
        }

        // Draw the encounter die
        if let Some(encounter) = self.encounter {
            let mut encounter_button = &self.clickables[ButtonId::Encounter as usize];
            encounter_button.draw();
        }

        // Mark the turns
        for turn_box in 10..10 + self.current_turn {
            let (x, y, w, h) = self.clickables[turn_box].to_screen();
            draw_rectangle(x, y, w, h, RED);
        }

        // Display all the visited locations
        for &visited in &self.visited_locations {
            let (x, y, w, h) = self.clickables[visited].to_screen();
            draw_rectangle(x, y, w, h, color_from_usize(139, 69, 19, 80));
        }

        // Display the current player location
        let (x, y, w, h) = self.clickables[self.player_location].to_screen();
        draw_rectangle(x, y, w, h, color_from_usize(139, 69, 19, 140));

        if matches!(self.state, BoardState::AssignDice) {
            // Draw the two dice
            let die1_button = &self.clickables[ButtonId::Die1 as usize];
            let die2_button = &self.clickables[ButtonId::Die2 as usize];

            if self.die1.is_some() {
                die1_button.draw();
            }

            if self.die2.is_some() {
                die2_button.draw();
            }

            // Highlight the selected die
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
        }

        // In Assign Die, highlight the selected die
        if matches!(self.state, BoardState::AssignDice) {
            if let Some(die) = self.selected_die {
                let die_id = match die {
                    1 => ButtonId::Die1 as usize,
                    2 => ButtonId::Die2 as usize,
                    _ => unreachable!()
                };

                let (x, y, w, h) = self.clickables[die_id].to_screen();
                draw_rectangle_lines(x, y, w, h, 10., BLACK);
            }
        }

        // If we are in draw_walls, draw the walls that the player is selecting
        if matches!(self.state, BoardState::DrawWalls) || 
                matches!(self.state, BoardState::ShortcutDrawWalls) {
            let (x, y, w, h) = self.clickables[self.player_location].to_screen();
            for wall in &self.selected_walls {
                match wall {
                    Wall::Top    => draw_line(x,     y,     x + w, y,     6., BLACK),
                    Wall::Right  => draw_line(x + w, y,     x + w, y + h, 6., BLACK),
                    Wall::Bottom => draw_line(x,     y + h, x + w, y + h, 6., BLACK),
                    Wall::Left   => draw_line(x,     y,     x,     y + h, 6., BLACK),
                }
            }
        }

        // If we are in choose_teleport, draw the locations that the player could
        // teleport to
        if matches!(self.state, BoardState::ChooseTeleport) {
            for &index in [36, 43].iter() {
                let (x, y, w, h) = self.clickables[index as usize].to_screen();
                draw_rectangle(x, y, w, h, color_from_usize(139, 69, 19, 240));
            }

            if let Some(index) = self.selected_teleport {
                let (x, y, w, h) = self.clickables[index].to_screen();
                draw_rectangle_lines(x, y, w, h, 10., GREEN);
            }
        }

        // If we are in draw_movement, draw the spaces available to the player
        if matches!(self.state, BoardState::Movement) ||
                matches!(self.state, BoardState::ShortcutMovement) {
            for (_wall, index) in self.get_neighbors() {
                let (x, y, w, h) = self.clickables[index].to_screen();
                draw_rectangle(x, y, w, h, color_from_usize(139, 69, 19, 240));
            }
        }

        /*
        if let Some(select) = self.selected {
            info!("Drawing {}", select);
            let selected_rect = &self.clickables[select];
            selected_rect.draw();
        }
        */

        // Draw all walls that are currently built
        for &wall in &self.built_walls {
            let (x, y, w, h) = self.clickables[wall].to_screen();
            draw_rectangle(x, y, w, h, BLACK);
        }

        // Highlight the seleted location
        if let Some((location, _health, _wall)) = self.selected_move {
            info!("Highlighting selected move: {:?}", location);
            let (x, y, w, h) = self.clickables[location].to_screen();
            draw_rectangle_lines(x, y, w, h, 8., GREEN);
        }

    }
}
