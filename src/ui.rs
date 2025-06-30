use nih_plug::context::gui::ParamSetter;
use nih_plug_egui::{egui, widgets};
use nih_plug_egui::{resizable_window::ResizableWindow};
use nih_plug_egui::EguiState;
use crate::egui::{Context, Slider, Vec2, vec2};
use crate::rotary_slider::RotarySlider;
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
        show_debug
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

fn debug_view(
    egui_ctx: &Context,
    logger: &EventCollector,
) {
    egui::Window::new("Logs").min_width(400.0).show(egui_ctx, |ui| {
        ui.label("Logs go under here");
        if ui.add(egui::Button::new("Clear")).clicked() {
            logger.clear();
        }
        ui.separator();
        
        egui::ScrollArea::vertical()
            .scroll_bar_visibility(egui::containers::scroll_area::ScrollBarVisibility::AlwaysVisible)
            .show(ui, |ui| {
                for evt in logger.events().iter() {
                    if let Some(field) = evt.fields.first_key_value() {
                        ui.label(field.1);
                    }
                }
            });
    });
}

fn synth_view(
    egui_ctx: &Context,
    setter: &ParamSetter,
    params: &RX11Params
) {
    egui::CentralPanel::default().show(egui_ctx, |ui| {
        // TODO - set dragged event on slider to test for changes
        // let mut my_value = 42f32;
        // let knob = ui.add(RotarySlider::new(&mut my_value, 0.0..=100.0).text("Volume"));
        // if knob.dragged() {
        //     tracing::info!("Dragging...");
        // }

        // if knob.clicked() {
        //     tracing::info!("clicked...");
        // }

        // custom_slider_ui(ui, &mut my_value, 0.0..=100.0);

        // let slider = Slider::new(&mut my_f32, 0.0..=100.0).text("Custom Slider");
        // ui.add(MyCustomSlider { slider });

        ui.separator();
        
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

                let raw_volume_val = &params.output_level.value();
                let mut volume_val = *raw_volume_val;
                // TODO - I should probably have a custom function to display as dB
                if ui
                    .add(RotarySlider::new(&mut volume_val, 0.0..=1.0)
                        .text("Volume")
                        .size(75.0)
                    )
                    .dragged()
                {
                    setter.begin_set_parameter(&params.output_level);
                    setter.set_parameter(&params.output_level, volume_val);
                    setter.end_set_parameter(&params.output_level);
                }
            })
    });// END CENTRAL PANEL
}

fn load_preset(preset: &Preset, setter: &ParamSetter, params: &RX11Params) {
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
