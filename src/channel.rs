use crate::error::Error;

type Callback = Box<dyn FnMut(&str)>;
pub struct CallBackListener {
    pub callback: Callback,
    pub event: String
}

impl CallBackListener {
    pub fn new(callback: Callback, event: impl Into<String>) -> Result<Self, Error> {
        Ok(CallBackListener {
            callback,
            event: event.into()
        })
    }
}

pub struct Channel {
  pub listeners: Vec<CallBackListener>,
  pub topic: String
}

impl Channel {
    pub fn new(topic: impl Into<String>) -> Result<Self, Error> {
        Ok(Channel {
            listeners: Vec::new(),
            topic: topic.into()
        })
    }

    pub fn on(&mut self, event: impl Into<String>, callback: Callback) -> Result<&mut Self, Error> {
        self.listeners.push(CallBackListener::new(callback, event.into())?);
        Ok(self)
    }
}

