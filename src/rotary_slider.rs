use crate::egui::{
    Align2, Color32, DragValue, emath, pos2, Label, lerp, Key,
    Painter, Pos2, Rangef, Rect, Response, remap, remap_clamp,
    Sense, Shape, Slider, Stroke, StrokeKind, style, TextStyle, TextWrapMode,
    Widget, WidgetInfo, WidgetText, 
    Ui, Vec2, vec2
};
use crate::egui::style::HandleShape;
use crate::egui::NumExt;

use std::f32::consts::{PI};

const INFINITY: f64 = f64::INFINITY;

/// When the user asks for an infinitely large range (e.g. logarithmic from zero),
/// give a scale that this many orders of magnitude in size.
const INF_RANGE_MAGNITUDE: f64 = 10.0;

use std::ops::RangeInclusive;

type NumFormatter<'a> = Box<dyn 'a + Fn(f64, RangeInclusive<usize>) -> String>;
type NumParser<'a> = Box<dyn 'a + Fn(&str) -> Option<f64>>;
type GetSetValue<'a> = Box<dyn 'a + FnMut(Option<f64>) -> f64>;

fn get(get_set_value: &mut GetSetValue<'_>) -> f64 {
    (get_set_value)(None)
}

fn set(get_set_value: &mut GetSetValue<'_>, value: f64) {
    (get_set_value)(Some(value));
}

#[derive(Clone)]
struct RotarySliderSpec {
    logarithmic: bool,
    smallest_positive: f64,
    largest_finite: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SliderClamping {
    Never,
    Edits,
    #[default]
    Always,
}

pub struct RotarySlider<'a> {
    get_set_value: GetSetValue<'a>,
    range: RangeInclusive<f64>,
    spec: RotarySliderSpec,
    clamping: SliderClamping,
    show_value: bool,
    prefix: String,
    suffix: String,
    text: WidgetText,
    step: Option<f64>,
    drag_value_speed: Option<f64>,
    min_decimals: usize,
    max_decimals: Option<usize>,
    custom_formatter: Option<NumFormatter<'a>>,
    custom_parser: Option<NumParser<'a>>,
    trailing_fill: Option<bool>,
    handle_shape: Option<HandleShape>, // Not needed, this was for the handle (circle / rect) of a slider
}

impl<'a> RotarySlider<'a> {
    pub fn new<Num: emath::Numeric>(value: &'a mut Num, range: RangeInclusive<Num>) -> Self {
        let range_f64 = range.start().to_f64()..=range.end().to_f64();
        let slf = Self::from_get_set(range_f64, move |v: Option<f64>| {
           if let Some(v) = v {
               *value = Num::from_f64(v);
           }
           value.to_f64()
        });

        if Num::INTEGRAL {
            slf.integer()
        } else {
            slf
        }
    }

    pub fn from_get_set(
        range: RangeInclusive<f64>,
        get_set_value: impl 'a + FnMut(Option<f64>) -> f64,
    ) -> Self {
        Self {
            get_set_value: Box::new(get_set_value),
            range,
            spec: RotarySliderSpec {
                logarithmic: false,
                smallest_positive: 1e-6,
                largest_finite: f64::INFINITY,
            },
            clamping: SliderClamping::default(),
            show_value: true,
            prefix: Default::default(),
            suffix: Default::default(),
            text: Default::default(),
            step: None,
            drag_value_speed: None,
            min_decimals: 0,
            max_decimals: None,
            custom_formatter: None,
            custom_parser: None,
            trailing_fill: None,
            handle_shape: None,
        }
    }

    #[inline]
    pub fn show_value(mut self, show_value: bool) -> Self {
        self.show_value = show_value;
        self
    }

    #[inline]
    pub fn prefix(mut self, prefix: impl ToString) -> Self {
        self.prefix = prefix.to_string();
        self
    }

    #[inline]
    pub fn suffix(mut self, suffix: impl ToString) -> Self {
        self.suffix = suffix.to_string();
        self
    }

    #[inline]
    pub fn text(mut self, text: impl Into<WidgetText>) -> Self {
        self.text = text.into();
        self
    }

    #[inline]
    pub fn text_color(mut self, text_color: Color32) -> Self {
        self.text = self.text.color(text_color);
        self
    }

    #[inline]
    pub fn logiarithmic(mut self, logarithmic: bool) -> Self {
        self.spec.logarithmic = logarithmic;
        self
    }

    #[inline]
    pub fn smallest_positive(mut self, smallest_positive: f64) -> Self {
        self.spec.smallest_positive = smallest_positive;
        self
    }

    #[inline]
    pub fn largest_finite(mut self, largest_finite: f64) -> Self {
        self.spec.largest_finite = largest_finite;
        self
    }

    #[inline]
    pub fn clamping(mut self, clamping: SliderClamping) -> Self {
        self.clamping = clamping;
        self
    }

    #[inline]
    pub fn step_by(mut self, step: f64) -> Self {
        self.step = if step != 0.0 { Some(step) } else { None };
        self
    }

    #[inline]
    pub fn drag_value_speed(mut self, drag_value_speed: f64) -> Self {
        self.drag_value_speed = Some(drag_value_speed);
        self
    }

    #[inline]
    pub fn min_decimals(mut self, min_decimals: usize) -> Self {
        self.min_decimals = min_decimals;
        self
    }

    #[inline]
    pub fn max_decimals(mut self, max_decimals: usize) -> Self {
        self.max_decimals = Some(max_decimals);
        self
    }

    #[inline]
    pub fn max_decimals_opt(mut self, max_decimals: Option<usize>) -> Self {
        self.max_decimals = max_decimals;
        self
    }

    #[inline]
    pub fn fixed_decimals(mut self, num_decimals: usize) -> Self {
        self.min_decimals = num_decimals;
        self.max_decimals = Some(num_decimals);
        self
    }

    #[inline]
    pub fn trailing_fill(mut self, trailing_fill: bool) -> Self {
        self.trailing_fill = Some(trailing_fill);
        self
    }

    #[inline]
    pub fn handle_shape(mut self, handle_shape: HandleShape) -> Self {
        self.handle_shape = Some(handle_shape);
        self
    }

    pub fn customer_formatter(
        mut self,
        formatter: impl 'a + Fn(f64, RangeInclusive<usize>) -> String,
    ) -> Self {
        self.custom_formatter = Some(Box::new(formatter));
        self
    }

    #[inline]
    pub fn customer_parser(mut self, parser: impl 'a + Fn(&str) -> Option<f64>) -> Self {
        self.custom_parser = Some(Box::new(parser));
        self
    }

    pub fn integer(self) -> Self {
        self.fixed_decimals(0).smallest_positive(1.0).step_by(1.0)
    }

    fn get_value(&mut self) -> f64 {
        let value = get(&mut self.get_set_value);

        if self.clamping == SliderClamping::Always {
            value
            //clamp_value_to_range(value, self.range.clone())
        } else {
            value
        }
    }

    fn set_value(&mut self, mut value: f64) {
        // if self.clamping != SliderClamping::Never {
        //     value = clamp_value_to_range(value, self.range.clone());
        // }

        if let Some(step) = self.step {
            let start = *self.range.start();
            value = start + ((value - start) / step).round() * step;
        }

        if let Some(max_decimals) = self.max_decimals {
            value = emath::round_to_decimals(value, max_decimals);
        }

        set(&mut self.get_set_value, value);
    }

    fn range(&self) -> RangeInclusive<f64> {
        self.range.clone()
    }

    fn value_from_position(&self, position: f32, position_range: Rangef) -> f64 {
        let normalized = remap_clamp(position, position_range, 0.0..=1.0) as f64;
        value_from_normalized(normalized, self.range(), &self.spec)
    }

    fn position_from_value(&self, value: f64, position_range: Rangef) -> f32 {
        let normalized = normalized_from_value(value, self.range(), &self.spec);
        lerp(position_range, normalized as f32)
    }
}

impl RotarySlider<'_> {
    fn allocate_slider_space(&self, ui: &mut Ui, thickness: f32) -> Response {
        // TODO - the size should probably be configurable
        let desired_size = vec2(64.0, 64.0);
        ui.allocate_response(desired_size, Sense::drag())
    }

    fn pointer_position(&self, pointer_position_2d: Pos2) -> f32 {
        pointer_position_2d.y
    }

    fn slider_ui(&mut self, ui: &mut Ui, response: &Response) {
        let rect = &response.rect;
        let handle_shape = self
            .handle_shape
            .unwrap_or_else(|| ui.style().visuals.handle_shape);

        let position_range = self.position_range(rect, &handle_shape);

        if let Some(pointer_position_2d) = response.interact_pointer_pos() {
            let position = self.pointer_position(pointer_position_2d);
            /*
            let new_value = if self.smart_aim {
                let aim_radius = ui.input(|i| i.aim_radius());
                emath::smart_aim::best_in_range_f64(
                    self.value_from_position(position - aim_radius, position_range),
                    self.value_from_position(position + aim_radius, position_range),
                )
            } else {
                self.value_from_position(position, position_range);    
            };
            */
            let new_value = self.value_from_position(position, position_range);    
            self.set_value(new_value);
        }

        let decrement = 0usize;
        let increment = 0usize;

        /*
        if response.has_focus() {
            ui.ctx().memory_mut(|m| {
                m.set_focus_lock_filter(
                    response.id,
                    EventFilter {
                        // pressing arrows in the orientation of the
                        // slider should not move focus to next widget
                        horizontal_arrows: matches!(
                            self.orientation,
                            SliderOrientation::Horizontal
                        ),
                        vertical_arrows: matches!(self.orientation, SliderOrientation::Vertical),
                        ..Default::default()
                    },
                );
            });

            let (dec_key, inc_key) = (Key::ArrowUp, Key::ArrowDown);

            ui.input(|input| {
                decrement += input.num_presses(dec_key);
                increment += input.num_presses(inc_key);
            });
        }
        */

        let kb_step = increment as f32 - decrement as f32;

        if kb_step != 0.0 {
            let ui_point_per_step = 1.0; // move this many ui points for each kb_step
            let prev_value = self.get_value();
            let prev_position = self.position_from_value(prev_value, position_range);
            let new_position = prev_position + ui_point_per_step * kb_step;
            /*
            let new_value = match self.step {
                Some(step) => prev_value + (kb_step as f64 * step),
                None if self.smart_aim => {
                    let aim_radius = 0.49 * ui_point_per_step; // Chosen so we don't include `prev_value` in the search.
                    emath::smart_aim::best_in_range_f64(
                        self.value_from_position(new_position - aim_radius, position_range),
                        self.value_from_position(new_position + aim_radius, position_range),
                    )
                }
                _ => self.value_from_position(new_position, position_range),
            };
            */
            let new_value = self.value_from_position(new_position, position_range);
            self.set_value(new_value);
        }

        if ui.is_rect_visible(response.rect) { 
            let val = self.get_value() / self.range.end();
            let visuals = ui.style().interact(response);
            let widget_visuals = &ui.visuals().widgets;
            let spacing = &ui.style().spacing;
            let corner_radius = widget_visuals.inactive.corner_radius;
            let radius = 25.0;
            let stroke = Stroke {
                width: 1.0, color: Color32::CYAN,
            };

            ui.painter()
                // .rect_filled(dial_rect, corner_radius, widget_visuals.inactive.bg_fill);
                .rect_stroke(*rect, corner_radius, stroke, StrokeKind::Outside);

            let center = response.rect.center();

            // Rotates counter clockwise
            let start_angle = PI * 2.25;
            let end_angle = PI * 0.75;

            let _ = &self.draw_arc(
                &ui.painter(),
                center,
                radius,
                start_angle,
                end_angle,
                64,              // number of segments (smoothness)
                Color32::LIGHT_BLUE,
                2.0,             // thickness
            );

            // Draw the line gauge
            let t = val as f32;
            let value_angle = start_angle + (1.0 - t) * (end_angle - start_angle);
            let x = center.x + radius * value_angle.cos();
            let y = center.y + radius * value_angle.sin();
            let mut line_points = Vec::with_capacity(2);
            line_points.push(Pos2 { x: center.x, y: center.y });
            line_points.push(Pos2 { x, y });

            ui.painter().add(Shape::line(line_points, Stroke::new(2.0, Color32::YELLOW)));
        }
    }

     fn current_gradient(&mut self, position_range: Rangef) -> f64 {
        let value = self.get_value();
        let value_from_pos = |position: f32| self.value_from_position(position, position_range);
        let pos_from_value = |value: f64| self.position_from_value(value, position_range);
        let left_value = value_from_pos(pos_from_value(value) - 0.5);
        let right_value = value_from_pos(pos_from_value(value) + 0.5);
        right_value - left_value
    }

    fn add_contents(&mut self, ui: &mut Ui) -> Response {
        let old_value = self.get_value();

        if self.clamping == SliderClamping::Always {
            self.set_value(old_value);
        }

        let thickness = ui
            .text_style_height(&TextStyle::Body)
            .at_least(ui.spacing().interact_size.y);

        let mut response = self.allocate_slider_space(ui, thickness);

        self.slider_ui(ui, &response);
        let value = self.get_value();

        if value != old_value {
            response.mark_changed();
        }

        response.widget_info(|| WidgetInfo::slider(ui.is_enabled(), value, self.text.text()));

        let slider_response = response.clone();
        let value_response = if self.show_value {
            let handle_shape = self
                .handle_shape
                .unwrap_or_else(|| ui.style().visuals.handle_shape);

            let position_range = self.position_range(&response.rect, &handle_shape);
            let value_response = self.value_ui(ui, position_range);

            if value_response.gained_focus()
                || value_response.has_focus()
                || value_response.lost_focus()
            {
                // Use the [`DragValue`] id as the id of the whole widget,
                // so that the focus events work as expected.
                response = value_response.union(response);
            } else {
                // Use the slider id as the id for the whole widget
                response = response.union(value_response.clone());
            }
            Some(value_response)
        } else {
            None
        };

        if !self.text.is_empty() {
            let label_response =
                ui.add(Label::new(self.text.clone()).wrap_mode(TextWrapMode::Extend));
            // The slider already has an accessibility label via widget info,
            // but sometimes it's useful for a screen reader to know
            // that a piece of text is a label for another widget,
            // e.g. so the text itself can be excluded from navigation.
            slider_response.labelled_by(label_response.id);
            if let Some(value_response) = value_response {
                value_response.labelled_by(label_response.id);
            }
        }

        response
    }

    
    fn value_ui(&mut self, ui: &mut Ui, position_range: Rangef) -> Response {
        // If [`DragValue`] is controlled from the keyboard and `step` is defined, set speed to `step`
        let change = ui.input(|input| {
            input.num_presses(Key::ArrowUp) as i32 + input.num_presses(Key::ArrowRight) as i32
                - input.num_presses(Key::ArrowDown) as i32
                - input.num_presses(Key::ArrowLeft) as i32
        });

        let any_change = change != 0;

        let speed = if let (Some(step), true) = (self.step, any_change) {
            // If [`DragValue`] is controlled from the keyboard and `step` is defined, set speed to `step`
            step
        } else {
            self.drag_value_speed
                .unwrap_or_else(|| self.current_gradient(position_range))
        };

        let mut value = self.get_value();

        let response = ui.add({
            let mut dv = DragValue::new(&mut value)
                .speed(speed)
                .min_decimals(self.min_decimals)
                .max_decimals_opt(self.max_decimals)
                .suffix(self.suffix.clone())
                .prefix(self.prefix.clone());

            /*
            match self.clamping {
                SliderClamping::Never => {}
                SliderClamping::Edits => {
                    dv = dv.range(self.range.clone()).clamp_existing_to_range(false);
                }

                SliderClamping::Always => {
                    dv = dv.range(self.range.clone()).clamp_existing_to_range(true);
                }
            }
            */

            if let Some(fmt) = &self.custom_formatter {
                dv = dv.custom_formatter(fmt);
            };

            if let Some(parser) = &self.custom_parser {
                dv = dv.custom_parser(parser);
            }
            dv
        });

        if value != self.get_value() {
            self.set_value(value);
        }

        response
    }

    // TODO: I can probably delete
    // This is for the little knob on the slider
    fn handle_radius(&self, rect: &Rect) -> f32 {
        /*
        let limit = match self.orientation {
            SliderOrientation::Horizontal => rect.height(),
            SliderOrientation::Vertical => rect.width(),
        };
        */

        rect.height() / 2.5
    }

    fn position_range(&self, rect: &Rect, handle_shape: &style::HandleShape) -> Rangef {
        let handle_radius = self.handle_radius(rect);

        let handle_radius = match handle_shape {
            style::HandleShape::Circle => handle_radius,
            style::HandleShape::Rect { aspect_ratio } => handle_radius * aspect_ratio,
        };

        // The vertical case has to be flipped because the largest slider value maps to the
        // lowest y value (which is at the top)
        rect.y_range().shrink(handle_radius).flip()
    }

    /// Draw an arc on the given painter
    fn draw_arc(
        &self,
        painter: &Painter,
        center: Pos2,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
        segments: usize,
        color: Color32,
        thickness: f32,
    ) {
        let mut points = Vec::with_capacity(segments + 1);

        // May need to account for clockwise vs counter clockwise drawing
        // based on start_angle <=> end_angle
        for i in 0..=segments {
            // t is the percentage of the total
            let t = i as f32/ segments as f32;

            // How this works when start_angle > end_angle
            // 0.75PI - 2.25PI = -1.5PI (Diff)
            // 2.25PI + 0 *-1.5PI = 2.25PI
            // 2.25PI + 0.0166 * -1.5PI = 2.22PI
            // ...
            // 2.25PI + 1 * -1.5PI = 0.75PI
            // It walks from the start angle and then goes counter clockwise
            let angle = start_angle + t * (end_angle - start_angle);

            // This is the actual segment that makes up the arc
            let x = center.x + radius * angle.cos();
            let y = center.y + radius * angle.sin();
            points.push(Pos2 { x, y });
        }

        painter.add(Shape::line(points, Stroke::new(thickness, color)));
    }
}


impl Widget for RotarySlider<'_> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let inner_response = ui.vertical(|ui| self.add_contents(ui));
        inner_response.inner | inner_response.response
    }
}


fn value_from_normalized(normalized: f64, range: RangeInclusive<f64>, spec: &RotarySliderSpec) -> f64 {
    let (min, max) = (*range.start(), *range.end());

    if min.is_nan() || max.is_nan() {
        f64::NAN
    } else if min == max {
        min
    } else if min > max {
        value_from_normalized(1.0 - normalized, max..=min, spec)
    } else if normalized <= 0.0 {
        min
    } else if normalized >= 1.0 {
        max
    } else if spec.logarithmic {
        if max <= 0.0 {
            // non-positive range
            -value_from_normalized(normalized, -min..=-max, spec)
        } else if 0.0 <= min {
            let (min_log, max_log) = range_log10(min, max, spec);
            let log = lerp(min_log..=max_log, normalized);
            10.0_f64.powf(log)
        } else {
            assert!(min < 0.0 && 0.0 < max);
            let zero_cutoff = logarithmic_zero_cutoff(min, max);

            if normalized < zero_cutoff {
                // negative
                value_from_normalized(
                    remap(normalized, 0.0..=zero_cutoff, 0.0..=1.0),
                    min..=0.0,
                    spec,
                )
            } else {
                // positive
                value_from_normalized(
                    remap(normalized, zero_cutoff..=1.0, 0.0..=1.0),
                    0.0..=max,
                    spec,
                )
            }
        }
    } else {
        debug_assert!(
            min.is_finite() && max.is_finite(),
            "You should use a logarithmic range"
        );

        lerp(range, normalized.clamp(0.0, 1.0))
    }
}

fn normalized_from_value(value: f64, range: RangeInclusive<f64>, spec: &RotarySliderSpec) -> f64 {
    let (min, max) = (*range.start(), *range.end());

    if min.is_nan() || max.is_nan() {
        f64::NAN
    } else if min == max {
        0.5 // empty range, show center of slider
    } else if min > max {
        1.0 - normalized_from_value(value, max..=min, spec)
    } else if value <= min {
        0.0
    } else if value >= max {
        1.0
    } else if spec.logarithmic {
        if max <= 0.0 {
            // non-positive range
            normalized_from_value(-value, -min..=-max, spec)
        } else if 0.0 <= min {
            let (min_log, max_log) = range_log10(min, max, spec);
            let value_log = value.log10();
            remap_clamp(value_log, min_log..=max_log, 0.0..=1.0)
        } else {
            assert!(min < 0.0 && 0.0 < max);
            let zero_cutoff = logarithmic_zero_cutoff(min, max);

            if value < 0.0 {
                // negative
                remap(
                    normalized_from_value(value, min..=0.0, spec),
                    0.0..=1.0,
                    0.0..=zero_cutoff,
                )
            } else {
                // positive side
                remap(
                    normalized_from_value(value, 0.0..=max, spec),
                    0.0..=1.0,
                    zero_cutoff..=1.0,
                )
            }
        }
    } else {
        debug_assert!(
            min.is_finite() && max.is_finite(),
            "You should use a logarithmic range"
        );

        remap_clamp(value, range, 0.0..=1.0)
    }
}

fn range_log10(min: f64, max: f64, spec: &RotarySliderSpec) -> (f64, f64) {
    assert!(spec.logarithmic);
    assert!(min <= max);

    if min == 0.0 && max == INFINITY {
        (spec.smallest_positive.log10(), INF_RANGE_MAGNITUDE)
    } else if min == 0.0 {
        if spec.smallest_positive < max {
            (spec.smallest_positive.log10(), max.log10())
        } else {
            (max.log10() - INF_RANGE_MAGNITUDE, max.log10())
        }
    } else if max == INFINITY {
        if min < spec.largest_finite {
            (min.log10(), spec.largest_finite.log10())
        } else {
            (min.log10(), min.log10() + INF_RANGE_MAGNITUDE)
        }
    } else {
        (min.log10(), max.log10())
    }
}

/// where to put the zero cutoff for logarithmic sliders
/// that crosses zero ?
fn logarithmic_zero_cutoff(min: f64, max: f64) -> f64 {
    assert!(min < 0.0 && 0.0 < max);

    let min_magnitude = if min == -INFINITY {
        INF_RANGE_MAGNITUDE
    } else {
        min.abs().log10().abs()
    };

    let max_magnitude = if max == INFINITY {
        INF_RANGE_MAGNITUDE
    } else {
        max.log10().abs()
    };

    let cutoff = min_magnitude / (min_magnitude + max_magnitude);

    debug_assert!(
        0.0 <= cutoff && cutoff <= 1.0,
        "Bad cutoff {cutoff:?} for min {min:?} and max {max:?}"
    );

    cutoff
}
