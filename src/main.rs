use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use uuid::Uuid;

const BATHROOM_SIZE: usize = 2;
const MAX_USE_TIME_THRESHOLD: Duration = Duration::new(5, 0);
const TIME_SCALE: u8 = 100;

#[derive(Copy, Clone, Debug, PartialEq)]
enum Gender {
    Male,
    Female,
}

#[derive(Debug)]
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
    cabins: [Option<Arc<Mutex<Person>>>; BATHROOM_SIZE],
    allowed_gender: Gender,
    use_count: u32,
    first_user_entered_at: Option<Instant>,
    male_queue: Vec<Arc<Mutex<Person>>>,
    female_queue: Vec<Arc<Mutex<Person>>>,
}

impl Bathroom {
    fn enqueue(&mut self, person_to_enqueue: Arc<Mutex<Person>>) {
        let mut lock = person_to_enqueue.lock().unwrap();
        let gender = lock.gender;
        lock.joined_queue_at = Some(Instant::now());
        drop(lock);

        match gender {
            Gender::Male => self.male_queue.push(person_to_enqueue),
            Gender::Female => self.female_queue.push(person_to_enqueue),
        }
    }

    fn allocate_cabin(&mut self, person: Arc<Mutex<Person>>) -> bool {
        let lock = person.lock().unwrap();
        let person_gender = lock.gender;
        let person_id = lock.id;
        drop(lock);

        if person_gender != self.allowed_gender
            || self.use_count == BATHROOM_SIZE.try_into().unwrap()
            || self
                .first_user_entered_at
                .unwrap_or(Instant::now())
                .elapsed()
                >= MAX_USE_TIME_THRESHOLD
        {
            println!("dropped person {} of gender {:?}", person_id, person_gender);
            return false;
        }

        let queue = match person_gender {
            Gender::Male => &mut self.male_queue,
            Gender::Female => &mut self.female_queue,
        };

        // Assures person is in queue, if not, we messed up
        queue
            .iter()
            .find(|p| p.lock().unwrap().id == person_id)
            .unwrap();

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

                let mut lock = person.lock().unwrap();
                lock.entered_bathroom_at = Some(Instant::now());
                drop(lock);

                queue.retain(|person_in_queue| person_in_queue.lock().unwrap().id != person_id);
                self.cabins[idx] = Some(person);
                true
            }
            None => false,
        };
    }
}

fn new_bathroom(g: Gender) -> Bathroom {
    const NO_PERSON: Option<Arc<Mutex<Person>>> = None;
    const NO_INSTANT: Option<Instant> = None;

    return Bathroom {
        cabins: [NO_PERSON; BATHROOM_SIZE],
        allowed_gender: g,
        use_count: 0,
        first_user_entered_at: NO_INSTANT,
        male_queue: vec![],
        female_queue: vec![],
    };
}

fn main() {
    let p = Arc::new(Mutex::new(new_person(Gender::Male)));
    println!("{:?}", p);
    let f = Arc::new(Mutex::new(new_person(Gender::Female)));
    println!("{:?}", f);
    let mut b = new_bathroom(Gender::Male);
    b.enqueue(p.clone());
    println!("joined queue: {:?}", p);
    b.enqueue(f.clone());
    println!("queue: {:?}, cabin: {:?}", b.male_queue, b.cabins);
    b.allocate_cabin(p.clone());
    b.allocate_cabin(f.clone());
    println!("queue: {:?}, cabin: {:?}", b.male_queue, b.cabins);
    let a1 = Arc::new(Mutex::new(new_person(Gender::Male)));
    let a2 = Arc::new(Mutex::new(new_person(Gender::Male)));
    let a3 = Arc::new(Mutex::new(new_person(Gender::Male)));
    let a4 = Arc::new(Mutex::new(new_person(Gender::Male)));
    b.enqueue(a1.clone());
    b.enqueue(a2.clone());
    b.enqueue(a3.clone());
    b.enqueue(a4.clone());
    b.allocate_cabin(a1);
    b.allocate_cabin(a2);
    b.allocate_cabin(a3);
    b.allocate_cabin(a4);
    println!("queue: {:?}, cabin: {:?}", b.male_queue, b.cabins);
}
