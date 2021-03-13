// Contents of this module is inspired by https://github.com/Srinivasa314/alcro/tree/master/src/chrome
use crate::worker::{Channel, PayloadChannel};
use async_executor::Executor;
use async_fs::File;
use async_oneshot::Receiver;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::ffi::CString;
use std::mem;
use std::os::raw::{c_char, c_int};
use std::os::unix::io::FromRawFd;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Copy, Clone, Error)]
pub enum ExitError {
    /// Generic error.
    #[error("Worker exited with generic error")]
    Generic,
    /// Settings error.
    #[error("Worker exited with settings error")]
    Settings,
    /// Unknown error.
    #[error("Worker exited with unknown error and status code {status_code}")]
    Unknown { status_code: i32 },
    /// Unexpected error.
    #[error("Worker exited unexpectedly")]
    Unexpected,
}

fn pipe() -> [c_int; 2] {
    unsafe {
        let mut fds = mem::MaybeUninit::<[c_int; 2]>::uninit();

        if libc::pipe(fds.as_mut_ptr() as *mut c_int) != 0 {
            panic!(
                "libc::pipe() failed with code {}",
                *libc::__errno_location()
            );
        }

        fds.assume_init()
    }
}

static SPAWNING: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

pub(super) struct WorkerRunResult {
    pub(super) channel: Channel,
    pub(super) payload_channel: PayloadChannel,
    pub(super) status_receiver: Receiver<Result<(), ExitError>>,
}

pub(super) fn run_worker_with_channels(
    executor: Arc<Executor<'static>>,
    args: Vec<String>,
) -> WorkerRunResult {
    // Take a lock to make sure we don't spawn workers from multiple threads concurrently, this
    // causes racy issues
    let _lock = SPAWNING.lock();
    let [producer_fd_read, producer_fd_write] = pipe();
    let [consumer_fd_read, consumer_fd_write] = pipe();
    let [producer_payload_fd_read, producer_payload_fd_write] = pipe();
    let [consumer_payload_fd_read, consumer_payload_fd_write] = pipe();
    let (status_sender, status_receiver) = async_oneshot::oneshot();

    std::thread::spawn(move || {
        let argc = args.len() as c_int;
        let args_cstring = args
            .into_iter()
            .map(|s| -> CString { CString::new(s).unwrap() })
            .collect::<Vec<_>>();
        let argv = args_cstring
            .iter()
            .map(|arg| arg.as_ptr() as *const c_char)
            .collect::<Vec<_>>();
        let version = CString::new(env!("CARGO_PKG_VERSION")).unwrap();
        let status_code = unsafe {
            mediasoup_sys::run_worker(
                argc,
                argv.as_ptr(),
                version.as_ptr(),
                false,
                producer_fd_read,
                consumer_fd_write,
                producer_payload_fd_read,
                consumer_payload_fd_write,
            )
        };

        let _ = status_sender.send(match status_code {
            0 => Ok(()),
            1 => Err(ExitError::Generic),
            42 => Err(ExitError::Settings),
            status_code => Err(ExitError::Unknown { status_code }),
        });
    });

    let producer_file = unsafe { File::from_raw_fd(producer_fd_write) };
    let consumer_file = unsafe { File::from_raw_fd(consumer_fd_read) };
    let producer_payload_file = unsafe { File::from_raw_fd(producer_payload_fd_write) };
    let consumer_payload_file = unsafe { File::from_raw_fd(consumer_payload_fd_read) };

    WorkerRunResult {
        channel: Channel::new(Arc::clone(&executor), consumer_file, producer_file),
        payload_channel: PayloadChannel::new(
            executor,
            consumer_payload_file,
            producer_payload_file,
        ),
        status_receiver,
    }
}
