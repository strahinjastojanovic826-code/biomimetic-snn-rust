use crate::rng::Xorshift32;

#[derive(Clone)]
pub struct Synapse {
    pub target_id: usize,
    pub weight: f32,
}

pub struct NetworkTopology {
    pub outgoing_synapses: Vec<Vec<Synapse>>,
}

impl NetworkTopology {
    pub fn create_sparse_network(
        num_neurons: usize,
        connectivity_prob: f32,
        excitatory_ratio: f32,
        rng: &mut Xorshift32,
    ) -> Self {
        let mut outgoing_synapses = vec![Vec::new(); num_neurons];
        let num_excitatory = (num_neurons as f32 * excitatory_ratio) as usize;

        for src in 0..num_neurons {
            let is_excitatory = src < num_excitatory;

            for tgt in 0..num_neurons {
                if src == tgt {
                    continue;
                }

                if rng.next_f32() < connectivity_prob {
                    let base_weight = rng.next_f32() * 0.5 + 0.1;
                    let weight = if is_excitatory {
                        base_weight
                    } else {
                        -base_weight * 2.0
                    };

                    outgoing_synapses[src].push(Synapse {
                        target_id: tgt,
                        weight,
                    });
                }
            }
        }

        NetworkTopology { outgoing_synapses }
    }
}