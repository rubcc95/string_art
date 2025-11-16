use futures::*;
use gloo_timers::future::TimeoutFuture;
use gloo_worker::{Spawnable, reactor::*};
use string_art_gui_common::*;

impl WebWorker {
    pub fn spawn(path: &str) -> ReactorBridge<Self> {
        WebWorker::spawner().spawn(path)
    }
}

#[derive(Copy, Clone)]
enum Counter {
    Smaller(usize),
    Bigger,
}

impl Counter {
    pub fn new() -> Self {
        Self::Smaller(0)
    }

    pub fn value(&mut self) -> usize {
        match *self {
            Counter::Smaller(val) => {
                if val < 2024 {
                    *self = Self::Smaller(val + 1);
                    val / 64
                } else {
                    *self = Self::Bigger;
                    32
                }
            }
            Counter::Bigger => 32,
        }
    }
}

#[reactor]
pub async fn WebWorker(mut scope: ReactorScope<Input, Output>) {
    let input = scope.next().await.unwrap();
    let mut computation = match computation(input) {
        Ok(computation) => computation,
        Err(err) => return scope.send(Output::Err(err.to_string())).await.unwrap(),
    };

    match computation.init() {
        Ok(data) => {
            scope.send(Output::Init(data)).await.unwrap();
            TimeoutFuture::new(0).await;

            let mut to_send = Vec::new();
            let mut count = Counter::new();

            while let Some(output) = computation.next() {
                match output {
                    Ok(step) => {
                        to_send.push(step);
                        if to_send.len() > count.value() {
                            scope.send(Output::Steps(to_send)).await.unwrap();
                            to_send = Vec::new();
                            gloo_timers::future::TimeoutFuture::new(0).await;
                        }

                    }
                    Err(err) => return scope.send(Output::Err(err.to_string())).await.unwrap(),
                }
            }
            scope
                .send(Output::Done(to_send, computation.start_anchors()))
                .await
                .unwrap();
        }
        Err(err) => return scope.send(Output::Err(err.to_string())).await.unwrap(),
    }
}
