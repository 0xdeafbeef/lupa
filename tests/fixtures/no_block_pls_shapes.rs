use std::future::Future;
use std::task::{Context, Poll};

struct TaskResult<T>(T);
struct LastOwnPoint;

struct Broadcaster {
    bcast_peers: Vec<PeerId>,
    receiver: Receiver<Update>,
}

struct PeerId;
struct Update;

struct Receiver<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Receiver<T> {
    async fn recv(&mut self) -> Option<T> {
        None
    }
}

impl Broadcaster {
    fn broadcast(&mut self, _peer: &PeerId) {}

    pub async fn run(&mut self) -> TaskResult<LastOwnPoint> {
        while let Some(update) = self.receiver.recv().await {
            let _ = update;
            for peer in std::mem::take(&mut self.bcast_peers) {
                self.broadcast(&peer);
            }
        }

        TaskResult(LastOwnPoint)
    }
}

struct Storage {
    gc_lock: MutexGuard,
}

struct MutexGuard;

impl MutexGuard {
    async fn lock_owned(&self) {}
}

impl Storage {
    #[tracing::instrument(skip(self))]
    pub async fn remove_outdated_states(&self, mc_seqno: u32) -> Result<(), Error> {
        let Some(top_blocks) = compute_recent_blocks(mc_seqno).await? else {
            return Ok(());
        };

        loop {
            let guard = self.gc_lock.lock_owned().await;
            if top_blocks.should_stop() {
                break;
            }
            drop(guard);
        }

        Ok(())
    }
}

struct TopBlocks;

impl TopBlocks {
    fn should_stop(&self) -> bool {
        false
    }
}

struct Error;

async fn compute_recent_blocks(_mc_seqno: u32) -> Result<Option<TopBlocks>, Error> {
    Ok(Some(TopBlocks))
}

fn poll_impl<'cx, Fut>(
    this_inner: &mut Option<Fut>,
    cx: &mut Context<'cx>,
) -> Poll<Fut::Output>
where
    Fut: Future + Unpin,
    Fut::Output: Clone,
{
    let Some(mut fut) = this_inner.take() else {
        return Poll::Pending;
    };

    let poll = Future::poll(std::pin::Pin::new(&mut fut), cx);
    *this_inner = Some(fut);
    poll
}
