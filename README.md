Gender-Switching Bathroom Simulation
This is a simulation of a gender-switching bathroom, written in Rust to handle concurrency problems. The simulation is based on the scenario where there is a single working bathroom in a very busy day in a hypothetic university. As people arrive to use it, they're told they have to wait until the bathroom is free for use based on their gender. Every now and then this gender changes, and different people may use said bathroom as it does.

The simulation generates people and assigns them to their queues. As certain thresholds are met (e.g., usage time by a single gender or the amount of a single gender that used the bathroom), the allowed gender of the bathroom changes, and people on the queue may use it. All of these events occur concurrently and are managed by a router that passes messages around to make things happen.

The router receives all messages and forwards them to the interested parties, which can register themselves in their topics of interest. For instance, to know how much time a single person has waited on queue, a `PERSON_ENTERED_THE_BATHROOM` event message is sent. Aside from the event name, several pieces of data are sent with it, allowing the metrics collector to compute the time it took for that person to be able to use the bathroom, from the time they joined the queue to the time they entered the bathroom. To receive these event messages, the metrics collector must register itself with the router as an interested destination for `PERSON_ENTERED_THE_BATHROOM` events.

The simulation is parameterized, and its parameters are constants defined in the `src/simulation.rs` file. The following are the key parameters:

- `TIME_SCALE`: How fast time will be simulated (wait times / statistical times will be divided by this constant);
- `RX_POLLING_WAIT`: Wait time for entities to check their "inbox" (polling interval);
- `MIN_PERSON_BATHROOM_SECONDS` / `MAX_PERSON_BATHROOM_SECONDS`: MIN/MAX time in seconds that a person will stay in the bathroom, each person stays in the bathroom for a random amount of time between these limits;
- `PERSON_GENERATION_INTERVAL`: How often new people may be generated;
- `PERSON_GENERATION_RATE`: The rate at which new people are generated after each `PERSON_GENERATION_INTERVAL`;
- `BATHROOM_SIZE`: How many booths the bathroom has;
- `MAX_USE_TIME_THRESHOLD`: Time the bathroom may be occupied by a single gender before switching;

When the simulation stops (which is itself an event), all threads are gracefully shut down. At this point, the metrics collector computes several metrics, such as average, ordered values, percentiles, etc., and writes them to a JSON file under `statistics_reports/`. For more details about which measures and metrics are taken and computed, see `src/simulation/metrics_collector.rs`.

How to Run
To run the simulation, simply run the following command:

```shell
cargo run
```

To stop the simulation gracefully, press Ctrl-c.
