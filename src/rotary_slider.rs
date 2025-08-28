// This is copied from the SFLT plugin source code.
// https://estrobiologist.gumroad.com/l/sflt
// SFLT source code copied it from
// https://github.com/obsqrbtz/egui_knob
// Both are excellent. I will be trying to customize additional behavior and just needed something for now.
// SFLT does windows specific stuff to hide the mouse and return it to the starting cursor position.
// I want to figure this out for cross platform or at least hide it behind build targets so I can still use
// this plugin on multiple platforms

#![allow(clippy::needless_pass_by_value)] // false positives with `impl ToString`
#![allow(dead_code)]

use crate::egui::{Align2, Color32, Rect, Response, Sense, Stroke, Ui, Vec2, Widget};
use crate::egui:: {
    emath, Button, FontId, NumExt, RichText,
    TextWrapMode, WidgetInfo,
};

use std::{cmp::Ordering, ops::RangeInclusive};

pub enum LabelPosition {
    Top,
    Bottom,
    Left,
    Right,
}


pub enum KnobStyle {
    Wiper,
    Dot,
}

pub struct Knob<'a> {
    value: &'a mut f32,
    min: f32,
    max: f32,
    size: f32,
    font_size: f32,
    stroke_width: f32,
    knob_color: Color32,
    line_color: Color32,
    text_color: Color32,
    label: Option<String>,
    label_position: LabelPosition,
    style: KnobStyle,
    label_offset: f32,
    step: Option<f32>,
    delta_offset: f32,
}

impl<'a> Knob<'a> {
    pub fn new(value: &'a mut f32, min: f32, max: f32, style: KnobStyle) -> Self{
        Self {
            value,
            min,
            max,
            size: 40.0,
            font_size: 12.0,
            stroke_width: 2.0,
            knob_color: Color32::GRAY,
            line_color: Color32::GRAY,
            text_color: Color32::WHITE,
            label: None,
            label_position: LabelPosition::Bottom,
            style,
            label_offset: 1.0,
            step: None,
            delta_offset: 0.0,
        }
    }

    pub fn with_delta_offset(mut self, delta_offset: f32) -> Self {
        self.delta_offset = delta_offset;
        self
    }

    pub fn with_size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    pub fn with_stroke_width(mut self, width: f32) -> Self {
        self.stroke_width = width;
        self
    }

    pub fn with_colors(mut self, knob_color: Color32, line_color: Color32, text_color: Color32) -> Self {
        self.knob_color = knob_color;
        self.line_color = line_color;
        self.text_color = text_color;
        self
    }

    pub fn with_label(mut self, label: impl Into<String>, position: LabelPosition) -> Self {
        self.label = Some(label.into());
        self.label_position = position;
        self
    }

    pub fn with_label_offset(mut self, offset: f32) -> Self {
        self.label_offset = offset;
        self
    }

    pub fn with_step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self
    }
}


impl Widget for Knob<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let knob_size = Vec2::splat(self.size);

        let label_size = if let Some(label) = &self.label {
            let font_id = FontId::proportional(self.font_size);
            let max_text = label.clone();
            ui.painter()
                .layout(max_text, font_id, Color32::WHITE, f32::INFINITY)
                .size()
        } else {
            Vec2::ZERO
        };

        let label_padding = 2.0;

        let adjusted_size = match self.label_position {
            LabelPosition::Top | LabelPosition::Bottom => Vec2::new(
                knob_size.x.max(label_size.x + label_padding * 2.0),
                knob_size.y + label_size.y + label_padding * 2.0 + self.label_offset,
            ),
            LabelPosition::Left | LabelPosition::Right => Vec2::new(
                  knob_size.x + label_size.x + label_padding * 2.0 + self.label_offset,
                  knob_size.y.max(label_size.y + label_padding * 2.0)      
            ),
        };

        let left_down = ui.input(|state| state.pointer.primary_down());
        let sense = if left_down { Sense::drag() } else { Sense::click_and_drag() };

        let (rect, mut response) = ui.allocate_exact_size(adjusted_size, sense);

        // Modify State of knob on drag
        if response.dragged() {
            let modifier = if ui.input(|i| i.modifiers.command_only()) { 0.1 } else { 1.0 };
            let delta = response.drag_delta().y + self.delta_offset;
            let range = self.max - self.min;
            let step = self.step.unwrap_or(range * 0.005) * modifier;
            let new_value = (*self.value - delta * step).clamp(self.min, self.max);

            *self.value = if let Some(step) = self.step {
                let steps = ((new_value - self.min) / step).round();
                (self.min + steps * step).clamp(self.min, self.max)
            } else {
                new_value
            };

            response.mark_changed();
        }

        // Start drawing the knob
        let painter = ui.painter();
        let knob_rect = match self.label_position {
            LabelPosition::Left => {
                Rect::from_min_size(rect.right_top() + Vec2::new(-knob_size.x, 0.0), knob_size)
            }
            LabelPosition::Right => Rect::from_min_size(rect.left_top(), knob_size),
            LabelPosition::Top => Rect::from_min_size(
                rect.left_bottom() + Vec2::new((rect.width() - knob_size.x) / 2.0, -knob_size.y),
                knob_size,      
            ),
            LabelPosition::Bottom => Rect::from_min_size(
                rect.left_top() + Vec2::new((rect.width() - knob_size.x) / 2.0, 0.0),
                knob_size,
            ),
        };

        let center = knob_rect.center();
        let radius = knob_size.x / 2.0;
        let angle = (*self.value - self.min) / (self.max - self.min)
            * std::f32::consts::PI * 1.5 - std::f32::consts::PI * 1.25;

        let knob_color = if response.hovered() && !response.dragged() {
            Color32::LIGHT_GRAY
        } else {
            Color32::GRAY
        };

        painter.circle_stroke(center, radius, Stroke::new(self.stroke_width, knob_color));

        match self.style {
            KnobStyle::Wiper => {
                let pointer = center + Vec2::angled(angle) * (radius * 0.7);
                painter.line_segment(
                    [center, pointer],
                    Stroke::new(self.stroke_width * 1.5, knob_color),
                );
            }
            KnobStyle::Dot => {
                let dot_pos = center + Vec2::angled(angle) * (radius * 0.7);
                painter.circle_filled(dot_pos, self.stroke_width * 1.5, knob_color);
            }
        }

        if let Some(label) = self.label {
            let label_text = label.clone();
            let font_id = FontId::proportional(self.font_size);

            let (label_pos, alignment) = match self.label_position {
                LabelPosition::Top => (
                  Vec2::new(rect.center().x, rect.min.y - self.label_offset + label_padding),
                  Align2::CENTER_TOP,
                ),
                LabelPosition::Bottom => (
                  Vec2::new(rect.center().x, rect.max.y + self.label_offset),
                  Align2::CENTER_BOTTOM,
                ),
                LabelPosition::Left => (
                  Vec2::new(rect.min.x - self.label_offset, rect.center().y),
                  Align2::LEFT_CENTER,
                ),
                LabelPosition::Right => (
                  Vec2::new(rect.max.x + self.label_offset, rect.center().y),
                  Align2::RIGHT_CENTER,
                ),
            };

            ui.painter().text(
                label_pos.to_pos2(),
                alignment,
                label_text,
                font_id,
                self.text_color,
            );
        }


        // DEBUG: Draw the bounding rect
        //painter.rect_stroke(rect, 0.0, Stroke::new(1.0, Color32::RED));

        response
    }
}

//////// Drag Value

type NumFormatter<'a> = Box<dyn 'a + Fn(f64, RangeInclusive<usize>) -> String>;
type NumParser<'a> = Box<dyn 'a + Fn(&str) -> Option<&f64>>;
type GetSetValue<'a> = Box<dyn 'a + FnMut(Option<f64>) -> f64>;

fn get(get_set_value: &mut GetSetValue<'_>) -> f64 {
    (get_set_value)(None)
}

fn set(get_set_value: &mut GetSetValue<'_>, value: f64) {
    (get_set_value)(Some(value));
}

pub struct DragValue<'a> {
    get_set_value: GetSetValue<'a>,
    speed: f64,
    prefix: String,
    suffix: String,
    range: RangeInclusive<f64>,
    clamp_existing_to_range: bool,
    min_decimals: usize,
    max_decimals: Option<usize>,
    custom_formatter: Option<NumFormatter<'a>>,
    delta_offset: f32,
}

impl <'a> DragValue<'a> {
    pub fn new<Num: emath::Numeric>(value: &'a mut Num) -> Self {
        let slf = Self::from_get_set(move |v: Option<f64>| {
            if let Some(v) = v {
                *value = Num::from_f64(v);
            }
            value.to_f64()
        });

        if Num::INTEGRAL {
            slf.max_decimals(0).range(Num::MIN..=Num::MAX).speed(0.25)
        } else {
            slf
        }
    }

    pub fn from_get_set(get_set_value: impl 'a + FnMut(Option<f64>) -> f64) -> Self {
        Self {
            get_set_value: Box::new(get_set_value),
            speed: 1.0,
            prefix: Default::default(),
            suffix: Default::default(),
            range: f64::NEG_INFINITY..=f64::INFINITY,
            clamp_existing_to_range: true,
            min_decimals: 0,
            max_decimals: None,
            custom_formatter: None,
            delta_offset: 0.0
        }
    }

    #[inline]
    pub fn speed(mut self, speed: impl Into<f64>) -> Self {
        self.speed = speed.into();
        self
    }

    #[inline]
    pub fn range<Num: emath::Numeric>(mut self, range: RangeInclusive<Num>) -> Self {
        self.range = range.start().to_f64()..=range.end().to_f64();
        self
    }

    #[inline]
    pub fn clamp_existing_to_range(mut self, clamp_existing_to_range: bool) -> Self {
        self.clamp_existing_to_range = clamp_existing_to_range;
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

    pub fn custom_formatter(
        mut self,
        formatter: impl 'a + Fn(f64, RangeInclusive<usize>) -> String,
    ) -> Self {
        self.custom_formatter = Some(Box::new(formatter));
        self
    }

    pub fn binary(self, min_width: usize, twos_compliment: bool) -> Self {
        assert!(min_width > 0, "DragValue::binary: `min_width` must be greater than 0");

        if twos_compliment {
            self.custom_formatter(move |n, _| format!("{:0>min_width$b}", n as i64))
        } else {
            self.custom_formatter(move |n, _| {
                let sign = if n < 0.0 { "-" } else { "" };
                format!("{sign}{:0>min_width$b}", n.abs() as i64)
            })
        }
    }

    // TODO: hex and octal

    pub fn with_delta_offset(mut self, delta_offset: f32) -> Self {
        self.delta_offset = delta_offset;
        self
    }
}


impl Widget for DragValue<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self {
            mut get_set_value,
            speed,
            range,
            clamp_existing_to_range,
            prefix,
            suffix,
            min_decimals,
            max_decimals,
            custom_formatter,
            delta_offset
        } = self;

        let shift = ui.input(|i| i.modifiers.command_only());
        let id = ui.next_auto_id();
        let is_slow_speed = shift && ui.ctx().is_being_dragged(id);

        if ui.memory_mut(|mem| !mem.had_focus_last_frame(id) && mem.has_focus(id)) {
            ui.data_mut(|data| data.remove::<String>(id));
        }

        let old_value = get(&mut get_set_value);
        let mut value = old_value;
        let aim_rad = ui.input(|i| i.aim_radius() as f64);

        let auto_decimals = (aim_rad / speed.abs()).log10().ceil().clamp(0.0, 15.0) as usize;
        let auto_decimals = auto_decimals + is_slow_speed as usize;
        let max_decimals = max_decimals
            .unwrap_or(auto_decimals + 2)
            .at_least(min_decimals);

        let auto_decimals = auto_decimals.clamp(min_decimals, max_decimals);

        if clamp_existing_to_range {
            value = clamp_value_to_range(value, range.clone());
        }

        if old_value != value {
            set(&mut get_set_value, value);
            ui.data_mut(|data| data.remove::<String>(id));
        }

        let value_text = match custom_formatter {
            Some(custom_formatter) => custom_formatter(value, auto_decimals..=max_decimals),
            None => ui.style().number_formatter.format(value, auto_decimals..=max_decimals),
        };

        let text_style = ui.style().drag_value_text_style.clone();

        #[allow(clippy::redundant_clone)]
        let mut response = {
            let button = Button::new(
                RichText::new(format!("{}{}{}", prefix, value_text.clone(), suffix))
                .text_style(text_style),
            )
            .wrap_mode(TextWrapMode::Extend)
            .sense(Sense::drag())
            .min_size(ui.spacing().interact_size);

            let response = ui.add(button);

            if ui.input(|i| i.pointer.any_pressed() || i.pointer.any_released()) {
                ui.data_mut(|data| data.remove::<f64>(id));
            }

            if response.dragged() {
                let mdelta = response.drag_delta() + Vec2::new(0.0, delta_offset);
                let delta_points = -mdelta.y;

                let speed = if is_slow_speed { speed / 10.0 } else { speed };
                let delta_value = delta_points as f64 * speed;

                if delta_value != 0.0 {
                    let precise_value = ui.data_mut(|data| data.get_temp::<f64>(id));
                    let precise_value = precise_value.unwrap_or(value);
                    let precise_value = precise_value + delta_value;

                    let aim_delta = aim_rad * speed;
                    let rounded_new_value = emath::smart_aim::best_in_range_f64(
                        precise_value - aim_delta,
                        precise_value + aim_delta,
                    );

                    let rounded_new_value = emath::round_to_decimals(rounded_new_value, auto_decimals);
                    let rounded_new_value = clamp_value_to_range(rounded_new_value, range.clone());
                    set(&mut get_set_value, rounded_new_value);

                    ui.data_mut(|data| data.insert_temp::<f64>(id, precise_value));
                }
            }

            response
        };

        if get(&mut get_set_value) != old_value {
            response.mark_changed();
        }

        response.widget_info(|| WidgetInfo::drag_value(ui.is_enabled(), value));
        response
    }   
}

fn parse(custom_parser: &Option<NumParser<'_>>, value_text: &str) -> Option<f64> {
    match &custom_parser {
        Some(parser) => parser(value_text).copied(),
        None => default_parser(value_text),
    }
}

fn default_parser(text: &str) -> Option<f64> {
    let text = text
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| if c == '-' { '-' } else { c})
        .collect::<String>();

    text.parse().ok()
}

pub(crate) fn clamp_value_to_range(x: f64, range: RangeInclusive<f64>) -> f64 {
    let (mut min, mut max) = (*range.start(), *range.end());

    if min.total_cmp(&max) == Ordering::Greater {
        (min, max) = (max, min);
    }

    match x.total_cmp(&min) {
        Ordering::Less | Ordering::Equal => min,
        Ordering::Greater => match x.total_cmp(&max) {
            Ordering::Greater | Ordering::Equal => max,
            Ordering::Less => x,
        },
    }
}

