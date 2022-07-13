mod simulation;
mod utils;

fn main() {
    let router = simulation::router::new_router();
    let router_tx = router.tx.clone();

    simulation::spawn_router_thread(router);
    simulation::spawn_bathroom_thread(router_tx.clone());
    simulation::randomly_generate_person_threads(router_tx.clone());
}
