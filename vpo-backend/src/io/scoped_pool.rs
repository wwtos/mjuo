use std::{
    any::Any,
    iter::repeat_with,
    num::NonZeroUsize,
    sync::mpsc,
    thread::{self, available_parallelism},
};

pub fn scoped_pool<T: Send, R: Send, F>(
    tasks: impl IntoIterator<Item = T>,
    f: &F,
) -> Result<Vec<R>, Box<dyn Any + Send>>
where
    F: Fn(T) -> R + Send + Sync,
{
    let threads = available_parallelism()
        .unwrap_or(NonZeroUsize::new(4).expect("wat"))
        .into();

    let (senders, receivers): (Vec<mpsc::Sender<Option<T>>>, Vec<mpsc::Receiver<Option<T>>>) =
        repeat_with(|| mpsc::channel()).take(threads).unzip();

    let merged: Result<Vec<R>, Box<dyn Any + Send>> = thread::scope(|s| {
        let guards: Vec<thread::ScopedJoinHandle<'_, Vec<R>>> = receivers
            .into_iter()
            .map(|receiver| {
                s.spawn(move || {
                    let mut results: Vec<R> = Vec::with_capacity(64);

                    while let Some(task) = receiver.recv().expect("not to be disconnected") {
                        results.push(f(task));
                    }

                    results
                })
            })
            .collect();

        // split up tasks between threads
        for (i, task) in tasks.into_iter().enumerate() {
            senders[i % threads]
                .send(Some(task))
                .expect("everything to be processed");
        }

        // be sure to send them all `None`s so they stop at the end
        for sender in senders {
            sender.send(None).unwrap();
        }

        let processed = guards
            .into_iter()
            .map(|guard| guard.join())
            .collect::<Result<Vec<Vec<R>>, Box<dyn Any + Send>>>()?;

        Ok(processed.into_iter().flatten().collect())
    });

    merged
}
