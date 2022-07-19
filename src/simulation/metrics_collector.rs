use std::ops::Div;
use uuid::Uuid;

#[derive(Debug)]
pub struct MetricsCollector {
    pub id: Uuid,
    pub male_queue_size: Statistic,
    pub female_queue_size: Statistic,
    pub gender_switches: u64,
    pub time_bathroom_was_male: Statistic,
    pub time_bathroom_was_female: Statistic,
    pub male_personal_total_time_spent: Statistic,
    pub female_personal_total_time_spent: Statistic,
}

pub fn new_metrics_collector() -> MetricsCollector {
    return MetricsCollector {
        id: Uuid::new_v4(),
        male_queue_size: new_statistic(),
        female_queue_size: new_statistic(),
        gender_switches: 0,
        time_bathroom_was_male: new_statistic(),
        time_bathroom_was_female: new_statistic(),
        male_personal_total_time_spent: new_statistic(),
        female_personal_total_time_spent: new_statistic(),
    };
}

impl MetricsCollector {
    pub fn update_statistics(&mut self) {
        self.male_queue_size.update_statistics();
        self.female_queue_size.update_statistics();
        self.time_bathroom_was_male.update_statistics();
        self.time_bathroom_was_female.update_statistics();
        self.male_personal_total_time_spent.update_statistics();
        self.female_personal_total_time_spent.update_statistics();
    }
}

#[derive(Debug)]
pub struct Statistic {
    pub measures: Vec<u64>,
    pub avg: u64,
    pub min: u64,
    pub max: u64,
    pub median: u64,
    pub ordered_measures: Vec<u64>,
}

fn new_statistic() -> Statistic {
    return Statistic {
        measures: vec![],
        avg: 0,
        min: 0,
        max: 0,
        median: 0,
        ordered_measures: vec![],
    };
}

impl Statistic {
    pub fn add_measure(&mut self, measure: u64) {
        self.measures.push(measure);

        self.ordered_measures.insert(
            self.ordered_measures
                .binary_search(&measure)
                .unwrap_or_else(|e| e),
            measure,
        );
    }

    pub fn update_avg(&mut self) -> u64 {
        let measures_len = self.measures.len() as u64;

        if measures_len == 0 {
            self.avg = 0;
        } else {
            self.avg = self.measures.iter().sum::<u64>().div(measures_len as u64);
        }

        return self.avg;
    }

    pub fn update_min(&mut self) -> u64 {
        self.min = match self.ordered_measures.first() {
            Some(i) => *i,
            None => 0,
        };

        return self.min;
    }

    pub fn update_max(&mut self) -> u64 {
        self.max = match self.ordered_measures.last() {
            Some(i) => *i,
            None => 0,
        };

        return self.max;
    }

    pub fn update_median(&mut self) -> u64 {
        self.median = match self
            .ordered_measures
            .get(self.ordered_measures.len() / 2 as usize)
        {
            Some(i) => *i,
            None => 0,
        };

        return self.median;
    }

    pub fn update_statistics(&mut self) {
        self.update_avg();
        self.update_min();
        self.update_max();
        self.update_median();
    }
}
