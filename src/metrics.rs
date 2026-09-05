use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

// --- 1. ZAPISNIK SPAJKOVA ---
#[derive(Clone, Debug)]
pub struct SpikeEvent {
    pub time_ms: f32,
    pub neuron_id: usize,
}

pub struct SpikeRecorder {
    events: Vec<SpikeEvent>,
}

impl SpikeRecorder {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn record(&mut self, spiked_ids: &[usize], current_time_ms: f32) {
        for &id in spiked_ids {
            self.events.push(SpikeEvent {
                time_ms: current_time_ms,
                neuron_id: id,
            });
        }
    }

    pub fn export_to_csv<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        writeln!(writer, "time_ms,neuron_id")?;
        for event in &self.events {
            writeln!(writer, "{:.2},{}", event.time_ms, event.neuron_id)?;
        }

        writer.flush()?;
        Ok(())
    }
}

// --- 2. PRAĆENJE FREKVENCIJE (HERCI) ---
pub struct MetricsTracker {
    num_excitatory: usize,
    num_inhibitory: usize,
    excitatory_spike_count: usize,
    inhibitory_spike_count: usize,
    window_duration_ms: f32,
    accumulated_time_ms: f32,
}

impl MetricsTracker {
    pub fn new(num_excitatory: usize, num_inhibitory: usize, window_duration_ms: f32) -> Self {
        Self {
            num_excitatory,
            num_inhibitory,
            excitatory_spike_count: 0,
            inhibitory_spike_count: 0,
            window_duration_ms,
            accumulated_time_ms: 0.0,
        }
    }

    pub fn update(&mut self, spiked_ids: &[usize], dt_ms: f32) {
        self.accumulated_time_ms += dt_ms;

        for &id in spiked_ids {
            if id < self.num_excitatory {
                self.excitatory_spike_count += 1;
            } else {
                self.inhibitory_spike_count += 1;
            }
        }

        if self.accumulated_time_ms >= self.window_duration_ms {
            let window_sec = self.accumulated_time_ms / 1000.0;

            let rate_exc = (self.excitatory_spike_count as f32) 
                / (self.num_excitatory as f32 * window_sec);

            let rate_inh = (self.inhibitory_spike_count as f32) 
                / (self.num_inhibitory as f32 * window_sec);

            println!(
                "[METRICS] Exc Frequency: {:.2} Hz | Inh Frequency: {:.2} Hz",
                rate_exc, rate_inh
            );

            self.excitatory_spike_count = 0;
            self.inhibitory_spike_count = 0;
            self.accumulated_time_ms = 0.0;
        }
    }
}