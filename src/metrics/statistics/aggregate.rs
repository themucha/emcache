use std::cmp::Ordering;

use super::AggregatedMetric;


// f64 does not have total ordering hence this convenience function which
// defaults to judging values equal if they cannot be compared
pub fn sort_f64(samples: &mut Vec<f64>) -> &mut Vec<f64> {
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    samples
}


pub fn compute_average(samples: &Vec<f64>) -> Option<f64> {
    match samples.is_empty() {
        true => None,
        false => {
            let sum = samples.iter().fold(0.0, |acc, x| acc + x);
            let avg = sum / (samples.len() as f64);
            Some(avg)
        }
    }
}


pub fn compute_px(samples: &Vec<f64>, len: usize, pct: f64) -> Option<f64> {
    if samples.len() < len {
        return None;
    }

    let mut clone = samples.clone();
    let sorted = sort_f64(&mut clone);
    let pos = ((samples.len() as f64) * pct) as usize;

    Some(sorted[pos])
}

pub fn compute_p0(samples: &Vec<f64>) -> Option<f64> {
    compute_px(samples, 1, 0.0)
}

pub fn compute_p90(samples: &Vec<f64>) -> Option<f64> {
    compute_px(samples, 10, 0.9)
}

pub fn compute_p99(samples: &Vec<f64>) -> Option<f64> {
    compute_px(samples, 100, 0.99)
}

pub fn compute_p999(samples: &Vec<f64>) -> Option<f64> {
    compute_px(samples, 1000, 0.999)
}


pub fn aggregate_metric(name: &str, samples: &Vec<f64>) -> AggregatedMetric {
    AggregatedMetric {
        name: name.to_string(),
        n: samples.len() as u64,
        avg: compute_average(&samples),
        p0: compute_p0(&samples),
        p90: compute_p90(&samples),
        p99: compute_p99(&samples),
        p999: compute_p999(&samples),
    }
}
