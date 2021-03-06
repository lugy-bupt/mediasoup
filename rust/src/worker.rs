//! A worker represents a mediasoup C++ subprocess that runs on a single CPU core and handles
//! [`Router`] instances.

// TODO: This is Unix-specific and doesn't support Windows in any way
mod channel;
mod common;
mod payload_channel;
mod utils;

use crate::data_structures::AppData;
use crate::messages::{
    RouterInternal, WorkerCreateRouterRequest, WorkerDumpRequest, WorkerGetResourceRequest,
    WorkerUpdateSettingsRequest,
};
use crate::ortc;
use crate::ortc::RtpCapabilitiesError;
use crate::router::{Router, RouterId, RouterOptions};
use crate::worker_manager::WorkerManager;
use async_executor::Executor;
use async_process::{Child, Command, ExitStatus, Stdio};
pub(crate) use channel::Channel;
pub(crate) use common::{SubscriptionHandler, SubscriptionTarget};
use event_listener_primitives::{Bag, BagOnce, HandlerId};
use futures_lite::io::BufReader;
use futures_lite::{future, AsyncBufReadExt, StreamExt};
use log::*;
use parking_lot::Mutex;
pub(crate) use payload_channel::{NotificationError, NotificationMessage, PayloadChannel};
use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use std::ops::RangeInclusive;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{env, io};
use thiserror::Error;
use utils::SpawnResult;

/// Error that caused request to mediasoup-worker subprocess to fail.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum RequestError {
    /// Channel already closed
    #[error("Channel already closed")]
    ChannelClosed,
    /// Message is too long
    #[error("Message is too long")]
    MessageTooLong,
    /// Payload is too long
    #[error("Payload is too long")]
    PayloadTooLong,
    /// Request timed out
    #[error("Request timed out")]
    TimedOut,
    /// Received response error
    #[error("Received response error: {reason}")]
    Response { reason: String },
    /// Failed to parse response from worker
    #[error("Failed to parse response from worker: {error}")]
    FailedToParse { error: String },
    /// Worker did not return any data in response
    #[error("Worker did not return any data in response")]
    NoData,
}

/// Logging level for logs generated by the media worker subprocesses (check the
/// [Debugging](https://mediasoup.org/documentation/v3/mediasoup/debugging/)
/// documentation on TypeScript implementation and generic
/// [Rust-specific](https://rust-lang-nursery.github.io/rust-cookbook/development_tools/debugging/log.html) [docs](https://docs.rs/env_logger)).
///
/// Default [`WorkerLogLevel::Error`].
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkerLogLevel {
    /// Log all severities.
    Debug,
    /// Log "warn" and "error" severities.
    Warn,
    /// Log "error" severity.
    Error,
    /// Do not log anything.
    None,
}

impl Default for WorkerLogLevel {
    fn default() -> Self {
        Self::Error
    }
}

impl WorkerLogLevel {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Debug => "debug",
            Self::Warn => "warn",
            Self::Error => "error",
            Self::None => "none",
        }
    }
}

/// Log tags for debugging. Check the meaning of each available tag in the
/// [Debugging](https://mediasoup.org/documentation/v3/mediasoup/debugging/) documentation.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkerLogTag {
    /// Logs about software/library versions, configuration and process information.
    Info,
    /// Logs about ICE.
    Ice,
    /// Logs about DTLS.
    Dtls,
    /// Logs about RTP.
    Rtp,
    /// Logs about SRTP encryption/decryption.
    Srtp,
    /// Logs about RTCP.
    Rtcp,
    /// Logs about RTP retransmission, including NACK/PLI/FIR.
    Rtx,
    /// Logs about transport bandwidth estimation.
    Bwe,
    /// Logs related to the scores of Producers and Consumers.
    Score,
    /// Logs about video simulcast.
    Simulcast,
    /// Logs about video SVC.
    Svc,
    /// Logs about SCTP (DataChannel).
    Sctp,
    /// Logs about messages (can be SCTP messages or direct messages).
    Message,
}

impl WorkerLogTag {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Ice => "ice",
            Self::Dtls => "dtls",
            Self::Rtp => "rtp",
            Self::Srtp => "srtp",
            Self::Rtcp => "rtcp",
            Self::Rtx => "rtx",
            Self::Bwe => "bwe",
            Self::Score => "score",
            Self::Simulcast => "simulcast",
            Self::Svc => "svc",
            Self::Sctp => "sctp",
            Self::Message => "message",
        }
    }
}

/// DTLS certificate and private key.
#[derive(Debug, Clone)]
pub struct WorkerDtlsFiles {
    /// Path to the DTLS public certificate file in PEM format.
    pub certificate: PathBuf,
    /// Path to the DTLS certificate private key file in PEM format.
    pub private_key: PathBuf,
}

/// Settings for worker to be created with.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct WorkerSettings {
    pub app_data: AppData,
    /// Logging level for logs generated by the media worker subprocesses.
    ///
    /// Default [`WorkerLogLevel::Error`].
    pub log_level: WorkerLogLevel,
    /// Log tags for debugging. Check the meaning of each available tag in the
    /// [Debugging](https://mediasoup.org/documentation/v3/mediasoup/debugging/) documentation.
    pub log_tags: Vec<WorkerLogTag>,
    /// RTC ports range for ICE, DTLS, RTP, etc. Default 10000..=59999.
    pub rtc_ports_range: RangeInclusive<u16>,
    /// DTLS certificate and private key.
    ///
    /// If `None`, a certificate is dynamically created.
    pub dtls_files: Option<WorkerDtlsFiles>,
}

impl Default for WorkerSettings {
    fn default() -> Self {
        Self {
            app_data: AppData::default(),
            log_level: WorkerLogLevel::default(),
            log_tags: Vec::new(),
            rtc_ports_range: 10000..=59999,
            dtls_files: None,
        }
    }
}

/// Worker settings that can be updated in runtime.
#[derive(Default, Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct WorkerUpdateSettings {
    /// Logging level for logs generated by the media worker subprocesses.
    ///
    /// If `None`, logging level will not be updated.
    pub log_level: Option<WorkerLogLevel>,
    /// Log tags for debugging. Check the meaning of each available tag in the
    /// [Debugging](https://mediasoup.org/documentation/v3/mediasoup/debugging/) documentation.
    ///
    /// If `None`, log tags will not be updated.
    pub log_tags: Option<Vec<WorkerLogTag>>,
}

/// CPU, memory and other resource usage information from mediasoup-worker subprocess.
#[derive(Debug, Copy, Clone, Deserialize)]
#[non_exhaustive]
pub struct WorkerResourceUsage {
    /// User CPU time used (in ms).
    pub ru_utime: u64,
    /// System CPU time used (in ms).
    pub ru_stime: u64,
    /// Maximum resident set size.
    pub ru_maxrss: u64,
    /// Integral shared memory size.
    pub ru_ixrss: u64,
    /// Integral unshared data size.
    pub ru_idrss: u64,
    /// Integral unshared stack size.
    pub ru_isrss: u64,
    /// Page reclaims (soft page faults).
    pub ru_minflt: u64,
    /// Page faults (hard page faults).
    pub ru_majflt: u64,
    /// Swaps.
    pub ru_nswap: u64,
    /// Block input operations.
    pub ru_inblock: u64,
    /// Block output operations.
    pub ru_oublock: u64,
    /// IPC messages sent.
    pub ru_msgsnd: u64,
    /// IPC messages received.
    pub ru_msgrcv: u64,
    /// Signals received.
    pub ru_nsignals: u64,
    /// Voluntary context switches.
    pub ru_nvcsw: u64,
    /// Involuntary context switches.
    pub ru_nivcsw: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[doc(hidden)]
#[non_exhaustive]
pub struct WorkerDump {
    pub pid: u32,
    pub router_ids: Vec<RouterId>,
}

/// Error that caused [`Worker::create_router`] to fail.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum CreateRouterError {
    /// RTP capabilities generation error
    #[error("RTP capabilities generation error: {0}")]
    FailedRtpCapabilitiesGeneration(RtpCapabilitiesError),
    /// Request to worker failed
    #[error("Request to worker failed: {0}")]
    Request(RequestError),
}

#[derive(Default)]
struct Handlers {
    new_router: Bag<Box<dyn Fn(&Router) + Send + Sync>>,
    dead: BagOnce<Box<dyn FnOnce(ExitStatus) + Send>>,
    close: BagOnce<Box<dyn FnOnce() + Send>>,
}

struct Inner {
    channel: Channel,
    payload_channel: PayloadChannel,
    child: Child,
    executor: Arc<Executor<'static>>,
    pid: u32,
    handlers: Handlers,
    app_data: AppData,
    closed: Arc<AtomicBool>,
    // Make sure worker is not dropped until this worker manager is not dropped
    _worker_manager: WorkerManager,
}

impl Drop for Inner {
    fn drop(&mut self) {
        debug!("drop()");

        let already_closed = self.closed.swap(true, Ordering::SeqCst);

        if matches!(self.child.try_status(), Ok(None)) {
            unsafe {
                libc::kill(self.pid as libc::pid_t, libc::SIGTERM);
            }
        }

        if !already_closed {
            self.handlers.close.call_simple();
        }
    }
}

impl Inner {
    async fn new(
        executor: Arc<Executor<'static>>,
        worker_binary: PathBuf,
        WorkerSettings {
            app_data,
            log_level,
            log_tags,
            rtc_ports_range,
            dtls_files,
        }: WorkerSettings,
        worker_manager: WorkerManager,
    ) -> io::Result<Arc<Self>> {
        debug!("new()");

        let mut spawn_args: Vec<OsString> = Vec::new();
        let spawn_bin: PathBuf = match env::var("MEDIASOUP_USE_VALGRIND") {
            Ok(value) if value.as_str() == "true" => {
                let binary = match env::var("MEDIASOUP_VALGRIND_BIN") {
                    Ok(binary) => binary.into(),
                    _ => "valgrind".into(),
                };

                spawn_args.push(worker_binary.into_os_string());

                binary
            }
            _ => worker_binary,
        };

        spawn_args.push(format!("--logLevel={}", log_level.as_str()).into());
        for log_tag in log_tags {
            spawn_args.push(format!("--logTag={}", log_tag.as_str()).into());
        }

        if rtc_ports_range.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid RTC ports range",
            ));
        }
        spawn_args.push(format!("--rtcMinPort={}", rtc_ports_range.start()).into());
        spawn_args.push(format!("--rtcMaxPort={}", rtc_ports_range.end()).into());

        if let Some(dtls_files) = dtls_files {
            {
                let mut arg = OsString::new();
                arg.push("--dtlsCertificateFile=");
                arg.push(dtls_files.certificate);
                spawn_args.push(arg);
            }
            {
                let mut arg = OsString::new();
                arg.push("--dtlsPrivateKeyFile=");
                arg.push(dtls_files.private_key);
                spawn_args.push(arg);
            }
        }

        debug!(
            "spawning worker process: {} {}",
            spawn_bin.to_string_lossy(),
            spawn_args
                .iter()
                .map(|arg| arg.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" ")
        );

        let mut command = Command::new(spawn_bin);
        command
            .args(spawn_args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env("MEDIASOUP_VERSION", env!("CARGO_PKG_VERSION"));

        let SpawnResult {
            child,
            channel,
            payload_channel,
        } = utils::spawn_with_worker_channels(Arc::clone(&executor), &mut command)?;

        let pid = child.id();
        let handlers = Handlers::default();

        let mut inner = Self {
            channel,
            payload_channel,
            child,
            executor,
            pid,
            handlers,
            app_data,
            closed: Arc::new(AtomicBool::new(false)),
            _worker_manager: worker_manager,
        };

        inner.setup_output_forwarding();

        inner.setup_message_handling();

        inner.wait_for_worker_process().await?;

        let status_fut = inner.child.status();
        let inner = Arc::new(inner);
        {
            let inner_weak = Arc::downgrade(&inner);
            inner
                .executor
                .spawn(async move {
                    let status = status_fut.await;

                    if let Some(inner) = inner_weak.upgrade() {
                        if let Ok(exit_status) = status {
                            warn!("exit status {}", exit_status);

                            if !inner.closed.swap(true, Ordering::SeqCst) {
                                inner.handlers.dead.call(|callback| {
                                    callback(exit_status);
                                });
                                inner.handlers.close.call_simple();
                            }
                        }
                    }
                })
                .detach();
        }

        Ok(inner)
    }

    fn setup_output_forwarding(&mut self) {
        let stdout = self.child.stdout.take().unwrap();
        self.executor
            .spawn(async move {
                let mut lines = BufReader::new(stdout).lines();
                while let Some(Ok(line)) = lines.next().await {
                    debug!("(stdout) {}", line);
                }
            })
            .detach();

        let stderr = self.child.stderr.take().unwrap();
        let closed = Arc::clone(&self.closed);
        self.executor
            .spawn(async move {
                let mut lines = BufReader::new(stderr).lines();
                while let Some(Ok(line)) = lines.next().await {
                    if !closed.load(Ordering::SeqCst) {
                        error!("(stderr) {}", line);
                    }
                }
            })
            .detach();
    }

    async fn wait_for_worker_process(&mut self) -> io::Result<()> {
        let status = self.child.status();
        future::or(
            async move {
                let status = status.await?;
                let error_message = format!(
                    "worker process exited before being ready, exit status {}, code {:?}",
                    status,
                    status.code(),
                );
                Err(io::Error::new(io::ErrorKind::NotFound, error_message))
            },
            self.wait_for_worker_ready(),
        )
        .await
    }

    async fn wait_for_worker_ready(&mut self) -> io::Result<()> {
        #[derive(Deserialize)]
        #[serde(tag = "event", rename_all = "lowercase")]
        enum Notification {
            Running,
        }

        let (sender, receiver) = async_oneshot::oneshot();
        let pid = self.pid;
        let sender = Mutex::new(Some(sender));
        let _handler =
            self.channel
                .subscribe_to_notifications(self.pid.into(), move |notification| {
                    let result = match serde_json::from_value(notification.clone()) {
                        Ok(Notification::Running) => {
                            debug!("worker process running [pid:{}]", pid);
                            Ok(())
                        }
                        Err(error) => Err(io::Error::new(
                            io::ErrorKind::Other,
                            format!(
                            "unexpected first notification from worker [pid:{}]: {:?}; error = {}",
                            pid, notification, error
                        ),
                        )),
                    };
                    let _ = sender
                        .lock()
                        .take()
                        .expect("Receiving more than one worker notification")
                        .send(result);
                });

        receiver.await.map_err(|_closed| {
            io::Error::new(io::ErrorKind::Other, "Worker dropped before it is ready")
        })?
    }

    fn setup_message_handling(&mut self) {
        let channel_receiver = self.channel.get_internal_message_receiver();
        let payload_channel_receiver = self.payload_channel.get_internal_message_receiver();
        let pid = self.pid;
        let closed = Arc::clone(&self.closed);
        self.executor
            .spawn(async move {
                while let Ok(message) = channel_receiver.recv().await {
                    match message {
                        channel::InternalMessage::Debug(text) => debug!("[pid:{}] {}", pid, text),
                        channel::InternalMessage::Warn(text) => warn!("[pid:{}] {}", pid, text),
                        channel::InternalMessage::Error(text) => {
                            if !closed.load(Ordering::SeqCst) {
                                error!("[pid:{}] {}", pid, text)
                            }
                        }
                        channel::InternalMessage::Dump(text) => eprintln!("{}", text),
                        channel::InternalMessage::Unexpected(data) => error!(
                            "worker[pid:{}] unexpected channel data: {}",
                            pid,
                            String::from_utf8_lossy(&data)
                        ),
                    }
                }
            })
            .detach();

        self.executor
            .spawn(async move {
                while let Ok(message) = payload_channel_receiver.recv().await {
                    match message {
                        payload_channel::InternalMessage::UnexpectedData(data) => error!(
                            "worker[pid:{}] unexpected payload channel data: {}",
                            pid,
                            String::from_utf8_lossy(&data)
                        ),
                    }
                }
            })
            .detach();
    }
}

/// A worker represents a mediasoup C++ subprocess that runs on a single CPU core and handles
/// [`Router`] instances.
#[derive(Clone)]
pub struct Worker {
    inner: Arc<Inner>,
}

impl Worker {
    pub(super) async fn new(
        executor: Arc<Executor<'static>>,
        worker_binary: PathBuf,
        worker_settings: WorkerSettings,
        worker_manager: WorkerManager,
    ) -> io::Result<Self> {
        let inner = Inner::new(executor, worker_binary, worker_settings, worker_manager).await?;

        Ok(Self { inner })
    }

    /// The PID of the worker process.
    pub fn pid(&self) -> u32 {
        self.inner.pid
    }

    /// Custom application data.
    pub fn app_data(&self) -> &AppData {
        &self.inner.app_data
    }

    /// Whether the worker is closed.
    pub fn closed(&self) -> bool {
        self.inner.closed.load(Ordering::SeqCst)
    }

    /// Dump Worker.
    #[doc(hidden)]
    pub async fn dump(&self) -> Result<WorkerDump, RequestError> {
        debug!("dump()");

        self.inner.channel.request(WorkerDumpRequest {}).await
    }

    /// Provides resource usage of the mediasoup-worker subprocess.
    pub async fn get_resource_usage(&self) -> Result<WorkerResourceUsage, RequestError> {
        debug!("get_resource_usage()");

        self.inner
            .channel
            .request(WorkerGetResourceRequest {})
            .await
    }

    /// Updates the worker settings in runtime. Just a subset of the worker settings can be updated.
    pub async fn update_settings(&self, data: WorkerUpdateSettings) -> Result<(), RequestError> {
        debug!("update_settings()");

        self.inner
            .channel
            .request(WorkerUpdateSettingsRequest { data })
            .await
    }

    /// Create a Router.
    ///
    /// Worker will be kept alive as long as at least one router instance is alive.
    pub async fn create_router(
        &self,
        router_options: RouterOptions,
    ) -> Result<Router, CreateRouterError> {
        debug!("create_router()");

        let RouterOptions {
            app_data,
            media_codecs,
        } = router_options;

        let rtp_capabilities = ortc::generate_router_rtp_capabilities(media_codecs)
            .map_err(CreateRouterError::FailedRtpCapabilitiesGeneration)?;

        let router_id = RouterId::new();
        let internal = RouterInternal { router_id };

        let _buffer_guard = self.inner.channel.buffer_messages_for(router_id.into());

        self.inner
            .channel
            .request(WorkerCreateRouterRequest { internal })
            .await
            .map_err(CreateRouterError::Request)?;

        let router = Router::new(
            router_id,
            Arc::clone(&self.inner.executor),
            self.inner.channel.clone(),
            self.inner.payload_channel.clone(),
            rtp_capabilities,
            app_data,
            self.clone(),
        );

        self.inner.handlers.new_router.call(|callback| {
            callback(&router);
        });

        Ok(router)
    }

    /// Callback is called when a new router is created.
    pub fn on_new_router<F: Fn(&Router) + Send + Sync + 'static>(&self, callback: F) -> HandlerId {
        self.inner.handlers.new_router.add(Box::new(callback))
    }

    /// Callback is called when the worker process unexpectedly dies.
    pub fn on_dead<F: FnOnce(ExitStatus) + Send + Sync + 'static>(&self, callback: F) -> HandlerId {
        self.inner.handlers.dead.add(Box::new(callback))
    }

    /// Callback is called when the worker is closed for whatever reason.
    ///
    /// NOTE: Callback will be called in place if worker is already closed.
    pub fn on_close<F: FnOnce() + Send + 'static>(&self, callback: F) -> HandlerId {
        let handler_id = self.inner.handlers.close.add(Box::new(callback));
        if self.inner.closed.load(Ordering::Relaxed) {
            self.inner.handlers.close.call_simple();
        }
        handler_id
    }
}
