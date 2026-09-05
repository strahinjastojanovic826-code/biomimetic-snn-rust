#[derive(Clone, Debug)]
pub struct DelayedSignal {
    pub target_id: usize,
    pub weight: f32,
}

pub struct DelayBuffer {
    /// Cirkularni bafer: redovi predstavljaju diskretne vremenske korake unapred
    slots: Vec<Vec<DelayedSignal>>,
    /// Trenutna pozicija glave u cirkularnom baferu
    head: usize,
    /// Maksimalno podržano kašnjenje u koracima
    max_delay_steps: usize,
}

impl DelayBuffer {
    pub fn new(max_delay_steps: usize) -> Self {
        Self {
            slots: vec![Vec::new(); max_delay_steps],
            head: 0,
            max_delay_steps,
        }
    }

    /// Dodaje signal u bafer sa određenim kašnjenjem (izraženim u koracima)
    pub fn schedule_signal(&mut self, target_id: usize, weight: f32, delay_steps: usize) {
        if delay_steps == 0 {
            return;
        }

        // Osiguravamo da kašnjenje ne prelazi veličinu bafera
        let clamped_delay = delay_steps.min(self.max_delay_steps);
        
        // Izračunavamo slot u cirkularnom baferu u koji upisujemo signal
        let slot_idx = (self.head + clamped_delay - 1) % self.max_delay_steps;
        
        self.slots[slot_idx].push(DelayedSignal { target_id, weight });
    }

    /// Preuzima sve signale koji su dospeli za isporuku u TRENUTNOM koraku i prazni taj slot
    pub fn pop_current_signals(&mut self) -> Vec<DelayedSignal> {
        let current_signals = std::mem::take(&mut self.slots[self.head]);

        // Pomera glavu bafera na sledeći korak za narednu iteraciju
        self.head = (self.head + 1) % self.max_delay_steps;

        current_signals
    }
}