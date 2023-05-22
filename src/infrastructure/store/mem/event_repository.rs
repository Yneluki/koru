use crate::application::store::{EventRepository, EventRepositoryError};
use crate::domain::Event;
use crate::infrastructure::store::mem::mem_store::{InMemTx, InMemoryStore, InnerEvent};
use crate::utils::date;
use anyhow::anyhow;
use async_trait::async_trait;
use std::cell::RefCell;
use std::sync::atomic::Ordering::Relaxed;
use uuid::Uuid;

#[async_trait]
impl EventRepository for InMemoryStore {
    type Tr = InMemTx;

    async fn save(
        &self,
        tx: &mut RefCell<Self::Tr>,
        events: &[Event],
    ) -> Result<(), EventRepositoryError> {
        if self.crash_events.load(Relaxed) {
            return Err(EventRepositoryError::CorruptedData("Crashed store"));
        }
        let mut evts = tx.get_mut().events.lock().unwrap();
        for evt in events {
            evts.push(InnerEvent::from(evt.clone()))
        }
        Ok(())
    }

    async fn find(&self, id: &Uuid) -> Result<Option<Event>, EventRepositoryError> {
        if self.crash_events.load(Relaxed) {
            return Err(EventRepositoryError::CorruptedData("Crashed store"));
        }
        self.events
            .lock()
            .unwrap()
            .iter()
            .find(|e| e.id == *id)
            .map(|e| e.clone().event())
            .map_or(Ok(None), |e| e.map(Some))
            .map_err(EventRepositoryError::CorruptedData)
    }

    async fn mark_processed(&self, id: &Uuid) -> Result<(), EventRepositoryError> {
        if self.crash_events.load(Relaxed) {
            return Err(EventRepositoryError::CorruptedData("Crashed store"));
        }
        let mut evts = self.events.lock().unwrap();
        let event = evts.iter_mut().find(|e| e.id == *id);
        match event {
            None => Err(EventRepositoryError::Fetch(anyhow!("Event not found."))),
            Some(event) => {
                event.processed_date = Some(date::now());
                Ok(())
            }
        }
    }
}
