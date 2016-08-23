
pub struct LatencyRecorder {
    pub last: i64,
    history: Vec<i64>,
}

impl LatencyRecorder {
    pub fn new() -> LatencyRecorder {
        LatencyRecorder {
            last: 0,
            history: vec!(),
        }
    }

    pub fn add(&mut self, latency: i64) {
        self.last = latency;
        self.history.push(latency);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn last_initial() {
        let subject = LatencyRecorder::new();
        assert_eq!(subject.last, 0);
    }

    #[test]
    fn last_after_add() {
        let mut subject = LatencyRecorder::new();
        subject.add(20);
        assert_eq!(subject.last, 20);
    }

    #[test]
    fn last_after_multiple_adds() {
        let mut subject = LatencyRecorder::new();
        subject.add(20);
        subject.add(40);
        subject.add(60);
        assert_eq!(subject.last, 60);
    }
}
