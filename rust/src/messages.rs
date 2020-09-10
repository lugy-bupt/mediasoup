use crate::data_structures::*;
use crate::router::RouterDumpResponse;
use crate::worker::{WorkerDumpResponse, WorkerResourceUsage, WorkerUpdateSettings};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::marker::PhantomData;

pub(crate) trait Request: Debug + Serialize {
    type Response: DeserializeOwned;

    fn as_method(&self) -> &'static str;
}

macro_rules! request_response {
    (
        $method: literal,
        $request_struct_name: ident { $( $field_name: ident: $field_type: ty, )* },
        $existing_response_type: ty $(,)?
    ) => {
        #[derive(Debug, Serialize)]
        pub(crate) struct $request_struct_name {
            $( pub(crate) $field_name: $field_type, )*
        }

        impl Request for $request_struct_name {
            type Response = $existing_response_type;

            fn as_method(&self) -> &'static str {
                $method
            }
        }
    };
    (
        $method: literal,
        $request_struct_name: ident $request_struct_impl: tt $(,)?
    ) => {
        // Call above macro with unit type as expected response
        request_response!($method, $request_struct_name $request_struct_impl, ());
    };
    (
        $method: literal,
        $request_struct_name: ident { $( $field_name: ident: $field_type: ty, )* },
        $response_struct_name: ident $response_struct_impl: tt,
    ) => {
        #[derive(Debug, Serialize)]
        pub(crate) struct $request_struct_name {
            $( pub(crate) $field_name: $field_type, )*
        }

        #[derive(Debug, Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub(crate) struct $response_struct_name $response_struct_impl

        impl Request for $request_struct_name {
            type Response = $response_struct_name;

            fn as_method(&self) -> &'static str {
                $method
            }
        }
    };
}

macro_rules! request_response_generic {
    (
        $method: literal,
        $request_struct_name: ident { $( $field_name: ident: $field_type: ty, )* },
        $generic_response: ident,
    ) => {
        #[derive(Debug, Serialize)]
        pub(crate) struct $request_struct_name<$generic_response>
        where
            $generic_response: Debug + DeserializeOwned,
        {
            $( pub(crate) $field_name: $field_type, )*
            #[serde(skip)]
            pub(crate) phantom_data: PhantomData<$generic_response>,
        }

        impl<$generic_response: Debug + DeserializeOwned> Request for $request_struct_name<$generic_response> {
            type Response = $generic_response;

            fn as_method(&self) -> &'static str {
                $method
            }
        }
    };
}

request_response!("worker.dump", WorkerDumpRequest {}, WorkerDumpResponse);

request_response!(
    "worker.getResourceUsage",
    WorkerGetResourceRequest {},
    WorkerResourceUsage,
);

request_response!(
    "worker.updateSettings",
    WorkerUpdateSettingsRequest {
        data: WorkerUpdateSettings,
    },
);

request_response!(
    "worker.createRouter",
    WorkerCreateRouterRequest {
        internal: RouterInternal,
    },
);

request_response!(
    "router.close",
    RouterCloseRequest {
        internal: RouterInternal,
    },
);

request_response!(
    "router.dump",
    RouterDumpRequest {
        internal: RouterInternal,
    },
    RouterDumpResponse,
);

request_response!(
    "router.createWebRtcTransport",
    RouterCreateWebrtcTransportRequest {
        internal: TransportInternal,
        data: RouterCreateWebrtcTransportData,
    },
    WebRtcTransportData,
);

request_response!(
    "router.createPlainTransport",
    RouterCreatePlainTransportRequest {
        internal: TransportInternal,
        data: RouterCreatePlainTransportData,
    },
    RouterCreatePlainTransportResponse {
        // TODO
    },
);

request_response!(
    "router.createPipeTransport",
    RouterCreatePipeTransportRequest {
        internal: TransportInternal,
        data: RouterCreatePipeTransportData,
    },
    RouterCreatePipeTransportResponse {
        // TODO
    },
);

request_response!(
    "router.createDirectTransport",
    RouterCreateDirectTransportRequest {
        internal: TransportInternal,
        data: RouterCreateDirectTransportData,
    },
    RouterCreateDirectTransportResponse {
        // TODO
    },
);

request_response!(
    "router.createAudioLevelObserver",
    RouterCreateAudioLevelObserverRequest {
        internal: RouterCreateAudioLevelObserverInternal,
        data: RouterCreateAudioLevelObserverData,
    },
    RouterCreateAudioLevelObserverResponse {
        // TODO
    },
);

request_response!(
    "transport.close",
    TransportCloseRequest {
        internal: TransportInternal,
    },
    TransportCloseResponse {
        // TODO
    },
);

request_response_generic!(
    "transport.dump",
    TransportDumpRequest {
        internal: TransportInternal,
    },
    Dump,
);

request_response_generic!(
    "transport.getStats",
    TransportGetStatsRequest {
        internal: TransportInternal,
    },
    Stats,
);

request_response!(
    "transport.connect",
    TransportConnectRequest {
        internal: TransportInternal,
        data: TransportConnectData,
    },
    TransportConnectResponse {
        // TODO
    },
);

request_response!(
    "transport.setMaxIncomingBitrate",
    TransportSetMaxIncomingBitrateRequest {
        internal: TransportInternal,
        data: TransportSetMaxIncomingBitrateData,
    },
    TransportSetMaxIncomingBitrateResponse {
        // TODO
    },
);

// TODO: Detail remaining methods, I got bored for now
request_response!(
    "transport.restartIce",
    TransportRestartIceRequest {
        internal: TransportInternal,
    },
    TransportRestartIceResponse {
        // TODO
    },
);

// request_response!(
//     TransportProduceRequest,
//     "transport.produce",
//     ;,
//     TransportProduceResponse,
//     {
//         // TODO
//     },
// );
//
// request_response!(
//     TransportConsumeRequest,
//     "transport.consume",
//     ;,
//     TransportConsumeResponse,
//     {
//         // TODO
//     },
// );
//
// request_response!(
//     TransportProduceDataRequest,
//     "transport.produceData",
//     ;,
//     TransportProduceDataResponse,
//     {
//         // TODO
//     },
// );
//
// request_response!(
//     TransportConsumeDataRequest,
//     "transport.consumeData",
//     ;,
//     TransportConsumeDataResponse,
//     {
//         // TODO
//     },
// );
//
// request_response!(
//     TransportEnableTraceEventRequest,
//     "transport.enableTraceEvent",
//     ;,
//     TransportEnableTraceEventResponse,
//     {
//         // TODO
//     },
// );
//
// request_response!(
//     ProducerCloseRequest,
//     "producer.close",
//     ;,
//     ProducerCloseResponse,
//     {
//         // TODO
//     },
// );
//
// request_response!(
//     ProducerDumpRequest,
//     "producer.dump",
//     ;,
//     ProducerDumpResponse,
//     {
//         // TODO
//     },
// );
//
// request_response!(
//     ProducerGetStatsRequest,
//     "producer.getStats",
//     ;,
//     ProducerGetStatsResponse,
//     {
//         // TODO
//     },
// );
//
// request_response!(
//     ProducerPauseRequest,
//     "producer.pause",
//     ;,
//     ProducerPauseResponse,
//     {
//         // TODO
//     },
// );
//
// request_response!(
//     ProducerResumeRequest,
//     "producer.resume",
//     ;,
//     ProducerResumeResponse,
//     {
//         // TODO
//     },
// );
//
// request_response!(
//     ProducerEnableTraceEventRequest,
//     "producer.enableTraceEvent",
//     ;,
//     ProducerEnableTraceEventResponse,
//     {
//         // TODO
//     },
// );
//
// request_response!(
//     ConsumerCloseRequest,
//     "consumer.close",
//     ;,
//     ConsumerCloseResponse,
//     {
//         // TODO
//     },
// );
//
// request_response!(
//     ConsumerDumpRequest,
//     "consumer.dump",
//     ;,
//     ConsumerDumpResponse,
//     {
//         // TODO
//     },
// );
//
// request_response!(
//     ConsumerGetStatsRequest,
//     "consumer.getStats",
//     ;,
//     ConsumerGetStatsResponse,
//     {
//         // TODO
//     },
// );
//
// request_response!(
//     ConsumerPauseRequest,
//     "consumer.pause",
//     ;,
//     ConsumerPauseResponse,
//     {
//         // TODO
//     },
// );
//
// request_response!(
//     ConsumerResumeRequest,
//     "consumer.resume",
//     ;,
//     ConsumerResumeResponse,
//     {
//         // TODO
//     },
// );
//
// request_response!(
//     ConsumerSetPreferredLayersRequest,
//     "consumer.setPreferredLayers",
//     ;,
//     ConsumerSetPreferredLayersResponse,
//     {
//         // TODO
//     },
// );
//
// request_response!(
//     ConsumerSetPriorityRequest,
//     "consumer.setPriority",
//     ;,
//     ConsumerSetPriorityResponse,
//     {
//         // TODO
//     },
// );
//
// request_response!(
//     ConsumerRequestKeyFrameRequest,
//     "consumer.requestKeyFrame",
//     ;,
//     ConsumerRequestKeyFrameResponse,
//     {
//         // TODO
//     },
// );
//
// request_response!(
//     ConsumerEnableTraceEventRequest,
//     "consumer.enableTraceEvent",
//     ;,
//     ConsumerEnableTraceEventResponse,
//     {
//         // TODO
//     },
// );
//
// request_response!(
//     DataProducerCloseRequest,
//     "dataProducer.close",
//     ;,
//     DataProducerCloseResponse,
//     {
//         // TODO
//     },
// );
//
// request_response!(
//     DataProducerDumpRequest,
//     "dataProducer.dump",
//     ;,
//     DataProducerDumpResponse,
//     {
//         // TODO
//     },
// );
//
// request_response!(
//     DataProducerGetStatsRequest,
//     "dataProducer.getStats",
//     ;,
//     DataProducerGetStatsResponse,
//     {
//         // TODO
//     },
// );
//
// request_response!(
//     DataConsumerCloseRequest,
//     "dataConsumer.close",
//     ;,
//     DataConsumerCloseResponse,
//     {
//         // TODO
//     },
// );
//
// request_response!(
//     DataConsumerDumpRequest,
//     "dataConsumer.dump",
//     ;,
//     DataConsumerDumpResponse,
//     {
//         // TODO
//     },
// );
//
// request_response!(
//     DataConsumerGetStatsRequest,
//     "dataConsumer.getStats",
//     ;,
//     DataConsumerGetStatsResponse,
//     {
//         // TODO
//     },
// );
//
// request_response!(
//     DataConsumerGetBufferedAmountRequest,
//     "dataConsumer.getBufferedAmount",
//     ;,
//     DataConsumerGetBufferedAmountResponse,
//     {
//         // TODO
//     },
// );
//
// request_response!(
//     DataConsumerSetBufferedAmountLowThresholdRequest,
//     "dataConsumer.setBufferedAmountLowThreshold",
//     ;,
//     DataConsumerSetBufferedAmountLowThresholdResponse,
//     {
//         // TODO
//     },
// );
//
// request_response!(
//     RtpObserverCloseRequest,
//     "rtpObserver.close",
//     ;,
//     RtpObserverCloseResponse,
//     {
//         // TODO
//     },
// );
//
// request_response!(
//     RtpObserverPauseRequest,
//     "rtpObserver.pause",
//     ;,
//     RtpObserverPauseResponse,
//     {
//         // TODO
//     },
// );
//
// request_response!(
//     RtpObserverResumeRequest,
//     "rtpObserver.resume",
//     ;,
//     RtpObserverResumeResponse,
//     {
//         // TODO
//     },
// );
//
// request_response!(
//     RtpObserverAddProducerRequest,
//     "rtpObserver.addProducer",
//     ;,
//     RtpObserverAddProducerResponse,
//     {
//         // TODO
//     },
// );
//
// request_response!(
//     RtpObserverRemoveProducerRequest,
//     "rtpObserver.removeProducer",
//     ;,
//     RtpObserverRemoveProducerResponse,
//     {
//         // TODO
//     },
// );
