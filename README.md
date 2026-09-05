# biomimetic-snn-rust

A zero-dependency, multi-threaded Spiking Neural Network (SNN) simulator written in pure Rust. Built purely as a personal hobby project out of curiosity to experiment with biophysically accurate neuron models and lock-free parallel execution on CPU hardware.

## Architecture & Biological Mechanisms

The core design focuses on low-overhead execution using a Data-Oriented Design (DoD) layout and pure standard library Rust.

* **Neuron Dynamics:** Leaky Integrate-and-Fire (LIF) model with adaptive thresholding and absolute/relative refractory states.
* **Synaptic Plasticity:** 
  * Spike-Timing-Dependent Plasticity (STDP) for long-term weight modifications.
  * Tsodyks-Markram Short-Term Plasticity (STP) modeling short-term depression and facilitation.
* **Transmission Delays:** Configurable discrete synaptic delays handled via ring buffers.
* **Homeostasis:** Synaptic scaling mechanism to maintain network activity within stable target regimes (~10–12 Hz).
* **Signal Encoding:** Time-To-First-Spike (TTFS) latency encoding for translating continuous inputs into precise temporal spike sequences.
* **Concurrency:** Parallel execution over contiguous slice splits (`split_at_mut`), avoiding mutex locks and thread contention during step updates.

## Performance & Verification

Tested on 100 neurons across 4 CPU threads:

* **Execution Time:** ~2.63ms per simulation pass.
* **Network Dynamics:** Reaches an Asynchronous Irregular (AI) firing state with average frequencies of ~12.6 Hz (Excitatory) and ~10.0 Hz (Inhibitory), conforming to Dale's Principle.

## Quickstart

Ensure you have a recent Rust toolchain installed.

Clone the repository and run the main entry point:

```bash
git clone [https://github.com/your-username/biomimetic-snn-rust.git](https://github.com/your-username/biomimetic-snn-rust.git)
cd biomimetic-snn-rust
cargo run --release