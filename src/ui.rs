use nih_plug::context::gui::ParamSetter;
use nih_plug_egui::egui;
use nih_plug_egui::resizable_window::ResizableWindow;
use nih_plug_egui::EguiState;

use crate::egui::{Color32, Context, Vec2};
use crate::rotary_slider::{Knob, KnobStyle, LabelPosition};
use std::sync::Arc;

use crate::{EventCollector, GlideMode, PolyMode, Preset, Presets, RX11Params, UiState};

#[derive(Clone)]
pub enum UiView {
    Synth,
    Debug,
}

pub fn rx11_egui_ui(
    egui_state: &Arc<EguiState>,
    egui_ctx: &Context,
    setter: &ParamSetter,
    state: &mut UiState,
    params: &RX11Params,
    presets: &Presets,
    logger: &EventCollector,
) {
    let UiState {
        selected_preset,
        loaded_preset_on_startup,
        _current_view,
        show_debug,
    } = state;

    ResizableWindow::new("res-wind")
        .min_size(Vec2::new(800.0, 600.0))
        .show(egui_ctx, egui_state.as_ref(), |_ui| { 

            // Load something that makes sound because the synth is initialized
            // without taking presets into consideration. This will also be a good
            // place to load the state of whatever the user last used
            if !*loaded_preset_on_startup {
                if let Some(preset) = presets.0.first() {
                    load_preset(&preset, &setter, &params);
                    *loaded_preset_on_startup = true;
                    tracing::debug!("The preset was loaded on startup.");
                } else {
                    tracing::debug!("There was no preset to load on startup");
                }
            }

            egui::TopBottomPanel::top("menu").show(egui_ctx, |ui| {
                ui.horizontal(|ui| {
                    // Displaying / selecting the presets
                    ui.menu_button("Presets", |ui| {
                        egui::ScrollArea::vertical()
                            .scroll_bar_visibility(
                                egui::containers::scroll_area::ScrollBarVisibility::VisibleWhenNeeded,
                            )
                            .show(ui, |ui| {
                                for preset in &presets.0 {
                                    if ui.button(&preset.name).clicked() {
                                        *selected_preset = preset.name.clone();
                                        load_preset(&preset, &setter, &params);
                                        // How do I close the dropdown on button click?
                                    }
                                }
                            })
                    });
                    ui.label(format!("Preset: {}", selected_preset));

                    if *show_debug {
                        if ui.add(egui::Button::new("Debug Hide")).clicked() {
                            *show_debug = false;
                        }
                    } else {
                        if ui.add(egui::Button::new("Debug Show")).clicked() {
                            *show_debug = true;
                        }
                    }
                })
            }); // MENU END

            synth_view(egui_ctx, setter, params);

            if *show_debug {
                debug_view(egui_ctx, logger);
            }
        
    });
}

fn debug_view(egui_ctx: &Context, logger: &EventCollector) {
    egui::Window::new("Logs")
        .min_width(400.0)
        .show(egui_ctx, |ui| {
            ui.label("Logs go under here");
            if ui.add(egui::Button::new("Clear")).clicked() {
                logger.clear();
            }
            ui.separator();

            egui::ScrollArea::vertical()
                .scroll_bar_visibility(
                    egui::containers::scroll_area::ScrollBarVisibility::AlwaysVisible,
                )
                .show(ui, |ui| {
                    for evt in logger.events().iter() {
                        if let Some(field) = evt.fields.first_key_value() {
                            ui.label(field.1);
                        }
                    }
                });
        });
}

fn synth_view(egui_ctx: &Context, setter: &ParamSetter, params: &RX11Params) {
    egui::CentralPanel::default().show(egui_ctx, |ui| {
        // TODO:
        // I don't want to have to copy the value and then do all the setter stuff.
        // This should be made into a nih_plug style widget like the other sliders
        // Also, setting the min/max range on both the Param and here is not ideal.

        let mut vol_raw = (&params.output_level.value()).clone();
        let vol_min = nih_plug::util::db_to_gain(-30.0);
        let vol_max = nih_plug::util::db_to_gain(6.0);
        if ui
            .add(
                Knob::new(
                    &params.output_level,
                    &mut vol_raw,
                    vol_min,
                    vol_max,
                    KnobStyle::Wiper,
                )
                .with_label("Volume", LabelPosition::Bottom),
            )
            .dragged()
        {
            setter.begin_set_parameter(&params.output_level);
            setter.set_parameter(&params.output_level, vol_raw);
            setter.end_set_parameter(&params.output_level);
        }

        ui.separator();

        egui::ScrollArea::vertical()
            .scroll_bar_visibility(
                egui::containers::scroll_area::ScrollBarVisibility::AlwaysVisible,
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let glide_mode = &params.glide_mode.value();

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

                ui.horizontal(|ui| {
                    ui.label("MODE");

                    let poly_mode = &params.poly_mode.value();

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

                ui.horizontal(|ui| {
                    ui.label("OSCILLATOR");

                    let mut osc_mix_raw = (&params.osc_mix.value()).clone();
                    let mix_min = 0.0;
                    let mix_max = 100.0;
                    if ui
                        .add(
                            Knob::new(
                                &params.osc_mix,
                                &mut osc_mix_raw,
                                mix_min,
                                mix_max,
                                KnobStyle::Wiper,
                            )
                            .with_label("Mix", LabelPosition::Bottom)
                            .with_colors(
                                Color32::from_hex("#8BDEAC").unwrap(),
                                Color32::from_hex("#8BDEAC").unwrap(),
                                Color32::from_hex("#8BDEAC").unwrap(),
                            ),
                        )
                        .dragged()
                    {
                        setter.begin_set_parameter(&params.osc_mix);
                        setter.set_parameter(&params.osc_mix, osc_mix_raw);
                        setter.end_set_parameter(&params.osc_mix);
                    }

                    let mut octave = (&params.octave.value()).clone();
                    if ui
                        .add(
                            Knob::new(&params.octave, &mut octave, -2.0, 20.0, KnobStyle::Wiper)
                                .with_label("osc1 octave", LabelPosition::Bottom)
                                .with_colors(
                                    Color32::from_hex("#8BDEAC").unwrap(),
                                    Color32::from_hex("#8BDEAC").unwrap(),
                                    Color32::from_hex("#8BDEAC").unwrap(),
                                ),
                        )
                        .dragged()
                    {
                        setter.begin_set_parameter(&params.octave);
                        setter.set_parameter(&params.octave, octave);
                        setter.end_set_parameter(&params.octave);
                    }

                    let mut tuning = (&params.tuning.value()).clone();
                    if ui
                        .add(
                            Knob::new(&params.tuning, &mut tuning, -100.0, 100.0, KnobStyle::Wiper)
                                .with_label("osc1 tuning", LabelPosition::Bottom)
                                .with_colors(
                                    Color32::from_hex("#8BDEAC").unwrap(),
                                    Color32::from_hex("#8BDEAC").unwrap(),
                                    Color32::from_hex("#8BDEAC").unwrap(),
                                ),
                        )
                        .dragged()
                    {
                        setter.begin_set_parameter(&params.tuning);
                        setter.set_parameter(&params.tuning, tuning);
                        setter.end_set_parameter(&params.tuning);
                    }

                    let mut osc_tune_raw = (&params.osc_tune.value()).clone();
                    let osc_tune_min = -24.0;
                    let osc_tune_max = 24.0;
                    if ui
                        .add(
                            Knob::new(
                                &params.osc_tune,
                                &mut osc_tune_raw,
                                osc_tune_min,
                                osc_tune_max,
                                KnobStyle::Wiper,
                            )
                            .with_label("osc2 tune", LabelPosition::Bottom)
                            .with_colors(
                                Color32::from_hex("#8BDEAC").unwrap(),
                                Color32::from_hex("#8BDEAC").unwrap(),
                                Color32::from_hex("#8BDEAC").unwrap(),
                            ),
                        )
                        .dragged()
                    {
                        setter.begin_set_parameter(&params.osc_tune);
                        setter.set_parameter(&params.osc_tune, osc_tune_raw);
                        setter.end_set_parameter(&params.osc_tune);
                    }

                    let mut osc_fine_tune_raw = (&params.osc_fine_tune.value()).clone();
                    let osc_fine_tune_min = -50.0;
                    let osc_fine_tune_max = 50.0;
                    if ui
                        .add(
                            Knob::new(
                                &params.osc_fine_tune,
                                &mut osc_fine_tune_raw,
                                osc_fine_tune_min,
                                osc_fine_tune_max,
                                KnobStyle::Wiper,
                            )
                            .with_label("osc2 fine tune", LabelPosition::Bottom)
                            .with_colors(
                                Color32::from_hex("#8BDEAC").unwrap(),
                                Color32::from_hex("#8BDEAC").unwrap(),
                                Color32::from_hex("#8BDEAC").unwrap(),
                            ),
                        )
                        .dragged()
                    {
                        setter.begin_set_parameter(&params.osc_fine_tune);
                        setter.set_parameter(&params.osc_fine_tune, osc_fine_tune_raw);
                        setter.end_set_parameter(&params.osc_fine_tune);
                    }

                    let mut glide_rate_raw = (&params.glide_rate.value()).clone();
                    let glide_rate_min = 0.0;
                    let glide_rate_max = 100.0;
                    if ui
                        .add(
                            Knob::new(
                                &params.glide_rate,
                                &mut glide_rate_raw,
                                glide_rate_min,
                                glide_rate_max,
                                KnobStyle::Wiper,
                            )
                            .with_label("glide rate", LabelPosition::Bottom)
                            .with_colors(
                                Color32::from_hex("#8BDEAC").unwrap(),
                                Color32::from_hex("#8BDEAC").unwrap(),
                                Color32::from_hex("#8BDEAC").unwrap(),
                            ),
                        )
                        .dragged()
                    {
                        setter.begin_set_parameter(&params.glide_rate);
                        setter.set_parameter(&params.glide_rate, glide_rate_raw);
                        setter.end_set_parameter(&params.glide_rate);
                    }

                    let mut glide_bend_raw = (&params.glide_bend.value()).clone();
                    let glide_bend_min = -36.0;
                    let glide_bend_max = 36.0;
                    if ui
                        .add(
                            Knob::new(
                                &params.glide_bend,
                                &mut glide_bend_raw,
                                glide_bend_min,
                                glide_bend_max,
                                KnobStyle::Wiper,
                            )
                            .with_label("glide bend", LabelPosition::Bottom)
                            .with_colors(
                                Color32::from_hex("#8BDEAC").unwrap(),
                                Color32::from_hex("#8BDEAC").unwrap(),
                                Color32::from_hex("#8BDEAC").unwrap(),
                            ),
                        )
                        .dragged()
                    {
                        setter.begin_set_parameter(&params.glide_bend);
                        setter.set_parameter(&params.glide_bend, glide_bend_raw);
                        setter.end_set_parameter(&params.glide_bend);
                    }

                    let mut noise = (&params.noise_level.value()).clone();
                    if ui
                        .add(
                            Knob::new(
                                &params.noise_level,
                                &mut noise,
                                0.0,
                                100.0,
                                KnobStyle::Wiper,
                            )
                            .with_label("noise", LabelPosition::Bottom)
                            .with_colors(
                                Color32::from_hex("#8BDEAC").unwrap(),
                                Color32::from_hex("#8BDEAC").unwrap(),
                                Color32::from_hex("#8BDEAC").unwrap(),
                            ),
                        )
                        .dragged()
                    {
                        setter.begin_set_parameter(&params.noise_level);
                        setter.set_parameter(&params.noise_level, noise);
                        setter.end_set_parameter(&params.noise_level);
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("FILTER");

                    let mut filter_freq_raw = (&params.filter_freq.value()).clone();
                    let filter_freq_min = 0.0;
                    let filter_freq_max = 100.0;
                    if ui
                        .add(
                            Knob::new(
                                &params.filter_freq,
                                &mut filter_freq_raw,
                                filter_freq_min,
                                filter_freq_max,
                                KnobStyle::Wiper,
                            )
                            .with_label("Freq", LabelPosition::Bottom),
                        )
                        .dragged()
                    {
                        setter.begin_set_parameter(&params.filter_freq);
                        setter.set_parameter(&params.filter_freq, filter_freq_raw);
                        setter.end_set_parameter(&params.filter_freq);
                    }

                    let mut filter_reso_raw = (&params.filter_reso.value()).clone();
                    let filter_reso_min = 0.0;
                    let filter_reso_max = 100.0;
                    if ui
                        .add(
                            Knob::new(
                                &params.filter_reso,
                                &mut filter_reso_raw,
                                filter_reso_min,
                                filter_reso_max,
                                KnobStyle::Wiper,
                            )
                            .with_label("Reso", LabelPosition::Bottom),
                        )
                        .dragged()
                    {
                        setter.begin_set_parameter(&params.filter_reso);
                        setter.set_parameter(&params.filter_reso, filter_reso_raw);
                        setter.end_set_parameter(&params.filter_reso);
                    }

                    let mut filter_lfo_raw = (&params.filter_lfo.value()).clone();
                    let filter_lfo_min = 0.0;
                    let filter_lfo_max = 100.0;
                    if ui
                        .add(
                            Knob::new(
                                &params.filter_lfo,
                                &mut filter_lfo_raw,
                                filter_lfo_min,
                                filter_lfo_max,
                                KnobStyle::Wiper,
                            )
                            .with_label("LFO", LabelPosition::Bottom),
                        )
                        .dragged()
                    {
                        setter.begin_set_parameter(&params.filter_lfo);
                        setter.set_parameter(&params.filter_lfo, filter_lfo_raw);
                        setter.end_set_parameter(&params.filter_lfo);
                    }

                    let mut filter_vel_raw = (&params.filter_velocity.value()).clone();
                    let filter_vel_min = -100.0;
                    let filter_vel_max = 100.0;
                    if ui
                        .add(
                            Knob::new(
                                &params.filter_velocity,
                                &mut filter_vel_raw,
                                filter_vel_min,
                                filter_vel_max,
                                KnobStyle::Wiper,
                            )
                            .with_label("Velocity", LabelPosition::Bottom),
                        )
                        .dragged()
                    {
                        setter.begin_set_parameter(&params.filter_velocity);
                        setter.set_parameter(&params.filter_velocity, filter_vel_raw);
                        setter.end_set_parameter(&params.filter_velocity);
                    }

                    let mut filter_env_raw = (&params.filter_env.value()).clone();
                    let filter_env_min = 0.0;
                    let filter_env_max = 100.0;
                    if ui
                        .add(
                            Knob::new(
                                &params.filter_env,
                                &mut filter_env_raw,
                                filter_env_min,
                                filter_env_max,
                                KnobStyle::Wiper,
                            )
                            .with_label("Env", LabelPosition::Bottom),
                        )
                        .dragged()
                    {
                        setter.begin_set_parameter(&params.filter_env);
                        setter.set_parameter(&params.filter_env, filter_env_raw);
                        setter.end_set_parameter(&params.filter_env);
                    }

                    ui.end_row();

                    let mut filter_attack_raw = (&params.filter_attack.value()).clone();
                    if ui
                        .add(
                            Knob::new(
                                &params.filter_attack,
                                &mut filter_attack_raw,
                                0.0,
                                100.0,
                                KnobStyle::Wiper,
                            )
                            .with_label("Attack", LabelPosition::Bottom),
                        )
                        .dragged()
                    {
                        setter.begin_set_parameter(&params.filter_attack);
                        setter.set_parameter(&params.filter_attack, filter_attack_raw);
                        setter.end_set_parameter(&params.filter_attack);
                    }

                    let mut filter_decay = (&params.filter_decay.value()).clone();
                    if ui
                        .add(
                            Knob::new(
                                &params.filter_decay,
                                &mut filter_decay,
                                0.0,
                                100.0,
                                KnobStyle::Wiper,
                            )
                            .with_label("Decay", LabelPosition::Bottom),
                        )
                        .dragged()
                    {
                        setter.begin_set_parameter(&params.filter_decay);
                        setter.set_parameter(&params.filter_decay, filter_decay);
                        setter.end_set_parameter(&params.filter_decay);
                    }

                    let mut filter_sustain = (&params.filter_sustain.value()).clone();
                    if ui
                        .add(
                            Knob::new(
                                &params.filter_sustain,
                                &mut filter_sustain,
                                0.0,
                                100.0,
                                KnobStyle::Wiper,
                            )
                            .with_label("Sustain", LabelPosition::Bottom),
                        )
                        .dragged()
                    {
                        setter.begin_set_parameter(&params.filter_sustain);
                        setter.set_parameter(&params.filter_sustain, filter_sustain);
                        setter.end_set_parameter(&params.filter_sustain);
                    }

                    let mut filter_release = (&params.filter_release.value()).clone();
                    if ui
                        .add(
                            Knob::new(
                                &params.filter_release,
                                &mut filter_release,
                                0.0,
                                100.0,
                                KnobStyle::Wiper,
                            )
                            .with_label("Sustain", LabelPosition::Bottom),
                        )
                        .dragged()
                    {
                        setter.begin_set_parameter(&params.filter_release);
                        setter.set_parameter(&params.filter_release, filter_release);
                        setter.end_set_parameter(&params.filter_release);
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("ENVELOPE");

                    let mut env_attack = (&params.env_attack.value()).clone();
                    if ui
                        .add(
                            Knob::new(
                                &params.env_attack,
                                &mut env_attack,
                                0.0,
                                100.0,
                                KnobStyle::Wiper,
                            )
                            .with_label("attack", LabelPosition::Bottom),
                        )
                        .dragged()
                    {
                        setter.begin_set_parameter(&params.env_attack);
                        setter.set_parameter(&params.env_attack, env_attack);
                        setter.end_set_parameter(&params.env_attack);
                    }

                    let mut env_decay = (&params.env_decay.value()).clone();
                    if ui
                        .add(
                            Knob::new(
                                &params.env_decay,
                                &mut env_decay,
                                0.0,
                                100.0,
                                KnobStyle::Wiper,
                            )
                            .with_label("decay", LabelPosition::Bottom),
                        )
                        .dragged()
                    {
                        setter.begin_set_parameter(&params.env_decay);
                        setter.set_parameter(&params.env_decay, env_decay);
                        setter.end_set_parameter(&params.env_decay);
                    }

                    let mut env_sustain = (&params.env_sustain.value()).clone();
                    if ui
                        .add(
                            Knob::new(
                                &params.env_sustain,
                                &mut env_sustain,
                                0.0,
                                100.0,
                                KnobStyle::Wiper,
                            )
                            .with_label("sustain", LabelPosition::Bottom),
                        )
                        .dragged()
                    {
                        setter.begin_set_parameter(&params.env_sustain);
                        setter.set_parameter(&params.env_sustain, env_sustain);
                        setter.end_set_parameter(&params.env_sustain);
                    }

                    let mut env_release = (&params.env_release.value()).clone();
                    if ui
                        .add(
                            Knob::new(
                                &params.env_release,
                                &mut env_release,
                                0.0,
                                100.0,
                                KnobStyle::Wiper,
                            )
                            .with_label("release", LabelPosition::Bottom),
                        )
                        .dragged()
                    {
                        setter.begin_set_parameter(&params.env_release);
                        setter.set_parameter(&params.env_release, env_release);
                        setter.end_set_parameter(&params.env_release);
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("MODULATION");

                    let mut lfo_rate = (&params.lfo_rate.value()).clone();
                    if ui
                        .add(
                            Knob::new(&params.lfo_rate, &mut lfo_rate, 0.0, 1.0, KnobStyle::Wiper)
                                .with_label("lfo rate", LabelPosition::Bottom),
                        )
                        .dragged()
                    {
                        setter.begin_set_parameter(&params.lfo_rate);
                        setter.set_parameter(&params.lfo_rate, lfo_rate);
                        setter.end_set_parameter(&params.lfo_rate);
                    }

                    let mut vibrato = (&params.vibrato.value()).clone();
                    if ui
                        .add(
                            Knob::new(
                                &params.vibrato,
                                &mut vibrato,
                                -100.0,
                                100.0,
                                KnobStyle::Wiper,
                            )
                            .with_label("vibrato", LabelPosition::Bottom),
                        )
                        .dragged()
                    {
                        setter.begin_set_parameter(&params.vibrato);
                        setter.set_parameter(&params.vibrato, vibrato);
                        setter.end_set_parameter(&params.vibrato);
                    }
                });
            })
    }); // END CENTRAL PANEL
}

fn load_preset(preset: &Preset, setter: &ParamSetter, params: &RX11Params) {
    for (param_name, param_value) in &preset.values {
        if &param_name[..] == "glide_mode" {
            setter.begin_set_parameter(&params.glide_mode);
            setter.set_parameter(&params.glide_mode, GlideMode::from_f32(*param_value));
            setter.end_set_parameter(&params.glide_mode);
        } else if &param_name[..] == "poly_mode" {
            setter.begin_set_parameter(&params.poly_mode);
            setter.set_parameter(&params.poly_mode, PolyMode::from_f32(*param_value));
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
