use std::sync::mpsc::{self, Receiver, Sender};

mod simulation;
mod utils;

fn main() {
    let mut router = simulation::router::new_router();
    let router_tx = router.tx.clone();
    let metrics_collector = simulation::metrics_collector::new_metrics_collector();

    let (metrics_collector_tx, metrics_collector_rx): (
        Sender<simulation::event::Event>,
        Receiver<simulation::event::Event>,
    ) = mpsc::channel();

    let all_events = vec![
        simulation::event::EV_NEW_BATHROOM,
        simulation::event::EV_NEW_PERSON,
        simulation::event::EV_PERSON_JOINED_THE_QUEUE,
        simulation::event::EV_PERSON_ENTERED_THE_BATHROOM,
        simulation::event::EV_PERSON_FINISHED_USING_BATHROOM,
        simulation::event::EV_PERSON_LEFT_THE_BATHROOM,
        simulation::event::EV_BATHROOM_SWITCHED_GENDERS,
    ];

    all_events.iter().for_each(|event| {
        let _ = router
            .listeners
            .insert(event.to_string(), vec![metrics_collector_tx.clone()]);
    });

    simulation::spawn_router_thread(router);
    simulation::spawn_metrics_collector_thread(
        router_tx.clone(),
        metrics_collector,
        metrics_collector_rx,
    );
    simulation::spawn_bathroom_thread(router_tx.clone());
    simulation::randomly_generate_person_threads(router_tx.clone());
}
