use std::collections::VecDeque;

use bracket_lib::prelude::*;
enum GameMode {
    Menu,
    Playing,
    Over,
}

struct Player {
    x: i32,
    y: i32,
    velocity: f32,
}

struct Obstacle {
    game_x: i32,
    gap_mid_y: i32,
    gap_size: i32,
}

struct State {
    mode: GameMode,
    player: Player,
    frame_time: f32,
    obstacles: VecDeque<Obstacle>,
    score: i32,
}

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const FRAME_DURATION: f32 = 75.0;

impl Obstacle {
    fn new(x: i32, score: i32) -> Self {
        let mut rng = RandomNumberGenerator::new();
        Self {
            game_x: x,
            gap_mid_y: rng.range(10, SCREEN_HEIGHT - 10),
            gap_size: i32::max(2, 20 - score),
        }
    }
    fn render(&self, ctx: &mut BTerm, player_x: i32) {
        let screen_x = self.game_x - player_x;
        if !(0..=SCREEN_WIDTH).contains(&screen_x) {
            return;
        }
        for y in 0..SCREEN_HEIGHT {
            if y < self.gap_mid_y - self.gap_size / 2 || y > self.gap_mid_y + self.gap_size / 2 {
                ctx.set(screen_x, y, RED, BLACK, to_cp437('|'));
            }
        }
    }

    fn player_hit(&self, player: &Player) -> bool {
        let half_size = self.gap_size / 2;

        if player.x == self.game_x
            && (player.y < self.gap_mid_y - half_size || player.y > self.gap_mid_y + half_size)
        {
            return true;
        }
        false
    }
}

impl Player {
    fn new(x: i32, y: i32) -> Self {
        Self {
            x,
            y,
            velocity: 0.0,
        }
    }
    fn render(&self, ctx: &mut BTerm) {
        ctx.set(0, self.y, YELLOW, BLACK, to_cp437('@'));
    }

    fn update(&mut self) {
        if self.velocity < 2.0 {
            self.velocity += 1.0;
        }
        self.y += self.velocity as i32;
        self.x += 1;
        if self.y < 0 {
            self.y = 0;
        }
    }

    fn flap(&mut self) {
        self.velocity = -2.0;
    }
}

impl State {
    fn new() -> Self {
        Self {
            mode: GameMode::Menu,
            player: Player::new(5, 25),
            frame_time: 0.0,
            obstacles: VecDeque::new(),
            score: 0,
        }
    }

    fn main_menu(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(4, "Welcome to bb game!");
        ctx.print_centered(6, "Press [P] to start");
        ctx.print_centered(9, "Press [Q] to exit");
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }

    fn restart(&mut self) {
        self.mode = GameMode::Playing;
        self.player = Player::new(5, 25);
        self.frame_time = 0.0;
        self.obstacles.clear();
        self.score = 0;
    }

    fn pop_obstacle(&mut self) -> bool {
        if !self.obstacles.is_empty() && self.obstacles[0].game_x < self.player.x {
            self.obstacles.pop_front();
            return true;
        }
        false
    }

    fn push_obstacle(&mut self) {
        if self.obstacles.is_empty() {
            self.obstacles
                .push_back(Obstacle::new(self.player.x + SCREEN_WIDTH, self.score));
        } else {
            let last_obstacle = &self.obstacles[self.obstacles.len() - 1];
            if last_obstacle.game_x >= self.player.x
                && last_obstacle.game_x < self.player.x + SCREEN_WIDTH
            {
                let mut rng = RandomNumberGenerator::new();
                let obstacle_game_pos = i32::max(
                    self.player.x + SCREEN_WIDTH,
                    last_obstacle.game_x + rng.range(i32::min(10, 20 - self.score), 20),
                );
                self.obstacles
                    .push_back(Obstacle::new(obstacle_game_pos, self.score));
            }
        }
    }

    fn play(&mut self, ctx: &mut BTerm) {
        ctx.cls_bg(NAVY);
        self.frame_time += ctx.frame_time_ms;
        if self.frame_time > FRAME_DURATION {
            self.frame_time = 0.0;
            self.player.update();
        }

        if self.pop_obstacle() {
            self.score += 1;
        }

        self.push_obstacle();

        if let Some(VirtualKeyCode::Space) = ctx.key {
            self.player.flap();
        }
        self.player.render(ctx);

        ctx.print(
            0,
            0,
            format!("Press SPACE to flap... Score: {}..!", self.score),
        );

        if self.player.y > SCREEN_HEIGHT {
            self.mode = GameMode::Over;
        }

        for obstacle in self.obstacles.iter() {
            obstacle.render(ctx, self.player.x);
            if obstacle.player_hit(&self.player) {
                self.mode = GameMode::Over;
            }
        }
    }

    fn game_over(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, format!("You are dead!.. Score: {}..!", self.score));
        ctx.print_centered(6, "Press [P] to start");
        ctx.print_centered(9, "Press [Q] to exit");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print(1, 1, "ola bb!");

        match self.mode {
            GameMode::Menu => self.main_menu(ctx),
            GameMode::Playing => self.play(ctx),
            GameMode::Over => self.game_over(ctx),
        }
    }
}

fn main() -> BError {
    let ctx = BTermBuilder::simple(SCREEN_WIDTH, SCREEN_HEIGHT)?
        .with_title("bb game..")
        .build()?;
    main_loop(ctx, State::new())
}
