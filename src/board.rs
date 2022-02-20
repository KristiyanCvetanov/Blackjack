use crate::card::{self, Card};
use ggez::{
    Context, 
    GameResult, 
    graphics,
    audio,
    mint::{Point2, Vector2}
};
use rand::seq::SliceRandom;


pub const DECK_POSITION: Point2<f32> = Point2 { x: 100.0, y: 160.0 };
const PLAYER_FIRST_POSITION: Point2<f32> = Point2 { x: 100.0, y: 770.0 };
const DEALER_FIRST_POSITION: Point2<f32> = Point2 { x: 100.0, y: 475.0 };
const MOVING_CARD_STEP: f32 = 1.0 / 75.0;
const CARD_SPACING: f32 = 170.0;

#[derive(Debug, Clone)]
pub enum Turn {
    Player,
    Dealer,
}

#[derive(Debug)]
pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        
        let mut vec = card::all();
        vec.shuffle(&mut rng);

        Deck {
            cards: vec,
        }
    }

    pub fn deal_card(&mut self, ctx: &mut Context) -> GameResult<Card> {
        let mut card = self.cards.pop().unwrap();

        card.load(ctx)?;

        card.position = DECK_POSITION;

        Ok(card)
    }

    pub fn get_top_card(&self) -> Card {
        self.cards.last().unwrap().clone()
    }
}

pub struct Assets {
    pub deck_image: graphics::Image,
    pub card_deal_sound: Box<dyn audio::SoundSource>,
    pub card_flip_sound: Box<dyn audio::SoundSource>,
}

impl Assets {
    pub fn new(ctx: &mut Context) -> GameResult<Assets> { 
        let deck_image = graphics::Image::new(ctx, "\\card_images\\card_back.png")?;
        let card_deal_sound = audio::Source::new(ctx, "\\sfx\\card_deal.wav")?;
        let card_flip_sound = audio::Source::new(ctx, "\\sfx\\card_flip.wav")?;

        Ok (
            Assets {
                deck_image, 
                card_deal_sound: Box::new(card_deal_sound), 
                card_flip_sound: Box::new(card_flip_sound),
            }
        )
    }
}

pub struct Board {
    pub deck: Deck,
    pub turn: Turn,
    pub dealed_cards_player: Vec<Card>,
    pub dealed_cards_dealer: Vec<Card>,
    pub assets: Assets,
    pub calculate_result: bool,
    pub card_moving: bool,
    next_card_position_player: Point2<f32>,
    next_card_position_dealer: Point2<f32>,
    translation: Vector2<f32>,
}

impl Board {
    fn get_translating_vector(next_pos: Point2<f32>) -> Vector2<f32> {
        let vec_x: f32 = (next_pos.x - DECK_POSITION.x) * MOVING_CARD_STEP;
        let vec_y: f32 = (next_pos.y - DECK_POSITION.y) * MOVING_CARD_STEP;

        Vector2 { x: vec_x, y: vec_y }
    }

    fn draw_deck(&self, ctx: &mut Context) -> GameResult<()> {
        let draw_params = graphics::DrawParam::default().
            dest(DECK_POSITION).
            offset(Point2 { x: 0.5, y: 0.5 }).
            scale(Vector2 {
                x: card::CARD_SCALE,
                y: card::CARD_SCALE,
            });
        graphics::draw(ctx, &self.assets.deck_image, draw_params)?;

        Ok(())
    }

    pub fn new(ctx: &mut Context) -> GameResult<Board> {
        let assets = Assets::new(ctx)?;

        Ok(
            Board {
                deck: Deck::new(),
                turn: Turn::Player,
                dealed_cards_player: Vec::new(),
                dealed_cards_dealer: Vec::new(),
                assets,
                calculate_result: false,
                next_card_position_player: PLAYER_FIRST_POSITION,
                next_card_position_dealer: DEALER_FIRST_POSITION,
                translation: Self::get_translating_vector(PLAYER_FIRST_POSITION),
                card_moving: false,
            }
        )   
    }

    fn change_next_position(&mut self) {
        match self.turn {
            Turn::Player => {
                self.next_card_position_player.x += CARD_SPACING;
            },
            Turn::Dealer => {
                self.next_card_position_dealer.x += CARD_SPACING;
            },
        }
    }

    fn change_translating_vector(&mut self) {
        match self.turn {
            Turn::Player => {
                self.translation = Self::get_translating_vector(self.next_card_position_player);
            },
            Turn::Dealer => {
                self.translation = Self::get_translating_vector(self.next_card_position_dealer);
            }
        }
    }

    pub fn set_card(&mut self, dealed_card: Card) {
        match self.turn {
            Turn::Player => self.dealed_cards_player.push(dealed_card),
            Turn::Dealer => self.dealed_cards_dealer.push(dealed_card),
        }
    }

    pub fn update(&mut self, ctx: &mut Context, time_delta: f32) {
        let mut is_moving: bool = false;
        let mut is_flipping: bool = false;

        for card in &mut self.dealed_cards_player {
            let mut vec = Vector2{ x: 0.0, y: 0.0 };
            
            if matches!(card.move_state, card::CardMoveState::Moving) {
                is_moving = true;
                vec = self.translation.clone();
            }
            
            if !matches!(card.animation.state, card::FlipAnimationState::Stopped) {
                is_flipping = true;
            }

            card.update(time_delta, vec, self.next_card_position_player);
        }

        for card in &mut self.dealed_cards_dealer {
            let mut vec = Vector2{ x: 0.0, y: 0.0 };
            
            if matches!(card.move_state, card::CardMoveState::Moving) {
                is_moving = true;
                vec = self.translation.clone();
            }
            
            if !matches!(card.animation.state, card::FlipAnimationState::Stopped) {
                is_flipping = true;
            }
            
            card.update(time_delta, vec, self.next_card_position_dealer);
        }

        if is_moving && !self.card_moving {
            // ако има движеща се карта, но флага е свален, то вдигаме флага(за да не могат да се раздават карти)

            self.card_moving = true;
        } else if !is_moving && !is_flipping && self.card_moving {
            // ако няма движеща се(или обръщаща се) карта, но флага е вдигнат, значи току що е спряла движеща се карта
            // в този случай:
            // 1. сваляме флага(вече няма движеща се карта)
            // 2. променяме позицията, където ще отиде следващата раздадена карта
            // 3. променяме транслиращия вектор
            // 4. смятаме наново резултата

            self.card_moving = false;
            self.change_next_position();
            self.change_translating_vector();
            self.calculate_result = true;

            let _ = self.assets.card_flip_sound.play(ctx);
        }
        // в другите два случая не правим нищо
    }

    pub fn draw(&self,  ctx: &mut Context) -> GameResult<()> {
        self.draw_deck(ctx)?;

        for card in &self.dealed_cards_player {
            card.draw(ctx)?;
        }

        for card in &self.dealed_cards_dealer {
            card.draw(ctx)?;
        }

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deck_get_top_card_returns_top_card() {
        let deck = Deck::new();
        let card = deck.get_top_card();

        assert_eq!(card.name, deck.cards.last().unwrap().name);
        assert_eq!(deck.cards.len(), 52);
    }
}
