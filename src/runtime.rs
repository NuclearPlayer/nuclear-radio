use crate::sinks::{Sink, SinkResult};

pub struct Runtime {
    sinks: Vec<Box<dyn Sink>>,
}

impl Runtime {
    pub fn new() -> Self {
        Self { sinks: Vec::new() }
    }

    pub fn add(&mut self, sink: impl Sink + 'static) -> &mut Self {
        self.sinks.push(Box::new(sink));
        self
    }

    pub async fn run(mut self) -> SinkResult {
        for sink in self.sinks.iter_mut() {
            sink.start().await?;
        }

        let shutdown = async {
            if tokio::signal::ctrl_c().await.is_err() {
                eprintln!("Failed to listen for shutdown signal");
            }

            for sink in self.sinks.iter_mut() {
                sink.cleanup().await;
            }
        };

        shutdown.await;
        Ok(())
    }
}
