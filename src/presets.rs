use std::collections::HashMap;

#[derive(Clone)]
pub struct Preset {
    pub name: String,
    pub values: HashMap<String, f32>,
}

impl Preset {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            values: HashMap::new(),
        }
    }

    pub fn add_param(&mut self, param_name: &str, param_val: f32) {
        self.values.insert(param_name.to_string(), param_val);
    }
}

#[derive(Clone)]
pub struct Presets(pub Vec<Preset>);

impl Presets {
    pub fn init() -> Self {
        let file_bytes = include_bytes!("../presets.csv");
        let csv = std::str::from_utf8(file_bytes).expect("Failed to convert csv to utf8 bytes");
        let lines = csv.lines();
        let headers = &lines.clone().take(1)
            .collect::<Vec<_>>()
            .first()
            .expect("failed to get headers")
            .split(',')
            .collect::<Vec<_>>();

        let presets = &lines
            .skip(1)
            .map(|line| {
                let values: Vec<_> = line.split(',').collect();
                let mut preset = Preset::new(values[0].trim());
                for (i, val) in values.iter().skip(1).enumerate() {
                    preset.add_param(headers[i + 1], val.trim().parse::<f32>().expect("Failed to parse f32"));
                }
                preset
            })
            .collect::<Vec<_>>();

        Self(presets.clone())
    }
}
