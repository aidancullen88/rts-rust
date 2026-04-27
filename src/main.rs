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
use crate::npc::Id;
use crate::npc::Npc;
use crate::point::Point;

struct App {
    gl: GlGraphics,
    mouse_pos: [f64; 2],
    npcs: NpcMap,
    game_state: GameState,
    cell_map: Cells,
    // The glyphs are init'd and stay the same throughout the program, hence 'static
    glyphs: GlyphCache<'static>,
    game_map: GameMap,
}

struct GameMap {
    cover: Vec<Cover>,
}

struct NpcMap {
    map: HashMap<Id, Npc>,
    selected: Option<Id>,
}

impl NpcMap {
    pub fn new() -> NpcMap {
        NpcMap {
            map: HashMap::new(),
            selected: None,
        }
    }

    pub fn add_npc(&mut self, id: Id, npc: Npc) {
        self.map.insert(id, npc);
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

    pub fn get_selected_npc(&self) -> Option<&Npc> {
        self.selected.and_then(|s| self.map.get(&s))
    }

    pub fn get_selected_npc_id(&self) -> Option<&Id> {
        self.selected.as_ref()
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
            crate::cover::render_grid(&self.cell_map, &c, gl, &mut self.glyphs);
            crate::npc::render_npcs(
                self.npcs.get_npc_iter(),
                self.npcs.get_selected_npc_id(),
                &c,
                gl,
            );
        })
    }

    fn update(&mut self, args: &UpdateArgs) {}
    fn control<E: GenericEvent>(&mut self, window_dims: &[u32; 2], event: &E) {
        // Save the current mouse position to use throughout the event handlers
        if let Some(pos) = event.mouse_cursor_args() {
            self.mouse_pos = pos;
        }

        if let Some(Button::Mouse(MouseButton::Left)) = event.press_args() {
            // check mouse pos against npc list to see which ones collide, and pick the first
            if let Some(npc_id) = self.cell_map.check_if_target_collides_with_npc(
                &Point::new(self.mouse_pos[0], self.mouse_pos[1]),
                &self.npcs,
            ) {
                println!("target collided");
                self.npcs.select_npc(npc_id);
            } else {
                // Set npcs that spawn on the left of the screen to look right and vice versa
                let look_dir = if self.mouse_pos[0] <= f64::from(window_dims[0]) / 2.0 {
                    [1.0, 0.0]
                } else {
                    [-1.0, 0.0]
                };
                let new_npc = Npc::new(
                    &mut self.game_state,
                    &mut self.cell_map,
                    self.mouse_pos.into(),
                )
                .set_look_dir(look_dir.into());
                self.npcs.add_npc(new_npc.get_id(), new_npc);
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
        npcs: NpcMap::new(),
        game_state: GameState {
            paused: false,
            entity_id_counter: 0,
        },
        cell_map: Cells::new(100.0),
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
