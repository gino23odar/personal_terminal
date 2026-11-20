//! Cooperative cancellation on the adapter's local executor; no threads required.
use std::{
    cell::{Cell, RefCell},
    future::Future,
    pin::Pin,
    rc::{Rc, Weak},
    task::{Context, Poll, Waker},
};

type Waiter = RefCell<Option<Waker>>;

#[derive(Default, Debug)]
struct State {
    cancelled: Cell<bool>,
    waiters: RefCell<Vec<Weak<Waiter>>>,
}

/// Create a fresh token per execution. Clones share cancellation; tokens cannot reset.
/// This is deliberately local (`!Send`/`!Sync`), matching command futures.
#[derive(Clone, Default, Debug)]
pub struct CancellationToken(Rc<State>);

impl CancellationToken {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.cancelled.get()
    }

    /// Idempotent; wake every live waiter without holding a RefCell borrow.
    pub fn cancel(&self) {
        if self.0.cancelled.replace(true) {
            return;
        }
        let waiters = std::mem::take(&mut *self.0.waiters.borrow_mut());
        for waiter in waiters.into_iter().filter_map(|weak| weak.upgrade()) {
            let waker = waiter.borrow_mut().take();
            if let Some(waker) = waker {
                waker.wake();
            }
        }
    }

    pub fn cancelled(&self) -> Cancelled {
        let waiter = Rc::new(RefCell::new(None));
        if !self.is_cancelled() {
            self.0.waiters.borrow_mut().push(Rc::downgrade(&waiter));
        }
        Cancelled {
            token: self.clone(),
            waiter,
        }
    }

    /// Poll cancellation before the operation. If cancelled, drop the operation.
    /// Adapters must make their I/O futures drop-safe; this cannot undo side effects.
    pub async fn run_until_cancelled<F: Future>(
        &self,
        future: F,
    ) -> Result<F::Output, crate::ShellError> {
        let mut future = std::pin::pin!(future);
        let mut cancelled = std::pin::pin!(self.cancelled());
        std::future::poll_fn(|cx| {
            if cancelled.as_mut().poll(cx).is_ready() {
                return Poll::Ready(Err(crate::ShellError::Cancelled));
            }
            future.as_mut().poll(cx).map(Ok)
        })
        .await
    }
}

/// A cancellation notification, with independent waker storage for each waiter.
pub struct Cancelled {
    token: CancellationToken,
    waiter: Rc<Waiter>,
}

impl Future for Cancelled {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if self.token.is_cancelled() {
            Poll::Ready(())
        } else {
            self.waiter.borrow_mut().replace(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl Drop for Cancelled {
    fn drop(&mut self) {
        let this = Rc::downgrade(&self.waiter);
        self.token
            .0
            .waiters
            .borrow_mut()
            .retain(|w| !w.ptr_eq(&this));
    }
}
