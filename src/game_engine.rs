use crate::card::Card;
use crate::board;

use ggez::{
    Context,
    GameResult,
    mint::Point2,
    graphics,
};


const SCORE_SIZE: f32 = 50.0;


#[derive(Debug, Clone)]
pub enum Outcome {
    Undecided,
    Win,
    Draw,
    Lose,
}

#[derive(Debug, Clone)]
pub enum HintStatus {
    Unused,
    Active,
    Exhausted
}

pub struct GameEngine {
    pub player_score: u32,
    pub dealer_score: u32,
    pub hint: HintStatus,
    pub dealer_handicap_active: bool,
    pub game_over: bool,
    pub outcome: Outcome
}

impl GameEngine {
    pub fn new() -> Self {
        GameEngine {
            player_score: 0,
            dealer_score: 0,
            hint: HintStatus::Unused,
            dealer_handicap_active: false,
            game_over: false,
            outcome: Outcome::Undecided,
        }
    }

    pub fn check_outcome(&mut self, turn: &mut board::Turn) {
        let handicap_addition: u32;
        if self.dealer_handicap_active {
            handicap_addition = 1;
        } else {
            handicap_addition = 0;
        }

        if self.player_score > 21 {
            // player has more than 21 -> player loses
            self.game_over = true;
            self.outcome = Outcome::Lose;
        } else if self.dealer_score > 21 {
            // dealer has more than 21 -> player wins
            self.game_over = true;
            self.outcome = Outcome::Win;
        } else if matches!(turn, board::Turn::Dealer) 
                && self.dealer_score >= 17 
                && self.player_score > self.dealer_score - handicap_addition {
            // dealer finished drawing(has >= 17) and player has more than dealer -> player wins
            self.game_over = true;
            self.outcome = Outcome::Win;
        } else if matches!(turn, board::Turn::Dealer) 
                && self.dealer_score >= 17 
                && self.player_score < self.dealer_score - handicap_addition {
            // dealer finished drawing(has >= 17) and player has less than dealer -> player loses  
            self.game_over = true;
            self.outcome = Outcome::Lose;
        } else if matches!(turn, board::Turn::Dealer) 
                && self.dealer_score >= 17 
                && self.player_score == self.dealer_score - handicap_addition {
            // dealer finished drawing(has >= 17) and player and dealer tied -> draw  
            self.game_over = true;
            self.outcome = Outcome::Draw;
        } else if matches!(turn, board::Turn::Player) 
                && self.player_score == 21{
            // player has a blackjack -> dealers turn
            *turn = board::Turn::Dealer;
        }
        // in the other cases, player or dealer are still drawing
    }

    pub fn score(&mut self, dealed_cards: &Vec<Card>, turn: board::Turn) -> GameResult<()> {
        let mut score: u32 = 0;
        let mut num_of_aces: u32 = 0;
        for card in dealed_cards {
            if card.is_an_ace() {
                num_of_aces += 1;
            } else {
                score += card.get_points().unwrap();
            }
        }
    
        if num_of_aces > 0 && score + 11 + (num_of_aces - 1) <= 21 {
            score += 11 + (num_of_aces - 1);
        } else {
            score += num_of_aces;
        }
    
        match turn {
            board::Turn::Player => self.player_score = score,
            board::Turn::Dealer => self.dealer_score = score,
        }

        Ok(())
    }

    pub fn draw_score(&self, ctx: &mut Context, pos_player: Point2<f32>, pos_dealer: Point2<f32>) -> GameResult<()> {
        let color;
        match self.dealer_handicap_active {
            true => color = graphics::Color::from_rgb(204, 0, 0),
            false => color = graphics::Color::from_rgb(255, 255, 255),
        }

        let font = graphics::Font::new(ctx, "\\font\\DejaVuSerif.ttf")?;

        let player_score_clone = self.player_score.clone();
        let dealer_score_clone = self.dealer_score.clone();

        let player_score_fragment = graphics::TextFragment::new(player_score_clone.to_string().as_str()).
                                                            font(font).
                                                            scale(graphics::PxScale::from(SCORE_SIZE));

        let dealer_score_fragment = graphics::TextFragment::new(dealer_score_clone.to_string().as_str()).
                                                            color(color). 
                                                            font(font).
                                                            scale(graphics::PxScale::from(SCORE_SIZE));

        graphics::draw(ctx, &graphics::Text::new(player_score_fragment), graphics::DrawParam::default().dest(pos_player))?;
        graphics::draw(ctx, &graphics::Text::new(dealer_score_fragment), graphics::DrawParam::default().dest(pos_dealer))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_outcome_player_over_21() {
        let mut engine = GameEngine::new();
        engine.player_score = 22;

        engine.check_outcome(&mut board::Turn::Player);
    
        assert!(matches!(engine.outcome, Outcome::Lose));
    }

    #[test]
    fn check_outcome_dealer_over_21() {
        let mut engine = GameEngine::new();
        engine.dealer_score = 22;

        engine.check_outcome(&mut board::Turn::Dealer);
    
        assert!(matches!(engine.outcome, Outcome::Win));
    }

    #[test]
    fn check_outcome_player_has_more_than_dealer() {
        let mut engine = GameEngine::new();
        engine.player_score = 20;
        engine.dealer_score = 18;

        engine.check_outcome(&mut board::Turn::Dealer);
    
        assert!(matches!(engine.outcome, Outcome::Win));
    }

    #[test]
    fn check_outcome_player_has_less_than_dealer() {
        let mut engine = GameEngine::new();
        engine.player_score = 20;
        engine.dealer_score = 21;

        engine.check_outcome(&mut board::Turn::Dealer);
    
        assert!(matches!(engine.outcome, Outcome::Lose));
    }

    #[test]
    fn check_outcome_player_and_dealer_equal() {
        let mut engine = GameEngine::new();
        engine.player_score = 19;
        engine.dealer_score = 19;

        engine.check_outcome(&mut board::Turn::Dealer);
    
        assert!(matches!(engine.outcome, Outcome::Draw));
    }

    #[test]
    fn check_outcome_player_has_21_and_its_players_turn() {
        let mut engine = GameEngine::new();
        engine.player_score = 21;

        let mut turn = board::Turn::Player;
        engine.check_outcome(&mut turn);
    
        assert!(matches!(turn, board::Turn::Dealer));
    }

    #[test]
    fn score_on_players_turn() {
        let mut engine = GameEngine::new();
        let v: Vec<Card> = vec![Card::new("king_of_diamonds")];

        engine.score(&v, board::Turn::Player).unwrap();

        assert!(engine.player_score > 0);
        assert_eq!(engine.dealer_score, 0);
    }

    #[test]
    fn score_on_dealers_turn() {
        let mut engine = GameEngine::new();
        let v: Vec<Card> = vec![Card::new("7_of_spades")];

        engine.score(&v, board::Turn::Dealer).unwrap();

        assert!(engine.dealer_score > 0);
        assert_eq!(engine.player_score, 0);
    }

    #[test]
    fn score_without_aces() {
        let mut engine = GameEngine::new();
        let v: Vec<Card> = vec![Card::new("king_of_diamonds"), Card::new("6_of_hearts"), Card::new("2_of_clubs")];

        engine.score(&v, board::Turn::Player).unwrap();

        assert_eq!(engine.player_score, 18);
    }

    #[test]
    fn score_ace_should_count_as_one() {
        let mut engine = GameEngine::new();
        let v: Vec<Card> = vec![Card::new("king_of_diamonds"), Card::new("6_of_hearts"), Card::new("ace_of_clubs")];

        engine.score(&v, board::Turn::Player).unwrap();

        assert_eq!(engine.player_score, 17);
    }

    #[test]
    fn score_ace_should_count_as_eleven() {
        let mut engine = GameEngine::new();
        let v: Vec<Card> = vec![Card::new("3_of_diamonds"), Card::new("6_of_hearts"), Card::new("ace_of_clubs")];

        engine.score(&v, board::Turn::Player).unwrap();

        assert_eq!(engine.player_score, 20);
    }

    #[test]
    fn score_with_more_than_one_aces() {
        let mut engine = GameEngine::new();
        let v: Vec<Card> = vec![Card::new("ace_of_diamonds"), Card::new("ace_of_hearts"), Card::new("ace_of_clubs"), Card::new("5_of_spades")];

        engine.score(&v, board::Turn::Player).unwrap();

        assert_eq!(engine.player_score, 18);
    }
}