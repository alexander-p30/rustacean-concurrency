# What is this?

This is a simulation of a gender-switching bathroom, written in Rust, to study concurrency. The simulation is based on the scenario where there is a single working bathroom in a very busy day in a hypothetic university. As people arrive to use it, they're told they have to wait until the bathroom is free for use based on their gender. Every now and then this gender changes, and different people may use the bathroom as it does.

# How it Works

The simulation generates people and assigns them to their queues. As certain thresholds are met (e.g., usage time by a single gender or the amount of people from a single gender that used the bathroom), the allowed gender of the bathroom changes, and people on the queue may use it. All of these events occur concurrently and are managed by a router that passes messages around to make things happen.

The router receives all messages and forwards them to the interested parties, which can register themselves in their topics of interest. For instance, to know how much time a single person has waited on queue, a `PERSON_ENTERED_THE_BATHROOM` event message is sent when a person enters the bathroom. Aside from the event name, several pieces of data are sent with it, allowing for the computation of a person's queue time from the time they joined the queue to the time they entered the bathroom. To receive these event messages, one must register itself with the router as an interested destination for `PERSON_ENTERED_THE_BATHROOM` events.

In fact, there is a metrics collector that listens to a bunch of events and use them to generate a more detailed report at the end of the simulation. Altough this was not implemented, it would also be possible for this metrics collector to emit events which in turn could contain data to be used to tweak parameters during runtime, in order to optimize the bathroom usage.

The simulation is parameterized, and its parameters are constants defined in the `src/simulation.rs` file. The following are the key parameters:

- `TIME_SCALE`: How fast time will be simulated (wait times and statistical time data will be divided by this constant);
- `RX_POLLING_WAIT`: Wait time for entities to check their "inbox" (polling interval);
- `MIN_PERSON_BATHROOM_SECONDS` / `MAX_PERSON_BATHROOM_SECONDS`: MIN/MAX time in seconds that a person will stay in the bathroom, each person stays in the bathroom for a random amount of time between these limits;
- `PERSON_GENERATION_INTERVAL`: How often new people may arrive;
- `PERSON_GENERATION_RATE`: The rate at which new people actually arrive after each `PERSON_GENERATION_INTERVAL`;
- `BATHROOM_SIZE`: How many booths the bathroom has;
- `MAX_USE_TIME_THRESHOLD`: Time the bathroom may be occupied by a single gender before switching;

When the simulation stops (which is itself an event), all threads are gracefully shut down. At this point, the metrics collector computes several metrics, such as average, ordered values, percentiles, etc., and writes them to a JSON file under `statistics_reports/`. For more details about which measures and metrics are taken and computed, see `src/simulation/metrics_collector.rs`.

So, to answer the question: why do bathrooms need routers? To solve concurrency problems, of course!

# How to Run

To run the simulation, simply run the following command:

```shell
cargo run
```

To stop the simulation gracefully, press Ctrl-c.
