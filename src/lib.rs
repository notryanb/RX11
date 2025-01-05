use nih_plug::prelude::*;
use std::sync::Arc;
// use parking_lot::Mutex;

const MAX_BLOCK_SIZE: usize = 64;

struct NoiseGenerator {
    noise_seed: u32,
}

impl NoiseGenerator {
    fn new() -> Self {
        Self { 
            noise_seed: 22222,
        }
    }
    fn reset(&mut self) {
        self.noise_seed = 22222;
    }

    fn next_value(&mut self) -> f32 {
        self.noise_seed = self.noise_seed * 196314165 + 907633515;
        let temp = ((self.noise_seed >> 7) as i32) - 16777216;
        temp as f32 / 16777216.0f32
    }
}

struct Synth {
    pub noise_mix: f32,
    noise_gen: NoiseGenerator,
}

impl Synth {
    fn new() -> Self {
        Self {
            noise_mix: 0.09,
            noise_gen: NoiseGenerator::new(),
        }
    }
    fn reset(&mut self) {
        self.noise_gen.reset();
    }

    fn render(&mut self, output_buffer: &mut Buffer) {
        // for each sample, 
        for channel_samples in output_buffer.iter_samples() {
            for sample in channel_samples {
                *sample = self.noise_gen.next_value() * self.noise_mix;
            }
        }
    }
}

struct RX11 {
    params: Arc<RX11Params>,
    synth: Synth,
}

#[derive(Params)]
struct RX11Params {
    #[id = "output"]
    pub output_level: FloatParam,

    #[id = "noise"]
    pub noise_level: FloatParam,
}

impl Default for RX11 {
    fn default() -> Self {
        Self {
            params: Arc::new(RX11Params::default()),
            synth: Synth::new(),
       }
    }
}

impl Default for RX11Params {
    fn default() -> Self {
        Self {
            output_level: FloatParam::new(
                    "Output",
                    util::db_to_gain(0.0),
                    FloatRange::Skewed {
                        min: util::db_to_gain(-30.0),
                        max: util::db_to_gain(6.0),
                        factor: FloatRange::gain_skew_factor(-30.0, 6.0),
                    }
                )
                .with_smoother(SmoothingStyle::Logarithmic(50.0))
                .with_unit("dB")
                .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
                .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            noise_level: FloatParam::new(
                "Noise",
                0.3,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_value_to_string(formatters::v2s_f32_rounded(2)),
        }
    }
}

impl Plugin for RX11 {
    const NAME: &'static str = "RX11 Synth";
    const VENDOR: &'static str = "RyanSoft";
    const URL: &'static str = "https://notryanb.dev";
    const EMAIL: &'static str = "notryanb@gmail.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),

            aux_input_ports: &[],
            aux_output_ports: &[],

            names: PortNames::const_default(),
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];

    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn process (
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>
    ) -> ProcessStatus {
        let num_samples = buffer.samples();
        let sample_rate = context.transport().sample_rate;
        let output = buffer.as_slice();
        
        let mut block_start: usize = 0;
        let mut block_end: usize = MAX_BLOCK_SIZE.min(num_samples);
        let mut next_event = context.next_event(); // Gets the next NoteEvent

        while block_start < num_samples {
            'events: loop {
                /*
                    TODO - The JUCE version of this does the following.
                    - Creates a buffer_offset to keep track of where we are in the block of audio. 
                        We want to split each audio block into smaller chunks
                    - For each event, check the following
                        - Get samples in current segment, which is sample_position(event.timing()?) - buffer_offset
                        - If there are samples
                            - render into the output buffer
                            - increase the buffer_offset by the samples in the current segment
                        - Handle the MIDI event
                            - For now, just test by printing info about the MIDI event
                            - NoteEvent enum docs: https://nih-plug.robbertvanderhelm.nl/nih_plug/midi/enum.NoteEvent.html
                            - timing, note, velocity, etc...
                    - Render the audio after the last MIDI event
                        This is known as "samples_last_segment" and is number of samples in the buffer - buffer_offset
                    - If there are samples_last_segment
                        - render into the output buffer

                    - Clear the MIDI messages if possible? I'm guessing this is getting the next event?
                */
                match next_event {
                    // event.timing() gets the note's timing within the buffer
                    Some(event) if (event.timing() as usize) <= block_start => {
                        match event {
                            NoteEvent::NoteOn {
                                timing,
                                voice_id,
                                channel,
                                note,
                                velocity,
                            } => {
                                self.synth.render(buffer);   
                            }
                            _ => { },
                        }

                        next_event = context.next_event();
                    }
                    Some(event) if (event.timing() as usize) < block_end => {
                        block_end = event.timing() as usize;
                        break 'events;
                    }
                    _ => break 'events,
                }
            }

            block_start = block_end;
            block_end = (block_start + MAX_BLOCK_SIZE).min(num_samples);
        }

       /*
        for channel_samples in buffer.iter_samples() {
            let output_level = self.params.output_level.smoothed.next();
            
            for sample in channel_samples {
                *sample *= output_level;
            }
        }
        */
        
        ProcessStatus::Normal
    }

    fn deactivate(&mut self) {}
}

impl ClapPlugin for RX11 {
    const CLAP_ID: &'static str = "com.ryansoft.rx11";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("The RX11 Synth");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::Synthesizer,
        ClapFeature::Instrument,
        ClapFeature::Stereo,
        ClapFeature::Mono,
        ClapFeature::Utility,
    ];
}

impl Vst3Plugin for RX11 {
    const VST3_CLASS_ID: [u8; 16] = *b"RX11PlugWhatGrok";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Synth, Vst3SubCategory::Instrument];
}

nih_export_clap!(RX11);
nih_export_vst3!(RX11);

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
