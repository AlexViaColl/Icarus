use icarus::glyph::{Glyph, GLYPHS, GLYPH_WIDTH};
use icarus::input::{ButtonId, InputState, KeyId};
use icarus::math::{Rect, Vec2};
use icarus::platform::{Config, Platform};
use icarus::vk::*;
use icarus::vk_util::{self, /*RenderCommand*/ VkContext};

use std::mem;
use std::time::Instant;

const MAX_FRAMES_IN_FLIGHT: usize = 2;
const WINDOW_WIDTH: f32 = 1600.0;
const WINDOW_HEIGHT: f32 = 900.0;
const MAX_ENTITIES: usize = 400;

const PLAYER_COUNT: usize = 2;
const PLAYER_2_AI: bool = true;
const PLAYER_COLOR: [(f32, f32, f32); PLAYER_COUNT] = [(1.0, 1.0, 1.0), (1.0, 0.0, 0.0)];

pub struct Game {
    pub running: bool,
    pub state: GameState,
    pub entities: Vec<Entity>,
    pub player: usize,
    pub tiles: [Option<usize>; 9],
    pub render_commands: Vec<RenderCommand>,
}
#[derive(PartialEq)]
pub enum GameState {
    Playing,
    Draw,
    Win(usize),
}
#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct Entity {
    pub transform: Transform,
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct Transform {
    pub pos: Vec2,
    pub size: Vec2,
}
#[repr(C)]
#[derive(Debug)]
pub struct GlobalState {
    width: u32,
    height: u32,
}

#[derive(Debug)]
pub enum RenderCommand {
    Rect(f32, f32, f32, f32, f32, f32, f32), // 7 * 4 = 28 bytes
}

#[repr(C)]
#[derive(Default)]
pub struct Vertex {
    pos: (f32, f32, f32),   // 12
    color: (f32, f32, f32), // 12
    uv: (f32, f32),         // 8
}

impl Vertex {
    fn get_binding_description() -> VkVertexInputBindingDescription {
        VkVertexInputBindingDescription {
            binding: 0,
            stride: mem::size_of::<Self>() as u32,
            inputRate: VK_VERTEX_INPUT_RATE_VERTEX,
        }
    }

    fn get_attribute_descriptions() -> [VkVertexInputAttributeDescription; 3] {
        [
            VkVertexInputAttributeDescription {
                binding: 0,
                location: 0,
                format: VK_FORMAT_R32G32B32_SFLOAT,
                offset: 0,
            },
            VkVertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: VK_FORMAT_R32G32B32_SFLOAT,
                offset: 3 * mem::size_of::<f32>() as u32,
            },
            VkVertexInputAttributeDescription {
                binding: 0,
                location: 2,
                format: VK_FORMAT_R32G32_SFLOAT,
                offset: (3 + 3) * mem::size_of::<f32>() as u32,
            },
        ]
    }
}

fn main() {
    #[rustfmt::skip]
    let vertices = [                                                            // CCW
        Vertex {pos: (-1.0, -1.0, 0.0), uv: (0.0, 0.0), color: (1.0, 1.0, 1.0), ..Vertex::default() },  // Top left
        Vertex {pos: (-1.0,  1.0, 0.0), uv: (0.0, 1.0), color: (1.0, 1.0, 1.0),..Vertex::default() },  // Bottom left
        Vertex {pos: ( 1.0,  1.0, 0.0), uv: (1.0, 1.0), color: (1.0, 1.0, 1.0),..Vertex::default() },  // Bottom right
        Vertex {pos: ( 1.0, -1.0, 0.0), uv: (1.0, 0.0), color: (1.0, 1.0, 1.0),..Vertex::default() },  // Top right
    ];
    let indices = [0, 1, 2, 2, 3, 0];

    let mut platform = Platform::init(Config {
        width: 1600,
        height: 900,
        app_name: String::from("Tic-Tac-Toe"),
    });
    let mut input = InputState::default();
    let mut game = Game::init();
    let mut vk_ctx = VkContext::init(
        &platform,
        mem::size_of::<Entity>() * MAX_ENTITIES,
        8, //mem::size_of::<GlobalState>(),
        Vertex::get_binding_description(),
        &Vertex::get_attribute_descriptions(),
    );
    vk_ctx.vertex_buffer = vk_util::create_vertex_buffer(&vk_ctx, &vertices);
    vk_ctx.index_buffer = vk_util::create_index_buffer(&vk_ctx, &indices);

    // Main loop
    let mut current_frame = 0;
    let start_time = Instant::now();
    let mut prev_frame_time = start_time;
    while game.running {
        input.reset_transitions();
        platform.process_messages(&mut input);

        let seconds_elapsed = prev_frame_time.elapsed().as_secs_f32();
        prev_frame_time = Instant::now();
        game.update(&input, seconds_elapsed);
        game.render();

        vk_ctx.render(game.render_commands.as_slice(), current_frame, indices.len());
        current_frame = (current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }

    vk_ctx.cleanup(&platform);
}

pub fn create_entity(game: &mut Game, transform: (f32, f32, f32, f32)) {
    game.entities.push(Entity {
        transform: Transform {
            pos: Vec2::new(transform.0, transform.1),
            size: Vec2::new(transform.2, transform.3),
        },
        ..Entity::default()
    });
}

pub fn push_rect<R: Into<Rect>>(render_commands: &mut Vec<RenderCommand>, r: R) {
    let r = r.into();
    render_commands.push(RenderCommand::Rect(r.offset.x, r.offset.y, r.extent.x, r.extent.y, 1.0, 1.0, 1.0));
}
pub fn push_rect_color<R: Into<Rect>>(render_commands: &mut Vec<RenderCommand>, r: R, c: (f32, f32, f32)) {
    let r = r.into();
    render_commands.push(RenderCommand::Rect(r.offset.x, r.offset.y, r.extent.x, r.extent.y, c.0, c.1, c.2));
}
pub const GLYPH_PIXEL_SIZE: f32 = 10.0;
pub fn push_glyph(cmd: &mut Vec<RenderCommand>, glyph: &Glyph, x: f32, y: f32, pixel_size: f32) {
    for row in 0..7 {
        for col in 0..5 {
            if glyph[row * 5 + col] != 0 {
                push_rect(
                    cmd,
                    Rect::offset_extent(
                        (x + pixel_size * (col as f32), y + pixel_size * (row as f32)),
                        (pixel_size, pixel_size),
                    ),
                );
            }
        }
    }
}
pub fn push_char(cmd: &mut Vec<RenderCommand>, c: char, x: f32, y: f32, pixel_size: f32) {
    assert!(c >= ' ' && c <= '~');
    let glyph_idx = c as usize - ' ' as usize;
    push_glyph(cmd, &GLYPHS[glyph_idx], x, y, pixel_size);
}
pub fn push_str(cmd: &mut Vec<RenderCommand>, s: &str, _x: f32, y: f32, pixel_size: f32) {
    let text_extent = (s.len() as f32) * 6.0 * pixel_size;
    let x = WINDOW_WIDTH / 2.0 - text_extent / 2.0;
    for (idx, c) in s.chars().enumerate() {
        push_char(cmd, c, x + (idx as f32) * pixel_size * (GLYPH_WIDTH as f32 + 1.0), y, pixel_size);
    }
}

impl Game {
    fn init() -> Self {
        Self {
            running: true,
            state: GameState::Playing,
            player: 0,
            entities: vec![],
            tiles: [None; 9],
            render_commands: vec![],
        }
    }

    // Advances the state of the game by dt seconds.
    fn update(&mut self, input: &InputState, _dt: f32) {
        if input.was_key_pressed(KeyId::Esc) {
            self.running = false;
            return;
        }

        if self.state != GameState::Playing {
            if input.was_key_pressed(KeyId::Any) {
                self.state = GameState::Playing;
                self.player = 0;
                self.tiles = [None; 9];
            }
            return;
        }

        if self.player == 1 && PLAYER_2_AI {
            let pieces_placed = self.tiles.iter().filter(|t| t.is_some()).count();
            match pieces_placed {
                1 => {
                    if self.tiles[4] == None {
                        self.tiles[4] = Some(self.player); // Pick the middle
                    } else {
                        self.tiles[0] = Some(self.player); // Pick any corner
                    }
                }
                3 | 5 | 7 => {
                    place_blocking(self);
                }
                n => panic!("Unreachable {}", n),
            }

            self.player = (self.player + 1) % PLAYER_COUNT;

            // Check for winner
            // Row complete
            if (self.tiles[0].is_some() && self.tiles[0] == self.tiles[1] && self.tiles[1] == self.tiles[2]) ||
                (self.tiles[3].is_some() && self.tiles[3] == self.tiles[4] && self.tiles[4] == self.tiles[5]) ||
                (self.tiles[6].is_some() && self.tiles[6] == self.tiles[7] && self.tiles[7] == self.tiles[8]) ||
                // Column complete
                (self.tiles[0].is_some() && self.tiles[0] == self.tiles[3] && self.tiles[3] == self.tiles[6]) ||
                (self.tiles[1].is_some() && self.tiles[1] == self.tiles[4] && self.tiles[4] == self.tiles[7]) ||
                (self.tiles[2].is_some() && self.tiles[2] == self.tiles[5] && self.tiles[5] == self.tiles[8]) ||
                // Diagonal complete
                (self.tiles[0].is_some() && self.tiles[0] == self.tiles[4] && self.tiles[4] == self.tiles[8]) ||
                (self.tiles[2].is_some() && self.tiles[2] == self.tiles[4] && self.tiles[4] == self.tiles[6])
            {
                self.state = GameState::Win((self.player + 1) % PLAYER_COUNT);
            } else if self.tiles.iter().all(|x| x.is_some()) {
                self.state = GameState::Draw;
            }
        }

        if self.player == 0 && input.was_button_pressed(ButtonId::Left) {
            let button = input.buttons[ButtonId::Left as usize];
            let center_x = WINDOW_WIDTH / 2.0;
            let center_y = WINDOW_HEIGHT / 2.0;
            let square_dim = WINDOW_HEIGHT / 3.0;

            for idx in 0..9 {
                let row = (idx / 3) as f32;
                let col = (idx % 3) as f32;
                let x_start = center_x - square_dim + col * square_dim;
                let y_start = center_y - square_dim + row * square_dim;
                let rect = Rect::center_extent((x_start, y_start), (0.8 * square_dim, 0.8 * square_dim));
                if self.tiles[idx].is_none() && rect.is_inside((button.x as f32, button.y as f32)) {
                    self.tiles[idx] = Some(self.player);
                    self.player = (self.player + 1) % PLAYER_COUNT;
                    break;
                }
            }

            // Check for winner
            // Row complete
            if (self.tiles[0].is_some() && self.tiles[0] == self.tiles[1] && self.tiles[1] == self.tiles[2]) ||
                (self.tiles[3].is_some() && self.tiles[3] == self.tiles[4] && self.tiles[4] == self.tiles[5]) ||
                (self.tiles[6].is_some() && self.tiles[6] == self.tiles[7] && self.tiles[7] == self.tiles[8]) ||
                // Column complete
                (self.tiles[0].is_some() && self.tiles[0] == self.tiles[3] && self.tiles[3] == self.tiles[6]) ||
                (self.tiles[1].is_some() && self.tiles[1] == self.tiles[4] && self.tiles[4] == self.tiles[7]) ||
                (self.tiles[2].is_some() && self.tiles[2] == self.tiles[5] && self.tiles[5] == self.tiles[8]) ||
                // Diagonal complete
                (self.tiles[0].is_some() && self.tiles[0] == self.tiles[4] && self.tiles[4] == self.tiles[8]) ||
                (self.tiles[2].is_some() && self.tiles[2] == self.tiles[4] && self.tiles[4] == self.tiles[6])
            {
                self.state = GameState::Win((self.player + 1) % PLAYER_COUNT);
            } else if self.tiles.iter().all(|x| x.is_some()) {
                self.state = GameState::Draw;
            }
        }
    }

    // Render the current state of the game.
    fn render(&mut self) {
        self.render_commands.clear();

        match self.state {
            GameState::Win(player) => {
                push_str(
                    &mut self.render_commands,
                    &format!("Player {} Won!", player + 1),
                    0.0,
                    WINDOW_HEIGHT / 2.0 - 150.0,
                    15.0,
                );
                push_str(
                    &mut self.render_commands,
                    &format!("Press any key to start"),
                    0.0,
                    WINDOW_HEIGHT / 2.0 + 100.0,
                    8.0,
                );
            }
            GameState::Draw => {
                push_str(&mut self.render_commands, &format!("Draw!"), 0.0, WINDOW_HEIGHT / 2.0 - 150.0, 15.0);
                push_str(
                    &mut self.render_commands,
                    &format!("Press any key to start"),
                    0.0,
                    WINDOW_HEIGHT / 2.0 + 100.0,
                    8.0,
                );
            }
            GameState::Playing => render_board(self),
        }
    }
}

fn render_board(game: &mut Game) {
    let cmd = &mut game.render_commands;
    let center_x = WINDOW_WIDTH / 2.0;
    let center_y = WINDOW_HEIGHT / 2.0;

    let square_dim = WINDOW_HEIGHT / 3.0;
    let half_square_dim = 0.5 * square_dim;
    let bar_dim = 0.05 * square_dim;

    // Horizontal bars
    push_rect(cmd, Rect::center_extent((center_x, center_y - half_square_dim), (3.0 * square_dim, bar_dim)));
    push_rect(cmd, Rect::center_extent((center_x, center_y + half_square_dim), (3.0 * square_dim, bar_dim)));

    // Vertical bars
    push_rect(cmd, Rect::center_extent((center_x - half_square_dim, center_y), (bar_dim, 3.0 * square_dim)));
    push_rect(cmd, Rect::center_extent((center_x + half_square_dim, center_y), (bar_dim, 3.0 * square_dim)));

    // Pieces
    for idx in 0..9 {
        let row = (idx / 3) as f32;
        let col = (idx % 3) as f32;
        match game.tiles[idx] {
            Some(player) => push_rect_color(
                cmd,
                Rect::center_extent(
                    (center_x - square_dim + col * square_dim, center_y - square_dim + row * square_dim),
                    (0.8 * square_dim, 0.8 * square_dim),
                ),
                PLAYER_COLOR[player],
            ),
            _ => {}
        }
    }
}

fn place_naive(game: &mut Game) {
    for tile in game.tiles.iter_mut() {
        if tile.is_none() {
            *tile = Some(game.player);
            break;
        }
    }
}

fn place_prefer_corners(game: &mut Game) {
    for idx in [0, 2, 6, 8] {
        if game.tiles[idx].is_none() {
            game.tiles[idx] = Some(game.player);
            return;
        }
    }
    place_naive(game);
}

fn place_blocking(game: &mut Game) {
    if let Some(idx) = get_win_tile(game, 1) {
        game.tiles[idx] = Some(1);
    } else if let Some(idx) = get_win_tile(game, 0) {
        game.tiles[idx] = Some(1);
    } else {
        // Prefer diagonal, adjacent to the other player
        if (game.tiles[5] == Some(0) || game.tiles[7] == Some(0)) && game.tiles[8] == None {
            game.tiles[8] = Some(1);
        } else {
            place_prefer_corners(game);
        }
    }
}

fn get_win_tile(game: &Game, player: usize) -> Option<usize> {
    // Rows
    for row in 0..3 {
        if game.tiles[row * 3 + 0] == Some(player)
            && game.tiles[row * 3 + 1] == Some(player)
            && game.tiles[row * 3 + 2] == None
        {
            return Some(row * 3 + 2);
        }
        if game.tiles[row * 3 + 0] == Some(player)
            && game.tiles[row * 3 + 1] == None
            && game.tiles[row * 3 + 2] == Some(player)
        {
            return Some(row * 3 + 1);
        }
        if game.tiles[row * 3 + 0] == None
            && game.tiles[row * 3 + 1] == Some(player)
            && game.tiles[row * 3 + 2] == Some(player)
        {
            return Some(row * 3 + 0);
        }
    }

    // Columns
    for col in 0..3 {
        if game.tiles[0 * 3 + col] == Some(player)
            && game.tiles[1 * 3 + col] == Some(player)
            && game.tiles[2 * 3 + col] == None
        {
            return Some(2 * 3 + col);
        }
        if game.tiles[0 * 3 + col] == Some(player)
            && game.tiles[1 * 3 + col] == None
            && game.tiles[2 * 3 + col] == Some(player)
        {
            return Some(1 * 3 + col);
        }
        if game.tiles[0 * 3 + col] == None
            && game.tiles[1 * 3 + col] == Some(player)
            && game.tiles[2 * 3 + col] == Some(player)
        {
            return Some(0 * 3 + col);
        }
    }

    // Diagonals
    if game.tiles[0] == Some(player) && game.tiles[4] == Some(player) && game.tiles[8] == None {
        return Some(8);
    }
    if game.tiles[0] == Some(player) && game.tiles[4] == None && game.tiles[8] == Some(player) {
        return Some(4);
    }
    if game.tiles[0] == None && game.tiles[4] == Some(player) && game.tiles[8] == Some(player) {
        return Some(0);
    }

    if game.tiles[2] == Some(player) && game.tiles[4] == Some(player) && game.tiles[6] == None {
        return Some(6);
    }
    if game.tiles[2] == Some(player) && game.tiles[4] == None && game.tiles[6] == Some(player) {
        return Some(4);
    }
    if game.tiles[2] == None && game.tiles[4] == Some(player) && game.tiles[6] == Some(player) {
        return Some(2);
    }

    None
}
