use crate::neuron::{NeuronType, SpikingNeuron};
use crate::synapse::Synapse;
use std::sync::{Arc, Mutex, Barrier};
use std::thread;

pub struct ParallelNetwork {
    pub num_neurons: usize,
    pub num_threads: usize,
}

impl ParallelNetwork {
    pub fn new(num_neurons: usize, num_threads: usize) -> Self {
        ParallelNetwork {
            num_neurons,
            num_threads,
        }
    }

    pub fn run_simulation(&self, steps: usize, dt: f64) {
        let num_neurons = self.num_neurons;
        let num_threads = self.num_threads;

        // 1. Inicijalizacija neurona (80% Ekscitatorni, 20% Inhibitorni)
        let num_excitatory = (num_neurons as f64 * 0.8) as usize;
        let mut neurons = Vec::new();
        for i in 0..num_neurons {
            let n_type = if i < num_excitatory {
                NeuronType::Excitatory
            } else {
                NeuronType::Inhibitory
            };
            neurons.push(SpikingNeuron::new(n_type));
        }

        // 2. Inicijalizacija sinapsi
        let mut synapses = vec![vec![None; num_neurons]; num_neurons];
        for i in 0..num_neurons {
            for j in 0..num_neurons {
                if i != j {
                    synapses[i][j] = Some(Synapse::new(0.3));
                }
            }
        }

        // Deljeni resursi obijeni u Arc<Mutex<T>> za bezbedan višenitni pristup
        let neurons_arc = Arc::new(Mutex::new(neurons));
        let synapses_arc = Arc::new(Mutex::new(synapses));
        
        // Stanje impulsa u tekućem koraku (Spike Buffer)
        let spikes_arc = Arc::new(Mutex::new(vec![false; num_neurons]));
        
        // Ulazne struje za svaki neuron
        let current_inputs_arc = Arc::new(Mutex::new(vec![0.0; num_neurons]));

        // Barijera osigurava da sve niti završe trenutni milisekundu pre prelaska na sledeću
        let barrier = Arc::new(Barrier::new(num_threads));

        let chunk_size = (num_neurons + num_threads - 1) / num_threads;
        let mut handles = vec![];

        for thread_id in 0..num_threads {
            let start_idx = thread_id * chunk_size;
            let end_idx = (start_idx + chunk_size).min(num_neurons);

            if start_idx >= num_neurons {
                break;
            }

            let neurons_ref = Arc::clone(&neurons_arc);
            let synapses_ref = Arc::clone(&synapses_arc);
            let spikes_ref = Arc::clone(&spikes_arc);
            let inputs_ref = Arc::clone(&current_inputs_arc);
            let barrier_ref = Arc::clone(&barrier);

            // Pokretanje zasebne RUST NITI za segment neurona [start_idx..end_idx]
            let handle = thread::spawn(move || {
                for t in 0..steps {
                    let current_time = t as f64;

                    // A) Pročitaj ulaznu struju i ažuriraj dodeljeni opseg neurona
                    let mut local_spikes = vec![(0, false); end_idx - start_idx];
                    {
                        let mut neurons = neurons_ref.lock().unwrap();
                        let mut inputs = inputs_ref.lock().unwrap();

                        for (i, neuron_idx) in (start_idx..end_idx).enumerate() {
                            // Spoljni podražaj u t=3ms za ulazni neuron 0
                            if t >= 3 && t <= 5 && neuron_idx == 0 {
                                inputs[neuron_idx] += 8.0;
                            }

                            let current = inputs[neuron_idx];
                            inputs[neuron_idx] = 0.0; // Resetuj unete struje za sledeći krug

                            let spiked = neurons[neuron_idx].update(current, dt);
                            local_spikes[i] = (neuron_idx, spiked);
                        }
                    }

                    // B) Zapiši ispaljene impulse u deljeni bafer
                    {
                        let mut spikes = spikes_ref.lock().unwrap();
                        for &(idx, spiked) in &local_spikes {
                            spikes[idx] = spiked;
                        }
                    }

                    // SINHRONIZACIJA: Čekaj da sve niti završe izračunavanje impulsa
                    barrier_ref.wait();

                    // C) Propagacija signala i STDP učenje
                    {
                        let mut synapses = synapses_ref.lock().unwrap();
                        let spikes = spikes_ref.lock().unwrap();
                        let mut inputs = inputs_ref.lock().unwrap();
                        let neurons = neurons_ref.lock().unwrap();

                        for &(pre_idx, spiked) in &local_spikes {
                            if spiked {
                                for post_idx in 0..num_neurons {
                                    if let Some(ref mut syn) = synapses[pre_idx][post_idx] {
                                        syn.on_pre_spike(current_time);

                                        let signal = match neurons[pre_idx].neuron_type {
                                            NeuronType::Excitatory => syn.weight * 15.0,
                                            NeuronType::Inhibitory => -syn.weight * 20.0,
                                        };
                                        inputs[post_idx] += signal;
                                    }
                                }
                            }

                            // Post-synaptic STDP provera
                            if spikes[pre_idx] {
                                for src_idx in 0..num_neurons {
                                    if let Some(ref mut syn) = synapses[src_idx][pre_idx] {
                                        syn.on_post_spike(current_time);
                                    }
                                }
                            }
                        }
                    }

                    // SINHRONIZACIJA: Čekaj da sve niti završe raspodelu signala pre novog koraka
                    barrier_ref.wait();
                }
            });

            handles.push(handle);
        }

        // Čekanje da sve niti završe kompletnu simulaciju
        for handle in handles {
            handle.join().unwrap();
        }
    }
}