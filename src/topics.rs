use crate::constants::SERVER_TYPE_NAME;

pub enum TopicId {
    Status,
    Error,
    Tx,
    Rx,
}

/// Topics used for MQTT communication with the power supply
#[derive(Debug, Clone)]
pub struct Topics {
    // ---
    /// Topic for status updates
    pub status: String,
    /// Topic for error messages
    /// pza_id match the one from the command that caused the error
    pub error: String,
    // ---
    /// Topic to send data to equipment
    pub tx: String,
    /// Topic to receive data from equipment
    pub rx: String,
    // ---
}

impl Topics {
    /// Create a new Topics instance with the given prefix
    pub fn new<A: AsRef<str>>(name: A) -> Self {
        let prefix = format!("{}/{}", SERVER_TYPE_NAME, name.as_ref());
        Self {
            status: format!("{}/status", prefix),
            error: format!("{}/error", prefix),
            tx: format!("{}/state/cmd", prefix),
            rx: format!("{}/state", prefix),
        }
    }

    /// Get a vector of all client subscription topics
    pub fn vec_sub_client(&self) -> Vec<String> {
        vec![self.status.clone(), self.error.clone(), self.rx.clone()]
    }

    /// Get a vector of all server subscription topics
    pub fn vec_sub_server(&self) -> Vec<String> {
        vec![self.tx.clone()]
    }

    pub fn topic_to_id(&self, topic: &str) -> Option<TopicId> {
        if topic == self.status {
            Some(TopicId::Status)
        } else if topic == self.error {
            Some(TopicId::Error)
        } else if topic == self.tx {
            Some(TopicId::Tx)
        } else if topic == self.rx {
            Some(TopicId::Rx)
        } else {
            None
        }
    }

    pub fn id_to_topic(&self, id: &TopicId) -> &str {
        match id {
            TopicId::Status => &self.status,
            TopicId::Error => &self.error,
            TopicId::Tx => &self.tx,
            TopicId::Rx => &self.rx,
        }
    }
}
