#![allow(dead_code)]
#![allow(unused)]

extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

mod cell_map;
mod cover;
mod effect;
mod event;
mod npc;
mod point;
mod vector;
mod planning;

use std::collections::{HashMap, VecDeque};

use glutin_window::GlutinWindow as Window;
use graphics::{Context, Graphics};
use opengl_graphics::{GlGraphics, GlyphCache, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
use piston::{Button, ButtonState, GenericEvent, MouseButton};

use crate::cell_map::Cells;
use crate::cover::Cover;
use crate::effect::{Effect, new_effect_queue};
use crate::event::EventQueue;
use crate::npc::{Id, NpcTeam, Task, TaskType};
use crate::npc::{Npc, NpcMap};
use crate::point::Point;

/// Generates the long statement that represents a key in piston
#[macro_export]
macro_rules! piston_key {
    ($i:ident) => {
        Button::Keyboard(piston::Key::$i)
    };
}

/// The effect queue is threaded through the update function. Effects can be added to the queue
/// here and then will be rendered in the render function later
pub type EffectQueue = Vec<Box<dyn Effect<GlGraphics>>>;

struct App {
    // generic graphics handler
    gl: GlGraphics,
    mouse_pos: [f64; 2],
    npcs: NpcMap,
    // governs if the game is paused and holds the entity id generator
    game_state: GameState,
    // queue to render effects
    effect_queue: EffectQueue,
    // Events are processed before npcs get to do any actions in update
    event_queue: EventQueue,
    // The glyphs are init'd and stay the same throughout the program, hence 'static
    glyphs: GlyphCache<'static>,
    // Holds the map details
    game_map: GameMap,
}

struct GameMap {
    cover: HashMap<Id, Cover>,
}

impl App {
    fn render(
        &mut self,
        args: &RenderArgs,
    ) {
        // c is the graphics context, gl is the graphics handler: in this case, the opengl handler
        // all render calls require both
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
            // Render any effects that have been added to the effect queue on the last update
            crate::effect::render_effects(&mut self.effect_queue, &c, gl, args.ext_dt);
        })
    }

    fn update(&mut self, args: &UpdateArgs) {
        // the npcs struct handles updating all of its npcs itself
        self.npcs.update_npcs(
            &self.game_map,
            &mut self.effect_queue,
            &mut self.event_queue,
            &args.dt,
        );
    }
    fn control<E: GenericEvent>(&mut self, window_dims: &[u32; 2], event: &E) {
        // Save the current mouse position to use throughout the event handlers (like on the next
        // update call)
        if let Some(pos) = event.mouse_cursor_args() {
            self.mouse_pos = pos;
        }

        if let Some(Button::Mouse(MouseButton::Left)) = event.press_args() {
            // check mouse pos against npc list to see which ones collide, and pick the first
            if let Some(npc_id) = self.npcs.cell_map.check_if_point_target_collides_with_npc(
                &self.mouse_pos.into(),
                &self.npcs.get_npc_info_map(),
            ) {
                match self.npcs.get_selected_npc_id() {
                    Some(id) if *id == npc_id => self.npcs.deselect_npc(),
                    _ => self.npcs.select_npc(npc_id),
                }
            // DEV: allows direct control of selected npcs
            } else if let Some(selected_npc) = self.npcs.get_selected_npc() {
                selected_npc.queue_task(Task::new(TaskType::Move(self.mouse_pos.into())));
                self.npcs.deselect_npc();
            // If there's no npc selected and we're clicking on blank space, spawn a new npc
            } else {
                // Set npcs that spawn on the left of the screen to look right and vice versa
                let (look_dir, team) = if self.mouse_pos[0] <= f64::from(window_dims[0]) / 2.0 {
                    ([1.0, 0.0], NpcTeam::Blue)
                } else {
                    ([-1.0, 0.0], NpcTeam::Red)
                };
                let new_npc = self.npcs.spawn_npc(
                    self.mouse_pos.into(),
                    look_dir.into(),
                    Some(look_dir.into()),
                    team,
                    &mut self.game_state,
                );
                new_npc.queue_task(Task::new(TaskType::FindCloseCover));
            }
        }
        if let Some(Button::Mouse(MouseButton::Right)) = event.press_args()
            && let Some(npc_id) = self.npcs.cell_map.check_if_point_target_collides_with_npc(
                &self.mouse_pos.into(),
                &self.npcs.get_npc_info_map(),
            )
        {
            self.npcs.delete_npc(&npc_id);
        }
        if let Some(button_args) = event.button_args() {
            match (button_args.button, button_args.state) {
                (piston_key!(P), ButtonState::Press) => self.game_state.toggle_pause(),
                (piston_key!(C), ButtonState::Press) => {
                    self.npcs.clear_npcs();
                }
                // DEV: selected npc moves to cover
                (piston_key!(F), ButtonState::Press) => {
                    if let Some(selected_npc) = self.npcs.get_selected_npc() {
                        selected_npc.queue_task(Task::new(TaskType::FindCloseCover));
                    }
                }
                (piston_key!(S), ButtonState::Press) => {
                    self.npcs
                        .queue_task_all_npcs(Task::new(TaskType::FindTarget));
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
    // const COVER_LIST: [[u32; 4]; 7] = [
    //     [250, 100, 250, 200],
    //     [150, 300, 150, 400],
    //     [250, 500, 250, 600],
    //     [750, 100, 750, 200],
    //     [850, 300, 850, 400],
    //     [750, 500, 750, 600],
    //     [500, 300, 500, 400],
    // ];

    // const COVER_LIST: [[u32; 4]; 12] = [
    //     [180, 120, 180, 230], // left column, top
    //     [180, 320, 180, 430], // left column, middle
    //     [180, 520, 180, 630], // left column, bottom
    //     [380, 150, 380, 260], // center-left column, top
    //     [380, 350, 380, 460], // center-left column, middle
    //     [380, 550, 380, 650], // center-left column, bottom
    //     [580, 120, 580, 230], // center-right column, top
    //     [580, 320, 580, 430], // center-right column, middle
    //     [580, 520, 580, 630], // center-right column, bottom
    //     [780, 150, 780, 260], // right column, top
    //     [780, 350, 780, 460], // right column, middle
    //     [780, 550, 780, 650], // right column, bottom
    // ];

    const COVER_LIST: [[u32; 4]; 0] = [];

    let mut game_state = GameState {
        paused: false,
        entity_id_counter: 0,
    };
    let covers = crate::cover::init_covers(&COVER_LIST, &mut game_state);

    let mut app = App {
        gl: GlGraphics::new(opengl),
        // Required for mouse updates: initialise to a sensible default
        mouse_pos: [0.0; 2],
        npcs: NpcMap::new(60.0),
        game_state,
        // Initialise the glpyh cache to use for drawing text
        glyphs: GlyphCache::new(
            "assets/Roboto-Regular.ttf",
            (),
            opengl_graphics::TextureSettings::new(),
        )
        .unwrap(),
        game_map: GameMap { cover: covers },
        effect_queue: effect::new_effect_queue(),
        event_queue: EventQueue::new(),
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
