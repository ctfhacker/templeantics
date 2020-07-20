use crate::*;

/// Game board storing game state
pub struct Board {
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
    dice_textures: [Texture2D; 7]
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

        let die1 = Some(5);
        let die2 = Some(6);

        let mut die1_button = &mut clickables[60];
        die1_button.texture = Some(dice_textures[die1.unwrap()]);

        let mut die2_button = &mut clickables[61]; 
        die2_button.texture = Some(dice_textures[die2.unwrap()]);

        Board { 
            texture, 
            clickables, 
            selected: None, 
            die1, 
            die2, 
            selected_die: None,
            next_tile: None, 
            encounter: None, 
            dice_textures 
        }
    } 

    /// Sets the value and the texture of die 1
    pub fn set_die1(&mut self, val: Option<usize>) {
        if let Some(new_val) = val {
            let die1_button = &mut self.clickables[60];
            die1_button.change_texture(self.dice_textures[new_val]);
        }
        self.die1 = val;
    }

    /// Sets the value and the texture of die 2
    pub fn set_die2(&mut self, val: Option<usize>) {
        if let Some(new_val) = val {
            let die2_button = &mut self.clickables[61];
            die2_button.change_texture(self.dice_textures[new_val]);
        }
        self.die2 = val;
    }
}

#[derive(Debug, Copy, Clone)]
/// Messages that are triggered by mouse clicks
pub enum BoardMessage {
    Select(usize),
    ChooseDie1,
    ChooseDie2,
    ChooseNextTile,
    ChooseEncounter,
}

impl Clickable for Board { 
    type Message = BoardMessage;

    fn handle_click(&mut self, pos: (f32, f32)) -> Option<Self::Message> {
        for (i, rect) in self.clickables.iter().enumerate() {
            if rect.contains(pos) {
                let msg = match i {
                     8 => BoardMessage::ChooseNextTile,
                     9 => BoardMessage::ChooseEncounter,
                    60 => BoardMessage::ChooseDie1,
                    61 => BoardMessage::ChooseDie2,
                    _ => BoardMessage::Select(i)
                };

                info!("Sending message: {:?}", msg);
                return Some(msg);
            }
        }

        None
    }

    fn handle_message(&mut self, message: Self::Message) { 
        info!("Handling message: {:?}", message);
        match message {
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
                        let next_tile_button = &mut self.clickables[8];

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
                        let encounter_button = &mut self.clickables[9];

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

        if let Some(select) = self.selected {
            let selected_rect = &self.clickables[select];
            selected_rect.draw();
        }

        let roll_button = Button { x: 0.03, y: 0.54, w: 0.12, h: 0.096, 
            text: Some(("ROLL".to_string(), RED)), texture: None };

        let die1_button = &self.clickables[60];
        let die2_button = &self.clickables[61];

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

        if self.die1.is_none() && self.die2.is_none() {
            roll_button.draw();
        }

        if let Some(next_tile) = self.next_tile {
            let mut next_tile_button = &self.clickables[8];
            next_tile_button.draw();
        }

        if let Some(encounter) = self.encounter {
            let mut encounter_button = &self.clickables[9];
            encounter_button.draw();
        }

    }
}
