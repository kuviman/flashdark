use super::*;

#[derive(Default)]
struct RngEntry {
    current_value: f32,
    next_change: f32,
    time: f32,
}

pub struct RngState {
    entries: RefCell<HashMap<u32, RngEntry>>,
}

impl RngState {
    pub fn new() -> Self {
        Self { entries: default() }
    }

    #[track_caller]
    pub fn get(&self, frequency: f32) -> f32 {
        let caller = std::panic::Location::caller();
        let seed = caller.line() * 1000 + caller.column();
        let mut entries = self.entries.borrow_mut();
        let entry = entries.entry(seed).or_default();
        entry.time = 1.0 / frequency;
        entry.current_value
    }

    pub fn update(&mut self, delta_time: f32) {
        for entry in self.entries.get_mut().values_mut() {
            entry.next_change -= delta_time;
            while entry.next_change < 0.0 {
                entry.next_change += entry.time;
                entry.current_value = global_rng().gen();
            }
        }
    }
}
