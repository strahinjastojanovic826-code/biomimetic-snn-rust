use crate::topology::NetworkTopology;

pub struct HomeostaticController {
    /// Ciljana prosečna frekvencija ispaljivanja za svaki neuron (u Hz)
    pub target_rate_hz: f32,
    /// Koliko brzo se sinapse prilagođavaju (learning rate za homeostazu)
    pub scaling_rate: f32,
    /// Minimalna dozvoljena težina sinapse
    pub w_min: f32,
    /// Maksimalna dozvoljena težina sinapse
    pub w_max: f32,
    /// Brojač spajkova po neuronu u tekućem prozoru
    spike_counts: Vec<usize>,
    /// Vreme akumulirano u tekućem prozoru (ms)
    accumulated_time_ms: f32,
}

impl HomeostaticController {
    pub fn new(
        num_neurons: usize,
        target_rate_hz: f32,
        scaling_rate: f32,
        w_min: f32,
        w_max: f32,
    ) -> Self {
        Self {
            target_rate_hz,
            scaling_rate,
            w_min,
            w_max,
            spike_counts: vec![0; num_neurons],
            accumulated_time_ms: 0.0,
        }
    }

    /// Beleži spajkove u tekućem koraku
    pub fn record_spikes(&mut self, spiked_ids: &[usize], dt_ms: f32) {
        self.accumulated_time_ms += dt_ms;
        for &id in spiked_ids {
            self.spike_counts[id] += 1;
        }
    }

    /// Izvršava Synaptic Scaling ako je prošao zadati prozor (npr. svakih 1000ms = 1s)
    pub fn update_synapses_if_needed(
        &mut self,
        topology: &mut NetworkTopology,
        window_duration_ms: f32,
    ) {
        if self.accumulated_time_ms < window_duration_ms {
            return; // Još nije vreme za skaliranje
        }

        let duration_sec = self.accumulated_time_ms / 1000.0;

        // Prolazimo kroz svaki neuron koji šalje sinapse
        for src_id in 0..topology.outgoing_synapses.len() {
            // Izračunaj njegovu trenutnu frekvenciju (Hz)
            let actual_rate_hz = (self.spike_counts[src_id] as f32) / duration_sec;

            // Razlika: ako je pozitivna -> neuron previše radi -> smanjuj težine
            // Ako je negativna -> neuron je neaktivan -> pojačaj težine
            let error = actual_rate_hz - self.target_rate_hz;
            let scale_factor = 1.0 - (self.scaling_rate * error);

            // Primenjujemo skaliranje na sve odlazne sinapse tog neurona
            for synapse in &mut topology.outgoing_synapses[src_id] {
                // Skaliranje
                synapse.weight *= scale_factor;

                // Tvrdo ograničenje (Clipping)
                if synapse.weight > 0.0 {
                    // Ekscitatorna sinapsa
                    synapse.weight = synapse.weight.clamp(self.w_min, self.w_max);
                } else if synapse.weight < 0.0 {
                    // Inhibitorna sinapsa (čuvamo negativan znak)
                    synapse.weight = synapse.weight.clamp(-self.w_max, -self.w_min);
                }
            }

            // Resetuj brojač spajkova za ovog neurona
            self.spike_counts[src_id] = 0;
        }

        // Resetuj akumulirano vreme
        self.accumulated_time_ms = 0.0;
    }
}