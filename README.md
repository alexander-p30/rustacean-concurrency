# What is this?

A simple simulation of a gender-switching bathroom.

Imagine this: there is a single working bathroom in a very busy day in a hypothetic university. As people arrive to use it, they're told they have to wait until the bathroom is free for use based on their gender. Every now and then this gender changes, and different people may use said bathroom as it does.

This is the situation simulated in this project: people being generated and assigned to their queues, as certain thresholds are met (usage time by a single gender or ammount of a single gender that used the bathroom, for instance), the allowed gender of the bathroom changes, and people on the queue may use it.

The catch is: everything happens concurrently, so in order to account for all this, there's a router passing messages around to make things happen. The router receives all messages and forwards them to the interested parties, which shall be registered in their topics of interest.

For instance, to know how much time a single person has waited on queue, a `PERSON_ENTERED_THE_BATHROOM` event message is sent. Aside from the event name, several pieces of data are sent with it, allowing for the metrics collector to compute the time it took for that person to be able to use the bathroom, from the time they joined the queue to the time they entered the bathroom. For this to reach the metrics collector, it has to register itself with the router as an interested destination for `PERSON_ENTERED_THE_BATHROOM` events. 

As the simulation stops (which is itself an event, after which all threads are gracefully "shut down"), this mentioned metrics collector computes several metrics, (such as average, ordered values, percentiles, etc...), and writes them to a json file under `statistics_reports/`. For more details about which meares and metrics are taken and computed, see `src/simulation/metrics_collector.rs`.

The simulation is fairly parameterized, and its parameters are constants defined on top of the `src/simulation.rs` file.

# How to run this?

Simply run
```shell
cargo run
```

Pressing `Ctrl-c` will stop the simulation gracefully. 
