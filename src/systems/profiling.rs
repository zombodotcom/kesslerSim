use bevy::prelude::*;

/// Resource to track system performance
#[derive(Resource, Default)]
pub struct SystemProfiler {
    pub system_timings: std::collections::HashMap<String, Vec<f64>>,
    pub frame_times: Vec<f64>,
    pub max_samples: usize,
}

impl SystemProfiler {
    pub fn new(max_samples: usize) -> Self {
        Self {
            system_timings: std::collections::HashMap::new(),
            frame_times: Vec::with_capacity(max_samples),
            max_samples,
        }
    }

    pub fn record_system(&mut self, name: String, duration: f64) {
        let timings = self.system_timings.entry(name).or_insert_with(Vec::new);
        timings.push(duration);
        if timings.len() > self.max_samples {
            timings.remove(0);
        }
    }

    pub fn record_frame(&mut self, frame_time: f64) {
        self.frame_times.push(frame_time);
        if self.frame_times.len() > self.max_samples {
            self.frame_times.remove(0);
        }
    }

    pub fn get_average_timing(&self, system_name: &str) -> Option<f64> {
        self.system_timings.get(system_name).map(|timings| {
            timings.iter().sum::<f64>() / timings.len() as f64
        })
    }

    pub fn get_average_frame_time(&self) -> f64 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        self.frame_times.iter().sum::<f64>() / self.frame_times.len() as f64
    }
}

/// System to record frame timing
pub fn profile_frame_system(
    mut profiler: ResMut<SystemProfiler>,
    time: Res<Time>,
) {
    let frame_time = time.delta_secs() as f64 * 1000.0; // Convert to milliseconds
    profiler.record_frame(frame_time);
}

/// System to log performance statistics periodically
pub fn log_performance_stats_system(
    profiler: Res<SystemProfiler>,
    time: Res<Time>,
    mut last_log: Local<f32>,
) {
    let current_time = time.elapsed_secs();
    
    // Log every 5 seconds
    if current_time - *last_log > 5.0 {
        *last_log = current_time;
        
        let avg_frame_time = profiler.get_average_frame_time();
        let avg_fps = if avg_frame_time > 0.0 {
            1000.0 / avg_frame_time
        } else {
            0.0
        };
        
        info!("Performance Stats:");
        info!("  Average FPS: {:.1}", avg_fps);
        info!("  Average Frame Time: {:.2} ms", avg_frame_time);
        
        // Log top systems by average time
        let mut system_times: Vec<_> = profiler.system_timings.iter()
            .filter_map(|(name, timings)| {
                if !timings.is_empty() {
                    let avg = timings.iter().sum::<f64>() / timings.len() as f64;
                    Some((name.clone(), avg))
                } else {
                    None
                }
            })
            .collect();
        
        system_times.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        info!("  Top Systems by Time:");
        for (name, avg_time) in system_times.iter().take(5) {
            info!("    {}: {:.3} ms", name, avg_time);
        }
    }
}

/// Macro helper for timing systems (would need to be used in system definitions)
#[macro_export]
macro_rules! timed_system {
    ($system_fn:ident, $profiler:expr) => {
        {
            let start = std::time::Instant::now();
            let result = $system_fn();
            let duration = start.elapsed().as_secs_f64() * 1000.0; // Convert to ms
            $profiler.record_system(stringify!($system_fn).to_string(), duration);
            result
        }
    };
}

