#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use rand::Rng;
use std::io::{stdout, Write};
use std::process::exit;
use std::thread;
use std::time;

use crossterm::{
    cursor,
    event::{poll, read, Event, KeyCode},
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
    terminal, ExecutableCommand, Result,
};

const FIELD_XMARGIN: usize = 4;
const FIELD_YMARGIN: usize = 2;
const FIELD_XMAX: usize = 40;
const FIELD_YMAX: usize = 20;
const FIELD_PIXELS: usize = (FIELD_XMAX * FIELD_YMAX);
const PIXEL_BORDER: u8 = b'#';
const SNAKE_MIN_LEN: usize = 4;
const SNAKE_MAX_LEN: usize = FIELD_PIXELS - 2 * FIELD_XMAX - 2 * (FIELD_YMAX - 2);

type SnakePixel = Option<[usize; 2]>;

enum SnakeOrientation {
    North,
    East,
    South,
    West,
}

fn main() -> Result<()> {
    let _ = terminal::enable_raw_mode()?;
    let mut rng = rand::thread_rng();
    let mut stdout = stdout();

    let mut field: [u8; FIELD_PIXELS] = [0; FIELD_PIXELS];
    let mut snake: [SnakePixel; SNAKE_MAX_LEN] = [None; SNAKE_MAX_LEN];
    let mut snake_orientation: SnakeOrientation = SnakeOrientation::East;
    let mut snake_speed: usize = 0;
    let mut snake_crashed: bool = false;

    let game_tick = time::Duration::from_millis(50);
    let tick_threshold: usize = 5;
    let mut tick_count: usize = 0;
    let mut grow_snake: bool;
    let mut move_snake: bool;

    // Init snake

    for i in 0..SNAKE_MIN_LEN {
        snake[i] = Some([SNAKE_MIN_LEN + 3 - i, FIELD_YMAX / 2]);
    }

    // Init field

    for y in 0..FIELD_YMAX {
        field[0 + y * FIELD_XMAX] = PIXEL_BORDER;
        field[FIELD_XMAX - 1 + y * FIELD_XMAX] = PIXEL_BORDER;
    }
    for x in 0..FIELD_XMAX {
        field[x + 0 * FIELD_XMAX] = PIXEL_BORDER;
        field[x + (FIELD_YMAX - 1) * FIELD_XMAX] = PIXEL_BORDER;
    }

    // Init terminal

    stdout
        .execute(terminal::Clear(terminal::ClearType::All))?
        .execute(SetBackgroundColor(Color::Black))?
        .execute(SetForegroundColor(Color::White))?
        .flush()?;

    // Game loop ...

    while !snake_crashed {
        tick_count += 1;
        move_snake = tick_count == tick_threshold;

        // Read user input

        if poll(game_tick)? {
            let event = read()?;

            if event == Event::Key(KeyCode::Esc.into()) {
                let _ = terminal::disable_raw_mode()?;
                exit(0x0);
            }
        }

        // Move snake

        if move_snake {}

        // Add snake to field

        for si in 0..snake.len() {
            match snake[si] {
                Some([sx, sy]) => field[sx + sy * FIELD_XMAX] = b'X',
                None => {}
            }
        }

        // Render field

        for fx in 0..FIELD_XMAX {
            for fy in 0..FIELD_YMAX {
                let i = fx + fy * FIELD_XMAX;
                stdout
                    .execute(cursor::MoveTo(
                        (fx + FIELD_XMARGIN) as u16,
                        (fy + FIELD_YMARGIN) as u16,
                    ))?
                    .execute(Print(field[i] as char))?;
            }
        }

        stdout.flush()?;
    }

    let _ = terminal::disable_raw_mode()?;
    Ok(())
}
