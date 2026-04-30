#![allow(dead_code)]
#![allow(unused)]

extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

mod cell_map;
mod cover;
mod npc;
mod point;
mod vector;

use std::collections::HashMap;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, GlyphCache, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
use piston::{Button, ButtonState, GenericEvent, MouseButton};

use crate::cell_map::Cells;
use crate::cover::Cover;
use crate::npc::{Id, Task, TaskType};
use crate::npc::Npc;
use crate::point::Point;

struct App {
    gl: GlGraphics,
    mouse_pos: [f64; 2],
    npcs: Npcs,
    game_state: GameState,
    // The glyphs are init'd and stay the same throughout the program, hence 'static
    glyphs: GlyphCache<'static>,
    game_map: GameMap,
}

struct GameMap {
    cover: Vec<Cover>,
}

struct Npcs {
    map: HashMap<Id, Npc>,
    cell_map: Cells,
    selected: Option<Id>,
}

// This can probably go in it's own file at some point
impl Npcs {
    pub fn new(cell_size: f64) -> Npcs {
        Npcs {
            map: HashMap::new(),
            selected: None,
            cell_map: Cells::new(cell_size),
        }
    }

    pub fn spawn_npc(&mut self, npc_pos: Point, look_dir: crate::vector::Vector, game_state: &mut GameState) -> Id {
        let npc_id = game_state.get_next_entity_id();
        let mut new_npc = Npc::new(npc_id, &mut self.cell_map, npc_pos.clone());
        new_npc.set_look_dir(look_dir);
        self.map.insert(npc_id, new_npc);
        npc_id
    }

    pub fn get_npc_by_id(&self, id: &Id) -> Option<&Npc> {
        self.map.get(id)
    }

    pub fn get_npc_iter(&self) -> impl Iterator<Item = &Npc> {
        self.map.values()
    }

    pub fn select_npc(&mut self, id: Id) {
        self.selected = Some(id);
    }

    pub fn deselect_npc(&mut self) {
        self.selected = None;
    }

    pub fn get_selected_npc(&mut self) -> Option<&mut Npc> {
        self.selected.and_then(|s| self.map.get_mut(&s))
    }

    pub fn get_selected_npc_id(&self) -> Option<&Id> {
        self.selected.as_ref()
    }

    pub fn update_npcs(&mut self, dt: &f64) {
        for npc in self.map.values_mut() {
            npc.act(&mut self.cell_map, dt);
        }
    }
}

impl App {
    fn render(
        &mut self,
        // TODO: This will be passed as part of a larger "environment/map" struct probably
        args: &RenderArgs,
    ) {
        // c is the graphics context, gl is the graphics handler: in this case, the opengl handler
        self.gl.draw(args.viewport(), |c, gl| {
            graphics::clear(graphics::color::BLACK, gl);
            crate::cover::render_covers(&self.game_map.cover, &c, gl);
            // If we're going to render text, need to pass the glyphs as well
            crate::cover::render_grid(&self.npcs.cell_map, &c, gl, &mut self.glyphs);
            crate::npc::render_npcs(
                self.npcs.get_npc_iter(),
                self.npcs.get_selected_npc_id(),
                &c,
                gl,
            );
        })
    }

    fn update(&mut self, args: &UpdateArgs) {
        self.npcs.update_npcs(&args.dt);
    }
    fn control<E: GenericEvent>(&mut self, window_dims: &[u32; 2], event: &E) {
        // Save the current mouse position to use throughout the event handlers
        if let Some(pos) = event.mouse_cursor_args() {
            self.mouse_pos = pos;
        }

        if let Some(Button::Mouse(MouseButton::Left)) = event.press_args() {
            // check mouse pos against npc list to see which ones collide, and pick the first
            if let Some(npc_id) = self.npcs.cell_map.check_if_target_collides_with_npc(
                &self.mouse_pos.into(),
                &self.npcs,
            ) {
                println!("target collided");
                self.npcs.select_npc(npc_id);
            } else if let Some(selected_npc) = self.npcs.get_selected_npc() {
                selected_npc.queue_task(Task::new(TaskType::Move(self.mouse_pos.into())));
                self.npcs.deselect_npc();
            } else {
                // Set npcs that spawn on the left of the screen to look right and vice versa
                let look_dir = if self.mouse_pos[0] <= f64::from(window_dims[0]) / 2.0 {
                    [1.0, 0.0]
                } else {
                    [-1.0, 0.0]
                };
                let new_npc_id = self.npcs.spawn_npc(self.mouse_pos.into(), look_dir.into(), &mut self.game_state);
                self.npcs.select_npc(new_npc_id);
            }
        }
        if let Some(Button::Mouse(MouseButton::Right)) = event.press_args() {}
        if let Some(button_args) = event.button_args() {
            match button_args.button {
                Button::Keyboard(piston::Key::P) if button_args.state == ButtonState::Press => {
                    self.game_state.toggle_pause()
                }
                _ => (),
            }
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

    fn get_next_entity_id(&mut self) -> Id {
        self.entity_id_counter += 1;
        Id(self.entity_id_counter)
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

    // This should come from a config/map file eventually
    const COVER_LIST: [[u32; 4]; 6] = [
        [250, 100, 250, 200],
        [150, 300, 150, 400],
        [250, 500, 250, 600],
        [750, 100, 750, 200],
        [850, 300, 850, 400],
        [750, 500, 750, 600],
    ];

    let mut app = App {
        gl: GlGraphics::new(opengl),
        // Required for mouse updates: initialise to a sensible default
        mouse_pos: [0.0; 2],
        npcs: Npcs::new(60.0),
        game_state: GameState {
            paused: false,
            entity_id_counter: 0,
        },
        // Initialise the glpyh cache to use for drawing text
        glyphs: GlyphCache::new(
            "assets/Roboto-Regular.ttf",
            (),
            opengl_graphics::TextureSettings::new(),
        )
        .unwrap(),
        game_map: GameMap {
            cover: crate::cover::init_covers(COVER_LIST),
        },
    };

    // let mut events = Events::new(EventSettings { max_fps: 180, ups: 180, ..EventSettings::default() });
    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        // Pass the event through to control to check for inputs
        app.control(&window_dims, &e);
        if let Some(args) = e.update_args()
            && !app.game_state.paused
        {
            app.update(&args);
        }
        if let Some(args) = e.render_args() {
            app.render(&args);
        }
    }
}
