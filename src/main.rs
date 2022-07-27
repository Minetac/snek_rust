// #![windows_subsystem = "windows"]

use macroquad::{prelude::*};
use ::rand::{thread_rng, Rng};


const SIZE: Vec2 = const_vec2!([50f32, 50f32]);
const YOFFSET: f32 = 100f32;

pub enum GameState {
    Menu,
    Game,
    Won,
    Dead,
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

fn check_input(currentDir: char) -> char {
    let dir: char = match (is_key_down(KeyCode::Left) || is_key_down(KeyCode::A), is_key_down(KeyCode::Right) || is_key_down(KeyCode::D)
                         , is_key_down(KeyCode::Up) || is_key_down(KeyCode::W), is_key_down(KeyCode::Down) || is_key_down(KeyCode::S)) {
                            (true, false, false, false) => 'w',
                            (false, true, false, false) => 'e',
                            (false, false, true, false) => 'n',
                            (false, false, false, true) => 's',
                            _ => currentDir,
                         };
    return dir;
}

struct SnakeHead {
    rect: Rect,
    last_dir: char,
    dir: char,
}

impl SnakeHead {
    pub fn new() -> Self {
        Self {
            rect: Rect::new(
                50f32 - 1f32,
                screen_height() * 0.5f32 + YOFFSET - 51f32,
                SIZE.x,
                SIZE.y
            ),
            last_dir: 'e',
            dir: 'e'
        }
    }
    pub fn set_dir(&mut self) {
        let tempdir: char = check_input(self.dir);
        if tempdir == 'e' && self.last_dir != 'w' {
            self.dir = tempdir;
        } else if tempdir == 'w' && self.last_dir != 'e' {
            self.dir = tempdir;
        } else if tempdir == 'n' && self.last_dir != 's' {
            self.dir = tempdir;
        } else if tempdir == 's' && self.last_dir != 'n' {
            self.dir = tempdir;
        }
    } 
    
    pub fn update(&mut self) {
        self.rect.y += match self.dir {
            'n' => -1f32,
            's' => 1f32,
            _ => 0f32
        } * SIZE.y;
        self.rect.x += match self.dir {
            'w' => -1f32,
            'e' => 1f32,
            _ => 0f32
        } * SIZE.x;
        self.last_dir = self.dir;
    }
    
    pub fn draw(&self) {
        draw_rectangle(self.rect.x, self.rect.y, self.rect.w, self.rect.h, GREEN);
    }
}

struct SnakeBody {
    rect: Rect,
}

impl SnakeBody {
    pub fn new() -> Self {
        Self {
            rect: Rect::new(
                0f32,
                0f32,
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
        draw_rectangle(self.rect.x, self.rect.y, self.rect.w, self.rect.h, GREEN);
    }
}

pub struct Apple {
    rect: Rect
}

impl Apple {
    pub fn new() -> Self {
        Self {
            rect: Rect::new(
                850f32 - 1f32,
                screen_height() * 0.5f32 + YOFFSET - 51f32,
                SIZE.x,
                SIZE.y,
            )
        }
    }

    pub fn respawn(&mut self, new_x: f32, new_y: f32) {
        (self.rect.x, self.rect.y) = (new_x, new_y);
    }

    pub fn draw(&self) {
        draw_rectangle(self.rect.x, self.rect.y, self.rect.w, self.rect.h, RED);
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

fn resolve_collision(a: &mut Rect, vel: &mut Vec2, b: &Rect) -> bool {
    
    // early exit
    let intersection = match a.intersect(*b) {
        Some(intersection) => intersection,
        None => return false,
    };

    let a_center = a.point() + a.size() * 0.5f32;
    let b_center = b.point() + b.size() * 0.5f32;
    
    let to = b_center - a_center;
    let to_signum = to.signum(); 

    if intersection.w > intersection.h {
        // bounce on y
        a.y -= to_signum.y * intersection.h;
        match to_signum.y > 0f32 {
            true => vel.y = -vel.y.abs(),
            false => vel.y = vel.y.abs(),
        }
        // vel.y = -to_signum.y * vel.y.abs();    
    } else {
        // bounce on x
        a.x -= to_signum.x * intersection.w;
        match to_signum.x < 0f32 {
            true => vel.x = vel.x.abs(),
            false => vel.x = -vel.x.abs(),
        }
        // vel.x = -to_signum.x * vel.x.abs();
    }
    true
}

#[macroquad::main(window_conf())]
async fn main() {
    let mut game_state: GameState = GameState::Menu;
    let tickrate: f32 = 1. / 10.;
    let mut head = SnakeHead::new();
    let mut frame_time_till_tick:f32 = 0.;
    let mut body_parts: Vec<SnakeBody> = Vec::new();
    let mut apfel = Apple::new();

    let mut rng = thread_rng();

    let font = load_ttf_font("res/Heebo-VariableFont_wght.ttf").await.unwrap();
    let mut score: u32 = 0;
    
    loop {
        match game_state {
            GameState::Menu => {
                if is_key_pressed(KeyCode::Space) {
                    game_state = GameState::Game;
                }
            },
            GameState::Game => {
                frame_time_till_tick += get_frame_time();
                // println!("{}", frame_time_till_tick);
                // println!("X: {}, Y: {}", head.rect.x, head.rect.y);
                head.set_dir();
                
                if is_key_pressed(KeyCode::Space) {
                    body_parts.push(SnakeBody::new());
                    println!("Space Pressed!");
                }
                
                if frame_time_till_tick >= tickrate {
                    if head.dir == 'w' && head.rect.x <= -1f32 || head.dir == 'e' && head.rect.x >= screen_width() - 1f32 - SIZE.x || head.dir == 'n' && head.rect.y <= YOFFSET || head.dir == 's' && head.rect.y >= screen_height() - SIZE.y  - 1f32{
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
                    
                    if head.rect.x == apfel.rect.x && head.rect.y == apfel.rect.y {
                        apfel.respawn(rng.gen_range(0..20) as f32 * SIZE.x - 1f32, rng.gen_range(0..10) as f32 * SIZE.y - 1f32 + YOFFSET);
                        body_parts.push(SnakeBody::new());
                        score += 1;
                    }

                    for part in body_parts.iter() {
                        if head.rect.x == part.rect.x && head.rect.y == part.rect.y {
                            game_state = GameState::Dead;
                        }
                    }
                    
                    frame_time_till_tick = tickrate - frame_time_till_tick;
                }
            },
            GameState::Won => {
                
            },
            GameState::Dead => {
                if is_key_pressed(KeyCode::Space) {
                    game_state = GameState::Game;
                }
            }
        }
        clear_background(GRAY);
        
        
        head.draw();        
        for part in body_parts.iter() {
            part.draw();
        }
        apfel.draw();
        draw_info_table();
        draw_grid();
        let score_text = &format!("Score: {score}");
        let score_text_dim = measure_text(&score_text, Some(font), 30u16, 1.0);
        
        draw_text_ex(
            score_text,
            10f32,
            40.0,
            TextParams{font, font_size: 30u16, color:WHITE, ..Default::default()}
        );

        // thread::sleep(Duration::from_millis(1000));
        // println!("{}", get_fps());
        next_frame().await;
    }
}
