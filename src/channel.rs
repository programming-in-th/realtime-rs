use crate::error::Error;
use serde_json::{json, Map, Value};

type Callback = Box<dyn FnMut(&Map<String, Value>)>;
pub struct CallBackListener {
    pub callback: Callback,
    pub event: String,
}

impl CallBackListener {
    pub fn new(callback: Callback, event: impl Into<String>) -> Self {
        CallBackListener {
            callback,
            event: event.into(),
        }
    }
}

fn generate_json(topic: &str) -> String {
    let json = json!({
        "topic": topic,
        "event": "phx_join",
        "payload": {},
        "ref": null
    });
    return json.to_string();
}

pub struct Channel {
    pub listeners: Vec<CallBackListener>,
    pub topic: String,
}

impl Channel {
    pub fn new(topic: impl Into<String>) -> Self {
        Channel {
            listeners: Vec::new(),
            topic: topic.into(),
        }
    }

    pub fn on(&mut self, event: impl Into<String>, callback: Callback) -> &mut Self {
        self.listeners
            .push(CallBackListener::new(callback, event.into()));
        self
    }

    pub fn off(&mut self, event: impl Into<String>) -> Result<&mut Self, Error> {
        let event = event.into();
        let index = self
            .listeners
            .iter()
            .position(|l| l.event == event)
            .ok_or(Error::NotFoundError)?;
        self.listeners.remove(index);
        Ok(self)
    }

    pub fn get_join(&self) -> String {
        generate_json(&self.topic)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn generate_json_correctly() {
        let topic = "random";
        let json = generate_json(topic);
        let expected = "{\"event\":\"phx_join\",\"playload\":{},\"ref\":null,\"topic\":\"random\"}";
        assert_eq!(json, expected);
    }
}
