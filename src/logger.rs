use std::collections::BTreeMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use tracing::field::{Field, Visit};
use tracing::{Event, Level, Metadata, Subscriber};

use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;

#[derive(Debug, Clone)]
pub struct CollectedEvent {
    pub target: String,
    pub level: tracing::Level,
    pub fields: BTreeMap<String, String>,
}

impl CollectedEvent {
    pub fn new(event: &Event, meta: &Metadata) -> Self {
        let mut fields = BTreeMap::new();
        event.record(&mut FieldVisitor(&mut fields));

        Self {
            level: meta.level().to_owned(),
            target: meta.target().to_owned(),
            fields,
        }
    }
}

struct FieldVisitor<'a>(&'a mut BTreeMap<String, String>);

impl<'a> Visit for FieldVisitor<'a> {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.0
            .insert(field.name().to_string(), format!("{:?}", value));
    }
}

#[derive(Debug, Clone)]
pub enum AllowedTargets {
    All,
    Selected(Vec<String>),
}

#[derive(Debug, Clone)]
pub struct EventCollector {
    allowed_targets: AllowedTargets,
    level: Level,
    events: Arc<Mutex<Vec<CollectedEvent>>>,
}

impl EventCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_level(self, level: Level) -> Self {
        Self { level, ..self }
    }

    pub fn allowed_targets(self, allowed_targets: AllowedTargets) -> Self {
        Self {
            allowed_targets,
            ..self
        }
    }

    pub fn events(&self) -> Vec<CollectedEvent> {
        self.events.lock().unwrap().clone()
    }

    pub fn clear(&self) {
        let mut events = self.events.lock().unwrap();
        *events = Vec::new();
    }

    fn collect(&self, event: CollectedEvent) {
        if event.level <= self.level {
            let should_collect = match self.allowed_targets {
                AllowedTargets::All => true,
                AllowedTargets::Selected(ref selection) => selection
                    .iter()
                    .any(|target| event.target.starts_with(target)),
            };

            if should_collect {
                self.events.lock().unwrap().push(event);
            }
        }
    }
}

impl Default for EventCollector {
    fn default() -> Self {
        Self {
            allowed_targets: AllowedTargets::All,
            events: Arc::new(Mutex::new(Vec::new())),
            level: Level::TRACE,
        }
    }
}

impl<S> Layer<S> for EventCollector
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let meta = event.metadata();

        self.collect(CollectedEvent::new(event, meta));
    }
}
