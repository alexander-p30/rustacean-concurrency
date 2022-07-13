use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, Sender};
use uuid::Uuid;

pub struct Router {
    pub outbox: HashMap<Uuid, Sender<super::event::Event>>,
    pub listeners: HashMap<String, Vec<Sender<super::event::Event>>>,
    pub rx: Receiver<super::event::Event>,
    pub tx: Sender<super::event::Event>,
}

pub fn new_router() -> Router {
    let (tx, rx): (Sender<super::event::Event>, Receiver<super::event::Event>) = mpsc::channel();
    return Router {
        outbox: HashMap::new(),
        listeners: HashMap::new(),
        rx,
        tx,
    };
}
