#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NeuronType {
    Excitatory, // Podstiče (pozitivan uticaj)
    Inhibitory, // Smiruje (negativan uticaj)
}

#[derive(Debug, Clone)]
pub struct SpikingNeuron {
    pub v_membrane: f64,
    pub v_rest: f64,
    pub v_threshold: f64,
    pub v_reset: f64,
    pub resistance: f64,
    pub capacitance: f64,
    pub neuron_type: NeuronType,
}

impl SpikingNeuron {
    pub fn new(neuron_type: NeuronType) -> Self {
        SpikingNeuron {
            v_membrane: -70.0,
            v_rest: -70.0,
            v_threshold: -55.0,
            v_reset: -75.0,
            resistance: 10.0,
            capacitance: 1.0,
            neuron_type,
        }
    }

    pub fn update(&mut self, input_current: f64, dt: f64) -> bool {
        let dv = (-(self.v_membrane - self.v_rest) + self.resistance * input_current) / (self.resistance * self.capacitance);
        self.v_membrane += dv * dt;

        if self.v_membrane >= self.v_threshold {
            self.v_membrane = self.v_reset;
            true // Impulse!
        } else {
            false
        }
    }
}