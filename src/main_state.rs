use crate::board::{self, Board};
use crate::card;
use crate::game_engine::{GameEngine, Outcome, HintStatus};

use rand::Rng;
use std::str::FromStr;

use ggez::{
    Context,
    GameResult,
    mint::Point2,
    event,
    graphics,
    input::{mouse, self},
    timer,
};

use std::io::{BufRead, Write, BufWriter};
use std::fs::OpenOptions;

const MENU_TITLE_POSITION: Point2<f32> = Point2 { x: 750.0, y: 300.0 };
const MENU_TITLE_SIZE: f32 = 60.0;
const MENU_PLAY_TEXT_POSITION: Point2<f32> = Point2 { x: 800.0, y: 500.0 }; 
const MENU_PLAY_TEXT_SIZE: f32 = 40.0;
const MENU_HELP_TEXT_POSITION: Point2<f32> = Point2 { x: 800.0, y: 700.0 };
const MENU_HELP_TEXT_SIZE: f32 = 40.0;

const HELP_TITLE_POSITION: Point2<f32> = Point2 { x: 800.0, y: 50.0 };
const HELP_TITLE_SIZE: f32 = 60.0;
const HELP_DESCRIPTION_POSITION: Point2<f32> = Point2 { x: 50.0, y: 200.0 };
const HELP_DESCRIPTION_SIZE: f32 = 30.0;
const HELP_BACK_TEXT_POSITION: Point2<f32> = Point2 { x: 1600.0, y: 800.0 };
const HELP_BACK_TEXT_SIZE: f32 = 40.0;

const PLAYER_SCORE_POSITION: Point2<f32> = Point2 { x: 450.0, y: 100.0 };
const PLAYER_TEXT_SCORE_POSITION: Point2<f32> = Point2 { x: 370.0, y: 50.0 };
const PLAYER_TEXT_SCORE_SIZE: f32 = 28.0;
const DEALER_SCORE_POSITION: Point2<f32> = Point2 { x: 850.0, y: 100.0 };
const DEALER_TEXT_SCORE_POSITION: Point2<f32> = Point2 { x: 765.0, y: 50.0 };
const DEALER_TEXT_SCORE_SIZE: f32 = 28.0;

const POWER_UPS_TEXT_POSITION: Point2<f32> = Point2 { x: 1100.0, y: 50.0 };
const POWER_UPS_TEXT_SIZE: f32 = 28.0;

const WINS_TEXT_POSITION:  Point2<f32> = Point2 { x: 1600.0, y: 50.0 };
const WINS_TEXT_SIZE: f32 = 28.0;

const HINT_RANGE_SIZE: u32 = 4;
const HINT_TEXT_POSITION: Point2<f32> = Point2 { x: 50.0, y: 400.0 };
const HINT_TEXT_SIZE: f32 = 35.0; 

const GAME_OVER_TEXT_POSITION: Point2<f32> = Point2 { x: 620.0, y: 420.0 };
const GAME_OVER_TEXT_SIZE: f32 = 100.0;

const SECONDS_TILL_GAME_OVER: f32 = 4.0;
const SECONDS_TILL_MENU: f32 = 3.0;



#[derive(Debug)]
pub enum GameStatus {
    Menu,
    Help,
    Play,
}

pub struct MainState {
    board: Board,
    engine: GameEngine,
    status: GameStatus,
    wins: u32,
    power_ups_count: (u32, u32),
    hint_range: Option<(u32, u32)>,
    time_till_game_over: f32,
    time_till_menu: f32,
    file_name: String,
}

impl MainState {
    fn load<B: BufRead>(mut reader: B) -> (u32, u32, u32) {
        let mut buffer = String::new();
        reader.read_line(&mut buffer).unwrap();

        let v: Vec<u32> = buffer.split(' ').map(|s| FromStr::from_str(s).unwrap()).collect();

        (v[0], v[1], v[2])
    }

    fn save(&self) {
        let f = OpenOptions::new().write(true).open(self.file_name.clone()).unwrap();
        let mut writer = BufWriter::new(f);

        writer.write(self.wins.to_string().as_bytes()).unwrap();
        writer.write(b" ").unwrap();
        writer.write(self.power_ups_count.0.to_string().as_bytes()).unwrap();
        writer.write(b" ").unwrap();
        writer.write(self.power_ups_count.1.to_string().as_bytes()).unwrap();

        writer.flush().unwrap();
    }

    pub fn new<B: BufRead>(ctx: &mut Context, reader: B, file: &str) -> GameResult<MainState> {
        let board = Board::new(ctx)?;
        let stats = Self::load(reader);

        Ok(
            MainState {
                board, 
                engine: GameEngine::new(),
                status: GameStatus::Menu,
                wins: stats.0, 
                power_ups_count: (stats.1, stats.2), 
                hint_range: None,
                time_till_game_over: SECONDS_TILL_GAME_OVER,
                time_till_menu: SECONDS_TILL_MENU,
                file_name: file.to_string(), // used for reset and exit(with esc)
            }
        )
    }

    fn deal_card(&mut self, ctx: &mut Context) -> GameResult<()> {
        let dealed_card = self.board.deck.deal_card(ctx)?; 
        self.board.set_card(dealed_card);                  
        self.board.assets.card_deal_sound.play(ctx)?;      

        Ok(())
    }

    fn mouse_over_play(&self, mouse_position: Point2<f32>) -> bool {
        let matches_horizontal = (mouse_position.x >= MENU_PLAY_TEXT_POSITION.x - 10.0) 
                                    && (mouse_position.x <= MENU_PLAY_TEXT_POSITION.x + 120.0);

        let matches_vertical = (mouse_position.y >= MENU_PLAY_TEXT_POSITION.y - 10.0) 
                                    && (mouse_position.y <= MENU_PLAY_TEXT_POSITION.y + 50.0);

        matches_horizontal && matches_vertical
    }

    fn mouse_over_help(&self, mouse_position: Point2<f32>) -> bool {
        let matches_horizontal = (mouse_position.x >= MENU_HELP_TEXT_POSITION.x - 10.0) 
                                    && (mouse_position.x <= MENU_HELP_TEXT_POSITION.x + 120.0);

        let matches_vertical = (mouse_position.y >= MENU_HELP_TEXT_POSITION.y - 10.0) 
                                    && (mouse_position.y <= MENU_HELP_TEXT_POSITION.y + 50.0);

        matches_horizontal && matches_vertical
    } 

    fn mouse_over_back(&self, mouse_position: Point2<f32>) -> bool {
        let matches_horizontal = (mouse_position.x >= HELP_BACK_TEXT_POSITION.x - 10.0) 
                                    && (mouse_position.x <= HELP_BACK_TEXT_POSITION.x + 120.0);

        let matches_vertical = (mouse_position.y >= HELP_BACK_TEXT_POSITION.y - 10.0) 
                                    && (mouse_position.y <= HELP_BACK_TEXT_POSITION.y + 50.0);

        matches_horizontal && matches_vertical
    }

    fn mouse_over_deck(&self, mouse_position: Point2<f32>) -> bool {
        let matches_horizontal = (mouse_position.x >= board::DECK_POSITION.x - card::CARD_DIMENSION_X / 2.0) 
                                    && (mouse_position.x <= board::DECK_POSITION.x + card::CARD_DIMENSION_X / 2.0);

        let matches_vertical = (mouse_position.y >= board::DECK_POSITION.y - card::CARD_DIMENSION_Y / 2.0) 
                                    && (mouse_position.y <= board::DECK_POSITION.y + card::CARD_DIMENSION_Y / 2.0);

        matches_horizontal && matches_vertical
    }

    fn increase_stats(&mut self) {
        self.wins += 1;

        if self.wins % 2 == 0 {
            // increase hints
            self.power_ups_count.0 += 1;
        }
        if self.wins % 3 == 0 {
            // increase dealer handicaps
            self.power_ups_count.1 += 1;
        }
    }

    fn reset(&mut self, ctx: &mut Context) -> GameResult<()> {
        if matches!(self.engine.outcome, Outcome::Win) {
            self.increase_stats();
        }

        self.save();

        self.board = Board::new(ctx)?;
        self.engine = GameEngine::new();
        self.status = GameStatus::Menu;
        self.time_till_game_over = SECONDS_TILL_GAME_OVER;
        self.time_till_menu = SECONDS_TILL_MENU;
        self.hint_range = None;

        Ok(())
    }

    fn update_score(&mut self) -> GameResult<()> {
        if self.board.calculate_result {
            // game engine calculates
           
            if matches!(self.board.turn, board::Turn::Player) {
                self.engine.score(&self.board.dealed_cards_player, board::Turn::Player)?;   
            } else {
                self.engine.score(&self.board.dealed_cards_dealer, board::Turn::Dealer)?; 
            }
            
            // check if game has reached an end state
            self.engine.check_outcome(&mut self.board.turn);   
            self.board.calculate_result = false;
        }

        Ok(())
    }
    
    fn update_menu(&mut self, ctx: &mut Context) {
        if mouse::button_pressed(ctx, mouse::MouseButton::Left) {
            let mouse_position = mouse::position(ctx);

            if self.mouse_over_play(mouse_position) {
                self.status = GameStatus::Play;
            } else if self.mouse_over_help(mouse_position) {
                self.status = GameStatus::Help;
            }
        }
    }  

    fn update_help(&mut self, ctx: &mut Context) {
        if mouse::button_pressed(ctx, mouse::MouseButton::Left) {
            let mouse_position = mouse::position(ctx);

            if self.mouse_over_back(mouse_position) {
                self.status = GameStatus::Menu;
            }
        }
    }

    fn update_game(&mut self, ctx: &mut Context, time_delta: f32) -> GameResult<()> {
        if self.time_till_game_over <= 0.0 { // check for game over
            if self.time_till_menu > 0.0 {
                self.time_till_menu -= time_delta;
            } else {
                self.reset(ctx)?;
            }
        } else if self.engine.game_over && self.time_till_game_over > 0.0 {
            self.time_till_game_over -= time_delta;
        }

        if matches!(self.board.turn, board::Turn::Dealer) { // dealer's turn
            if !self.engine.game_over && !self.board.card_moving {
                self.deal_card(ctx)?;
            }
        } else { // player's turn
            if mouse::button_pressed(ctx, mouse::MouseButton::Left) {
                let mouse_position = mouse::position(ctx);

                if !self.engine.game_over && self.mouse_over_deck(mouse_position) && !self.board.card_moving {
                    self.deal_card(ctx)?;
                    
                    if matches!(self.engine.hint, HintStatus::Active) {
                        self.engine.hint = HintStatus::Exhausted;
                    }
                }
            }
        }

        self.update_score()?; // update score if needed

        self.board.update(ctx, time_delta);

        Ok(())
    }

    fn use_hint(&mut self) {
        if self.power_ups_count.0 == 0 {
            return;
        }

        if matches!(self.engine.hint, HintStatus::Unused) {
            self.engine.hint = HintStatus::Active;
            self.power_ups_count.0 -= 1;
            
            let top_card_points = self.board.deck.get_top_card().get_points().unwrap();
            let mut rng = rand::thread_rng();
            let rand_num: u32 = rng.gen_range(0..HINT_RANGE_SIZE);

            if top_card_points - rand_num + HINT_RANGE_SIZE > 11 {
                self.hint_range = Some((7, 11));
            } else if (top_card_points as i32) - (rand_num as i32) < 2 {
                self.hint_range = Some((2, 6));
            } else {
                self.hint_range = Some((top_card_points - rand_num, top_card_points - rand_num + HINT_RANGE_SIZE));
            }
        }
    }

    fn use_handicap(&mut self) {
        if self.power_ups_count.1 == 0 {
            return;
        }

        if self.engine.dealer_handicap_active == false {
            self.engine.dealer_handicap_active = true;
            self.power_ups_count.1 -= 1;
        }
    }

    fn draw_menu(&self, ctx: &mut Context) -> GameResult<()> {
        let font = graphics::Font::new(ctx, "\\font\\DejaVuSerif.ttf")?;

        let mut title = graphics::Text::new("MENU");
        title.set_font(font, graphics::PxScale::from(MENU_TITLE_SIZE));

        let mut play_button_text = graphics::Text::new("PLAY");
        play_button_text.set_font(font, graphics::PxScale::from(MENU_PLAY_TEXT_SIZE));

        let mut help_button_text = graphics::Text::new("HELP");
        help_button_text.set_font(font, graphics::PxScale::from(MENU_HELP_TEXT_SIZE));

        graphics::draw(ctx, &title, graphics::DrawParam::default().dest(MENU_TITLE_POSITION))?;
        graphics::draw(ctx, &play_button_text, graphics::DrawParam::default().dest(MENU_PLAY_TEXT_POSITION))?;
        graphics::draw(ctx, &help_button_text, graphics::DrawParam::default().dest(MENU_HELP_TEXT_POSITION))
    }

    fn draw_help(&self, ctx: &mut Context) -> GameResult<()> {
        let font = graphics::Font::new(ctx, "\\font\\DejaVuSerif.ttf")?;

        let help_description_str = "        Standard blackjack rules.

        hit = Left-Mouse-Click over deck
        stand = Space 
        use hint = Key1
        use handicap = Key2
        exit = Escape
        
        hint: gives approximation of next card's points
        handicap: dealer's score is reduced with 1 point";

        let mut title = graphics::Text::new("HELP");
        title.set_font(font, graphics::PxScale::from(HELP_TITLE_SIZE));

        let mut help_description = graphics::Text::new(help_description_str);
        help_description.set_font(font, graphics::PxScale::from(HELP_DESCRIPTION_SIZE));

        let mut back_button_text = graphics::Text::new("BACK");
        back_button_text.set_font(font, graphics::PxScale::from(HELP_BACK_TEXT_SIZE));

        // create and draw a rectangle for button

        graphics::draw(ctx, &title, graphics::DrawParam::default().dest(HELP_TITLE_POSITION))?;
        graphics::draw(ctx, &help_description, graphics::DrawParam::default().dest(HELP_DESCRIPTION_POSITION))?;
        graphics::draw(ctx, &back_button_text, graphics::DrawParam::default().dest(HELP_BACK_TEXT_POSITION))
    }

    fn draw_score(&self, ctx: &mut Context) -> GameResult<()> {  
        self.engine.draw_score(ctx, PLAYER_SCORE_POSITION, DEALER_SCORE_POSITION)?;

        let font = graphics::Font::new(ctx, "\\font\\DejaVuSerif.ttf")?;
        
        let mut text_player = graphics::Text::new("PLAYER SCORE:");
        text_player.set_font(font, graphics::PxScale::from(PLAYER_TEXT_SCORE_SIZE));

        let mut text_dealer = graphics::Text::new("DEALER SCORE:");
        text_dealer.set_font(font, graphics::PxScale::from(DEALER_TEXT_SCORE_SIZE));

        graphics::draw(ctx, &text_player, graphics::DrawParam::default().dest(PLAYER_TEXT_SCORE_POSITION))?;
        graphics::draw(ctx, &text_dealer, graphics::DrawParam::default().dest(DEALER_TEXT_SCORE_POSITION))
    }

    fn draw_power_ups(&self, ctx: &mut Context) -> GameResult<()> {
        let font = graphics::Font::new(ctx, "\\font\\DejaVuSerif.ttf")?;

        let available_power_ups = "AVAILABLE POWER UPS:\n".to_string();
        let first_power_up = "1. Next card approximation x".to_owned() + self.power_ups_count.0.to_string().as_str() + "\n";
        let second_power_up = "2. Activate dealer handicap x".to_owned() + self.power_ups_count.1.to_string().as_str() + "\n";
        let text = available_power_ups + first_power_up.as_str() + second_power_up.as_str(); 

        
        let mut text_power_ups = graphics::Text::new(text.as_str());
        text_power_ups.set_font(font, graphics::PxScale::from(POWER_UPS_TEXT_SIZE));
        
        graphics::draw(ctx, &text_power_ups, graphics::DrawParam::default().dest(POWER_UPS_TEXT_POSITION))
    }

    fn draw_hint_text(&self, ctx: &mut Context) -> GameResult<()> {
        if self.hint_range.is_none() {
            return Ok(())
        }

        let begin = self.hint_range.unwrap().0;
        let end = self.hint_range.unwrap().1;

        let begin_str = begin.clone().to_string();
        let end_str = end.clone().to_string();
        let text = "NEXT CARD GIVES BETWEEN: ".to_owned() + begin_str.as_str() + "-" + end_str.as_str();

        let font = graphics::Font::new(ctx, "\\font\\DejaVuSerif.ttf")?;
        
        let mut hint_text = graphics::Text::new(text);
        hint_text.set_font(font, graphics::PxScale::from(HINT_TEXT_SIZE));
        
        graphics::draw(ctx, &hint_text, graphics::DrawParam::default().dest(HINT_TEXT_POSITION))
    }

    fn draw_wins(&self, ctx: &mut Context) -> GameResult<()> {
        let text = "WINS: ".to_owned() + self.wins.to_string().as_str();

        let font = graphics::Font::new(ctx, "\\font\\DejaVuSerif.ttf")?;
        
        let mut wins_text = graphics::Text::new(text);
        wins_text.set_font(font, graphics::PxScale::from(WINS_TEXT_SIZE));
        
        graphics::draw(ctx, &wins_text, graphics::DrawParam::default().dest(WINS_TEXT_POSITION))
    }

    fn draw_game_over_text(&self, ctx: &mut Context) -> GameResult<()> {
        let text;
        let color;
        match self.engine.outcome {
            Outcome::Win => {
                text = "YOU WIN!";
                color = graphics::Color::from_rgb(255, 163, 26);
            },
            Outcome::Draw => {
                text = "YOU DRAW!";
                color = graphics::Color::from_rgb(255, 255, 255);
            }
            Outcome::Lose => {
                text = "YOU LOSE!";
                color = graphics::Color::from_rgb(204, 0, 0);
            },
            _ => {
                text = "should not be possible";
                color = graphics::Color::from_rgb(0, 0, 0);
            },
        }
        
        let font = graphics::Font::new(ctx, "\\font\\DejaVuSerif.ttf")?;

        let game_over_text = graphics::TextFragment::new(text).
                                                     color(color).
                                                     font(font).
                                                     scale(graphics::PxScale::from(GAME_OVER_TEXT_SIZE));

        graphics::draw(ctx, &graphics::Text::new(game_over_text), graphics::DrawParam::default().dest(GAME_OVER_TEXT_POSITION))?;

        Ok(())
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        const DESIRED_FPS: u32 = 60;
        let time_delta = 1.0 / (DESIRED_FPS as f32);

        while timer::check_update_time(ctx, DESIRED_FPS) {
            match self.status {
                GameStatus::Menu => self.update_menu(ctx),
                GameStatus::Help => self.update_help(ctx),
                GameStatus::Play => self.update_game(ctx, time_delta)?,    
            }
        }

        Ok(())
    }

    fn key_down_event(&mut self,
                      ctx: &mut Context,
                      keycode: event::KeyCode,
                      _keymod: input::keyboard::KeyMods,
                      _repeat: bool) {
            match keycode {
                event::KeyCode::Space => self.board.turn = board::Turn::Dealer,
                event::KeyCode::Key1 => self.use_hint(),
                event::KeyCode::Key2 => self.use_handicap(),
                event::KeyCode::Escape => {
                    self.save();
                    event::quit(ctx)
                },
                _ => (), 
            }
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let casino_green = graphics::Color::from_rgb(21, 50, 30);
        graphics::clear(ctx, casino_green);

        match self.status {
            GameStatus::Menu => self.draw_menu(ctx)?,
            GameStatus::Help => self.draw_help(ctx)?,
            GameStatus::Play => {
                if self.time_till_game_over <= 0.0 {
                    self.draw_game_over_text(ctx)?;
                } else {
                    self.board.draw(ctx)?;
                    self.draw_score(ctx)?;
                    self.draw_power_ups(ctx)?;
                    self.draw_wins(ctx)?;
                    if matches!(self.engine.hint, HintStatus::Active) {
                        self.draw_hint_text(ctx)?
                    }
                }
            },
        }
        
        graphics::present(ctx)?;

        Ok(())
    }
}
