pub enum EncodingType {
    Rate,    // Uspostavlja frekvenciju (češći impulsi)
    Latency, // Uspostavlja vreme prvog okidanja (brži impulsi)
}

pub struct SpikeEncoder {
    pub encoding_type: EncodingType,
    pub time_window: f64, // Prozor trajanja simulacije u ms (npr. 50.0 ms)
}

impl SpikeEncoder {
    pub fn new(encoding_type: EncodingType, time_window: f64) -> Self {
        SpikeEncoder {
            encoding_type,
            time_window,
        }
    }

    /// Konvertuje niz brojeva [0.0 - 1.0] u struju (Current) ili direktan impuls za svaki vremenski korak
    pub fn encode_value(&self, value: f64, current_time: f64) -> f64 {
        let value = value.clamp(0.0, 1.0); // Normalizacija ulaza

        match self.encoding_type {
            EncodingType::Rate => {
                // Veća vrednost = češći impulsi
                // Simuliramo struju koja ulazi u neuron srazmerno vrednosti
                value * 8.0
            }
            EncodingType::Latency => {
                // Veća vrednost (npr. 1.0) = okida odmah na startu (t ~ 0 ms)
                // Manja vrednost (npr. 0.1) = okida kasno (t ~ 45 ms)
                let target_time = (1.0 - value) * self.time_window;

                // Šaljemo kratak jak strujni impuls ako je trenutno vreme blizu target vremena
                if (current_time - target_time).abs() < 1.0 {
                    12.0 // Jak strujni impuls
                } else {
                    0.0
                }
            }
        }
    }
}