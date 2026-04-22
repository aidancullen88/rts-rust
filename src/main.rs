extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

mod cell_map;
mod cover;
mod npc;
mod point;
mod vector;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
use piston::{Button, GenericEvent, MouseButton};

use crate::cell_map::Cells;
use crate::npc::Npc;

struct App {
    gl: GlGraphics,
    mouse_pos: [f64; 2],
    npc_list: Vec<Npc>,
    game_state: GameState,
    cell_map: Cells,
}

impl App {
    fn render(
        &mut self,
        cover_list: &Vec<crate::cover::Cover>,
        // game_state: &mut GameState,
        args: &RenderArgs,
    ) {
        self.gl.draw(args.viewport(), |c, gl| {
            graphics::clear(graphics::color::BLACK, gl);
            crate::cover::render_covers(cover_list, &c, gl);
            crate::cover::render_grid(&self.cell_map, &c, gl);
            crate::npc::render_npcs(&self.npc_list, &c, gl);
        })
    }

    fn update(&mut self, args: &UpdateArgs) {}
    fn control<E: GenericEvent>(&mut self, window_dims: &[u32; 2], event: &E) {
        // Get the current mouse position
        if let Some(pos) = event.mouse_cursor_args() {
            self.mouse_pos = pos;
        }

        if let Some(Button::Mouse(MouseButton::Left)) = event.press_args() {
            let look_dir = if self.mouse_pos[0] <= f64::from(window_dims[0]) / 2.0 {
                [1.0, 0.0]
            } else {
                [-1.0, 0.0]
            };
            self.npc_list.push(
                Npc::new(
                    &mut self.game_state,
                    &mut self.cell_map,
                    self.mouse_pos.into(),
                )
                .set_look_dir(look_dir.into()),
            );
        }
    }
}

struct GameState {
    paused: bool,
    entity_id_counter: usize,
}

impl GameState {
    fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    fn get_next_entity_id(&mut self) -> usize {
        self.entity_id_counter += 1;
        self.entity_id_counter
    }
}

fn main() {
    let opengl = OpenGL::V3_2;

    let window_dims: [u32; 2] = [1024, 768];

    let mut window: Window = WindowSettings::new("rts", window_dims)
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut app = App {
        gl: GlGraphics::new(opengl),
        mouse_pos: [0.0; 2],
        npc_list: Vec::new(),
        game_state: GameState {
            paused: false,
            entity_id_counter: 0,
        },
        cell_map: Cells::new(),
    };

    const COVER_LIST: [[u32; 4]; 6] = [
        [250, 100, 250, 200],
        [150, 300, 150, 400],
        [250, 500, 250, 600],
        [750, 100, 750, 200],
        [850, 300, 850, 400],
        [750, 500, 750, 600],
    ];

    let cover_list = crate::cover::init_covers(COVER_LIST);

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        // Pass the event through to control to check for inputs
        app.control(&window_dims, &e);
        if let Some(args) = e.update_args() {
            app.update(&args);
        }
        if let Some(args) = e.render_args() {
            app.render(&cover_list, &args);
        }
    }
}
