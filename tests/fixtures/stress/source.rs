pub mod pipeline {
    pub trait Stage {
        fn name(&self) -> &'static str;
    }

    pub enum Event {
        Started,
        Finished,
    }

    pub struct Runner<T> {
        stage: T,
    }

    impl<T: Stage> Runner<T> {
        pub fn new(stage: T) -> Self {
            Self { stage }
        }

        pub async fn run(&self, events: &[Event]) -> Vec<String> {
            let format = |event: &Event| match event {
                Event::Started => format!("{}:started", self.stage.name()),
                Event::Finished => format!("{}:finished", self.stage.name()),
            };
            events.iter().map(format).collect()
        }
    }

    pub fn build_runner<T: Stage>(stage: T) -> Runner<T> {
        Runner::new(stage)
    }
}
