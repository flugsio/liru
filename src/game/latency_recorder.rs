
pub struct LatencyRecorder {
    average: f64,
    history: Vec<i64>,
}

impl LatencyRecorder {
    pub fn new() -> LatencyRecorder {
        LatencyRecorder {
            average: 0.0,
            history: vec!(),
        }
    }

    pub fn add(&mut self, latency: i64) {
        self.average += (latency as f64 - self.average) * self.sensitivity();
        self.history.push(latency);
    }

    #[allow(dead_code)]
    pub fn last(&self) -> i64 {
        *self.history.last().unwrap_or(&0)
    }

    pub fn average(&self) -> i64 {
        self.average as i64
    }

    fn sensitivity(&self) -> f64 {
        (10.0 - (self.history.len() as f64).min(8.0)) / 10.0
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn last_initial() {
        let subject = LatencyRecorder::new();
        assert_eq!(subject.last(), 0);
    }

    #[test]
    fn last_after_add() {
        let mut subject = LatencyRecorder::new();
        subject.add(20);
        assert_eq!(subject.last(), 20);
    }

    #[test]
    fn last_after_multiple_adds() {
        let mut subject = LatencyRecorder::new();
        subject.add(20);
        subject.add(40);
        subject.add(60);
        assert_eq!(subject.last(), 60);
    }

    #[test]
    fn average_one() {
        let mut subject = LatencyRecorder::new();
        subject.add(20);
        assert_eq!(subject.average(), 20);
    }

    #[test]
    fn average_more() {
        let mut subject = LatencyRecorder::new();
        subject.add(20);
        subject.add(40);
        assert_eq!(subject.average(), 38);
        subject.add(40);
        assert_eq!(subject.average(), 39);
        subject.add(26);
        assert_eq!(subject.average(), 30);
        subject.add(120);
        assert_eq!(subject.average(), 84);
        subject.add(22);
        assert_eq!(subject.average(), 53);
        subject.add(24);
        assert_eq!(subject.average(), 41);
        subject.add(24);
        assert_eq!(subject.average(), 36);
        subject.add(24);
        assert_eq!(subject.average(), 33);
        subject.add(24);
        assert_eq!(subject.average(), 31);
        subject.add(24);
        assert_eq!(subject.average(), 30);
    }

    #[test]
    fn average_sentitivity() {
        // more sensitive in the beginning
        let mut subject = LatencyRecorder::new();
        subject.add(20);
        subject.add(20);
        subject.add(100);
        assert_eq!(subject.average(), 84);

        let mut subject = LatencyRecorder::new();
        subject.add(20);
        subject.add(20);
        subject.add(20);
        subject.add(20);
        subject.add(20);
        subject.add(20);
        subject.add(100);
        assert_eq!(subject.average(), 52);
    }

    #[test]
    fn sensitivity_declines() {
        let mut subject = LatencyRecorder::new();
        assert_eq!(subject.sensitivity(), 1.0);
        subject.add(20);
        assert_eq!(subject.sensitivity(), 0.9);
        subject.add(20);
        assert_eq!(subject.sensitivity(), 0.8);
        subject.add(20);
        assert_eq!(subject.sensitivity(), 0.7);
        subject.add(20);
        assert_eq!(subject.sensitivity(), 0.6);
        subject.add(20);
        subject.add(20);
        subject.add(20);
        subject.add(20);
        subject.add(20);
        assert_eq!(subject.sensitivity(), 0.2);
        subject.add(20);
        assert_eq!(subject.sensitivity(), 0.2);
    }
    
}
