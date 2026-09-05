use std::sync::Arc;
use std::thread;
use crate::topology::NetworkTopology;
use crate::parallel_sim::thread::ScopedJoinHandle;

pub struct ParallelSimulator {
    pub num_neurons: usize,
    pub num_threads: usize,
    pub potentials_current: Vec<f32>,
    pub potentials_next: Vec<f32>,
    pub chunk_size: usize,
    pub worker_slices: Vec<Vec<f32>>,
}

impl ParallelSimulator {
    pub fn new(num_neurons: usize, num_threads: usize) -> Self {
        let chunk_size = num_neurons / num_threads;
        Self {
            num_neurons,
            num_threads,
            potentials_current: vec![-65.0; num_neurons], // Početni potencijal -65mV
            potentials_next: vec![-65.0; num_neurons],
            chunk_size,                          // <-- DODAJ
            worker_slices: vec![vec![-65.0; chunk_size]; num_threads],
        }
    }

    /// Izvršava jedan simulacioni korak u potpunosti paralelno bez Mutex-a
    pub fn step(&mut self, topology: &NetworkTopology, dt_ms: f32) -> Vec<usize> {
        let current_potentials = self.potentials_current.clone();
        let mut all_spikes = Vec::new();

        // std::thread::scope omogućava bezbedno korišćenje referenci unutar niti
        thread::scope(|s| {
            let mut handles: Vec<ScopedJoinHandle<Vec<usize>>> = Vec::new();

            // Podelu worker_slices vršimo odjednom pre petlje pomoću iteratora
            for (thread_idx, current_worker_slice) in self.worker_slices.iter_mut().enumerate() {
                let start_id: usize = thread_idx * self.chunk_size;

                // Referenca umesto move celog niza
                let current_potentials = &current_potentials;

                let handle = s.spawn(move || {
                    let mut local_spikes: Vec<usize> = Vec::new();

                    for local_idx in 0..current_worker_slice.len() {
                        let global_id: usize = start_id + local_idx;

                        // 1. Integracija potencijala (LIF jednačina)
                        let v: f32 = current_potentials[global_id];
                        let mut v_new: f32 = v + (-65.0 - v) * (dt_ms / 10.0);

                        // 2. Dodaj sinaptičke ulaze iz prethodnog koraka
                        for src in 0..topology.outgoing_synapses.len() {
                            if current_potentials[src] >= -50.0 {
                                for syn in &topology.outgoing_synapses[src] {
                                    if syn.target_id == global_id {
                                        v_new += syn.weight;
                                    }
                                }
                            }
                        }

                        // 3. Provera praga za spajk
                        if v_new >= -50.0 {
                            local_spikes.push(global_id);
                            v_new = -65.0; // Reset
                        }

                        // Upisujemo direktno u radni slice
                        current_worker_slice[local_idx] = v_new;
                    }

                    local_spikes
                });

                handles.push(handle);
            }

            // Skupljanje spajkova iz svih niti unutar opsega
            for handle in handles {
                if let Ok(spikes) = handle.join() {
                    all_spikes.extend(spikes);
                }
            }
        });

        all_spikes
    }
}