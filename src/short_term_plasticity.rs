#[derive(Clone, Debug)]
pub struct StpSynapseState {
    /// Trenutni nivo raspoloživih neurotransmitera R (raspon 0.0 do 1.0)
    pub r: f32,
    /// Trenutna iskorišćenost / verovatnoća oslobađanja u (raspon 0.0 do 1.0)
    pub u: f32,
}

pub struct ShortTermPlasticity {
    pub states: Vec<StpSynapseState>,
    /// Bazna iskorišćenost u stanju mirovanja (npr. 0.2)
    pub u_base: f32,
    /// Vremenska konstanta oporavka resursa R (ms) - npr. 800.0 ms (Depresija)
    pub tau_rec: f32,
    /// Vremenska konstanta opadanja facilitacije u (ms) - npr. 200.0 ms (Facilitacija)
    pub tau_fac: f32,
}

impl ShortTermPlasticity {
    pub fn new(
        num_synapses: usize,
        u_base: f32,
        tau_rec: f32,
        tau_fac: f32,
    ) -> Self {
        Self {
            states: vec![
                StpSynapseState {
                    r: 1.0,      // Potpuno napunjeni resursi na početku
                    u: u_base,   // Bazna verovatnoća
                };
                num_synapses
            ],
            u_base,
            tau_rec,
            tau_fac,
        }
    }

    /// Kontinuirani oporavak resursa i opadanje facilitacije tokom proteklog vremena (dt)
    pub fn update(&mut self, dt_ms: f32) {
        for state in self.states.iter_mut() {
            // R se oporavlja ka 1.0
            state.r += (1.0 - state.r) * (dt_ms / self.tau_rec);
            // u opada ka u_base
            state.u += (self.u_base - state.u) * (dt_ms / self.tau_fac);
        }
    }

    /// Izračunava efektivnu težinu sinapse kada stigne spajk i ažurira interno stanje (u i R)
    pub fn process_spike(&mut self, synapse_idx: usize, base_weight: f32) -> f32 {
        let state = &mut self.states[synapse_idx];

        // 1. Skok u facilitaciji pre oslobađanja
        state.u += self.u_base * (1.0 - state.u);

        // 2. Izračunavanje efektivne težine (koliko se zapravo prenese)
        let effective_weight = base_weight * state.u * state.r;

        // 3. Potrošnja resursa R usled oslobađanja neurotransmitera
        state.r -= state.u * state.r;

        effective_weight
    }
}