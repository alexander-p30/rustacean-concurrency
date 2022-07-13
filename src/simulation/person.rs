use core::fmt;
use rand::distributions::Standard;
use rand::prelude::*;
use std::time::Instant;
use uuid::Uuid;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Gender {
    Male,
    Female,
}

impl fmt::Display for Gender {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Gender::Male => write!(f, "Male"),
            Gender::Female => write!(f, "Female"),
        }
    }
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
pub struct Person {
    pub id: Uuid,
    pub gender: Gender,
    pub joined_queue_at: Option<Instant>,
    pub entered_bathroom_at: Option<Instant>,
    pub left_bathroom_at: Option<Instant>,
}

pub fn new_person(g: Gender) -> Person {
    const NO_INSTANT: Option<Instant> = None;
    return Person {
        id: Uuid::new_v4(),
        gender: g,
        joined_queue_at: NO_INSTANT,
        entered_bathroom_at: NO_INSTANT,
        left_bathroom_at: NO_INSTANT,
    };
}
