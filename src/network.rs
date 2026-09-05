use crate::neuron::{NeuronType, SpikingNeuron};
use crate::synapse::Synapse;

pub struct Network {
    pub neurons: Vec<SpikingNeuron>,
    pub synapses: Vec<Vec<Option<Synapse>>>, // Matrica sinapsi [pre_id][post_id]
}

impl Network {
    pub fn new(num_neurons: usize, excitatory_ratio: f64) -> Self {
        let mut neurons = Vec::new();
        let num_excitatory = (num_neurons as f64 * excitatory_ratio) as usize;

        // 1. Kreiranje neurona (80% Ekscitatorni, 20% Inhibitorni)
        for i in 0..num_neurons {
            let n_type = if i < num_excitatory {
                NeuronType::Excitatory
            } else {
                NeuronType::Inhibitory
            };
            neurons.push(SpikingNeuron::new(n_type));
        }

        // 2. Kreiranje matrica sinapsi (svak sa svakim, izuzev sa samim sobom)
        let mut synapses = vec![vec![None; num_neurons]; num_neurons];
        for i in 0..num_neurons {
            for j in 0..num_neurons {
                if i != j {
                    // Početna nasumična/fiksna težina sinapse
                    synapses[i][j] = Some(Synapse::new(0.3));
                }
            }
        }

        Network { neurons, synapses }
    }

    // Simulacija jednog vremenskog koraka (dt milisekunda)
    pub fn step(&mut self, external_inputs: &[f64], current_time: f64, dt: f64) -> Vec<bool> {
        let num_neurons = self.neurons.len();
        let mut current_inputs = vec![0.0; num_neurons];

        // Dodajemo spoljne podražaje
        for i in 0..num_neurons {
            current_inputs[i] += external_inputs[i];
        }

        // Prolazimo kroz sve neurone i proveravamo ko okida
        let mut spikes = vec![false; num_neurons];
        for i in 0..num_neurons {
            spikes[i] = self.neurons[i].update(current_inputs[i], dt);

            if spikes[i] {
                // Ako je neuron okinuo, ažuriraj STDP i pošalji struju susedima za sledeći korak
                for j in 0..num_neurons {
                    if let Some(ref mut synapse) = self.synapses[i][j] {
                        synapse.on_pre_spike(current_time);

                        // Signal prema susedu zavisi od tipa neurona!
                        let signal_strength = match self.neurons[i].neuron_type {
                            NeuronType::Excitatory => synapse.weight * 15.0,  // Podstiče (+ napon)
                            NeuronType::Inhibitory => -synapse.weight * 20.0, // Inhibira (- napon)
                        };
                        current_inputs[j] += signal_strength;
                    }
                }
            }
        }

        // Ažuriramo STDP za post-synaptic impulse
        for j in 0..num_neurons {
            if spikes[j] {
                for i in 0..num_neurons {
                    if let Some(ref mut synapse) = self.synapses[i][j] {
                        synapse.on_post_spike(current_time);
                    }
                }
            }
        }

        spikes
    }
}