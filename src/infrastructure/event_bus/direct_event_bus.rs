use crate::application::event_bus::{EventBus, EventBusError, EventListener};
use crate::domain::usecases::event_processor::EventProcessor;
use anyhow::Error;
use async_trait::async_trait;
use log::warn;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use uuid::Uuid;

#[derive(Default)]
pub struct DirectEventBus {
    pub events: Mutex<VecDeque<Uuid>>,
}

impl DirectEventBus {
    pub fn new() -> Self {
        Self {
            events: Mutex::new(VecDeque::new()),
        }
    }
}

#[async_trait]
impl EventBus for DirectEventBus {
    #[tracing::instrument(name = "Publishing events", skip(self))]
    async fn publish(&self, event_ids: &[Uuid]) -> Result<(), EventBusError> {
        {
            let mut evts = self.events.lock().unwrap();
            for evt in event_ids {
                evts.push_back(*evt);
            }
        }
        Ok(())
    }
}

pub struct DirectEventListener {
    event_bus: Arc<DirectEventBus>,
    processors: Vec<Arc<dyn EventProcessor>>,
}

pub enum ExecutionOutcome {
    TaskCompleted,
    EmptyQueue,
}

impl DirectEventListener {
    pub fn new(event_bus: Arc<DirectEventBus>) -> Self {
        Self {
            event_bus,
            processors: Vec::new(),
        }
    }

    #[tracing::instrument(name = "Handling event", skip(self))]
    async fn handle_event(&self, event: Uuid) {
        for processor in &self.processors {
            processor.handle(&event).await.unwrap_or_else(|failure| {
                warn!("{}", failure);
            });
        }
    }

    async fn execute_task(&self) -> ExecutionOutcome {
        let event;
        {
            event = self.event_bus.events.lock().unwrap().pop_front();
        }
        match event {
            None => ExecutionOutcome::EmptyQueue,
            Some(event) => {
                self.handle_event(event).await;
                ExecutionOutcome::TaskCompleted
            }
        }
    }
}

#[async_trait]
impl EventListener for DirectEventListener {
    fn register(&mut self, processor: impl EventProcessor + 'static) {
        self.processors.push(Arc::new(processor))
    }

    #[tracing::instrument(name = "Listening to events", skip(self))]
    async fn listen(self) -> Result<(), Error> {
        loop {
            match self.execute_task().await {
                ExecutionOutcome::EmptyQueue => {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
                ExecutionOutcome::TaskCompleted => {}
            }
        }
    }
}
