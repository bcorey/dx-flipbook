#[derive(Clone, PartialEq, Debug)]
pub struct Stopwatch {
    lap_start: Option<web_time::SystemTime>,
    elapsed: web_time::Duration,
}

impl Stopwatch {
    pub fn new() -> Self {
        Self {
            lap_start: None,
            elapsed: web_time::Duration::ZERO,
        }
    }

    pub fn stop(&mut self) {
        self.lap_start = None;
    }

    pub fn start(&mut self) {
        self.lap_start = Some(web_time::SystemTime::now());
    }

    fn lap(&mut self) {
        if let Some(lap_start) = self.lap_start {
            let lap_elapsed = lap_start.elapsed().unwrap();
            self.elapsed = self.elapsed.checked_add(lap_elapsed).unwrap();
            self.start();
        }
    }

    pub fn get_elapsed(&mut self) -> web_time::Duration {
        self.lap();
        self.elapsed
    }

    pub fn clear(&mut self) {
        self.stop();
        self.elapsed = web_time::Duration::ZERO;
    }
}
