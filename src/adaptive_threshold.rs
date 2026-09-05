pub struct AdaptiveThreshold {
    /// Trenutni pragovi za svaki neuron (mV)
    pub current_thresholds: Vec<f32>,
    /// Bazni (mirovanja) prag (npr. -50.0 mV)
    pub base_threshold: f32,
    /// Vremenska konstanta opadanja praga (ms) - npr. 100.0 ms
    pub tau_adaptive: f32,
    /// Koliko se prag poveća nakon svakog ispaljenog spajka (mV) - npr. 2.0 mV
    pub spike_step: f32,
}

impl AdaptiveThreshold {
    pub fn new(num_neurons: usize, base_threshold: f32, tau_adaptive: f32, spike_step: f32) -> Self {
        Self {
            current_thresholds: vec![base_threshold; num_neurons],
            base_threshold,
            tau_adaptive,
            spike_step,
        }
    }

    /// Ažurira pragove za jedan korak vremena (opadanje ka baznoj vrednosti)
    pub fn update(&mut self, dt_ms: f32) {
        for v_th in self.current_thresholds.iter_mut() {
            // Relaksacija praga nazad ka base_threshold
            *v_th += (self.base_threshold - *v_th) * (dt_ms / self.tau_adaptive);
        }
    }

    /// Kada neuron ispali spajk, dodaje se skok na njegov prag
    pub fn on_spike(&mut self, neuron_id: usize) {
        self.current_thresholds[neuron_id] += self.spike_step;
    }

    /// Vraća trenutni prag za zadati neuron
    pub fn get_threshold(&self, neuron_id: usize) -> f32 {
        self.current_thresholds[neuron_id]
    }
}