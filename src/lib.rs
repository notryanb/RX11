use nih_plug::midi::control_change::{
    ALL_NOTES_OFF, ALL_SOUND_OFF, MAIN_VOLUME_MSB, MODULATION_MSB,
};
use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui, widgets, EguiState};

use std::sync::Arc;

mod envelope;
mod noise_generator;
mod oscillator;
mod presets;
mod state_variable_filter;
mod synth;
mod voice;

use crate::presets::Presets;
use crate::synth::Synth;

const MAX_BLOCK_SIZE: usize = 64;

#[derive(Clone, Enum, PartialEq)]
pub enum PolyMode {
    #[id = "mono"]
    Mono,

    #[id = "poly"]
    Poly,
}

impl PolyMode {
    pub fn to_f32(pm: PolyMode) -> f32 {
        match pm {
            PolyMode::Mono => 0.0,
            PolyMode::Poly => 1.0,
        }
    }

    pub fn from_f32(i: f32) -> Self {
        match i {
            1.0 => PolyMode::Poly,
            _ => PolyMode::Mono,
        }
    }
}

#[derive(Clone, Enum, PartialEq)]
pub enum GlideMode {
    #[id = "off"]
    Off,

    #[id = "legato"]
    Legato,

    #[id = "always"]
    Always,
}

impl GlideMode {
    pub fn to_f32(gm: GlideMode) -> f32 {
        match gm {
            GlideMode::Off => 0.0,
            GlideMode::Legato => 1.0,
            GlideMode::Always => 2.0,
        }
    }

    pub fn from_f32(i: f32) -> Self {
        match i {
            2.0 => GlideMode::Always,
            1.0 => GlideMode::Legato,
            _ => GlideMode::Off,
        }
    }
}

pub struct RX11 {
    params: Arc<RX11Params>,
    synth: Synth,
    presets: Presets,
}

#[derive(Params)]
pub struct RX11Params {
    /// The editor state, saved together with the parameter state so the custom scaling can be
    /// restored.
    #[persist = "editor-state"]
    editor_state: Arc<EguiState>,

    #[id = "osc_mix"]
    pub osc_mix: FloatParam,

    #[id = "osc_tune"]
    pub osc_tune: FloatParam,

    #[id = "osc_fine_tune"]
    pub osc_fine_tune: FloatParam,

    #[id = "glide_mode"]
    pub glide_mode: EnumParam<GlideMode>,

    #[id = "poly_mode"]
    pub poly_mode: EnumParam<PolyMode>,

    #[id = "glide_rate"]
    pub glide_rate: FloatParam,

    #[id = "glide_bend"]
    pub glide_bend: FloatParam,

    #[id = "filter_freq"]
    pub filter_freq: FloatParam,

    #[id = "filter_reso"]
    pub filter_reso: FloatParam,

    #[id = "filter_env"]
    pub filter_env: FloatParam,

    #[id = "filter_lfo"]
    pub filter_lfo: FloatParam,

    #[id = "filter_velocity"]
    pub filter_velocity: FloatParam,

    #[id = "filter_attack"]
    pub filter_attack: FloatParam,

    #[id = "filter_decay"]
    pub filter_decay: FloatParam,

    #[id = "filter_sustain"]
    pub filter_sustain: FloatParam,

    #[id = "filter_release"]
    pub filter_release: FloatParam,

    #[id = "env_attack"]
    pub env_attack: FloatParam,

    #[id = "env_decay"]
    pub env_decay: FloatParam,

    #[id = "env_sustain"]
    pub env_sustain: FloatParam,

    #[id = "env_release"]
    pub env_release: FloatParam,

    #[id = "lfo_rate"]
    pub lfo_rate: FloatParam,

    #[id = "vibrato"]
    pub vibrato: FloatParam,

    #[id = "noise"]
    pub noise_level: FloatParam,

    #[id = "octave"]
    pub octave: FloatParam,

    #[id = "tuning"]
    pub tuning: FloatParam,

    #[id = "output"]
    pub output_level: FloatParam,
}

impl Default for RX11 {
    fn default() -> Self {
        Self {
            params: Arc::new(RX11Params::default()),
            synth: Synth::new(),
            presets: Presets::init(),
        }
    }
}

impl Default for RX11Params {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(800, 600),

            osc_mix: FloatParam::new(
                "Osc Mix",
                0.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 100.0,
                },
            )
            .with_value_to_string(Arc::new(|value| {
                format!("{:.0}:{:.0}", 100.0 - 0.5 * value, 0.5 * value)
            })),

            osc_tune: FloatParam::new(
                "Osc Tune",
                -12.0,
                FloatRange::Linear {
                    min: -24.0,
                    max: 24.0,
                },
            )
            .with_unit("semi")
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            osc_fine_tune: FloatParam::new(
                "Osc Fine Tune",
                0.0,
                FloatRange::SymmetricalSkewed {
                    min: -50.0,
                    max: 50.0,
                    factor: 0.3, //FloatRange::skew_factor(-50.0, 50.0),
                    center: 0.0,
                },
            )
            .with_step_size(0.1)
            .with_unit("cent")
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            poly_mode: EnumParam::new("Poly Mode", PolyMode::Mono),

            glide_mode: EnumParam::new("Glide Mode", GlideMode::Off),

            glide_rate: FloatParam::new(
                "Glide Rate",
                35.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 100.0,
                },
            )
            .with_step_size(1.0)
            .with_unit("%")
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            glide_bend: FloatParam::new(
                "Glide Bend",
                0.0,
                FloatRange::SymmetricalSkewed {
                    min: -36.0,
                    max: 36.0,
                    factor: 0.01,
                    center: 0.0,
                },
            )
            .with_step_size(0.4)
            .with_unit("semi")
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            filter_freq: FloatParam::new(
                "Filter Freq",
                100.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 100.0,
                },
            )
            .with_unit("%")
            .with_step_size(0.1)
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            filter_reso: FloatParam::new(
                "Filter Reso",
                15.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 100.0,
                },
            )
            .with_unit("%")
            .with_step_size(1.0)
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            filter_env: FloatParam::new(
                "Filter Env",
                50.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 100.0,
                },
            )
            .with_unit("%")
            .with_step_size(0.1)
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            filter_lfo: FloatParam::new(
                "Filter LFO",
                0.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 100.0,
                },
            )
            .with_unit("%")
            .with_step_size(1.0)
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            filter_velocity: FloatParam::new(
                "Filter Velocity",
                0.0,
                FloatRange::Linear {
                    min: -100.0,
                    max: 100.0,
                },
            )
            .with_step_size(1.0)
            .with_value_to_string(Arc::new(|value| {
                if value < -90.0 {
                    String::from("Off")
                } else {
                    format!("{value:.2}")
                }
            })),

            filter_attack: FloatParam::new(
                "Filter Attack",
                0.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 100.0,
                },
            )
            .with_unit("%")
            .with_step_size(1.0)
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            filter_decay: FloatParam::new(
                "Filter Decay",
                30.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 100.0,
                },
            )
            .with_unit("%")
            .with_step_size(1.0)
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            filter_sustain: FloatParam::new(
                "Filter Sustain",
                0.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 100.0,
                },
            )
            .with_unit("%")
            .with_step_size(1.0)
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            filter_release: FloatParam::new(
                "Filter Release",
                25.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 100.0,
                },
            )
            .with_unit("%")
            .with_step_size(1.0)
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            env_attack: FloatParam::new(
                "Env Attack",
                0.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 100.0,
                },
            )
            .with_unit("%")
            .with_step_size(1.0)
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            env_decay: FloatParam::new(
                "Env Decay",
                50.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 100.0,
                },
            )
            .with_unit("%")
            .with_step_size(1.0)
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            env_sustain: FloatParam::new(
                "Env Sustain",
                100.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 100.0,
                },
            )
            .with_unit("%")
            .with_step_size(1.0)
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            env_release: FloatParam::new(
                "Env Release",
                30.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 100.0,
                },
            )
            .with_unit("%")
            .with_step_size(1.0)
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            lfo_rate: FloatParam::new("LFO Rate", 0.81, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_unit("Hz")
                .with_step_size(0.01)
                .with_value_to_string(Arc::new(|value| {
                    format!("{:.2}", (7.0 * value - 4.0).exp())
                })),

            vibrato: FloatParam::new(
                "Vibrato",
                0.0,
                FloatRange::Linear {
                    min: -100.0,
                    max: 100.0,
                },
            )
            .with_unit("Hz")
            .with_step_size(0.1)
            .with_value_to_string(Arc::new(|value| {
                if value < 0.0 {
                    format!("PWM {:.2}", -value)
                } else {
                    format!("{value:.2}")
                }
            })),

            octave: FloatParam::new(
                "Octave",
                0.0,
                FloatRange::Linear {
                    min: -2.0,
                    max: 2.0,
                },
            )
            .with_unit("''")
            .with_step_size(1.0)
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            tuning: FloatParam::new(
                "Tuning",
                0.0,
                FloatRange::Linear {
                    min: -100.0,
                    max: 100.0,
                },
            )
            .with_unit("cent")
            .with_step_size(0.1)
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            output_level: FloatParam::new(
                "Output",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(6.0),
                    factor: FloatRange::gain_skew_factor(-30.0, 6.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit("dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            noise_level: FloatParam::new(
                "Noise",
                0.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 100.0,
                },
            )
            .with_step_size(1.0)
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

    fn reset(&mut self) {
        self.synth.reset(&self.params);
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let params = self.params.clone();
        let presets = self.presets.clone();

        create_egui_editor(
            self.params.editor_state.clone(),
            (),
            |_, _| {},
            move |egui_ctx, setter, _state| {
                egui::TopBottomPanel::top("menu").show(egui_ctx, |ui| {
                    ui.horizontal(|ui| {
                        // Displaying / selecting the presets
                        ui.menu_button("Presets", |ui| {
                            for preset in &presets.0 {
                                if ui.button(&preset.name).clicked() {
                                    for (param_name, param_value) in &preset.values {
                                        if &param_name[..] == "glide_mode" {
                                            setter.begin_set_parameter(&params.glide_mode);
                                            setter.set_parameter(
                                                &params.glide_mode,
                                                GlideMode::from_f32(*param_value),
                                            );
                                            setter.end_set_parameter(&params.glide_mode);
                                        } else if &param_name[..] == "poly_mode" {
                                            setter.begin_set_parameter(&params.poly_mode);
                                            setter.set_parameter(
                                                &params.poly_mode,
                                                PolyMode::from_f32(*param_value),
                                            );
                                            setter.end_set_parameter(&params.poly_mode);
                                        } else {
                                            let param = match &param_name[..] {
                                                "osc_mix" => Some(&params.osc_mix),
                                                "osc_tune" => Some(&params.osc_tune),
                                                "osc_fine_tune" => Some(&params.osc_fine_tune),
                                                "glide_rate" => Some(&params.glide_rate),
                                                "glide_bend" => Some(&params.glide_bend),
                                                "filter_freq" => Some(&params.filter_freq),
                                                "filter_reso" => Some(&params.filter_reso),
                                                "filter_env" => Some(&params.filter_env),
                                                "filter_lfo" => Some(&params.filter_lfo),
                                                "filter_velocity" => Some(&params.filter_velocity),
                                                "filter_attack" => Some(&params.filter_attack),
                                                "filter_decay" => Some(&params.filter_decay),
                                                "filter_sustain" => Some(&params.filter_sustain),
                                                "filter_release" => Some(&params.filter_release),
                                                "env_attack" => Some(&params.env_attack),
                                                "env_decay" => Some(&params.env_decay),
                                                "env_sustain" => Some(&params.env_sustain),
                                                "env_release" => Some(&params.env_release),
                                                "lfo_rate" => Some(&params.lfo_rate),
                                                "vibrato" => Some(&params.vibrato),
                                                "noise" => Some(&params.noise_level),
                                                "octave" => Some(&params.octave),
                                                "tuning" => Some(&params.tuning),
                                                "output" => Some(&params.output_level),
                                                _ => None,
                                            };

                                            if let Some(param) = param {
                                                setter.begin_set_parameter(param);
                                                setter.set_parameter(param, *param_value);
                                                setter.end_set_parameter(param);
                                            }
                                        }
                                    }
                                }
                            }
                        });
                        ui.label("RX11: I think this is where I'll put menu stuff.");
                    })
                });
                egui::CentralPanel::default().show(egui_ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .scroll_bar_visibility(
                            egui::containers::scroll_area::ScrollBarVisibility::AlwaysVisible,
                        )
                        .show(ui, |ui| {
                            let glide_mode = &params.glide_mode.value();
                            ui.horizontal(|ui| {
                                if ui
                                    .add(egui::widgets::SelectableLabel::new(
                                        *glide_mode == GlideMode::Off,
                                        "Glide Off",
                                    ))
                                    .clicked()
                                {
                                    setter.begin_set_parameter(&params.glide_mode);
                                    setter.set_parameter(&params.glide_mode, GlideMode::Off);
                                    setter.end_set_parameter(&params.glide_mode);
                                }
                                if ui
                                    .add(egui::widgets::SelectableLabel::new(
                                        *glide_mode == GlideMode::Legato,
                                        "Legato",
                                    ))
                                    .clicked()
                                {
                                    setter.begin_set_parameter(&params.glide_mode);
                                    setter.set_parameter(&params.glide_mode, GlideMode::Legato);
                                    setter.end_set_parameter(&params.glide_mode);
                                }
                                if ui
                                    .add(egui::widgets::SelectableLabel::new(
                                        *glide_mode == GlideMode::Always,
                                        "Glide Always",
                                    ))
                                    .clicked()
                                {
                                    setter.begin_set_parameter(&params.glide_mode);
                                    setter.set_parameter(&params.glide_mode, GlideMode::Always);
                                    setter.end_set_parameter(&params.glide_mode);
                                }
                            });
                            ui.end_row();

                            ui.separator();

                            let poly_mode = &params.poly_mode.value();
                            ui.horizontal(|ui| {
                                if ui
                                    .add(egui::widgets::SelectableLabel::new(
                                        *poly_mode == PolyMode::Mono,
                                        "Mono",
                                    ))
                                    .clicked()
                                {
                                    setter.begin_set_parameter(&params.poly_mode);
                                    setter.set_parameter(&params.poly_mode, PolyMode::Mono);
                                    setter.end_set_parameter(&params.poly_mode);
                                }
                                if ui
                                    .add(egui::widgets::SelectableLabel::new(
                                        *poly_mode == PolyMode::Poly,
                                        "Poly",
                                    ))
                                    .clicked()
                                {
                                    setter.begin_set_parameter(&params.poly_mode);
                                    setter.set_parameter(&params.poly_mode, PolyMode::Poly);
                                    setter.end_set_parameter(&params.poly_mode);
                                }
                            });
                            ui.end_row();

                            ui.separator();

                            ui.label("Oscillator Mix");
                            ui.add(widgets::ParamSlider::for_param(&params.osc_mix, setter));

                            ui.label("Oscillator Tune");
                            ui.add(widgets::ParamSlider::for_param(&params.osc_tune, setter));

                            ui.label("Oscillator Fine Tune");
                            ui.add(widgets::ParamSlider::for_param(
                                &params.osc_fine_tune,
                                setter,
                            ));

                            ui.label("Glide Rate");
                            ui.add(widgets::ParamSlider::for_param(&params.glide_rate, setter));

                            ui.label("Glide Bend");
                            ui.add(widgets::ParamSlider::for_param(&params.glide_bend, setter));

                            ui.label("Filter Frequency");
                            ui.add(widgets::ParamSlider::for_param(&params.filter_freq, setter));

                            ui.label("Filter Resonance");
                            ui.add(widgets::ParamSlider::for_param(&params.filter_reso, setter));

                            ui.label("Filter LFO");
                            ui.add(widgets::ParamSlider::for_param(&params.filter_lfo, setter));

                            ui.label("Filter Velocity");
                            ui.add(widgets::ParamSlider::for_param(
                                &params.filter_velocity,
                                setter,
                            ));

                            ui.label("Filter ADSR");
                            ui.add(widgets::ParamSlider::for_param(&params.filter_env, setter));

                            ui.label("Filter Attack");
                            ui.add(widgets::ParamSlider::for_param(
                                &params.filter_attack,
                                setter,
                            ));

                            ui.label("Filter Decay");
                            ui.add(widgets::ParamSlider::for_param(
                                &params.filter_decay,
                                setter,
                            ));

                            ui.label("Filter Sustain");
                            ui.add(widgets::ParamSlider::for_param(
                                &params.filter_sustain,
                                setter,
                            ));

                            ui.label("Filter Release");
                            ui.add(widgets::ParamSlider::for_param(
                                &params.filter_release,
                                setter,
                            ));

                            ui.label("Envelope Attack");
                            ui.add(widgets::ParamSlider::for_param(&params.env_attack, setter));

                            ui.label("Envelope Decay");
                            ui.add(widgets::ParamSlider::for_param(&params.env_decay, setter));

                            ui.label("Envelope Sustain");
                            ui.add(widgets::ParamSlider::for_param(&params.env_sustain, setter));

                            ui.label("Envelope Release");
                            ui.add(widgets::ParamSlider::for_param(&params.env_release, setter));

                            ui.label("LFO Rate");
                            ui.add(widgets::ParamSlider::for_param(&params.lfo_rate, setter));

                            ui.label("Vibrato");
                            ui.add(widgets::ParamSlider::for_param(&params.vibrato, setter));

                            ui.label("Noise");
                            ui.add(widgets::ParamSlider::for_param(&params.noise_level, setter));

                            ui.label("Octave");
                            ui.add(widgets::ParamSlider::for_param(&params.octave, setter));

                            ui.label("Tuning");
                            ui.add(widgets::ParamSlider::for_param(&params.tuning, setter));

                            ui.label("Volume");
                            ui.add(widgets::ParamSlider::for_param(
                                &params.output_level,
                                setter,
                            ));
                        })
                });
            },
        )
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let sample_rate = context.transport().sample_rate;

        // TODO - discover if there is a way to get the sample_rate in RX11::default
        // voices also rely on default impl, so I'd need to pass it down.
        self.synth.sample_rate = sample_rate;
        for voice in &mut self.synth.voices {
            voice.filter.sample_rate = sample_rate;
        }

        let inverse_sample_rate = 1.0 / sample_rate;
        let inverse_update_rate = inverse_sample_rate * crate::synth::LFO_MAX;

        let num_samples = buffer.samples();
        let output = buffer.as_slice();

        let mut block_start: usize = 0;
        let mut block_end: usize = MAX_BLOCK_SIZE.min(num_samples);
        let mut next_event = context.next_event(); // Gets the next NoteEvent

        while block_start < num_samples {
            'events: loop {
                match next_event {
                    // event.timing() gets the note's timing within the buffer
                    Some(event) if (event.timing() as usize) <= block_start => {
                        match event {
                            NoteEvent::NoteOn {
                                timing: _,
                                voice_id: _,
                                channel: _,
                                note, // values are 0..128, maybe I can store as i8 instead of i32?
                                velocity, // values are normalized 0..1, multiply by 127 to get back to original range
                            } => {
                                self.synth.note_on(note.into(), velocity * 127.0);
                            }
                            NoteEvent::NoteOff {
                                timing: _,
                                voice_id: _,
                                channel: _,
                                note,
                                velocity: _,
                            } => {
                                self.synth.note_off(note.into());
                            }
                            NoteEvent::MidiPitchBend {
                                timing: _,
                                channel: _,
                                value, // Normalized 0..1. 0.5 is no pitch bend
                            } => {
                                // Express pitch bend in semitones
                                // TODO - Need to turn 0..1 into semitones 0.89..1.12 according to the book
                                // A value of pitchbend = 1 means the multiplier won't change
                                // the pitch.
                                self.synth.pitch_bend = value + 0.5;
                            }
                            NoteEvent::MidiChannelPressure {
                                timing: _,
                                channel: _,
                                pressure,
                            } => {
                                self.synth.pressure = 0.0001 * pressure * pressure;
                            }
                            NoteEvent::MidiCC {
                                timing: _,
                                channel: _,
                                cc,
                                value, // 0..1. Normally 0..127 for typical midi, but can be mapped back by multiplying by 127.
                                       // The pedals will usually be off for the first half of the range and on for the second half.
                            } => {
                                if cc == MODULATION_MSB {
                                    self.synth.mod_wheel = 0.000005 * value;
                                }

                                if cc == 0x4A {
                                    // Filter inc
                                    self.synth.filter_ctrl = 0.02 * value;
                                }

                                if cc == 0x4B {
                                    // Filter dec
                                    self.synth.filter_ctrl = -0.03 * value;
                                }

                                if cc == MAIN_VOLUME_MSB {
                                    // TODO - Figure out how to notify the UI...
                                    // ie. write to the params Arc.
                                    let _volume_ctrl = value;
                                }

                                // TODO - unsure of the const for footpedal in nih_plug
                                if cc == 0x40 {
                                    self.synth.is_sustained = value >= 0.5;

                                    if !self.synth.is_sustained {
                                        // release the sustained voices
                                        self.synth.note_off(crate::synth::SUSTAIN);
                                    }
                                }

                                // All Notes Off (aka. PANIC!!!) Message
                                if cc == ALL_NOTES_OFF || cc == ALL_SOUND_OFF {
                                    for voice in &mut self.synth.voices {
                                        voice.reset();
                                    }

                                    self.synth.is_sustained = false;
                                }
                            }
                            _ => {}
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

            // Parameter stuff...
            // TODO: Eventually all other params will be here and can potentially be expensive
            // to calculate their values on every process block iteration.
            // JUCE has a way to check if the parameter raw value changed and only perform calculations
            // when necessary.
            // Essentially an atomic boolean is used in the JUCE examples which indicates if a parameter changed.

            // ADSR Envelope
            self.synth.env_attack =
                (-inverse_sample_rate * (5.5 - 0.075 * self.params.env_attack.value()).exp()).exp();

            self.synth.env_decay =
                (-inverse_sample_rate * (5.5 - 0.075 * self.params.env_decay.value()).exp()).exp();

            self.synth.env_sustain = self.params.env_sustain.value() / 100.0;

            let env_release = self.params.env_release.value();
            if env_release < 1.0 {
                self.synth.env_release = 0.75; // Extra fast release
            } else {
                self.synth.env_release =
                    (-inverse_sample_rate * (5.5 - 0.075 * env_release).exp()).exp();
            }

            // Voices
            match self.params.poly_mode.value() {
                PolyMode::Mono => self.synth.num_voices = 1,
                PolyMode::Poly => self.synth.num_voices = crate::synth::MAX_VOICES,
            }

            // Oscillator Tuning
            let octave = self.params.octave.value();
            let tuning = self.params.tuning.value();
            let tune_in_semi = -36.3763 - 12.0 * octave - tuning / 100.0;
            self.synth.tune = sample_rate * (0.05776226505 * tune_in_semi).exp();

            let semi = self.params.osc_tune.value();
            let cent = self.params.osc_fine_tune.value();
            self.synth.detune = 1.059463094359_f32.powf(-semi - 0.01 * cent); // Total detuning in semitones
            self.synth.osc_mix = self.params.osc_mix.value() / 100.0;

            // Filter
            let filter_velocity = self.params.filter_velocity.value();
            if filter_velocity < -90.0 {
                self.synth.velocity_sensitivity = 0.0;
                self.synth.ignore_velocity = true;
            } else {
                self.synth.velocity_sensitivity = 0.0005 * filter_velocity;
                self.synth.ignore_velocity = false;
            }

            let filter_lfo = self.params.filter_lfo.value() / 100.0;
            self.synth.filter_lfo_depth = 2.5 * filter_lfo * filter_lfo;

            // Convert range from -1.5..6.5
            self.synth.filter_key_tracking = 0.08 * self.params.filter_freq.value() - 1.5;

            let filter_resonance = self.params.filter_reso.value() / 100.0;
            self.synth.filter_resonance = (3.0 * filter_resonance).exp();

            self.synth.filter_attack = (-inverse_update_rate
                * (5.5 - 0.075 * self.params.filter_attack.value()).exp())
            .exp();
            self.synth.filter_decay = (-inverse_update_rate
                * (5.5 - 0.075 * self.params.filter_decay.value()).exp())
            .exp();
            self.synth.filter_release = (-inverse_update_rate
                * (5.5 - 0.075 * self.params.filter_release.value()).exp())
            .exp();
            let filter_sustain = self.params.filter_sustain.value() / 100.0;
            self.synth.filter_sustain = filter_sustain * filter_sustain;
            self.synth.filter_env_depth = 0.06 * self.params.filter_env.value();

            // LFO & Vibrato: Phase increment = 2PI * freq / sample rate
            let lfo_rate = (7.0 * self.params.lfo_rate.value() - 4.0).exp();
            self.synth.lfo_phase_increment = lfo_rate * inverse_update_rate * std::f32::consts::TAU;

            let vibrato = self.params.vibrato.value() / 200.0;
            self.synth.vibrato = 0.2 * vibrato * vibrato;

            self.synth.pwm_depth = self.synth.vibrato;
            if self.synth.vibrato < 0.0 {
                self.synth.vibrato = 0.0;
            }

            self.synth.glide_mode = self.params.glide_mode.value();
            let glide_rate = self.params.glide_rate.value();
            if glide_rate < 2.0 {
                self.synth.glide_rate = 1.0; // No glide
            } else {
                self.synth.glide_rate =
                    1.0 - (-inverse_update_rate * (6.0 - 0.07 * glide_rate).exp()).exp();
            }

            // Noise
            let mut noise_mix = self.params.noise_level.value() / 100.0;
            noise_mix *= noise_mix;
            self.synth.noise_mix = noise_mix * 0.06;

            // Volume
            self.synth.volume_trim = 0.0008
                * (3.2 - self.synth.osc_mix - 25.0 * self.synth.noise_mix)
                * (1.5 - 0.5 * filter_resonance);

            //self.synth.volume_trim = 1.0;
            //self.synth.output_level = self.params.output_level.smoothed.next();

            // Do I really need to clear the buffer before rendering into it?
            output[0][block_start..block_end].fill(0.0);
            output[1][block_start..block_end].fill(0.0);

            self.synth
                .render(output, block_start, block_end, &self.params);

            block_start = block_end;
            block_end = (block_start + MAX_BLOCK_SIZE).min(num_samples);
        }

        ProcessStatus::Normal
    }

    fn deactivate(&mut self) {
        nih_log!("Deactivation");
    }
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
