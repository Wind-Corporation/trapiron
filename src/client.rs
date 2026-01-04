mod control;
mod view;

use std::time::{Duration, Instant};

use crate::{
    client::{control::Control, view::View},
    gui::{Dcf, Drawable},
    logic::Logic,
    world::{Event, World},
};

struct TickStats {
    last_duration: Duration,
    last_timestamp: Option<Instant>,
    completed: u64,
}

impl TickStats {
    pub fn start_tick(&mut self, now: Instant) {
        if let Some(time) = self.last_timestamp {
            self.last_duration = now - time;
        };
    }

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

pub struct Game {
    world: World,
    view: View,
    control: Control,
    logic: Logic,

    buffered_events: Vec<Event>,

    logic_ticks: TickStats,
    presentation_ticks: TickStats,
}

pub const TARGET_TPS: u32 = 20;

pub fn target_tick_duration() -> Duration {
    Duration::from_secs(1) / TARGET_TPS
}

impl Game {
    pub fn new(gui: &mut crate::gui::Gui) -> Self {
        Self {
            world: World::new(),
            view: View::new(gui),
            control: Control::new(),
            logic: Logic::new(),

            buffered_events: Vec::with_capacity(256),

            logic_ticks: TickStats {
                last_duration: target_tick_duration(),
                ..Default::default()
            },
            presentation_ticks: TickStats {
                last_duration: Duration::from_secs(1) / 60,
                ..Default::default()
            },
        }
    }

    pub fn tick(&mut self, now: Instant) {
        crate::crash::with_context(("", || "Game tick"), || {
            loop {
                let last_logic_tick = *self.logic_ticks.last_timestamp.get_or_insert(now);
                let next_logic_tick = last_logic_tick + target_tick_duration();
                if next_logic_tick >= now {
                    break;
                }
                self.tick_presentation(next_logic_tick);
                self.tick_logic(now);
            }

            self.tick_presentation(now);
        });
    }

    fn tick_logic(&mut self, now: Instant) {
        crate::crash::with_context(("Tick phase", || "logic"), || {
            self.logic_ticks.start_tick(now);

            while let Some(event) = self.buffered_events.pop() {
                self.world.process(event, &self.logic);
            }

            self.world.process(Event::Tick, &self.logic);

            self.logic_ticks.end_tick(now);
        });
    }

    fn tick_presentation(&mut self, now: Instant) {
        crate::crash::with_context(("Tick phase", || "presentation"), || {
            self.presentation_ticks.start_tick(now);

            let new_events_begin = self.buffered_events.len();
            self.buffered_events.push(Event::PresentationTick {
                duration: self.presentation_ticks.last_duration,
            });
            self.control.fetch_into(&mut self.buffered_events);
            let new_events_end = self.buffered_events.len();

            for event in &self.buffered_events[new_events_begin..new_events_end] {
                self.world.process_presentation(event, &self.logic);
            }

            self.presentation_ticks.end_tick(now);
        });
    }
}

impl Drawable for Game {
    fn draw(&mut self, dcf: &mut Dcf) {
        crate::crash::with_context(("", || "Game draw"), || {
            self.view.draw(dcf, &self.world);
        });
    }
}
