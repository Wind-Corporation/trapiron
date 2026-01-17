//! Parts of the game that only make sense with a GUI.
//!
//! Compare [`crate::world`] and [`crate::logic`] that can be used headless.

mod control;
mod view;

use std::{
    rc::Rc,
    time::{Duration, Instant},
};

use crate::{
    client::{control::Control, view::View},
    content::Resources,
    gui::{Dcf, Drawable},
    logic::Logic,
    world::{Event, World},
};

/// Generalized statistics tracker for the two types of regular update routines: logic ticks and
/// presentation ticks.
///
/// It should be informed about beginning and end of each tick so it can compute various useful
/// properties such as tick duration.
///
/// Even though the processing of a tick takes time, each tick represents an instant.
struct TickStats {
    /// Duration of the last tick measured from end of penultimate tick to end of last tick.
    last_duration: Duration,

    /// The instant of the last tick that occurred, if any.
    last_timestamp: Option<Instant>,

    /// Number of ticks fully processed. Zero before and during first tick.
    completed: u64,
}

impl TickStats {
    /// Report that processing of a tick representing instant _now_ has begun.
    pub fn start_tick(&mut self, now: Instant) {
        if let Some(time) = self.last_timestamp {
            self.last_duration = now - time;
        };
    }

    /// Report that processing of a tick representing instant _now_ has ended.
    pub fn end_tick(&mut self, now: Instant) {
        self.last_timestamp = Some(now);
        self.completed += 1;
    }
}

impl Default for TickStats {
    fn default() -> Self {
        Self {
            last_duration: Duration::from_secs(0),
            last_timestamp: None,
            completed: 0,
        }
    }
}

/// An active play session in a world, controlled and presented in realtime with GUI.
pub struct Game {
    world: World,
    view: View,
    view_settings: view::Parameters,
    control: Control,
    logic: Logic,

    resources: Rc<Resources>,

    logic_ticks: TickStats,
    presentation_ticks: TickStats,
}

impl Game {
    /// tmp: should accept World and Logic externally probably
    pub fn new(resources: Rc<Resources>, gui: &mut crate::gui::Gui) -> Self {
        Self {
            world: World::new(&resources),
            view: View::new(gui),
            view_settings: view::Parameters {
                camera: view::Camera::PlayerCharacter,
                fov: (75.0 as crate::gui::Float).to_radians(),
            },
            control: Control::new(),
            logic: Logic::new(),

            resources,

            logic_ticks: TickStats {
                last_duration: crate::world::target_tick_duration(),
                ..Default::default()
            },
            presentation_ticks: TickStats {
                // FPS is unknown during first frame, assume 60
                last_duration: Duration::from_secs(1) / 60,
                ..Default::default()
            },
        }
    }

    /// Run at least one presentation tick and possibly some logic ticks to advance simulation to
    /// _now_.
    ///
    /// Should be called exactly once per frame.
    pub fn tick(&mut self, now: Instant) {
        crate::crash::with_context(("", || "Game tick"), || {
            loop {
                let last_logic_tick = *self.logic_ticks.last_timestamp.get_or_insert(now);
                let next_logic_tick = last_logic_tick + crate::world::target_tick_duration();
                if next_logic_tick >= now {
                    break;
                }
                self.tick_presentation(next_logic_tick);
                self.tick_logic(now);
            }

            self.tick_presentation(now);
        });
    }

    /// Execute a single logic tick and flush [`Game::buffered_events`].
    fn tick_logic(&mut self, now: Instant) {
        crate::crash::with_context(("Tick phase", || "logic"), || {
            self.logic_ticks.start_tick(now);
            self.world.process(Event::LogicTick, &self.logic);
            self.logic_ticks.end_tick(now);
        });
    }

    /// Execute a single presentation tick and populate [`Game::buffered_events`] with new events.
    fn tick_presentation(&mut self, now: Instant) {
        crate::crash::with_context(("Tick phase", || "presentation"), || {
            self.presentation_ticks.start_tick(now);

            let presentation_tick = Event::PresentationTick {
                duration: self.presentation_ticks.last_duration,
            };

            for event in self.control.pending_events().chain([presentation_tick]) {
                self.world.process(event, &self.logic);
            }

            self.presentation_ticks.end_tick(now);
        });
    }

    /// React to GUI input.
    pub fn on_input(&mut self, input: crate::gui::Input, gui: &mut crate::gui::Gui) {
        self.control.on_input(input, gui, &self.world);
    }
}

impl Drawable for Game {
    fn draw(&mut self, dcf: &mut Dcf) {
        crate::crash::with_context(("", || "Game draw"), || {
            self.control.draw(dcf);
            let mut parameters = self.view_settings.clone();
            self.control.tweak_view_parameters(&mut parameters);
            self.view
                .draw(dcf, &self.world, &self.resources, &parameters);
        });
    }
}
