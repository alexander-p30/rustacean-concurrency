use core::fmt;
use rand::distributions::Standard;
use rand::prelude::*;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use uuid::Uuid;

const BATHROOM_SIZE: usize = 12;
const MAX_USE_TIME_THRESHOLD: Duration = Duration::from_secs(5);
const TIME_SCALE: f64 = 80.0;
const RX_POLLING_WAIT: Duration = Duration::from_micros(500);
const PERSON_GENERATION_INTERVAL: Duration = Duration::from_secs(10);
const ENABLE_LOGGING: bool = false;
const MIN_PERSON_BATHROOM_SECONDS: u64 = 20;
const MAX_PERSON_BATHROOM_SECONDS: u64 = MIN_PERSON_BATHROOM_SECONDS * 20;
const PERSON_GENERATION_RATE: f64 = 0.3;

// Person events
const EV_NEW_PERSON: &str = "new_person";
const EV_PERSON_JOINED_THE_QUEUE: &str = "person_joined_the_queue";
const EV_PERSON_ENTERED_THE_BATHROOM: &str = "person_entered_the_bathroom";
const EV_PERSON_FINISHED_USING_BATHROOM: &str = "person_finished_using_bathroom";
const EV_PERSON_LEFT_THE_BATHROOM: &str = "person_left_the_bathroom";
// Bathroom events
const EV_NEW_BATHROOM: &str = "new_bathroom";

fn blue(msg: String) -> String {
    return format!("\u{001b}[34m{}\u{001b}[0m", msg);
}

fn magenta(msg: String) -> String {
    return format!("\u{001b}[31m{}\u{001b}[0m", msg);
}

fn timestamp() -> chrono::format::DelayedFormat<chrono::format::StrftimeItems<'static>> {
    return chrono::offset::Local::now().format("%Y-%m-%d %H:%M:%S.%3f");
}

fn log(msg: String) {
    if ENABLE_LOGGING {
        println!("[{}] {}", timestamp(), msg);
    }
}
fn wait(d: Duration) {
    thread::sleep(d.div_f64(TIME_SCALE));
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Gender {
    Male,
    Female,
}

impl Distribution<Gender> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Gender {
        if rng.gen_bool(0.5) {
            Gender::Female
        } else {
            Gender::Male
        }
    }
}

#[derive(Debug, Clone)]
struct Person {
    id: Uuid,
    gender: Gender,
    joined_queue_at: Option<Instant>,
    entered_bathroom_at: Option<Instant>,
    left_bathroom_at: Option<Instant>,
}

fn new_person(g: Gender) -> Person {
    const NO_INSTANT: Option<Instant> = None;
    return Person {
        id: Uuid::new_v4(),
        gender: g,
        joined_queue_at: NO_INSTANT,
        entered_bathroom_at: NO_INSTANT,
        left_bathroom_at: NO_INSTANT,
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

impl fmt::Display for Bathroom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Customize so only `x` and `y` are denoted.
        let cabins_str = self.cabins.iter().fold("[".to_string(), |acc, cabin| {
            if cabin.is_some() {
                match cabin.as_ref().unwrap().gender {
                    Gender::Male => {
                        acc + &" |".to_string() + &blue("o̲".to_string()) + &"|".to_string()
                    }
                    Gender::Female => {
                        acc + &" |".to_string() + &magenta("o̲".to_string()) + &"|".to_string()
                    }
                }
            } else {
                acc + &" |/|".to_string()
            }
        }) + " ]";

        let occupied_cabins_count = self.cabins.iter().filter(|cabin| cabin.is_some()).count();

        let female_queue_str = self
            .female_queue
            .iter()
            .fold("".to_string(), |acc, _| acc + &magenta(" o".to_string()));

        let male_queue_str = self
            .male_queue
            .iter()
            .fold("".to_string(), |acc, _| acc + &blue(" o".to_string()));

        let gender = match self.allowed_gender {
            Gender::Male => blue("Male".to_string()),
            Gender::Female => magenta("Female".to_string()),
        };

        write!(
            f,
            "Bathroom {{\n\toccupation: \t\t [used_cabins: {}, time_since_first_user: {:?}]\n\tgender: \t\t{}\n\t[{occupied_cabins_count:0>2}/{}] cabins: \t{}\n\t[{}] female_queue: \t{}\n\t[{}] male_queue: \t{}\n}}",
            self.use_count, self.first_user_entered_at.unwrap_or(Instant::now()).elapsed().mul_f64(TIME_SCALE), gender, self.cabins.len(), cabins_str, self.female_queue.len(), female_queue_str, self.male_queue.len(), male_queue_str
        )
    }
}

impl Bathroom {
    fn log(&self, msg: String) {
        println!("[{}] {}", timestamp(), msg);
    }

    fn display(&self) {
        println!("[{}] {}", timestamp(), self);
    }

    fn enqueue(&mut self, mut person_to_enqueue: Person) {
        person_to_enqueue.joined_queue_at = Some(Instant::now());

        match person_to_enqueue.gender {
            Gender::Male => self.male_queue.push(person_to_enqueue),
            Gender::Female => self.female_queue.push(person_to_enqueue),
        }

        self.display();

        if self.should_switch_genders() {
            self.switch_genders();
            self.display();
        }
    }

    fn allocate_cabin(&mut self, gender: Gender) -> Option<Person> {
        let first_in_queue = match gender {
            Gender::Male => self.male_queue.first(),
            Gender::Female => self.female_queue.first(),
        };

        if first_in_queue.is_none() {
            return None;
        }

        let mut person = first_in_queue.unwrap().to_owned();

        if person.gender != self.allowed_gender
            || self.use_count == BATHROOM_SIZE.try_into().unwrap()
            || self
                .first_user_entered_at
                .unwrap_or(Instant::now())
                .elapsed()
                >= MAX_USE_TIME_THRESHOLD
        {
            return None;
        }

        let queue = match person.gender {
            Gender::Male => &mut self.male_queue,
            Gender::Female => &mut self.female_queue,
        };

        // Assures person is in queue, if not, we messed up
        queue.iter().find(|p| p.id == person.id).unwrap();

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
                self.cabins[idx] = Some(person.clone());
                self.display();
                Some(person)
            }
            None => None,
        };
    }

    fn free_cabin(&mut self, person_id: Uuid) {
        let cabin_idx = self
            .cabins
            .iter()
            .position(|cabin| match cabin {
                Some(person) => person.id == person_id,
                None => false,
            })
            .unwrap();
        self.cabins[cabin_idx] = None;

        self.display();

        if self.should_switch_genders() {
            self.switch_genders();
            self.display();
        }
    }

    fn should_switch_genders(&mut self) -> bool {
        let other_gender_queue = match self.allowed_gender {
            Gender::Male => &self.female_queue,
            Gender::Female => &self.male_queue,
        };

        let other_gender_queue_empty = other_gender_queue.is_empty();

        if other_gender_queue_empty {
            self.log("Other gender's queue is empty, resetting usage statistics".to_string());
            self.use_count = 0;
            self.first_user_entered_at = None;
            self.display();
        }

        let all_cabins_empty = self.cabins.iter().all(|cabin| cabin.is_none());

        let current_gender_queue = match self.allowed_gender {
            Gender::Male => &self.male_queue,
            Gender::Female => &self.female_queue,
        };

        let current_gender_queue_empty = current_gender_queue.is_empty();

        return (all_cabins_empty && !other_gender_queue_empty)
            || (all_cabins_empty && current_gender_queue_empty);
    }

    fn switch_genders(&mut self) {
        match self.allowed_gender {
            Gender::Male => self.allowed_gender = Gender::Female,
            Gender::Female => self.allowed_gender = Gender::Male,
        };

        self.use_count = 0;
        self.first_user_entered_at = None;

        self.display();
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

#[derive(Debug, Clone)]
struct Event {
    name: String,
    producer_id: Uuid,
    destination_id: Option<Uuid>,
    producer_sender: Option<Sender<Event>>,
    person_data: Option<Person>,
}

fn new_event(
    name: String,
    producer_id: Uuid,
    destination_id: Option<Uuid>,
    person: Option<Person>,
) -> Event {
    return Event {
        name,
        producer_id,
        destination_id,
        producer_sender: None,
        person_data: person,
    };
}

fn new_creation_event(
    name: String,
    producer_id: Uuid,
    destination_id: Option<Uuid>,
    producer_sender: Sender<Event>,
    person: Option<Person>,
) -> Event {
    return Event {
        name,
        producer_id,
        destination_id,
        producer_sender: Some(producer_sender),
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

fn spawn_person_thread(
    router_tx: Sender<Event>,
    gender: Gender,
) -> (JoinHandle<()>, Uuid, Sender<Event>) {
    let (tx_person, rx_person): (Sender<Event>, Receiver<Event>) = mpsc::channel();
    let mut person = new_person(gender);
    log(format!(
        "Person {} of gender {:?} spawned!",
        person.id, person.gender
    ));
    router_tx
        .send(new_creation_event(
            EV_NEW_PERSON.to_string(),
            person.id,
            None,
            tx_person.clone(),
            Some(person.clone()),
        ))
        .unwrap();

    let person_id = person.id;

    let person_t = thread::spawn(move || loop {
        let mut rand = rand::thread_rng();

        match &rx_person.try_recv() {
            Ok(msg) => match msg.name.as_str() {
                EV_PERSON_JOINED_THE_QUEUE => {
                    person.joined_queue_at = msg.person_data.as_ref().unwrap().joined_queue_at
                }
                EV_PERSON_ENTERED_THE_BATHROOM => {
                    person.entered_bathroom_at =
                        msg.person_data.as_ref().unwrap().entered_bathroom_at;
                    wait(Duration::new(
                        rand.gen_range(MIN_PERSON_BATHROOM_SECONDS..MAX_PERSON_BATHROOM_SECONDS),
                        0,
                    ));
                    let _ = router_tx
                        .send(new_event(
                            EV_PERSON_FINISHED_USING_BATHROOM.to_string(),
                            person.id,
                            None,
                            Some(person.clone()),
                        ))
                        .unwrap();
                }
                EV_PERSON_LEFT_THE_BATHROOM => {
                    person.left_bathroom_at = msg.person_data.as_ref().unwrap().left_bathroom_at;
                    break;
                }
                &_ => todo!(),
            },
            Err(_) => wait(RX_POLLING_WAIT),
        };
    });

    return (person_t, person_id, tx_person);
}

fn main() {
    // sobe thread que tem Bathroom
    // sobe n threads que tem Person
    // comunica threads de person com Bathroom
    // Bathroom <-Router-> Person
    // Coletor de métricas
    let mut router = new_router();
    let router_tx = router.tx.clone();

    let _router_t = thread::spawn(move || {
        log("Router spawned!".to_string());

        let bathroom_interesting_events = vec![EV_NEW_PERSON, EV_PERSON_FINISHED_USING_BATHROOM];

        loop {
            match router.rx.try_recv() {
                Ok(ref msg) => {
                    match msg.name.as_str() {
                        EV_NEW_BATHROOM => {
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
                        EV_NEW_PERSON => {
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
    });

    let bathroom_router = router_tx.clone();

    let _bathroom_t = thread::spawn(move || {
        log("Bathroom spawned!".to_string());
        let mut bathroom = new_bathroom(Gender::Male);
        let (tx_bathroom, rx_bathroom): (Sender<Event>, Receiver<Event>) = mpsc::channel();

        bathroom_router
            .send(new_creation_event(
                EV_NEW_BATHROOM.to_string(),
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
                    let _ = bathroom_router
                        .send(new_event(
                            EV_PERSON_ENTERED_THE_BATHROOM.to_string(),
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
                    EV_NEW_PERSON => {
                        let person_data = msg.person_data.as_ref().unwrap().clone();
                        bathroom.enqueue(person_data.clone());
                        log(format!(
                            "Person {} joined the {:?} queue",
                            person_data.id, person_data.gender
                        ));
                        let _ = bathroom_router.send(new_event(
                            EV_PERSON_JOINED_THE_QUEUE.to_string(),
                            bathroom.id,
                            Some(msg.producer_id),
                            Some(msg.person_data.as_ref().unwrap().clone()),
                        ));
                    }
                    EV_PERSON_FINISHED_USING_BATHROOM => {
                        let person_data = msg.person_data.to_owned().unwrap();
                        log(format!(
                            "Person {} left the {:?} bathroom",
                            person_data.id, person_data.gender
                        ));
                        bathroom.free_cabin(person_data.id);
                        let _ = bathroom_router.send(new_event(
                            EV_PERSON_LEFT_THE_BATHROOM.to_string(),
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

    // thread::sleep(Duration::new(1, 0));

    let mut rand = rand::thread_rng();
    loop {
        if rand.gen_bool(PERSON_GENERATION_RATE) {
            let g = rand.gen::<Gender>();
            let (_person_t, _person_id, _person_tx) = spawn_person_thread(router_tx.clone(), g);
        }
        wait(PERSON_GENERATION_INTERVAL);
    }
}
