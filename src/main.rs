mod neuron;
mod synapse;
mod network;
mod encoder;
mod network_parallel;
mod homeostasis;
mod parallel_sim;
mod rng;
mod topology;
mod metrics; 
mod adaptive_threshold;
mod synaptic_delays;
mod refractory_period;
mod short_term_plasticity;

//Silicon is a thing of the past 
//The neuron is the future

use metrics::{MetricsTracker, SpikeRecorder};
use rng::Xorshift32;
use topology::NetworkTopology;
use encoder::{EncodingType, SpikeEncoder};
use network::Network;
use network_parallel::ParallelNetwork;
use neuron::{NeuronType, SpikingNeuron};
use synapse::Synapse;
use std::time::Instant;
use homeostasis::HomeostaticController; 
use parallel_sim::ParallelSimulator;
use adaptive_threshold::AdaptiveThreshold;
use synaptic_delays::DelayBuffer;
use refractory_period::RefractoryState;
use short_term_plasticity::ShortTermPlasticity;

fn main() {
    println!("==================================================");
    println!("1. PARALLEL SIMULATION (MULTITHREADED OPERATION)");
    println!("==================================================");
    run_parallel_demo();

    println!("\n==================================================");
    println!("2. SPIKE ENCODING (TRANSLATION OF DATA INTO IMPULSES)");
    println!("==================================================");
    run_encoding_demo();

    println!("\n==================================================");
    println!("3. BASIC STDP TWO-NEURON TEST");
    println!("==================================================");
    run_stdp_demo();

    println!("\n==================================================");
    println!("4. NETWORK WITH EXCITATORY AND INHIBITORY NEURONS");
    println!("==================================================");
    if let Err(e) = run_network_demo() {
        eprintln!("Error running network simulation: {:?}", e);
    }
}

// 1. Paralelna simulacija na više niti
fn run_parallel_demo() {
    let num_neurons: usize = 100;
    let num_threads: usize = 4;
    let steps: usize = 50;

    println!(
        "Initializing the network with {} neurons on the {} RUST niti...",
        num_neurons, num_threads
    );

    let net = ParallelNetwork::new(num_neurons, num_threads);
    let start_time = Instant::now();
    net.run_simulation(steps, 1.0);
    let duration = start_time.elapsed();

    println!("Simulation completed successfully for: {:?}", duration);
}

// 2. Latency Encoding demo
fn run_encoding_demo() {
    let mut net = Network::new(5, 0.8);
    let dt: f64 = 1.0;
    let time_window: f64 = 30.0;

    let input_pixels: Vec<f64> = vec![0.1, 0.5, 0.9];
    let encoder = SpikeEncoder::new(EncodingType::Latency, time_window);

    println!("Input pixels: {:?}", input_pixels);
    println!("Pixel 2 (0.9) okida FIRST, Pixel 1 (0.5) SECOND, Pixel 0 (0.1) THE LAST.\n");

    for t in 0..30 {
        let current_time: f64 = t as f64;
        let mut external_inputs: Vec<f64> = vec![0.0; 5];

        for (i, &pixel_val) in input_pixels.iter().enumerate() {
            external_inputs[i] = encoder.encode_value(pixel_val, current_time);
        }

        let spikes: Vec<bool> = net.step(&external_inputs, current_time, dt);

        let mut visual = String::new();
        for (i, spiked) in spikes.iter().enumerate() {
            if *spiked {
                let val: f64 = if i < 3 { input_pixels[i] } else { 0.0 };
                visual.push_str(&format!(" N{}(val:{:.1}) ", i, val));
            }
        }

        if !visual.is_empty() {
            println!("t = {:2} ms | Fired an impulse: {}", t, visual);
        }
    }
}

// 3. Osnovni STDP test na 2 neurona
fn run_stdp_demo() {
    let mut pre_neuron = SpikingNeuron::new(NeuronType::Excitatory);
    let mut post_neuron = SpikingNeuron::new(NeuronType::Excitatory);
    let mut synapse = Synapse::new(0.5);

    println!("Initial synaptic weight: {:.4}\n", synapse.weight);

    let dt: f64 = 1.0;
    for t in 0..60 {
        let time: f64 = t as f64;

        let current_pre: f64 = if t >= 10 && t <= 12 { 5.0 } else { 0.0 };
        let pre_spiked: bool = pre_neuron.update(current_pre, dt);
        if pre_spiked {
            synapse.on_pre_spike(time);
        }

        let current_post: f64 = if t >= 15 && t <= 17 { 5.0 } else { 0.0 };
        let post_spiked: bool = post_neuron.update(current_post, dt);
        if post_spiked {
            synapse.on_post_spike(time);
        }

        if pre_spiked || post_spiked {
            println!(
                "t = {:2} ms | Pre: {:5} | Post: {:5} | New weight: {:.4}",
                t, pre_spiked, post_spiked, synapse.weight
            );
        }
    }

    println!("\nFinal synaptic weight: {:.4}", synapse.weight);
}

// 4. Mreža sa neurona i balansiranom inhibicijom
fn run_network_demo() -> std::io::Result<()> {
    // 1. Parametri mreže
    let num_neurons: usize = 100;
    let dt_ms: f32 = 1.0;
    let total_steps: i32 = 1000;           // Simuliramo 1000 ms (1 sekundu)
    let poisson_rate_hz: f32 = 10.0;       // Poisson-ov šum od 10 Hz
    let poisson_input_current: f32 = 15.0; // Jačina spoljašnjeg impulsa

    // 2. Inicijalizacija RNG-a, topologije i instrumenata
    let mut global_rng = Xorshift32::new(12345);
    let mut topology = NetworkTopology::create_sparse_network(
        num_neurons,
        0.10, // 10% popunjenost konekcija
        0.80, // 80% ekscitatorni, 20% inhibitorni
        &mut global_rng,
    );

    let mut rngs: Vec<Xorshift32> = (0..num_neurons)
        .map(|i| Xorshift32::new((i + 1) as u32 * 999))
        .collect();

    let mut potentials: Vec<f32> = vec![-65.0; num_neurons];

    // Zapisnici i metrika
    let mut recorder = SpikeRecorder::new();
    let mut tracker = MetricsTracker::new(80, 20, 100.0); // 80 exc, 20 inh, 100ms prozor
    let mut homeostasis = HomeostaticController::new(num_neurons, 5.0, 0.01, 3.0, 1000.0);

    println!("Simulacija započeta...");

    // 3. Glavna simulaciona petlja
    for step in 0..total_steps {
        let current_time_ms: f32 = step as f32 * dt_ms;
        let mut spiked_this_step: Vec<usize> = Vec::new();

        // KORAK A: Dodaj Poisson-ov šum i proveri spajkove
        for i in 0..num_neurons {
            if rngs[i].poisson_spike(poisson_rate_hz, dt_ms) {
                potentials[i] += poisson_input_current;
            }

            // Provera praga (-50 mV)
            if potentials[i] >= -50.0 {
                spiked_this_step.push(i);
                potentials[i] = -65.0; // Reset na odmor
            }
        }

        // KORAK B: Propagacija spajkova kroz retku mrežu
        for &src_id in &spiked_this_step {
            for synapse in &topology.outgoing_synapses[src_id] {
                potentials[synapse.target_id] += synapse.weight;
            }
        }

        // KORAK C: Praćenje i Homeostaza
        recorder.record(&spiked_this_step, current_time_ms);
        tracker.update(&spiked_this_step, dt_ms);
        homeostasis.record_spikes(&spiked_this_step, dt_ms);
        homeostasis.update_synapses_if_needed(&mut topology, 1000.0);

        // Ispis samo ako je neko ispalio spajk
        if !spiked_this_step.is_empty() {
            println!("Step {} ms | Neurons fired: {:?}", step, spiked_this_step);
        }
    }

    println!("Simulation completed successfully.!");

    // 4. Snimanje rezultata u CSV
    recorder.export_to_csv("spikes_output.csv")?;
    println!("Spikes were successfully recorded in 'spikes_output.csv'!");

    Ok(())
}
