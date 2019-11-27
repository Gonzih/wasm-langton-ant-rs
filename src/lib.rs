extern crate rand;
extern crate web_sys;

mod utils;

use rand::seq::SliceRandom;
use rand::Rng;
use std::fmt;
use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const EMPTY_COLOR: &str = "rgb(255, 255, 255)";
const FILL_COLOR: &str = "rgb(0, 0, 0)";
const ANT_COLOR: &str = "rgb(200, 0, 0)";

#[derive(Clone, Debug)]
enum Rotate {
    Clockwise,
    CounterClockwise,
    Noop,
    Uturn,
}

#[derive(Clone)]
struct Decision {
    rotate: Rotate,
    color: bool,
    state: bool,
}

impl Decision {
    fn new(rotate: Rotate, color: bool, state: bool) -> Decision {
        Decision {
            rotate,
            color,
            state,
        }
    }
}

macro_rules! decision_table {
    ($name:expr, $( [$rotate:ident, $color:ident, $state:ident] ),*) => {
        DecisionTable {
            name: $name,
            table: [
                $( Decision::new($rotate, $color, $state) ),*
            ],
        }
    }
}

#[derive(Clone)]
struct DecisionTable {
    name: &'static str,
    table: [Decision; 4],
}

impl DecisionTable {
    fn random() -> DecisionTable {
        use Rotate::*;

        let tables = vec![
            decision_table!(
                "fibonacci",
                [CounterClockwise, true, true],
                [CounterClockwise, true, true],
                [Clockwise, true, true],
                [Noop, false, false]
            ),
            decision_table!(
                "langton",
                [Clockwise, true, false],
                [CounterClockwise, false, false],
                [Clockwise, true, false],
                [CounterClockwise, false, false]
            ),
            decision_table!(
                "chaotic_one",
                [Clockwise, true, false],
                [Clockwise, true, true],
                [Noop, false, false],
                [Noop, false, true]
            ),
            decision_table!(
                "chaotic_two",
                [Clockwise, true, true],
                [CounterClockwise, false, true],
                [Noop, true, false],
                [Noop, false, true]
            ),
            decision_table!(
                "chaotic_three",
                [CounterClockwise, true, true],
                [CounterClockwise, false, true],
                [Clockwise, true, true],
                [CounterClockwise, false, false]
            ),
            decision_table!(
                "chaotic_four",
                [CounterClockwise, true, true],
                [CounterClockwise, false, true],
                [Noop, true, false],
                [Noop, true, true]
            ),
            decision_table!(
                "coral",
                [Clockwise, true, true],
                [CounterClockwise, true, true],
                [Clockwise, true, true],
                [CounterClockwise, false, false]
            ),
            decision_table!(
                "square_one",
                [CounterClockwise, true, false],
                [Clockwise, true, true],
                [Clockwise, false, false],
                [CounterClockwise, false, true]
            ),
            decision_table!(
                "square_two",
                [Clockwise, false, true],
                [CounterClockwise, false, false],
                [Noop, true, false],
                [Uturn, true, true]
            ),
            decision_table!(
                "counter_one",
                [Noop, false, true],
                [Uturn, false, true],
                [Clockwise, true, true],
                [Noop, false, true]
            ),
            decision_table!(
                "counter_two",
                [Clockwise, true, true],
                [Noop, false, true],
                [Noop, false, false],
                [CounterClockwise, true, true]
            ),
            decision_table!(
                "spiral_one",
                [Noop, true, true],
                [CounterClockwise, true, false],
                [Clockwise, true, true],
                [Noop, false, false]
            ),
            decision_table!(
                "spiral_two",
                [CounterClockwise, true, false],
                [Clockwise, false, true],
                [Clockwise, true, false],
                [CounterClockwise, false, true]
            ),
            decision_table!(
                "spiral_three",
                [Uturn, true, false],
                [Noop, false, true],
                [CounterClockwise, false, false],
                [Clockwise, false, true]
            ),
            decision_table!(
                "ladder",
                [Noop, false, true],
                [Uturn, true, true],
                [CounterClockwise, true, false],
                [Noop, true, true]
            ),
            decision_table!(
                "dixie",
                [Clockwise, false, true],
                [CounterClockwise, false, false],
                [Uturn, true, true],
                [Clockwise, false, false]
            ),
        ];

        tables
            .choose(&mut rand::thread_rng())
            .expect("Could not get random move table")
            .clone()
    }

    fn decide(&self, x: usize, y: usize) -> Decision {
        self.table[x * 2 + y].clone()
    }
}

#[derive(Clone)]
enum Orientation {
    Up,
    Right,
    Down,
    Left,
}

impl Orientation {
    fn uturn(&self) -> Orientation {
        use Orientation::*;

        match self {
            Up => Down,
            Right => Left,
            Down => Up,
            Left => Right,
        }
    }

    fn clockwise(&self) -> Orientation {
        use Orientation::*;

        match self {
            Up => Right,
            Right => Down,
            Down => Left,
            Left => Up,
        }
    }

    fn counter_clockwise(&self) -> Orientation {
        use Orientation::*;

        match self {
            Up => Left,
            Left => Down,
            Down => Right,
            Right => Up,
        }
    }
}

#[wasm_bindgen]
pub struct Turmite {
    x: usize,
    y: usize,
    orientation: Orientation,
    behavior: DecisionTable,
    state: bool,
    width: usize,
    height: usize,
    field: Vec<Vec<bool>>,
    pixel_ratio: usize,
    is_active: bool,
}

#[wasm_bindgen]
impl Turmite {
    pub fn new(canvas_width: usize, canvas_height: usize, pixel_ratio: usize) -> Turmite {
        let width = canvas_width / pixel_ratio;
        let height = canvas_height / pixel_ratio;
        let field = vec![vec![false; height]; width];

        let mut rng = rand::thread_rng();
        let state = rng.gen();

        let behavior = DecisionTable::random();
        log!("Using {} as behavior table", behavior.name);

        let mut turmite = Turmite {
            x: width / 2,
            y: height / 2,
            orientation: Orientation::Right,
            is_active: true,
            behavior,
            state,
            field,
            width,
            height,
            pixel_ratio,
        };

        turmite.set_color(rng.gen());

        turmite
    }

    fn cur_color(&self) -> bool {
        self.field[self.x][self.y]
    }

    fn set_color(&mut self, v: bool) {
        self.field[self.x][self.y] = v;
    }

    fn move_by(&mut self, dx: i32, dy: i32) {
        // log!("Moving by {} {}", dx, dy);
        let mut new_x = self.x as i32 + dx;
        if new_x < 0 {
            self.is_active = false;
            new_x = 0;
        }

        let mut new_y = self.y as i32 + dy;
        if new_y < 0 {
            self.is_active = false;
            new_y = 0;
        }

        self.x = new_x as usize;
        self.y = new_y as usize;
    }

    fn tick_state(&mut self) {
        let x = if self.state { 1 } else { 0 };
        let y = if self.cur_color() { 1 } else { 0 };
        let Decision {
            rotate,
            color,
            state,
        } = self.behavior.decide(x, y);

        self.state = state;
        self.rotate(rotate);
        self.set_color(color);
    }

    fn rotate(&mut self, rotation: Rotate) {
        use Rotate::*;
        // log!("Rotating with {:?}", rotation);

        self.orientation = match rotation {
            Noop => self.orientation.clone(),
            Clockwise => self.orientation.clockwise(),
            CounterClockwise => self.orientation.counter_clockwise(),
            Uturn => self.orientation.uturn(),
        }
    }

    fn tick_pos(&mut self) {
        use Orientation::*;

        if self.x < self.width && self.y < self.height {
            match &self.orientation {
                Up => self.move_by(0, -1),
                Right => self.move_by(1, 0),
                Down => self.move_by(0, 1),
                Left => self.move_by(-1, 0),
            };
        } else {
            self.is_active = false;
        }
    }

    fn render(&self, ctx: &CanvasRenderingContext2d, color: &str) {
        if self.is_active() {
            ctx.set_fill_style(&JsValue::from(color));
            ctx.fill_rect(
                (self.x * self.pixel_ratio) as f64,
                (self.y * self.pixel_ratio) as f64,
                self.pixel_ratio as f64,
                self.pixel_ratio as f64,
            );
        }
    }

    fn render_self(&self, ctx: &CanvasRenderingContext2d) {
        self.render(ctx, ANT_COLOR);
    }

    fn render_cell(&self, ctx: &CanvasRenderingContext2d) {
        let color = if self.cur_color() {
            FILL_COLOR
        } else {
            EMPTY_COLOR
        };

        self.render(ctx, color);
    }

    pub fn log(&self) {
        log!("{}", self);
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn tick(&mut self, ctx: &CanvasRenderingContext2d) {
        if self.is_active() {
            self.tick_state();
            self.render_cell(ctx);
            self.tick_pos();
            self.render_self(ctx);
        }
    }
}

impl fmt::Display for Turmite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut field_output = "".to_string();

        for row in &self.field {
            for cell in row {
                field_output += if *cell { "1" } else { "0" };
            }
        }

        write!(
            f,
            "Turmite {{ width: {}, height: {} }}\n{}",
            self.width, self.height, field_output
        )
    }
}

#[wasm_bindgen]
pub fn debug() {
    utils::set_panic_hook();
}
