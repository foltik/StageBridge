use stagebridge::midi::device::launchpad_x::*;
use stagebridge::util::pipeline::Pipeline;

pub fn cancel_off() -> Pipeline<Output, Output> {
    Pipeline::<Output>::new()
        .with_cancel(async move |o, tx| {
            tx.send(o).await;
            // TODO: Better way of parking task, or directly awaiting cancellation future
            tokio::time::sleep(std::time::Duration::from_secs(99999999)).await;
        }, async move |o, tx| {
            if let Output::Light(p, _) = o {
                tx.send(Output::Off(p)).await;
            }
        })
}