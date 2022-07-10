use rand::prelude::*;
use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use std::{
    sync::{Arc, Mutex},
    thread::{self, spawn},
};
use uuid::Uuid;

const BATHROOM_SIZE: usize = 2;
const MAX_USE_TIME_THRESHOLD: Duration = Duration::new(5, 0);
const TIME_SCALE: u8 = 100;
const RX_POLLING_WAIT: Duration = Duration::new(0, 5000);

#[derive(Copy, Clone, Debug, PartialEq)]
enum Gender {
    Male,
    Female,
}

#[derive(Debug, Clone)]
struct Person {
    id: Uuid,
    gender: Gender,
    joined_queue_at: Option<Instant>,
    entered_bathroom_at: Option<Instant>,
}

fn new_person(g: Gender) -> Person {
    const NO_INSTANT: Option<Instant> = None;
    return Person {
        id: Uuid::new_v4(),
        gender: g,
        joined_queue_at: NO_INSTANT,
        entered_bathroom_at: NO_INSTANT,
    };
}

#[derive(Debug)]
struct Bathroom {
    id: Uuid,
    cabins: [Option<Person>; BATHROOM_SIZE],
    allowed_gender: Gender,
    use_count: u32,
    first_user_entered_at: Option<Instant>,
    male_queue: Vec<Person>,
    female_queue: Vec<Person>,
}

impl Bathroom {
    fn enqueue(&mut self, mut person_to_enqueue: Person) {
        person_to_enqueue.joined_queue_at = Some(Instant::now());

        match person_to_enqueue.gender {
            Gender::Male => self.male_queue.push(person_to_enqueue),
            Gender::Female => self.female_queue.push(person_to_enqueue),
        }
    }

    fn allocate_cabin(&mut self, mut person: Person) -> bool {
        if person.gender != self.allowed_gender
            || self.use_count == BATHROOM_SIZE.try_into().unwrap()
            || self
                .first_user_entered_at
                .unwrap_or(Instant::now())
                .elapsed()
                >= MAX_USE_TIME_THRESHOLD
        {
            println!("dropped person {} of gender {:?}", person.id, person.gender);
            return false;
        }

        let queue = match person.gender {
            Gender::Male => &mut self.male_queue,
            Gender::Female => &mut self.female_queue,
        };

        // Assures person is in queue, if not, we messed up
        queue.iter().find(|p| p.id == person.id).unwrap();

        println!("use_count: {}", self.use_count);
        let first_free_cabin_idx = self
            .cabins
            .iter()
            .position(|cabin_occupation| cabin_occupation.is_none());

        return match first_free_cabin_idx {
            Some(idx) => {
                if self.use_count == 0 {
                    self.first_user_entered_at = Some(Instant::now());
                }

                self.use_count += 1;

                person.entered_bathroom_at = Some(Instant::now());

                queue.retain(|person_in_queue| person_in_queue.id != person.id);
                self.cabins[idx] = Some(person);
                true
            }
            None => false,
        };
    }
}

fn new_bathroom(g: Gender) -> Bathroom {
    const NO_PERSON: Option<Person> = None;
    const NO_INSTANT: Option<Instant> = None;

    return Bathroom {
        id: Uuid::new_v4(),
        cabins: [NO_PERSON; BATHROOM_SIZE],
        allowed_gender: g,
        use_count: 0,
        first_user_entered_at: NO_INSTANT,
        male_queue: vec![],
        female_queue: vec![],
    };
}

#[derive(Debug)]
struct Event {
    name: String,
    producer_id: Uuid,
    destination_id: Uuid,
    person_data: Option<Person>,
}

fn new_event(
    name: String,
    producer_id: Uuid,
    destination_id: Uuid,
    person: Option<Person>,
) -> Event {
    return Event {
        name,
        producer_id,
        destination_id,
        person_data: person,
    };
}

struct Router {
    outbox: HashMap<Uuid, Sender<Event>>,
    listeners: HashMap<String, Vec<Sender<Event>>>,
    rx: Receiver<Event>,
    tx: Sender<Event>,
}

fn new_router() -> Router {
    let (tx, rx): (Sender<Event>, Receiver<Event>) = mpsc::channel();
    return Router {
        outbox: HashMap::new(),
        listeners: HashMap::new(),
        rx,
        tx,
    };
}

impl Router {
    fn register(&mut self, id: Uuid, tx: Sender<Event>) {
        self.outbox.insert(id, tx);
    }
}

fn spawn_person_thread(r: &mut Router) -> JoinHandle<()> {
    let (tx_person, rx_person): (Sender<Event>, Receiver<Event>) = mpsc::channel();
    let mut person = new_person(Gender::Male);
    r.register(person.id, mpsc::Sender::clone(&tx_person));

    let person_t = thread::spawn(move || {
        loop {
            match &rx_person.try_recv() {
                Ok(ev) => {
                    println!("person received message: {:?}", ev);
                    match ev.name.as_str() {
                        "person_joined_the_queue" => person.joined_queue_at = ev.person_data.as_ref().unwrap().joined_queue_at,
                        "person_entered_the_bathroom" => person.entered_bathroom_at = ev.person_data.as_ref().unwrap().entered_bathroom_at,
                        "person_left_the_bathroom" => break,
                        &_ => todo!(),
                    }
                }
                Err(_) => (),
            };
            thread::sleep(RX_POLLING_WAIT);
        }
    });

    return person_t;
}

fn main() {
    // sobe thread que tem Bathroom
    // sobe n threads que tem Person
    // comunica threads de person com Bathroom
    // Bathroom <-Router-> Person
    // Coletor de métricas
    let (tx_person, rx_person): (Sender<Event>, Receiver<Event>) = mpsc::channel();
    let mut r = new_router();
    let p = new_person(Gender::Male);
    r.register(p.id, tx_person);
    let person_router = mpsc::Sender::clone(&r.tx);

    let router_t = thread::spawn(move || loop {
        match r.rx.try_recv() {
            Ok(ev) => {
                let rx = r.outbox.get(&ev.destination_id).unwrap();
                rx.send(ev).unwrap();
                0
            }
            Err(_) => 1,
        };
        thread::sleep(RX_POLLING_WAIT);
    });

    let person_t = thread::spawn(move || loop {
        loop {
            match &rx_person.try_recv() {
                Ok(ev) => {
                    println!("person received message: {:?}", ev);
                }
                Err(_) => (),
            };
            thread::sleep(RX_POLLING_WAIT);
        }
    });

    let mut b = new_bathroom(Gender::Male);
    let (tx_bathroom, rx_bathroom): (Sender<Event>, Receiver<Event>) = mpsc::channel();

    let bathroom_t = thread::spawn(move || loop {
        match &rx_bathroom.try_recv() {
            Ok(ev) => match ev.name.as_str() {
                "new_person" => b.enqueue(ev.person_data.as_ref().unwrap().clone()),
                &_ => todo!(),
            },
            Err(_) => (),
        };
        thread::sleep(RX_POLLING_WAIT);
    });

    let e = new_event("hi".to_string(), p.id, p.id, Some(p.clone()));
    person_router.send(e).unwrap();
    router_t.join().unwrap();
    person_t.join().unwrap();
}
