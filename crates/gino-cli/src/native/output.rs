use gino_shell::{CommandIo, LocalFuture, OutputStream, ShellError, TerminalEvent};
use std::{
    cell::RefCell,
    io::{self, Write},
    rc::Rc,
    task::{Poll, Waker},
};

const CHUNK: usize = 4096;
enum Request {
    Bytes(OutputStream, Vec<u8>),
    Clear,
}
#[derive(Default)]
struct Slot {
    request: Option<Request>,
    result: Option<Result<(), ShellError>>,
    waker: Option<Waker>,
}
/// At most one 4 KiB output chunk is held; each chunk waits for renderer acknowledgement.
#[derive(Clone, Default)]
pub(super) struct Bridge(Rc<RefCell<Slot>>);
struct Pending(Bridge);
impl Drop for Pending {
    fn drop(&mut self) {
        *self.0.0.borrow_mut() = Slot::default();
    }
}
impl Bridge {
    async fn send(&mut self, request: Request) -> Result<(), ShellError> {
        let _pending = Pending(self.clone());
        self.0.borrow_mut().request = Some(request);
        std::future::poll_fn(|cx| {
            let mut slot = self.0.borrow_mut();
            if let Some(result) = slot.result.take() {
                Poll::Ready(result)
            } else {
                slot.waker = Some(cx.waker().clone());
                Poll::Pending
            }
        })
        .await
    }
    pub(super) fn drain(&self, renderer: &mut Renderer) -> bool {
        let Some(request) = self.0.borrow_mut().request.take() else {
            return false;
        };
        let result = match request {
            Request::Bytes(stream, bytes) => renderer.write(stream, &bytes),
            Request::Clear => {
                let mut stdout = io::stdout().lock();
                stdout
                    .write_all(b"\x1b[2J\x1b[H")
                    .and_then(|()| stdout.flush())
            }
        }
        .map_err(|_| ShellError::Io("terminal output failed".into()));
        let waker = {
            let mut slot = self.0.borrow_mut();
            slot.result = Some(result);
            slot.waker.take()
        };
        if let Some(waker) = waker {
            waker.wake();
        }
        true
    }
}
impl CommandIo for Bridge {
    fn read<'a>(&'a mut self, _: &'a mut [u8]) -> LocalFuture<'a, Result<usize, ShellError>> {
        // Keyboard editing is not command stdin. No input-consuming built-ins exist yet.
        Box::pin(async { Ok(0) })
    }
    fn write<'a>(
        &'a mut self,
        stream: OutputStream,
        bytes: &'a [u8],
    ) -> LocalFuture<'a, Result<(), ShellError>> {
        Box::pin(async move {
            for chunk in bytes.chunks(CHUNK) {
                self.send(Request::Bytes(stream, chunk.to_vec())).await?;
            }
            Ok(())
        })
    }
    fn emit(&mut self, _: TerminalEvent) -> LocalFuture<'_, Result<(), ShellError>> {
        Box::pin(async move { self.send(Request::Clear).await })
    }
}

/// Keep UTF-8 boundaries intact across transport chunks; terminal control bytes
/// are displayed as escaped text. Only the Clear event can emit ANSI controls.
#[derive(Default)]
pub(super) struct Renderer {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}
impl Renderer {
    /// A truncated UTF-8 suffix cannot leak into the next command's output.
    pub(super) fn finish(&mut self) -> io::Result<()> {
        for stream in [OutputStream::Stdout, OutputStream::Stderr] {
            let pending = match stream {
                OutputStream::Stdout => &mut self.stdout,
                OutputStream::Stderr => &mut self.stderr,
            };
            if !pending.is_empty() {
                pending.clear();
                self.write(stream, "�".as_bytes())?;
            }
        }
        Ok(())
    }

    fn write(&mut self, stream: OutputStream, bytes: &[u8]) -> io::Result<()> {
        let pending = match stream {
            OutputStream::Stdout => &mut self.stdout,
            OutputStream::Stderr => &mut self.stderr,
        };
        let text = decode(pending, bytes);
        match stream {
            OutputStream::Stdout => {
                let mut out = io::stdout().lock();
                out.write_all(text.as_bytes())?;
                out.flush()
            }
            OutputStream::Stderr => {
                let mut out = io::stderr().lock();
                out.write_all(text.as_bytes())?;
                out.flush()
            }
        }
    }
}
fn decode(pending: &mut Vec<u8>, bytes: &[u8]) -> String {
    pending.extend_from_slice(bytes);
    let mut text = String::new();
    let mut offset = 0;
    while offset < pending.len() {
        let (valid, invalid) = match std::str::from_utf8(&pending[offset..]) {
            Ok(s) => (s, Some(0)),
            Err(error) => (
                std::str::from_utf8(&pending[offset..offset + error.valid_up_to()])
                    .expect("validated prefix"),
                error.error_len(),
            ),
        };
        for c in valid.chars() {
            if (c.is_control() && !matches!(c, '\n' | '\t')) || matches!(c, '\u{2028}' | '\u{2029}')
            {
                text.extend(c.escape_default());
            } else {
                text.push(c);
            }
        }
        offset += valid.len();
        match invalid {
            None => break,
            Some(0) => break,
            Some(n) => {
                text.push('\u{fffd}');
                offset += n;
            }
        }
    }
    pending.drain(..offset);
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::task::Context;
    #[test]
    fn utf8_and_controls_are_safe_across_chunks() {
        let mut pending = Vec::new();
        let crab = "🦀".as_bytes();
        assert_eq!(decode(&mut pending, &crab[..2]), "");
        assert_eq!(decode(&mut pending, &crab[2..]), "🦀");
        assert_eq!(
            decode(&mut pending, b"a\x1b[2J\r\n\t\xff"),
            "a\\u{1b}[2J\\r\n\t�"
        );
        assert!(pending.is_empty());
    }
    #[test]
    fn output_is_bounded_and_dropped_writes_clear_pending_work() {
        let mut bridge = Bridge::default();
        let observer = bridge.clone();
        let bytes = vec![b'a'; CHUNK * 3];
        let mut future = bridge.write(OutputStream::Stdout, &bytes);
        let mut cx = Context::from_waker(Waker::noop());
        assert!(future.as_mut().poll(&mut cx).is_pending());
        assert!(
            matches!(&observer.0.borrow().request, Some(Request::Bytes(_, b)) if b.len() == CHUNK)
        );
        drop(future);
        assert!(observer.0.borrow().request.is_none());
        assert!(observer.0.borrow().waker.is_none());
    }
}
