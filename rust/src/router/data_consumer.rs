//! A data consumer represents an endpoint capable of receiving data messages from a mediasoup
//! [`Router`](crate::router::Router).
//!
//! A data consumer can use [SCTP](https://tools.ietf.org/html/rfc4960) (AKA
//! DataChannel) to receive those messages, or can directly receive them in the Rust application if
//! the data consumer was created on top of a
//! [`DirectTransport`](crate::direct_transport::DirectTransport).

use crate::data_producer::DataProducerId;
use crate::data_structures::{AppData, WebRtcMessage};
use crate::messages::{
    DataConsumerCloseRequest, DataConsumerDumpRequest, DataConsumerGetBufferedAmountRequest,
    DataConsumerGetStatsRequest, DataConsumerInternal, DataConsumerSendRequest,
    DataConsumerSendRequestData, DataConsumerSetBufferedAmountLowThresholdData,
    DataConsumerSetBufferedAmountLowThresholdRequest,
};
use crate::sctp_parameters::SctpStreamParameters;
use crate::transport::Transport;
use crate::uuid_based_wrapper_type;
use crate::worker::{
    Channel, NotificationMessage, PayloadChannel, RequestError, SubscriptionHandler,
};
use async_executor::Executor;
use event_listener_primitives::{Bag, BagOnce, HandlerId};
use log::*;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};

uuid_based_wrapper_type!(
    /// Data consumer identifier.
    DataConsumerId
);

/// Data consumer options.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DataConsumerOptions {
    // The id of the DataProducer to consume.
    pub(super) data_producer_id: DataProducerId,
    /// Just if consuming over SCTP.
    /// Whether data messages must be received in order. If true the messages will be sent reliably.
    /// Defaults to the value in the DataProducer if it has type `Sctp` or to true if it has type
    /// `Direct`.
    pub(super) ordered: Option<bool>,
    /// Just if consuming over SCTP.
    /// When ordered is false indicates the time (in milliseconds) after which a SCTP packet will
    /// stop being retransmitted.
    /// Defaults to the value in the DataProducer if it has type `Sctp` or unset if it has type
    /// `Direct`.
    pub(super) max_packet_life_time: Option<u16>,
    /// Just if consuming over SCTP.
    /// When ordered is false indicates the maximum number of times a packet will be retransmitted.
    /// Defaults to the value in the DataProducer if it has type `Sctp` or unset if it has type
    /// `Direct`.
    pub(super) max_retransmits: Option<u16>,
    /// Custom application data.
    pub app_data: AppData,
}

impl DataConsumerOptions {
    /// Inherits parameters of corresponding DataProducer.
    pub fn new_sctp(data_producer_id: DataProducerId) -> Self {
        Self {
            data_producer_id,
            ordered: None,
            max_packet_life_time: None,
            max_retransmits: None,
            app_data: AppData::default(),
        }
    }

    /// For DirectTransport.
    pub fn new_direct(data_producer_id: DataProducerId) -> Self {
        Self {
            data_producer_id,
            ordered: Some(true),
            max_packet_life_time: None,
            max_retransmits: None,
            app_data: AppData::default(),
        }
    }

    /// Messages will be sent reliably in order.
    pub fn new_sctp_ordered(data_producer_id: DataProducerId) -> Self {
        Self {
            data_producer_id,
            ordered: None,
            max_packet_life_time: None,
            max_retransmits: None,
            app_data: AppData::default(),
        }
    }

    /// Messages will be sent unreliably with time (in milliseconds) after which a SCTP packet will
    /// stop being retransmitted.
    pub fn new_sctp_unordered_with_life_time(
        data_producer_id: DataProducerId,
        max_packet_life_time: u16,
    ) -> Self {
        Self {
            data_producer_id,
            ordered: None,
            max_packet_life_time: Some(max_packet_life_time),
            max_retransmits: None,
            app_data: AppData::default(),
        }
    }

    /// Messages will be sent unreliably with a limited number of retransmission attempts.
    pub fn new_sctp_unordered_with_retransmits(
        data_producer_id: DataProducerId,
        max_retransmits: u16,
    ) -> Self {
        Self {
            data_producer_id,
            ordered: None,
            max_packet_life_time: None,
            max_retransmits: Some(max_retransmits),
            app_data: AppData::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[doc(hidden)]
#[non_exhaustive]
pub struct DataConsumerDump {
    pub id: DataConsumerId,
    pub data_producer_id: DataProducerId,
    pub r#type: DataConsumerType,
    pub label: String,
    pub protocol: String,
    pub sctp_stream_parameters: Option<SctpStreamParameters>,
    pub buffered_amount_low_threshold: u32,
}

/// RTC statistics of the data consumer.
#[derive(Debug, Clone, PartialOrd, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct DataConsumerStat {
    // `type` field is present in worker, but ignored here
    pub timestamp: u64,
    pub label: String,
    pub protocol: String,
    pub messages_sent: usize,
    pub bytes_sent: usize,
    pub buffered_amount: u32,
}

/// Data consumer type.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DataConsumerType {
    /// The endpoint receives messages using the SCTP protocol.
    Sctp,
    /// Messages are received directly by the Rust process over a direct transport.
    Direct,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "event", rename_all = "lowercase", content = "data")]
enum Notification {
    DataProducerClose,
    SctpSendBufferFull,
    #[serde(rename_all = "camelCase")]
    BufferedAmountLow {
        buffered_amount: u32,
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "event", rename_all = "lowercase", content = "data")]
enum PayloadNotification {
    Message { ppid: u32 },
}

#[derive(Default)]
struct Handlers {
    message: Bag<Box<dyn Fn(&WebRtcMessage) + Send + Sync>>,
    sctp_send_buffer_full: Bag<Box<dyn Fn() + Send + Sync>>,
    buffered_amount_low: Bag<Box<dyn Fn(u32) + Send + Sync>>,
    data_producer_close: BagOnce<Box<dyn FnOnce() + Send>>,
    transport_close: BagOnce<Box<dyn FnOnce() + Send>>,
    close: BagOnce<Box<dyn FnOnce() + Send>>,
}

struct Inner {
    id: DataConsumerId,
    r#type: DataConsumerType,
    sctp_stream_parameters: Option<SctpStreamParameters>,
    label: String,
    protocol: String,
    data_producer_id: DataProducerId,
    direct: bool,
    executor: Arc<Executor<'static>>,
    channel: Channel,
    payload_channel: PayloadChannel,
    handlers: Arc<Handlers>,
    app_data: AppData,
    transport: Box<dyn Transport>,
    closed: AtomicBool,
    // Drop subscription to consumer-specific notifications when consumer itself is dropped
    _subscription_handlers: Vec<Option<SubscriptionHandler>>,
    _on_transport_close_handler: Mutex<HandlerId>,
}

impl Drop for Inner {
    fn drop(&mut self) {
        debug!("drop()");

        self.close();
    }
}

impl Inner {
    fn close(&self) {
        if !self.closed.swap(true, Ordering::SeqCst) {
            debug!("close()");

            self.handlers.close.call_simple();

            {
                let channel = self.channel.clone();
                let request = DataConsumerCloseRequest {
                    internal: DataConsumerInternal {
                        router_id: self.transport.router_id(),
                        transport_id: self.transport.id(),
                        data_consumer_id: self.id,
                        data_producer_id: self.data_producer_id,
                    },
                };
                let transport = self.transport.clone();
                self.executor
                    .spawn(async move {
                        if let Err(error) = channel.request(request).await {
                            error!("consumer closing failed on drop: {}", error);
                        }

                        drop(transport);
                    })
                    .detach();
            }
        }
    }
}

/// Data consumer created on transport other than
/// [`DirectTransport`](crate::direct_transport::DirectTransport).
#[derive(Clone)]
pub struct RegularDataConsumer {
    inner: Arc<Inner>,
}

impl From<RegularDataConsumer> for DataConsumer {
    fn from(producer: RegularDataConsumer) -> Self {
        DataConsumer::Regular(producer)
    }
}

/// Data consumer created on [`DirectTransport`](crate::direct_transport::DirectTransport).
#[derive(Clone)]
pub struct DirectDataConsumer {
    inner: Arc<Inner>,
}

impl From<DirectDataConsumer> for DataConsumer {
    fn from(producer: DirectDataConsumer) -> Self {
        DataConsumer::Direct(producer)
    }
}

/// A data consumer represents an endpoint capable of receiving data messages from a mediasoup
/// [`Router`](crate::router::Router).
///
/// A data consumer can use [SCTP](https://tools.ietf.org/html/rfc4960) (AKA
/// DataChannel) to receive those messages, or can directly receive them in the Rust application if
/// the data consumer was created on top of a
/// [`DirectTransport`](crate::direct_transport::DirectTransport).
#[derive(Clone)]
#[non_exhaustive]
pub enum DataConsumer {
    /// Data consumer created on transport other than
    /// [`DirectTransport`](crate::direct_transport::DirectTransport).
    Regular(RegularDataConsumer),
    /// Data consumer created on [`DirectTransport`](crate::direct_transport::DirectTransport).
    Direct(DirectDataConsumer),
}

impl DataConsumer {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        id: DataConsumerId,
        r#type: DataConsumerType,
        sctp_stream_parameters: Option<SctpStreamParameters>,
        label: String,
        protocol: String,
        data_producer_id: DataProducerId,
        executor: Arc<Executor<'static>>,
        channel: Channel,
        payload_channel: PayloadChannel,
        app_data: AppData,
        transport: Box<dyn Transport>,
        direct: bool,
    ) -> Self {
        debug!("new()");

        let handlers = Arc::<Handlers>::default();

        let inner_weak = Arc::<Mutex<Option<Weak<Inner>>>>::default();
        let subscription_handler = {
            let handlers = Arc::clone(&handlers);
            let inner_weak = Arc::clone(&inner_weak);

            channel.subscribe_to_notifications(id.into(), move |notification| {
                match serde_json::from_value::<Notification>(notification) {
                    Ok(notification) => match notification {
                        Notification::DataProducerClose => {
                            handlers.data_producer_close.call_simple();
                            if let Some(inner) = inner_weak
                                .lock()
                                .as_ref()
                                .and_then(|weak_inner| weak_inner.upgrade())
                            {
                                inner.close();
                            }
                        }
                        Notification::SctpSendBufferFull => {
                            handlers.sctp_send_buffer_full.call_simple();
                        }
                        Notification::BufferedAmountLow { buffered_amount } => {
                            handlers.buffered_amount_low.call(|callback| {
                                callback(buffered_amount);
                            });
                        }
                    },
                    Err(error) => {
                        error!("Failed to parse notification: {}", error);
                    }
                }
            })
        };

        let payload_subscription_handler = {
            let handlers = Arc::clone(&handlers);

            payload_channel.subscribe_to_notifications(id.into(), move |notification| {
                let NotificationMessage { message, payload } = notification;
                match serde_json::from_value::<PayloadNotification>(message) {
                    Ok(notification) => match notification {
                        PayloadNotification::Message { ppid } => {
                            let message = WebRtcMessage::new(ppid, payload);

                            handlers.message.call(|callback| {
                                callback(&message);
                            });
                        }
                    },
                    Err(error) => {
                        error!("Failed to parse payload notification: {}", error);
                    }
                }
            })
        };

        let on_transport_close_handler = transport.on_close({
            let inner_weak = Arc::clone(&inner_weak);

            Box::new(move || {
                if let Some(inner) = inner_weak
                    .lock()
                    .as_ref()
                    .and_then(|weak_inner| weak_inner.upgrade())
                {
                    inner.handlers.transport_close.call_simple();
                    inner.close();
                }
            })
        });
        let inner = Arc::new(Inner {
            id,
            r#type,
            sctp_stream_parameters,
            label,
            protocol,
            data_producer_id,
            direct,
            executor,
            channel,
            payload_channel,
            handlers,
            app_data,
            transport,
            closed: AtomicBool::new(false),
            _subscription_handlers: vec![subscription_handler, payload_subscription_handler],
            _on_transport_close_handler: Mutex::new(on_transport_close_handler),
        });

        inner_weak.lock().replace(Arc::downgrade(&inner));

        if direct {
            Self::Direct(DirectDataConsumer { inner })
        } else {
            Self::Regular(RegularDataConsumer { inner })
        }
    }

    /// Data consumer identifier.
    pub fn id(&self) -> DataConsumerId {
        self.inner().id
    }

    /// The associated data producer identifier.
    pub fn data_producer_id(&self) -> DataProducerId {
        self.inner().data_producer_id
    }

    /// The type of the data consumer.
    pub fn r#type(&self) -> DataConsumerType {
        self.inner().r#type
    }

    /// The SCTP stream parameters (just if the data consumer type is `Sctp`).
    pub fn sctp_stream_parameters(&self) -> Option<SctpStreamParameters> {
        self.inner().sctp_stream_parameters
    }

    /// The data consumer label.
    pub fn label(&self) -> &String {
        &self.inner().label
    }

    /// The data consumer sub-protocol.
    pub fn protocol(&self) -> &String {
        &self.inner().protocol
    }

    /// Custom application data.
    pub fn app_data(&self) -> &AppData {
        &self.inner().app_data
    }

    /// Whether the data consumer is closed.
    pub fn closed(&self) -> bool {
        self.inner().closed.load(Ordering::SeqCst)
    }

    /// Dump DataConsumer.
    #[doc(hidden)]
    pub async fn dump(&self) -> Result<DataConsumerDump, RequestError> {
        debug!("dump()");

        self.inner()
            .channel
            .request(DataConsumerDumpRequest {
                internal: self.get_internal(),
            })
            .await
    }

    /// Returns current statistics of the data consumer.
    ///
    /// Check the [RTC Statistics](https://mediasoup.org/documentation/v3/mediasoup/rtc-statistics/)
    /// section for more details (TypeScript-oriented, but concepts apply here as well).
    pub async fn get_stats(&self) -> Result<Vec<DataConsumerStat>, RequestError> {
        debug!("get_stats()");

        self.inner()
            .channel
            .request(DataConsumerGetStatsRequest {
                internal: self.get_internal(),
            })
            .await
    }

    /// Returns the number of bytes of data currently buffered to be sent over the underlying SCTP
    /// association.
    ///
    /// # Notes on usage
    /// The underlying SCTP association uses a common send buffer for all data consumers, hence the
    /// value given by this method indicates the data buffered for all data consumers in the
    /// transport.
    pub async fn get_buffered_amount(&self) -> Result<u32, RequestError> {
        debug!("get_buffered_amount()");

        let response = self
            .inner()
            .channel
            .request(DataConsumerGetBufferedAmountRequest {
                internal: self.get_internal(),
            })
            .await?;

        Ok(response.buffered_amount)
    }

    // Whenever the underlying SCTP association buffered bytes drop to this value,
    // `on_buffered_amount_low` callback is called.
    pub async fn set_buffered_amount_low_threshold(
        &self,
        threshold: u32,
    ) -> Result<(), RequestError> {
        debug!(
            "set_buffered_amount_low_threshold() [threshold:{}]",
            threshold
        );

        self.inner()
            .channel
            .request(DataConsumerSetBufferedAmountLowThresholdRequest {
                internal: self.get_internal(),
                data: DataConsumerSetBufferedAmountLowThresholdData { threshold },
            })
            .await
    }

    /// Callback is called when a message has been received from the corresponding data producer.
    ///
    /// # Notes on usage
    /// Just available in direct transports, this is, those created via
    /// [`Router::create_direct_transport`](crate::router::Router::create_direct_transport).
    pub fn on_message<F: Fn(&WebRtcMessage) + Send + Sync + 'static>(
        &self,
        callback: F,
    ) -> HandlerId {
        self.inner().handlers.message.add(Box::new(callback))
    }

    /// Callback is called when a message could not be sent because the SCTP send buffer was full.
    pub fn on_sctp_send_buffer_full<F: Fn() + Send + Sync + 'static>(
        &self,
        callback: F,
    ) -> HandlerId {
        self.inner()
            .handlers
            .sctp_send_buffer_full
            .add(Box::new(callback))
    }

    /// Emitted when the underlying SCTP association buffered bytes drop down to the value set with
    /// [`DataConsumer::set_buffered_amount_low_threshold`].
    ///
    /// # Notes on usage
    /// Only applicable for consumers of type `Sctp`.
    pub fn on_buffered_amount_low<F: Fn(u32) + Send + Sync + 'static>(
        &self,
        callback: F,
    ) -> HandlerId {
        self.inner()
            .handlers
            .buffered_amount_low
            .add(Box::new(callback))
    }

    /// Callback is called when the associated data producer is closed for whatever reason. The data
    /// consumer itself is also closed.
    pub fn on_data_producer_close<F: FnOnce() + Send + 'static>(&self, callback: F) -> HandlerId {
        self.inner()
            .handlers
            .data_producer_close
            .add(Box::new(callback))
    }

    /// Callback is called when the transport this data consumer belongs to is closed for whatever
    /// reason. The data consumer itself is also closed.
    pub fn on_transport_close<F: FnOnce() + Send + 'static>(&self, callback: F) -> HandlerId {
        self.inner()
            .handlers
            .transport_close
            .add(Box::new(callback))
    }

    /// Callback is called when the data consumer is closed for whatever reason.
    ///
    /// NOTE: Callback will be called in place if data consumer is already closed.
    pub fn on_close<F: FnOnce() + Send + 'static>(&self, callback: F) -> HandlerId {
        let handler_id = self.inner().handlers.close.add(Box::new(callback));
        if self.inner().closed.load(Ordering::Relaxed) {
            self.inner().handlers.close.call_simple();
        }
        handler_id
    }

    /// Downgrade `DataConsumer` to [`WeakDataConsumer`] instance.
    pub fn downgrade(&self) -> WeakDataConsumer {
        WeakDataConsumer {
            inner: Arc::downgrade(&self.inner()),
        }
    }

    fn inner(&self) -> &Arc<Inner> {
        match self {
            DataConsumer::Regular(data_consumer) => &data_consumer.inner,
            DataConsumer::Direct(data_consumer) => &data_consumer.inner,
        }
    }

    fn get_internal(&self) -> DataConsumerInternal {
        DataConsumerInternal {
            router_id: self.inner().transport.router_id(),
            transport_id: self.inner().transport.id(),
            data_consumer_id: self.inner().id,
            data_producer_id: self.inner().data_producer_id,
        }
    }
}

impl DirectDataConsumer {
    /// Sends direct messages from the Rust process.
    pub async fn send(&self, message: WebRtcMessage) -> Result<(), RequestError> {
        let (ppid, payload) = message.into_ppid_and_payload();

        self.inner
            .payload_channel
            .request(
                DataConsumerSendRequest {
                    internal: DataConsumerInternal {
                        router_id: self.inner.transport.router_id(),
                        transport_id: self.inner.transport.id(),
                        data_consumer_id: self.inner.id,
                        data_producer_id: self.inner.data_producer_id,
                    },
                    data: DataConsumerSendRequestData { ppid },
                },
                payload,
            )
            .await
    }
}

/// [`WeakDataConsumer`] doesn't own data consumer instance on mediasoup-worker and will not prevent
/// one from being destroyed once last instance of regular [`DataConsumer`] is dropped.
///
/// [`WeakDataConsumer`] vs [`DataConsumer`] is similar to [`Weak`] vs [`Arc`].
#[derive(Clone)]
pub struct WeakDataConsumer {
    inner: Weak<Inner>,
}

impl WeakDataConsumer {
    /// Attempts to upgrade `WeakDataConsumer` to [`DataConsumer`] if last instance of one wasn't
    /// dropped yet.
    pub fn upgrade(&self) -> Option<DataConsumer> {
        let inner = self.inner.upgrade()?;

        let data_consumer = if inner.direct {
            DataConsumer::Direct(DirectDataConsumer { inner })
        } else {
            DataConsumer::Regular(RegularDataConsumer { inner })
        };

        Some(data_consumer)
    }
}
