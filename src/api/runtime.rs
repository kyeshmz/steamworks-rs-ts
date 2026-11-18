use std::sync::Mutex;
use std::time::Instant;

#[derive(Default)]
struct Pump {
    last: Option<Instant>,
    runs: u64,
    gaps: u64,
    total_gap_us: u128,
    max_gap_us: u128,
    last_gap_us: u128,
    max_run_us: u128,
}

lazy_static! {
    static ref PUMP: Mutex<Pump> = Mutex::new(Pump::default());
}

pub(crate) fn dispatch() {
    // All Node workers share Steam's manual-dispatch pipe and its last callback buffer.
    if let Ok(mut pump) = PUMP.try_lock() {
        let start = Instant::now();
        crate::client::get_client().run_callbacks();
        if let Some(last) = pump.last.replace(start) {
            let gap = start.duration_since(last).as_micros();
            pump.gaps += 1;
            pump.total_gap_us += gap;
            pump.last_gap_us = gap;
            pump.max_gap_us = pump.max_gap_us.max(gap);
        }
        pump.runs += 1;
        pump.max_run_us = pump.max_run_us.max(start.elapsed().as_micros());
    }
}

use napi_derive::napi;

#[napi]
pub mod runtime {
    #[napi(object)]
    pub struct PumpStats {
        pub runs: f64,
        pub gaps: f64,
        pub mean_gap_us: f64,
        pub max_gap_us: f64,
        pub last_gap_us: f64,
        pub max_run_us: f64,
    }

    /// Timing of completed Steam callback pumps. A reset starts a fresh measurement window.
    #[napi]
    pub fn pump_stats(reset: Option<bool>) -> PumpStats {
        let mut p = super::PUMP.lock().unwrap();
        let stats = PumpStats {
            runs: p.runs as f64,
            gaps: p.gaps as f64,
            mean_gap_us: if p.gaps == 0 {
                0.0
            } else {
                p.total_gap_us as f64 / p.gaps as f64
            },
            max_gap_us: p.max_gap_us as f64,
            last_gap_us: p.last_gap_us as f64,
            max_run_us: p.max_run_us as f64,
        };
        if reset.unwrap_or(false) {
            *p = Default::default();
        }
        stats
    }
}
