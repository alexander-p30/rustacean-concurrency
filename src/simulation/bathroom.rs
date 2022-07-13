use core::fmt;
use std::time::Instant;
use uuid::Uuid;

#[derive(Debug)]
pub struct Bathroom {
    pub id: Uuid,
    pub cabins: [Option<super::person::Person>; super::BATHROOM_SIZE],
    pub allowed_gender: super::person::Gender,
    pub use_count: u32,
    pub first_user_entered_at: Option<Instant>,
    pub male_queue: Vec<super::person::Person>,
    pub female_queue: Vec<super::person::Person>,
}

impl fmt::Display for Bathroom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Customize so only `x` and `y` are denoted.
        let cabins_str = self.cabins.iter().fold("[".to_string(), |acc, cabin| {
            if cabin.is_some() {
                match cabin.as_ref().unwrap().gender {
                    super::person::Gender::Male => {
                        acc + &" |".to_string()
                            + &crate::utils::color::blue("👦".to_string())
                            + &"|".to_string()
                    }
                    super::person::Gender::Female => {
                        acc + &" |".to_string()
                            + &crate::utils::color::magenta("👧".to_string())
                            + &"|".to_string()
                    }
                }
            } else {
                acc + &" |🚽|".to_string()
            }
        }) + " ]";

        let occupied_cabins_count = self.cabins.iter().filter(|cabin| cabin.is_some()).count();

        let female_queue_str = self.female_queue.iter().fold("".to_string(), |acc, _| {
            acc + &crate::utils::color::magenta(" 👧".to_string())
        });

        let male_queue_str = self.male_queue.iter().fold("".to_string(), |acc, _| {
            acc + &crate::utils::color::blue(" 👦".to_string())
        });

        let gender = match self.allowed_gender {
            super::person::Gender::Male => crate::utils::color::blue("Male 🛉".to_string()),
            super::person::Gender::Female => crate::utils::color::magenta("Female 🛊".to_string()),
        };

        write!(
            f,
            "Bathroom 🚾 {{\n\toccupation: \t\t [used_cabins: {}, time_since_first_user: {:?}]\n\tgender: \t\t{}\n\t[{occupied_cabins_count:0>2}/{}] cabins: \t{}\n\t[{}] female_queue: \t{}\n\t[{}] male_queue: \t{}\n}}",
            self.use_count, self.first_user_entered_at.unwrap_or(Instant::now()).elapsed().mul_f64(super::TIME_SCALE), gender, self.cabins.len(), cabins_str, self.female_queue.len(), female_queue_str, self.male_queue.len(), male_queue_str
        )
    }
}

impl Bathroom {
    pub fn log(&self, msg: String) {
        println!("[{}] {}", super::timestamp(), msg);
    }

    pub fn display(&self) {
        println!("[{}] {}", super::timestamp(), self);
    }

    pub fn enqueue(&mut self, mut person_to_enqueue: super::person::Person) {
        person_to_enqueue.joined_queue_at = Some(Instant::now());

        match person_to_enqueue.gender {
            super::person::Gender::Male => self.male_queue.push(person_to_enqueue),
            super::person::Gender::Female => self.female_queue.push(person_to_enqueue),
        }

        self.display();

        if self.should_switch_genders() {
            self.switch_genders();
            self.display();
        }
    }

    pub fn allocate_cabin(
        &mut self,
        gender: super::person::Gender,
    ) -> Option<super::person::Person> {
        let first_in_queue = match gender {
            super::person::Gender::Male => self.male_queue.first(),
            super::person::Gender::Female => self.female_queue.first(),
        };

        if first_in_queue.is_none() {
            return None;
        }

        let mut person = first_in_queue.unwrap().to_owned();

        if person.gender != self.allowed_gender
            || self.use_count == super::BATHROOM_SIZE.try_into().unwrap()
            || self
                .first_user_entered_at
                .unwrap_or(Instant::now())
                .elapsed()
                .mul_f64(super::TIME_SCALE)
                >= super::MAX_USE_TIME_THRESHOLD
        {
            return None;
        }

        let queue = match person.gender {
            super::person::Gender::Male => &mut self.male_queue,
            super::person::Gender::Female => &mut self.female_queue,
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

    pub fn free_cabin(&mut self, person_id: Uuid) {
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

    pub fn should_switch_genders(&mut self) -> bool {
        let other_gender_queue = match self.allowed_gender {
            super::person::Gender::Male => &self.female_queue,
            super::person::Gender::Female => &self.male_queue,
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
            super::person::Gender::Male => &self.male_queue,
            super::person::Gender::Female => &self.female_queue,
        };

        let current_gender_queue_empty = current_gender_queue.is_empty();

        return (all_cabins_empty && !other_gender_queue_empty)
            || (all_cabins_empty && current_gender_queue_empty);
    }

    pub fn switch_genders(&mut self) {
        match self.allowed_gender {
            super::person::Gender::Male => self.allowed_gender = super::person::Gender::Female,
            super::person::Gender::Female => self.allowed_gender = super::person::Gender::Male,
        };

        self.use_count = 0;
        self.first_user_entered_at = None;

        self.display();
    }
}

pub fn new_bathroom(g: super::person::Gender) -> Bathroom {
    const NO_PERSON: Option<super::person::Person> = None;
    const NO_INSTANT: Option<Instant> = None;

    return Bathroom {
        id: Uuid::new_v4(),
        cabins: [NO_PERSON; super::BATHROOM_SIZE],
        allowed_gender: g,
        use_count: 0,
        first_user_entered_at: NO_INSTANT,
        male_queue: vec![],
        female_queue: vec![],
    };
}
