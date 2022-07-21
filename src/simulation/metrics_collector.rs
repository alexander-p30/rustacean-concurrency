use serde::Serialize;
use std::ops::Div;

#[derive(Debug, Serialize)]
pub struct MetricsCollector {
    pub male_queue_size: Statistic,
    pub female_queue_size: Statistic,
    pub gender_switches: u64,
    pub time_bathroom_was_male: Statistic,
    pub time_bathroom_was_female: Statistic,
    pub male_personal_total_time_spent: Statistic,
    pub female_personal_total_time_spent: Statistic,
    pub male_personal_total_wait_time: Statistic,
    pub female_personal_total_wait_time: Statistic,
}

pub fn new_metrics_collector() -> MetricsCollector {
    return MetricsCollector {
        male_queue_size: new_statistic(),
        female_queue_size: new_statistic(),
        gender_switches: 0,
        time_bathroom_was_male: new_statistic(),
        time_bathroom_was_female: new_statistic(),
        male_personal_total_time_spent: new_statistic(),
        female_personal_total_time_spent: new_statistic(),
        male_personal_total_wait_time: new_statistic(),
        female_personal_total_wait_time: new_statistic(),
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
        self.male_personal_total_wait_time.update_statistics();
        self.female_personal_total_wait_time.update_statistics();
    }
}

#[derive(Debug, Serialize)]
pub struct Statistic {
    pub measures: Vec<u64>,
    pub avg: u64,
    pub min: u64,
    pub max: u64,
    pub median: u64,
    pub percentile_10: u64,
    pub percentile_25: u64,
    pub percentile_75: u64,
    pub percentile_90: u64,
    pub ordered_measures: Vec<u64>,
}

fn new_statistic() -> Statistic {
    return Statistic {
        measures: vec![],
        avg: 0,
        min: 0,
        max: 0,
        median: 0,
        percentile_10: 0,
        percentile_25: 0,
        percentile_75: 0,
        percentile_90: 0,
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

    pub fn update_percentile_10(&mut self) -> u64 {
        self.percentile_10 = match self
            .ordered_measures
            .get(self.ordered_measures.len() / 10 as usize)
        {
            Some(i) => *i,
            None => 0,
        };

        return self.percentile_10;
    }

    pub fn update_percentile_25(&mut self) -> u64 {
        self.percentile_25 = match self
            .ordered_measures
            .get(self.ordered_measures.len() / 4 as usize)
        {
            Some(i) => *i,
            None => 0,
        };

        return self.percentile_25;
    }

    pub fn update_percentile_75(&mut self) -> u64 {
        self.percentile_75 = match self
            .ordered_measures
            .get((self.ordered_measures.len() / 4 as usize) * 3 as usize)
        {
            Some(i) => *i,
            None => 0,
        };

        return self.percentile_75;
    }

    pub fn update_percentile_90(&mut self) -> u64 {
        self.percentile_90 = match self
            .ordered_measures
            .get((self.ordered_measures.len() / 10 as usize) * 9 as usize)
        {
            Some(i) => *i,
            None => 0,
        };

        return self.percentile_90;
    }

    pub fn update_statistics(&mut self) {
        self.update_avg();
        self.update_min();
        self.update_max();
        self.update_median();
        self.update_percentile_10();
        self.update_percentile_25();
        self.update_percentile_75();
        self.update_percentile_90();
    }
}
