pub struct RefractoryState {
    /// Vreme proteklo od poslednjeg spajka za svaki neuron (u ms)
    pub time_since_last_spike: Vec<f32>,
    /// Trajanje apsolutnog refraktornog perioda (ms) - npr. 2.0 ms
    pub t_abs: f32,
    /// Trajanje relativnog refraktornog perioda (ms) - npr. 10.0 ms
    pub t_rel: f32,
    /// Amplituda podizanja praga tokom relativnog perioda (mV) - npr. 20.0 mV
    pub rel_amplitude: f32,
    /// Vremenska konstanta opadanja relativnog praga (ms) - npr. 5.0 ms
    pub tau_rel: f32,
}

impl RefractoryState {
    pub fn new(
        num_neurons: usize,
        t_abs: f32,
        t_rel: f32,
        rel_amplitude: f32,
        tau_rel: f32,
    ) -> Self {
        Self {
            // Inicijalno postavljamo veliku vrednost da neuroni ne bi bili u refraktornom stanju na startu
            time_since_last_spike: vec![1000.0; num_neurons],
            t_abs,
            t_rel,
            rel_amplitude,
            tau_rel,
        }
    }

    /// Ažurira tajmere za protekli vremenski korak
    pub fn update(&mut self, dt_ms: f32) {
        for t in self.time_since_last_spike.iter_mut() {
            *t += dt_ms;
        }
    }

    /// Zabeleži spajk i resetuj tajmer neurona
    pub fn on_spike(&mut self, neuron_id: usize) {
        self.time_since_last_spike[neuron_id] = 0.0;
    }

    /// Proverava da li je neuron u apsolutnom refraktornom periodu (ne sme da ispali)
    pub fn is_in_absolute(&self, neuron_id: usize) -> bool {
        self.time_since_last_spike[neuron_id] < self.t_abs
    }

    /// Izračunava efektivni prag neurona uzimajući u obzir relativni refraktorni period
    pub fn get_effective_threshold(&self, neuron_id: usize, base_threshold: f32) -> f32 {
        let t = self.time_since_last_spike[neuron_id];

        if t < self.t_abs {
            // U apsolutnom periodu prag je beskonačno visok
            f32::INFINITY
        } else if t < (self.t_abs + self.t_rel) {
            // U relativnom periodu prag opada eksponencijalno
            let dt_rel = t - self.t_abs;
            base_threshold + self.rel_amplitude * (-dt_rel / self.tau_rel).exp()
        } else {
            // Izvan refraktornog perioda prag je bazni
            base_threshold
        }
    }
}