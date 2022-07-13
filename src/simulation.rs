pub mod bathroom;
pub mod event;
pub mod person;
pub mod router;

use rand::prelude::*;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use self::event::Event;
use self::person::Gender;

const ENABLE_LOGGING: bool = false;
pub const TIME_SCALE: f64 = 80.0;
pub const RX_POLLING_WAIT: Duration = Duration::from_micros(500);

pub const MIN_PERSON_BATHROOM_SECONDS: u64 = 20;
pub const MAX_PERSON_BATHROOM_SECONDS: u64 = MIN_PERSON_BATHROOM_SECONDS * 20;
//
// Person constants
pub const PERSON_GENERATION_INTERVAL: Duration = Duration::from_secs(10);
pub const PERSON_GENERATION_RATE: f64 = 0.3;

// Bathroom constants
pub const BATHROOM_SIZE: usize = 12;
pub const MAX_USE_TIME_THRESHOLD: Duration = Duration::from_secs(MAX_PERSON_BATHROOM_SECONDS * 3);

pub fn timestamp() -> chrono::format::DelayedFormat<chrono::format::StrftimeItems<'static>> {
    return chrono::offset::Local::now().format("%Y-%m-%d %H:%M:%S.%3f");
}

pub fn log(msg: String) {
    if ENABLE_LOGGING {
        println!("[{}] {}", timestamp(), msg);
    }
}

pub fn wait(d: Duration) {
    thread::sleep(d.div_f64(TIME_SCALE));
}

pub fn spawn_person_thread(router_tx: Sender<Event>, gender: Gender) -> JoinHandle<()> {
    let (tx_person, rx_person): (Sender<Event>, Receiver<Event>) = mpsc::channel();
    let mut person = person::new_person(gender);
    log(format!(
        "Person {} of gender {} spawned!",
        person.id, person.gender
    ));
    router_tx
        .send(event::new_creation_event(
            event::EV_NEW_PERSON.to_string(),
            person.id,
            None,
            tx_person.clone(),
            Some(person.clone()),
        ))
        .unwrap();

    let person_t = thread::spawn(move || loop {
        let mut rand = rand::thread_rng();

        match &rx_person.try_recv() {
            Ok(msg) => match msg.name.as_str() {
                event::EV_PERSON_JOINED_THE_QUEUE => {
                    person.joined_queue_at = msg.person_data.as_ref().unwrap().joined_queue_at
                }
                event::EV_PERSON_ENTERED_THE_BATHROOM => {
                    person.entered_bathroom_at =
                        msg.person_data.as_ref().unwrap().entered_bathroom_at;
                    wait(Duration::new(
                        rand.gen_range(MIN_PERSON_BATHROOM_SECONDS..MAX_PERSON_BATHROOM_SECONDS),
                        0,
                    ));
                    let _ = router_tx
                        .send(event::new_event(
                            event::EV_PERSON_FINISHED_USING_BATHROOM.to_string(),
                            person.id,
                            None,
                            Some(person.clone()),
                        ))
                        .unwrap();
                }
                event::EV_PERSON_LEFT_THE_BATHROOM => {
                    person.left_bathroom_at = msg.person_data.as_ref().unwrap().left_bathroom_at;
                    break;
                }
                &_ => todo!(),
            },
            Err(_) => wait(RX_POLLING_WAIT),
        };
    });

    return person_t;
}

pub fn spawn_bathroom_thread(router_tx: Sender<Event>) {
    let _ = thread::spawn(move || {
        log("Bathroom spawned!".to_string());
        let mut bathroom = bathroom::new_bathroom(Gender::Female);
        let (tx_bathroom, rx_bathroom): (Sender<Event>, Receiver<Event>) = mpsc::channel();

        router_tx
            .send(event::new_creation_event(
                event::EV_NEW_BATHROOM.to_string(),
                bathroom.id,
                None,
                tx_bathroom.clone(),
                None,
            ))
            .unwrap();

        loop {
            match bathroom.allocate_cabin(bathroom.allowed_gender) {
                Some(person) => {
                    log(format!("Person {} entered the bathroom", person.id));
                    let _ = router_tx
                        .send(event::new_event(
                            event::EV_PERSON_ENTERED_THE_BATHROOM.to_string(),
                            bathroom.id,
                            Some(person.id),
                            Some(person),
                        ))
                        .unwrap();
                }
                None => (),
            }

            match &rx_bathroom.try_recv() {
                Ok(msg) => match msg.name.as_str() {
                    event::EV_NEW_PERSON => {
                        let person_data = msg.person_data.as_ref().unwrap().clone();
                        bathroom.enqueue(person_data.clone());
                        log(format!(
                            "Person {} joined the {} queue",
                            person_data.id, person_data.gender
                        ));
                        let _ = router_tx.send(event::new_event(
                            event::EV_PERSON_JOINED_THE_QUEUE.to_string(),
                            bathroom.id,
                            Some(msg.producer_id),
                            Some(msg.person_data.as_ref().unwrap().clone()),
                        ));
                    }
                    event::EV_PERSON_FINISHED_USING_BATHROOM => {
                        let person_data = msg.person_data.to_owned().unwrap();
                        log(format!(
                            "Person {} left the {} bathroom",
                            person_data.id, person_data.gender
                        ));
                        bathroom.free_cabin(person_data.id);
                        let _ = router_tx.send(event::new_event(
                            event::EV_PERSON_LEFT_THE_BATHROOM.to_string(),
                            bathroom.id,
                            Some(msg.producer_id),
                            Some(msg.person_data.as_ref().unwrap().clone()),
                        ));
                    }
                    &_ => todo!(),
                },
                Err(_) => wait(RX_POLLING_WAIT),
            };
        }
    });
}

pub fn spawn_router_thread(mut router: router::Router) -> JoinHandle<()> {
    thread::spawn(move || {
        log("Router spawned!".to_string());

        let bathroom_interesting_events = vec![
            event::EV_NEW_PERSON,
            event::EV_PERSON_FINISHED_USING_BATHROOM,
        ];

        loop {
            match router.rx.try_recv() {
                Ok(ref msg) => {
                    match msg.name.as_str() {
                        event::EV_NEW_BATHROOM => {
                            log(format!(
                                "Registering bathroom {} in the router",
                                msg.producer_id
                            ));
                            bathroom_interesting_events.iter().for_each(|event| {
                                let _ = &router.listeners.insert(
                                    event.to_string(),
                                    vec![msg.producer_sender.as_ref().unwrap().clone()],
                                );
                            })
                        }
                        event::EV_NEW_PERSON => {
                            log(format!(
                                "Registering person {} in the router",
                                msg.producer_id
                            ));
                            let _ = router.outbox.insert(
                                msg.producer_id,
                                msg.producer_sender.as_ref().unwrap().clone(),
                            );
                        }
                        &_ => (),
                    }

                    if msg.destination_id.is_some() {
                        let rx = router.outbox.get(&msg.destination_id.unwrap()).unwrap();
                        rx.send(msg.clone()).unwrap();
                    }

                    match router.listeners.get(&msg.name) {
                        Some(interested_parties) => interested_parties
                            .iter()
                            .for_each(|tx| tx.send(msg.clone()).unwrap()),
                        None => (),
                    }
                }
                Err(_) => wait(RX_POLLING_WAIT),
            };
        }
    })
}

pub fn randomly_generate_person_threads(router_tx: Sender<Event>) {
    let mut rand = rand::thread_rng();
    loop {
        if rand.gen_bool(PERSON_GENERATION_RATE) {
            let g = rand.gen::<Gender>();
            let _person_t = spawn_person_thread(router_tx.clone(), g);
        }
        wait(PERSON_GENERATION_INTERVAL);
    }
}
