use nih_plug::prelude::*;
use std::sync::Arc;
// use parking_lot::Mutex;

struct RX11 {
    params: Arc<RX11Params>,
}

#[derive(Params)]
struct RX11Params {
    #[id = "output_level"]
    pub output_level: FloatParam,
}

impl Default for RX11 {
    fn default() -> Self {
        Self {
            params: Arc::new(RX11Params::default())
       }
    }
}

impl Default for RX11Params {
    fn default() -> Self {
        Self {
            output_level: FloatParam::new(
                "Output Level",
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

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn process (
        &mut self,
        _buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>
    ) -> ProcessStatus {
        // TODO: Process Audio here...
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
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Mono,
        ClapFeature::Utility,
    ];
}

impl Vst3Plugin for RX11 {
    const VST3_CLASS_ID: [u8; 16] = *b"RX11PlugWhatFuck";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Tools];
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
