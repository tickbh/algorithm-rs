use super::Timer;

pub struct StepTimer<T: Timer, V> {
    pub step: T,
    pub val: V,
}

impl<T: Timer, V> StepTimer<T, V> {
    pub fn new(val: V, step: T) -> Self {
        Self { val, step }
    }
}

impl<T: Timer, V> Timer for StepTimer<T, V> {
    fn when(&self) -> u64 {
        self.step.when()
    }
}
