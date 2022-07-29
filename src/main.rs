#![windows_subsystem = "windows"]

use macroquad::{prelude::*};
use ::rand::{thread_rng, Rng};

const SIZE: Vec2 = const_vec2!([50f32, 50f32]);
const FIELD: Vec2 = const_vec2!([20f32, 10f32]);
const YOFFSET: f32 = 100f32;

pub enum GameState {
    Menu,
    Game,
    Won,
    Dead,
}

#[derive(PartialEq, Clone, Copy)]
pub enum Direction {
    North,
    South,
    West,
    East
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Snek".to_owned(),
        window_width: 1000i32,
        window_height: 600i32,
        window_resizable: false,
        ..Default::default()
    }
}

fn check_input(current_dir: Direction) -> Direction {
    let dir: Direction = match (is_key_down(KeyCode::Left) || is_key_down(KeyCode::A), is_key_down(KeyCode::Right) || is_key_down(KeyCode::D)
                         , is_key_down(KeyCode::Up) || is_key_down(KeyCode::W), is_key_down(KeyCode::Down) || is_key_down(KeyCode::S)) {
                            (true, false, false, false) => Direction::West,
                            (false, true, false, false) => Direction::East,
                            (false, false, true, false) => Direction::North,
                            (false, false, false, true) => Direction::South,
                            _ => current_dir,
                         };
    return dir;
}

struct SnakeHead {
    rect: Rect,
    last_dir: Direction,
    dir: Direction,
}

impl SnakeHead {
    pub fn new() -> Self {
        Self {
            rect: Rect::new(
                1f32,
                FIELD.y / 2.,
                SIZE.x,
                SIZE.y
            ),
            last_dir: Direction::East,
            dir: Direction::East
        }
    }
    
    pub fn set_dir(&mut self) {
        let tempdir: Direction = check_input(self.dir);
        if tempdir == Direction::East && self.last_dir != Direction::West {
            self.dir = tempdir;
        } else if tempdir == Direction::West && self.last_dir != Direction::East {
            self.dir = tempdir;
        } else if tempdir == Direction::North && self.last_dir != Direction::South {
            self.dir = tempdir;
        } else if tempdir == Direction::South && self.last_dir != Direction::North {
            self.dir = tempdir;
        }
    } 
    
    pub fn update(&mut self) {
        self.rect.y += match self.dir {
            Direction::North => -1f32,
            Direction::South => 1f32,
            _ => 0f32
        };
        self.rect.x += match self.dir {
            Direction::West => -1f32,
            Direction::East => 1f32,
            _ => 0f32
        };
        self.last_dir = self.dir;
    }
    
    pub fn draw(&self) {
        // draw_rectangle(self.rect.x, self.rect.y, self.rect.w, self.rect.h, GREEN);
        draw_rectangle(self.rect.x*50.-1., self.rect.y*50.-1.+YOFFSET, self.rect.w, self.rect.h, GREEN);
        match self.last_dir {
            Direction::East => draw_rectangle(self.rect.x*50.-1.+35., self.rect.y*50.-1.+5.+YOFFSET, 10., 10., BLACK),
            Direction::North => draw_rectangle(self.rect.x*50.-1.+5., self.rect.y*50.-1.+5.+YOFFSET, 10., 10., BLACK),
            Direction::West => draw_rectangle(self.rect.x*50.-1.+5., self.rect.y*50.-1.+35.+YOFFSET, 10., 10., BLACK),
            Direction::South => draw_rectangle(self.rect.x*50.-1.+35., self.rect.y*50.-1.+35.+YOFFSET, 10., 10., BLACK)
        }
    }
}

struct SnakeBody {
    rect: Rect,
}

impl SnakeBody {
    pub fn new() -> Self {
        Self {
            rect: Rect::new(
                -1f32,
                -1f32,
                SIZE.x,
                SIZE.y
            ),
        }
    }

    pub fn update(&mut self, new_x: f32, new_y: f32) {
        self.rect.x = new_x;
        self.rect.y = new_y;
    }

    pub fn draw(&self) {
        draw_rectangle(self.rect.x*50.-1., self.rect.y*50.-1.+YOFFSET, self.rect.w, self.rect.h, GREEN);
    }
}

pub struct Apple {
    rect: Rect
}

impl Apple {
    pub fn new() -> Self {
        Self {
            rect: Rect::new(
                17f32,
                FIELD.y / 2.,
                SIZE.x,
                SIZE.y,
            )
        }
    }

    pub fn respawn(&mut self, new_x: f32, new_y: f32) {
        (self.rect.x, self.rect.y) = (new_x, new_y);
    }

    pub fn draw(&self) {
        draw_rectangle(self.rect.x*50.-1., self.rect.y*50.-1.+YOFFSET, self.rect.w, self.rect.h, RED);
    }
}

fn draw_info_table() {
    draw_line(0f32, 99f32, screen_width(), 100f32, 1f32, BLACK);
    draw_rectangle(0f32, 0f32, screen_width(), 100f32, DARKGRAY);
}

fn draw_grid() {
    let x_lines: i32 = (screen_width() / 50f32) as i32;
    for i in 0..x_lines {
        let x: f32 = i as f32;
        draw_line(x * 50f32 - 1f32, YOFFSET, x * 50f32 - 1f32, screen_height(), 3f32, BLACK)
    }
    let y_lines: i32 = ((screen_height() - YOFFSET) / 50f32) as i32;
    for i in 0..y_lines {
        let y: f32 = i as f32;
        draw_line(0f32, YOFFSET + y * 50f32 - 1f32, screen_width(), YOFFSET + y * 50f32 - 1f32, 3f32, BLACK);
    }
}

pub fn draw_title_text(text: &str, font: Font) {
    let dims = measure_text(text, Some(font), 50u16, 1.0f32);
    draw_text_ex(
        text,
        screen_width() * 0.5f32 - dims.width * 0.5f32,
        YOFFSET / 2.,
        TextParams {font: font, font_size: 50u16, color: DARKGREEN, ..Default::default()}
    );
}


#[macroquad::main(window_conf())]
async fn main() {
    let font = load_ttf_font("res/Heebo-VariableFont_wght.ttf").await.unwrap();
    let mut game_state: GameState = GameState::Menu;
    let tickrate: f32 = 1. / 7.5;
    let mut frame_time_till_tick:f32 = 0.;
    let mut rng = thread_rng();

    // init objects
    let mut head = SnakeHead::new();
    let mut body_parts: Vec<SnakeBody> = Vec::new();
    let mut apfel = Apple::new();
    let mut score: u32 = 0;
    
    loop {
        clear_background(GRAY);
        draw_info_table();
        draw_text_ex(
            &format!("Score: {score}"),
            10f32,
            50.0,
            TextParams{font, font_size: 50u16, color:WHITE, ..Default::default()}
        );
        match game_state {
            GameState::Menu => {
                draw_title_text("Press Space to Start!", font);
                head = SnakeHead::new();
                body_parts = Vec::new();
                apfel = Apple::new();
                score = 0;
                if is_key_pressed(KeyCode::Space) {
                    game_state = GameState::Game;
                }
            },
            GameState::Game => {
                frame_time_till_tick += get_frame_time();

                // debug
                // println!("{}", frame_time_till_tick);
                // println!("X: {}, Y: {}", head.rect.x, head.rect.y);

                head.set_dir();
                
                if is_key_pressed(KeyCode::Space) {
                    body_parts.push(SnakeBody::new());
                    println!("Space Pressed!");
                    score += 1;
                }
                
                // simulating slower tickrate
                if frame_time_till_tick >= tickrate {
                    if head.dir == Direction::West && head.rect.x <= 0f32 || head.dir == Direction::East && head.rect.x >= FIELD.x - 1f32 || head.dir == Direction::North && head.rect.y <= 0f32 || head.dir == Direction::South && head.rect.y >= FIELD.y - 1f32 {
                        game_state = GameState::Dead;
                        continue;
                    }

                    if !body_parts.is_empty() {
                        for i in (0..body_parts.len()).rev() {
                            if i == 0 {
                                body_parts[i].update(head.rect.x, head.rect.y);
                            } else {
                                let x: f32 = body_parts[i-1].rect.x;
                                let y: f32 = body_parts[i-1].rect.y;
                                body_parts[i].update(x, y);
                            }
                        }
                    }

                    head.update();
                    

                    // Check Apple Collision
                    if head.rect.x == apfel.rect.x && head.rect.y == apfel.rect.y {
                        let mut x: f32;
                        let mut y: f32;
                        loop {
                            let mut check = true;
                            x = rng.gen_range(0..20) as f32;
                            y = rng.gen_range(0..10) as f32;

                            for part in body_parts.iter() {
                                if x == part.rect.x && y == part.rect.y {
                                    check = false;
                                    println!("set new apple location");
                                }
                            }
                            if check{ break; }
                        }
                        apfel.respawn(x, y);
                        body_parts.push(SnakeBody::new());
                        score += 1;
                    }

                    for part in body_parts.iter() {
                        if head.rect.x == part.rect.x && head.rect.y == part.rect.y {
                            game_state = GameState::Dead;
                            break;
                        }
                    }
                    
                    if score >= 198 {
                        game_state = GameState::Won;
                    }

                    frame_time_till_tick = tickrate - frame_time_till_tick;
                }
            },
            GameState::Won => {
                draw_title_text("You WON the Game, Space to restart", font);
                if is_key_pressed(KeyCode::Space) {
                    game_state = GameState::Menu
                }
            },
            GameState::Dead => {
                draw_title_text("You Died, Space to restart", font);
                if is_key_pressed(KeyCode::Space) {
                    game_state = GameState::Menu;
                }
            }
        }
        
        head.draw();        
        for part in body_parts.iter() {
            part.draw();
        }

        apfel.draw();
        draw_grid();

        // println!("{}", get_fps());
        next_frame().await;
    }
}
