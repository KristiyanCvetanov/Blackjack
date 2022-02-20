use ggez::{Context, GameResult, graphics};
use ggez::mint::{Point2, Vector2};

use std::fmt;
use std::error::Error;

pub const CARD_DIMENSION_X: f32 = 150.0;
pub const CARD_DIMENSION_Y: f32 = 200.0;
pub const CARD_SCALE: f32 = 0.27;  
const FLIP_DURATION: f32 = 0.3;

#[derive(Debug, Clone)]
pub enum CardFlipState {
    Front,
    Back,
}

#[derive(Debug, Clone)]
pub enum CardMoveState {
    Moving,
    Stopped,
}

#[derive(Debug, Clone)]
pub struct CardNameError {
    details: String,
}

impl CardNameError {
    fn new(msg: &str) -> Self {
        CardNameError {
            details: msg.to_string()
        }
    }
}

impl fmt::Display for CardNameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.details)
    }
}

impl Error for CardNameError {
    fn description(&self) -> &str {
        &self.details
    }
}

#[derive(Debug, Clone)]
pub struct Card {
    pub flip_state: CardFlipState,
    pub move_state: CardMoveState,
    pub name: String,
    pub position: Point2<f32>,
    pub animation: FlipAnimation,
    image_front: Option<graphics::Image>,
    image_back: Option<graphics::Image>,
    flipped: bool,
}

impl Card {
    pub fn new(card_name: &str) -> Self {
        Card {
            flip_state: CardFlipState::Back,
            move_state: CardMoveState::Moving,
            name: String::from(card_name),
            position: Point2 { x: 0.0, y: 0.0 },
            animation: FlipAnimation::new(FLIP_DURATION),
            flipped: false,
            image_back: None,
            image_front: None,  
        }
    }

    pub fn load(&mut self, ctx: &mut Context) -> GameResult<()> {
        let path = format!("\\card_images\\{}.png", self.name);

        self.image_front = Some(graphics::Image::new(ctx, path)?);
        self.image_back  = Some(graphics::Image::new(ctx, "\\card_images\\card_back.png")?);

        Ok(())
    }

    pub fn update(&mut self, time_delta: f32, translation: Vector2<f32>, dest_point: Point2<f32>) {
        self.animation.update(time_delta);

        let new_pos_x = self.position.x + translation.x;
        let new_pos_y = self.position.y + translation.y;

        self.position.x = nalgebra::clamp(new_pos_x, 0.0, dest_point.x);
        self.position.y = nalgebra::clamp(new_pos_y, 0.0, dest_point.y);

        if self.position == dest_point {
            self.move_state = CardMoveState::Stopped;

            if !self.flipped {
                self.animation.state = FlipAnimationState::Started;
                self.flipped = true;
            }

            if matches!(self.animation.state, FlipAnimationState::BeforeFlip) {
                self.flip_state = CardFlipState::Front;
                self.animation.state = FlipAnimationState::AfterFlip;
            }
        }
    }

    pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        if let Some(image) = self.get_visible_image() {
            let draw_params = graphics::DrawParam::default().
                dest(self.position).
                offset(Point2 { x: 0.5, y: 0.5 }).
                scale(Vector2 {
                    x: self.animation.scale_x * CARD_SCALE,
                    y: CARD_SCALE,
                });
            graphics::draw(ctx, image, draw_params)?;
        }

        Ok(())
    }

    pub fn get_points(&self) -> Result<u32, CardNameError> {
        let c = self.name.chars().next().unwrap();
        match c {
            '2' => Ok(2),
            '3' => Ok(3),
            '4' => Ok(4),
            '5' => Ok(5),
            '6' => Ok(6),
            '7' => Ok(7),
            '8' => Ok(8),
            '9' => Ok(9),
            '1' => Ok(10),
            'j' => Ok(10),
            'q' => Ok(10),
            'k' => Ok(10),
            'a' => Ok(11),
            _   => Err(CardNameError::new("Invalid card name!")),
        }
    }
    
    pub fn is_an_ace(&self) -> bool {
        let c: char = self.name.chars().next().unwrap();
        
        c == 'a'
    }

    fn get_visible_image(&self) -> Option<&graphics::Image> {
        match self.flip_state {
            CardFlipState::Front => self.image_front.as_ref(),
            CardFlipState::Back  => self.image_back.as_ref(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum FlipAnimationState {
    /// The animation will play
    Started,

    /// The animation is at the point where the card should be flipped
    BeforeFlip,

    /// The card has (hopefully) been flipped now
    AfterFlip,

    /// The animation is not playing
    Stopped,
}

#[derive(Debug, Clone)]
pub struct FlipAnimation {
    pub scale_x: f32,
    pub state: FlipAnimationState,

    /// Number of seconds to animate in one direction
    duration: f32,

    /// Progress of the animation: 0 <= progress <= duration
    progress: f32,

    /// Positive or negative change in direction: -1.0 or +1.0
    direction: f32,
}

impl FlipAnimation {
    fn new(duration: f32) -> Self {
        FlipAnimation {
            scale_x: 1.0,
            state: FlipAnimationState::Stopped,
            progress: 0.0,
            direction: 1.0,
            duration,
        }
    }

    pub fn update(&mut self, time_delta: f32) {
        if matches!(self.state, FlipAnimationState::Stopped) {
            return;
        }

        self.progress += self.direction * time_delta;

        // Flip conditions:
        if self.progress >= self.duration {
            self.progress = self.duration;
            self.direction = -1.0;
            self.state = FlipAnimationState::BeforeFlip;
        } else if self.progress <= 0.0 {
            self.progress = 0.0;
            self.direction = 1.0;
            self.state = FlipAnimationState::Stopped;
        }

        self.scale_x = 1.0 - (self.progress / self.duration);
    }
}

pub fn all() -> Vec<Card> {
    vec![
        Card::new("ace_of_clubs"),
        Card::new("ace_of_diamonds"),
        Card::new("ace_of_hearts"),
        Card::new("ace_of_spades"),
        Card::new("2_of_clubs"),
        Card::new("2_of_diamonds"),
        Card::new("2_of_hearts"),
        Card::new("2_of_spades"),
        Card::new("3_of_clubs"),
        Card::new("3_of_diamonds"),
        Card::new("3_of_hearts"),
        Card::new("3_of_spades"),
        Card::new("4_of_clubs"),
        Card::new("4_of_diamonds"),
        Card::new("4_of_hearts"),
        Card::new("4_of_spades"),
        Card::new("5_of_clubs"),
        Card::new("5_of_diamonds"),
        Card::new("5_of_hearts"),
        Card::new("5_of_spades"),
        Card::new("6_of_clubs"),
        Card::new("6_of_diamonds"),
        Card::new("6_of_hearts"),
        Card::new("6_of_spades"),
        Card::new("7_of_clubs"),
        Card::new("7_of_diamonds"),
        Card::new("7_of_hearts"),
        Card::new("7_of_spades"),
        Card::new("8_of_clubs"),
        Card::new("8_of_diamonds"),
        Card::new("8_of_hearts"),
        Card::new("8_of_spades"),
        Card::new("9_of_clubs"),
        Card::new("9_of_diamonds"),
        Card::new("9_of_hearts"),
        Card::new("9_of_spades"),
        Card::new("10_of_clubs"),
        Card::new("10_of_diamonds"),
        Card::new("10_of_hearts"),
        Card::new("10_of_spades"),
        Card::new("jack_of_clubs"),
        Card::new("jack_of_diamonds"),
        Card::new("jack_of_hearts"),
        Card::new("jack_of_spades"),
        Card::new("queen_of_clubs"),
        Card::new("queen_of_diamonds"),
        Card::new("queen_of_hearts"),
        Card::new("queen_of_spades"),
        Card::new("king_of_clubs"),
        Card::new("king_of_diamonds"),
        Card::new("king_of_hearts"),
        Card::new("king_of_spades"),
    ]
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_points_2() {
        let card = Card::new("2_of_something");

        let points = card.get_points().unwrap();

        assert_eq!(points, 2, "a 2 should give 2 points");
    }

    #[test]
    fn get_points_3() {
        let card = Card::new("3_of_something");

        let points = card.get_points().unwrap();

        assert_eq!(points, 3, "a 3 should give 3 points");
    }

    #[test]
    fn get_points_4() {
        let card = Card::new("4_of_something");

        let points = card.get_points().unwrap();

        assert_eq!(points, 4, "a 4 should give 4 points");
    }

    #[test]
    fn get_points_5() {
        let card = Card::new("5_of_something");

        let points = card.get_points().unwrap();

        assert_eq!(points, 5, "a 5 should give 5 points");
    }

    #[test]
    fn get_points_6() {
        let card = Card::new("6_of_something");

        let points = card.get_points().unwrap();

        assert_eq!(points, 6, "a 6 should give 6 points");
    }

    #[test]
    fn get_points_7() {
        let card = Card::new("7_of_something");

        let points = card.get_points().unwrap();

        assert_eq!(points, 7, "a 7 should give 7 points");
    }

    #[test]
    fn get_points_8() {
        let card = Card::new("8_of_something");

        let points = card.get_points().unwrap();

        assert_eq!(points, 8, "a 8 should give 8 points");
    }

    #[test]
    fn get_points_9() {
        let card = Card::new("9_of_something");

        let points = card.get_points().unwrap();

        assert_eq!(points, 9, "a 9 should give 9 points");
    }

    #[test]
    fn get_points_10() {
        let card = Card::new("10_of_something");

        let points = card.get_points().unwrap();

        assert_eq!(points, 10, "a 10 should give 10 points");
    }

    #[test]
    fn get_points_jack() {
        let card = Card::new("jack_of_something");

        let points = card.get_points().unwrap();

        assert_eq!(points, 10, "jack should give 10 points");
    }

    #[test]
    fn get_points_queen() {
        let card = Card::new("queen_of_something");

        let points = card.get_points().unwrap();

        assert_eq!(points, 10, "queen should give 10 points");
    }

    #[test]
    fn get_points_king() {
        let card = Card::new("king_of_something");

        let points = card.get_points().unwrap();

        assert_eq!(points, 10, "king should give 10 points");
    }

    #[test]
    fn get_points_ace() {
        let card = Card::new("ace_of_something");

        let points = card.get_points().unwrap();

        assert_eq!(points, 11, "ace should give 11 points");
    }

    #[test]
    #[should_panic]
    fn get_points_invalid_card_name() {
        let card = Card::new("some invalid name");

        let _ = card.get_points().unwrap();
    }

    #[test]
    fn is_an_ace_correct_case() {
        let card = Card::new("ace_of_something");

        assert!(card.is_an_ace(), "card should have been an ace");
    }

    #[test]
    #[should_panic]
    fn is_an_ace_wrong_case() {
        let card = Card::new("not an ace");

        assert!(card.is_an_ace(), "card should not have been an ace");
    }

    #[test]
    fn all_should_return_52_cards() {
        assert_eq!(all().len(), 52);
    }
}