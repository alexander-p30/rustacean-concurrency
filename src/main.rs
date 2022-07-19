use std::sync::mpsc::{self, Receiver, Sender};

mod simulation;
mod utils;

fn main() {
    let (main_tx, main_rx): (
        Sender<simulation::event::Event>,
        Receiver<simulation::event::Event>,
    ) = mpsc::channel();

    let mut router = simulation::router::new_router();
    let router_tx = router.tx.clone();

    router.listeners.insert(
        simulation::event::EV_SIMULATION_FINISHED.to_string(),
        vec![main_tx.clone()],
    );

    let (metrics_collector_tx, metrics_collector_rx): (
        Sender<simulation::event::Event>,
        Receiver<simulation::event::Event>,
    ) = mpsc::channel();

    simulation::event::ALL_EVENTS.iter().for_each(|event| {
        let _ = router
            .listeners
            .insert(event.to_string(), vec![metrics_collector_tx.clone()]);
    });

    simulation::spawn_router_thread(router);
    simulation::spawn_metrics_collector_thread(router_tx.clone(), metrics_collector_rx);
    simulation::spawn_bathroom_thread(router_tx.clone());
    simulation::randomly_generate_person_threads(router_tx.clone(), main_rx);
}
