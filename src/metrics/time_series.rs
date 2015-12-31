use std::collections::HashMap;

use super::Duration;
use super::Second;
use super::StartTime;
use super::statistics::ComputedMetric;
use super::statistics::compute_metric;


#[derive(Debug, Clone, PartialEq)]
pub struct TimeSeries {
    timers: HashMap<String, HashMap<Second, Vec<Duration>>>,
}

impl TimeSeries {
    pub fn new() -> TimeSeries {
        TimeSeries { timers: HashMap::new() }
    }

    pub fn get_timers(&self) -> &HashMap<String, HashMap<Second, Vec<Duration>>> {
        &self.timers
    }


    pub fn add_timer(&mut self,
                     name: &str,
                     start_time: StartTime,
                     dur: Duration) {
        // does the name series exist?
        if !self.timers.contains_key(name) {
            self.timers.insert(name.to_string(), HashMap::new());
        }

        // does the second series exist?
        let sec = start_time as Second;
        if !self.timers.get(name).unwrap().contains_key(&sec) {
            self.timers.get_mut(name).unwrap().insert(sec, vec![]);
        }

        // insert the value
        self.timers.get_mut(name).unwrap().get_mut(&sec).unwrap().push(dur);
    }

    pub fn clear(&mut self) {
        self.timers.clear();
    }

    pub fn merge(&mut self, other: &TimeSeries) {
        let ref ts = other.timers;
        for (name, seconds) in ts.iter() {

            // If we don't have this name we just add it
            if !self.timers.contains_key(name) {
                self.timers.insert(name.to_string(), seconds.clone());

            } else {
                // Else we need to merge it
                for (second, intervals) in seconds.iter() {

                    // If we don't have this second we add it
                    if !self.timers.get(name).unwrap().contains_key(second) {
                        self.timers
                            .get_mut(name)
                            .unwrap()
                            .insert(second.clone(), intervals.clone());
                    } else {
                        // Else we need to merge it
                        self.timers
                            .get_mut(name)
                            .unwrap()
                            .get_mut(second)
                            .unwrap()
                            .extend(intervals);
                    }

                }
            }

        }
    }

    pub fn compute_metrics(&self) -> HashMap<String, ComputedMetric> {
        let mut comp_mets = HashMap::new();

        for (name, seconds) in self.timers.iter() {
            for (_, samples) in seconds.iter() {
                let comp = compute_metric(name, samples);
                comp_mets.insert(name.to_string(), comp);
            }
        }

        comp_mets
    }
}