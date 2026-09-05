#[derive(Debug, Clone)]
pub struct Synapse {
    pub weight: f64,
    pub weight_max: f64,
    pub weight_min: f64,
    pub last_spike_pre: f64,
    pub last_spike_post: f64,
    
    // STDP parametri
    pub a_plus: f64,
    pub a_minus: f64,
    pub tau_stdp: f64,
}

impl Synapse {
    pub fn new(initial_weight: f64) -> Self {
        Synapse {
            weight: initial_weight,
            weight_max: 1.0,
            weight_min: 0.0,
            last_spike_pre: -1000.0,
            last_spike_post: -1000.0,
            a_plus: 0.05,
            a_minus: 0.055,
            tau_stdp: 20.0,
        }
    }

    // Kada pre-sinaptički neuron okine
    pub fn on_pre_spike(&mut self, current_time: f64) {
        self.last_spike_pre = current_time;
        
        let dt = current_time - self.last_spike_post;
        if dt >= 0.0 {
            // Pre-synaptic je okinuo POSLE post-synaptic -> VEZA SLABI (LTD)
            let dw = -self.a_minus * (-dt / self.tau_stdp).exp();
            self.weight = (self.weight + dw).clamp(self.weight_min, self.weight_max);
        }
    }

    // Kada post-sinaptički neuron okine
    pub fn on_post_spike(&mut self, current_time: f64) {
        self.last_spike_post = current_time;
        
        let dt = current_time - self.last_spike_pre;
        if dt >= 0.0 {
            // Post-synaptic je okinuo POSLE pre-synaptic -> VEZA OJAČAVA (LTP)
            let dw = self.a_plus * (-dt / self.tau_stdp).exp();
            self.weight = (self.weight + dw).clamp(self.weight_min, self.weight_max);
        }
    }
}