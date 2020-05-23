#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

extern crate itertools;

use itertools::join;
use rand::{prelude::ThreadRng, Rng};

use std::io::{stdout, Write};
use std::time;

use crossterm::{
    cursor,
    event::{poll, read, Event, KeyCode},
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
    terminal, ExecutableCommand, Result,
};

const FIELD_XMARGIN: usize = 2;
const FIELD_YMARGIN: usize = 1;
const FIELD_XMAX: usize = 30;
const FIELD_YMAX: usize = 20;
const FIELD_PIXELS: usize = (FIELD_XMAX * FIELD_YMAX);
const PIXEL_EMPTY: u8 = b' ';
const PIXEL_BORDER: u8 = b'#';
const PIXEL_SNAKE: u8 = b'X';
const PIXEL_FRUIT: u8 = b'O';
const SNAKE_MIN_LEN: usize = 8;
const SNAKE_MAX_LEN: usize = FIELD_PIXELS - 2 * FIELD_XMAX - 2 * (FIELD_YMAX - 2);
const FRUITS_MAX: usize = 2;
const FRUITS_MIN: usize = 2;

type ObjectPixel = Option<[usize; 2]>;
type PixelContent = Option<u8>;
type Snake = [ObjectPixel; SNAKE_MAX_LEN];
type Fruits = [ObjectPixel; FRUITS_MAX];

#[derive(PartialEq)]
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

    let mut field: [PixelContent; FIELD_PIXELS] = [None; FIELD_PIXELS];
    let mut fruits: Fruits = [None; FRUITS_MAX];
    let mut snake: Snake = [None; SNAKE_MAX_LEN];
    let mut snake_orientation: SnakeOrientation = SnakeOrientation::East;
    let mut snake_speed: usize = 1;
    let mut snake_crashed: bool = false;
    let mut snake_drop_tail: ObjectPixel = None; // Tail to remove, to avoid complete re-render

    let speed_increase_count: usize = 0;
    let speed_increase_amount: usize = 10;
    let mut game_tick = time::Duration::from_millis(250);
    let mut grow_snake: bool;

    let mut is_paused: bool = false;
    let mut score: usize = 0;

    // Init snake

    for i in 0..SNAKE_MIN_LEN {
        snake[i] = Some([SNAKE_MIN_LEN + 3 - i, FIELD_YMAX / 2]);
    }

    // Init field

    for y in 0..FIELD_YMAX {
        field[0 + y * FIELD_XMAX] = Some(PIXEL_BORDER);
        field[FIELD_XMAX - 1 + y * FIELD_XMAX] = Some(PIXEL_BORDER);
    }
    for x in 0..FIELD_XMAX {
        field[x + 0 * FIELD_XMAX] = Some(PIXEL_BORDER);
        field[x + (FIELD_YMAX - 1) * FIELD_XMAX] = Some(PIXEL_BORDER);
    }

    // Init fruits

    for i in 0..FRUITS_MAX {
        fruits[i] = get_random_fruit(&mut rng, &snake);
    }

    // Init terminal

    stdout
        .execute(terminal::Clear(terminal::ClearType::All))?
        .execute(SetBackgroundColor(Color::Black))?
        .execute(SetForegroundColor(Color::White))?
        .flush()?;

    // Game loop

    'outer: while !snake_crashed {
        // Read user input

        if poll(game_tick)? {
            let event = read()?;

            if event == Event::Key(KeyCode::Esc.into()) {
                break 'outer;
            }
            if event == Event::Key(KeyCode::Char('p').into()) {
                is_paused = !is_paused;
            }
            if event == Event::Key(KeyCode::Up.into()) {
                if snake_orientation != SnakeOrientation::South {
                    snake_orientation = SnakeOrientation::North;
                }
            }
            if event == Event::Key(KeyCode::Right.into()) {
                if snake_orientation != SnakeOrientation::West {
                    snake_orientation = SnakeOrientation::East;
                }
            }
            if event == Event::Key(KeyCode::Down.into()) {
                if snake_orientation != SnakeOrientation::North {
                    snake_orientation = SnakeOrientation::South;
                }
            }
            if event == Event::Key(KeyCode::Left.into()) {
                if snake_orientation != SnakeOrientation::East {
                    snake_orientation = SnakeOrientation::West;
                }
            }
        }

        if is_paused {
            continue;
        }

        // Move snake

        for si in (1..SNAKE_MAX_LEN).rev() {
            match snake[si - 1] {
                Some(p) => {
                    if snake[si].is_some() {
                        snake[si] = Some(p);
                    } else {
                        snake_drop_tail = Some(p);
                    }
                }
                None => {}
            }
        }
        let mut sx = snake[0].expect("Missing snake pixel")[0];
        let mut sy = snake[0].expect("Missing snake pixel")[1];
        match snake_orientation {
            SnakeOrientation::North => sy -= 1,
            SnakeOrientation::East => sx += 1,
            SnakeOrientation::South => sy += 1,
            SnakeOrientation::West => sx -= 1,
        }
        snake[0] = Some([sx, sy]);

        // Check colission

        match snake[0] {
            Some([shx, shy]) => {
                // Wall colission
                if shx == 0 || shx == FIELD_XMAX - 1 || shy == 0 || shy == FIELD_YMAX - 1 {
                    snake_crashed = true;
                    break 'outer;
                }
                // Self colission
                for si in 1..snake.len() {
                    match snake[si] {
                        Some([tx, ty]) => {
                            if shx == tx && shy == ty {
                                snake_crashed = true;
                                break 'outer;
                            }
                        }
                        None => {}
                    }
                }
            }
            None => {}
        }

        // Check snake eats fruit

        match snake[0] {
            Some([sx, sy]) => {
                for i in 0..fruits.len() {
                    match fruits[i] {
                        Some([fx, fy]) => {
                            if fx == sx && fy == sy {
                                score += 10;
                                fruits[i] = get_random_fruit(&mut rng, &snake);
                            }
                        }
                        None => {}
                    }
                }
            }
            None => {}
        }

        // Add fruits  to field

        for i in 0..fruits.len() {
            match fruits[i] {
                Some([fx, fy]) => field[fx + fy * FIELD_XMAX] = Some(PIXEL_FRUIT),
                None => {}
            }
        }

        // Add snake to field

        for si in 0..snake.len() {
            match snake[si] {
                Some([sx, sy]) => field[sx + sy * FIELD_XMAX] = Some(PIXEL_SNAKE),
                None => {}
            }
        }
        match snake_drop_tail {
            Some([sx, sy]) => field[sx + sy * FIELD_XMAX] = Some(PIXEL_EMPTY),
            None => {}
        }

        // Render field

        for fx in 0..FIELD_XMAX {
            for fy in 0..FIELD_YMAX {
                let fi = fx + fy * FIELD_XMAX;
                match field[fi] {
                    Some(content) => {
                        let tx = (fx + FIELD_XMARGIN) as u16;
                        let ty = (fy + FIELD_YMARGIN) as u16;
                        stdout
                            .execute(cursor::MoveTo(tx, ty))?
                            .execute(Print(content as char))?;
                    }
                    None => {}
                }
            }
        }

        stdout
            .execute(cursor::MoveTo(
                FIELD_XMARGIN as u16 + 3,
                FIELD_YMARGIN as u16 + 0,
            ))?
            .execute(Print(format!("[{}]", score)))?
            .flush()?;
    }

    if snake_crashed {
        let msg = "Snake crashed!";
        let spc_len = (FIELD_XMAX - msg.len()) / 2;
        let spc = join((0..spc_len).map(|_| " "), "");
        let x = (FIELD_XMARGIN) as u16;
        let y = (FIELD_YMARGIN + (FIELD_YMAX / 2)) as u16;
        stdout
            .execute(SetBackgroundColor(Color::White))?
            .execute(SetForegroundColor(Color::Black))?
            .execute(cursor::MoveTo(x, y))?
            .execute(Print(format!("{}{}{}", spc, msg, spc)))?
            .flush()?;
    }

    stdout
        .execute(cursor::MoveTo(0, (FIELD_YMARGIN + FIELD_YMAX + 1) as u16))?
        .flush()?;

    let _ = terminal::disable_raw_mode()?;
    Ok(())
}

fn get_random_fruit(rng: &mut ThreadRng, snake: &Snake) -> ObjectPixel {
    let mut x: usize;
    let mut y: usize;
    loop {
        x = rng.gen_range(0, FIELD_XMAX);
        y = rng.gen_range(0, FIELD_YMAX);

        // not on the field border
        if x == 0 || x == FIELD_XMAX - 1 || y == 0 || y == FIELD_YMAX - 1 {
            continue;
        }

        // not on the snake
        for si in 0..snake.len() {
            match snake[si] {
                Some([sx, sy]) => {
                    if sx == x || sy == y {
                        continue;
                    }
                }
                None => {}
            }
        }

        // all okay
        return Some([x, y]);
    }
}
