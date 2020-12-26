//! RTP capabilities supported by Mediasoup.

use crate::rtp_parameters::{
    MediaKind, MimeTypeAudio, MimeTypeVideo, RtcpFeedback, RtpCapabilities, RtpCodecCapability,
    RtpCodecParametersParameters, RtpHeaderExtension, RtpHeaderExtensionDirection,
    RtpHeaderExtensionUri,
};
use std::num::{NonZeroU32, NonZeroU8};

/// Get a Mediasoup supported RTP capabilities.
///
/// # Notes on usage
/// Those are NOT the RTP capabilities needed by mediasoup-client's
/// [device.load()](https://mediasoup.org/documentation/v3/mediasoup-client/api/#device-load) and
/// libmediasoupclient's [device.Load()](https://mediasoup.org/documentation/v3/libmediasoupclient/api/#device-Load)
/// methods. There you must use [`Router::rtp_capabilities`](crate::router::Router::rtp_capabilities)
/// getter instead.
pub fn get_supported_rtp_capabilities() -> RtpCapabilities {
    RtpCapabilities {
        codecs: vec![
            RtpCodecCapability::Audio {
                mime_type: MimeTypeAudio::Opus,
                preferred_payload_type: None,
                clock_rate: NonZeroU32::new(48000).unwrap(),
                channels: NonZeroU8::new(2).unwrap(),
                parameters: RtpCodecParametersParameters::new(),
                rtcp_feedback: vec![RtcpFeedback::TransportCC],
            },
            RtpCodecCapability::Audio {
                mime_type: MimeTypeAudio::PCMU,
                preferred_payload_type: Some(0),
                clock_rate: NonZeroU32::new(8000).unwrap(),
                channels: NonZeroU8::new(1).unwrap(),
                parameters: RtpCodecParametersParameters::new(),
                rtcp_feedback: vec![RtcpFeedback::TransportCC],
            },
            RtpCodecCapability::Audio {
                mime_type: MimeTypeAudio::PCMA,
                preferred_payload_type: Some(8),
                clock_rate: NonZeroU32::new(8000).unwrap(),
                channels: NonZeroU8::new(1).unwrap(),
                parameters: RtpCodecParametersParameters::new(),
                rtcp_feedback: vec![RtcpFeedback::TransportCC],
            },
            RtpCodecCapability::Audio {
                mime_type: MimeTypeAudio::ISAC,
                preferred_payload_type: None,
                clock_rate: NonZeroU32::new(32000).unwrap(),
                channels: NonZeroU8::new(1).unwrap(),
                parameters: RtpCodecParametersParameters::new(),
                rtcp_feedback: vec![RtcpFeedback::TransportCC],
            },
            RtpCodecCapability::Audio {
                mime_type: MimeTypeAudio::ISAC,
                preferred_payload_type: None,
                clock_rate: NonZeroU32::new(16000).unwrap(),
                channels: NonZeroU8::new(1).unwrap(),
                parameters: RtpCodecParametersParameters::new(),
                rtcp_feedback: vec![RtcpFeedback::TransportCC],
            },
            RtpCodecCapability::Audio {
                mime_type: MimeTypeAudio::G722,
                preferred_payload_type: Some(9),
                clock_rate: NonZeroU32::new(8000).unwrap(),
                channels: NonZeroU8::new(1).unwrap(),
                parameters: RtpCodecParametersParameters::new(),
                rtcp_feedback: vec![RtcpFeedback::TransportCC],
            },
            RtpCodecCapability::Audio {
                mime_type: MimeTypeAudio::iLBC,
                preferred_payload_type: None,
                clock_rate: NonZeroU32::new(8000).unwrap(),
                channels: NonZeroU8::new(1).unwrap(),
                parameters: RtpCodecParametersParameters::new(),
                rtcp_feedback: vec![RtcpFeedback::TransportCC],
            },
            RtpCodecCapability::Audio {
                mime_type: MimeTypeAudio::SILK,
                preferred_payload_type: None,
                clock_rate: NonZeroU32::new(24000).unwrap(),
                channels: NonZeroU8::new(1).unwrap(),
                parameters: RtpCodecParametersParameters::new(),
                rtcp_feedback: vec![RtcpFeedback::TransportCC],
            },
            RtpCodecCapability::Audio {
                mime_type: MimeTypeAudio::SILK,
                preferred_payload_type: None,
                clock_rate: NonZeroU32::new(16000).unwrap(),
                channels: NonZeroU8::new(1).unwrap(),
                parameters: RtpCodecParametersParameters::new(),
                rtcp_feedback: vec![RtcpFeedback::TransportCC],
            },
            RtpCodecCapability::Audio {
                mime_type: MimeTypeAudio::SILK,
                preferred_payload_type: None,
                clock_rate: NonZeroU32::new(12000).unwrap(),
                channels: NonZeroU8::new(1).unwrap(),
                parameters: RtpCodecParametersParameters::new(),
                rtcp_feedback: vec![RtcpFeedback::TransportCC],
            },
            RtpCodecCapability::Audio {
                mime_type: MimeTypeAudio::SILK,
                preferred_payload_type: None,
                clock_rate: NonZeroU32::new(8000).unwrap(),
                channels: NonZeroU8::new(1).unwrap(),
                parameters: RtpCodecParametersParameters::new(),
                rtcp_feedback: vec![RtcpFeedback::TransportCC],
            },
            RtpCodecCapability::Audio {
                mime_type: MimeTypeAudio::CN,
                preferred_payload_type: Some(13),
                clock_rate: NonZeroU32::new(32000).unwrap(),
                channels: NonZeroU8::new(1).unwrap(),
                parameters: RtpCodecParametersParameters::new(),
                rtcp_feedback: vec![],
            },
            RtpCodecCapability::Audio {
                mime_type: MimeTypeAudio::CN,
                preferred_payload_type: Some(13),
                clock_rate: NonZeroU32::new(16000).unwrap(),
                channels: NonZeroU8::new(1).unwrap(),
                parameters: RtpCodecParametersParameters::new(),
                rtcp_feedback: vec![],
            },
            RtpCodecCapability::Audio {
                mime_type: MimeTypeAudio::CN,
                preferred_payload_type: Some(13),
                clock_rate: NonZeroU32::new(8000).unwrap(),
                channels: NonZeroU8::new(1).unwrap(),
                parameters: RtpCodecParametersParameters::new(),
                rtcp_feedback: vec![],
            },
            RtpCodecCapability::Audio {
                mime_type: MimeTypeAudio::TelephoneEvent,
                preferred_payload_type: None,
                clock_rate: NonZeroU32::new(48000).unwrap(),
                channels: NonZeroU8::new(1).unwrap(),
                parameters: RtpCodecParametersParameters::new(),
                rtcp_feedback: vec![],
            },
            RtpCodecCapability::Audio {
                mime_type: MimeTypeAudio::TelephoneEvent,
                preferred_payload_type: None,
                clock_rate: NonZeroU32::new(32000).unwrap(),
                channels: NonZeroU8::new(1).unwrap(),
                parameters: RtpCodecParametersParameters::new(),
                rtcp_feedback: vec![],
            },
            RtpCodecCapability::Audio {
                mime_type: MimeTypeAudio::TelephoneEvent,
                preferred_payload_type: None,
                clock_rate: NonZeroU32::new(16000).unwrap(),
                channels: NonZeroU8::new(1).unwrap(),
                parameters: RtpCodecParametersParameters::new(),
                rtcp_feedback: vec![],
            },
            RtpCodecCapability::Audio {
                mime_type: MimeTypeAudio::TelephoneEvent,
                preferred_payload_type: None,
                clock_rate: NonZeroU32::new(8000).unwrap(),
                channels: NonZeroU8::new(1).unwrap(),
                parameters: RtpCodecParametersParameters::new(),
                rtcp_feedback: vec![],
            },
            RtpCodecCapability::Video {
                mime_type: MimeTypeVideo::VP8,
                preferred_payload_type: None,
                clock_rate: NonZeroU32::new(90000).unwrap(),
                parameters: RtpCodecParametersParameters::new(),
                rtcp_feedback: vec![
                    RtcpFeedback::Nack,
                    RtcpFeedback::NackPli,
                    RtcpFeedback::CcmFir,
                    RtcpFeedback::GoogRemb,
                    RtcpFeedback::TransportCC,
                ],
            },
            RtpCodecCapability::Video {
                mime_type: MimeTypeVideo::VP9,
                preferred_payload_type: None,
                clock_rate: NonZeroU32::new(90000).unwrap(),
                parameters: RtpCodecParametersParameters::new(),
                rtcp_feedback: vec![
                    RtcpFeedback::Nack,
                    RtcpFeedback::NackPli,
                    RtcpFeedback::CcmFir,
                    RtcpFeedback::GoogRemb,
                    RtcpFeedback::TransportCC,
                ],
            },
            RtpCodecCapability::Video {
                mime_type: MimeTypeVideo::H264,
                preferred_payload_type: None,
                clock_rate: NonZeroU32::new(90000).unwrap(),
                parameters: RtpCodecParametersParameters::from([
                    ("packetization-mode", 1u32.into()),
                    ("level-asymmetry-allowed", 1u32.into()),
                ]),
                rtcp_feedback: vec![
                    RtcpFeedback::Nack,
                    RtcpFeedback::NackPli,
                    RtcpFeedback::CcmFir,
                    RtcpFeedback::GoogRemb,
                    RtcpFeedback::TransportCC,
                ],
            },
            RtpCodecCapability::Video {
                mime_type: MimeTypeVideo::H264,
                preferred_payload_type: None,
                clock_rate: NonZeroU32::new(90000).unwrap(),
                parameters: RtpCodecParametersParameters::from([
                    ("packetization-mode", 0u32.into()),
                    ("level-asymmetry-allowed", 1u32.into()),
                ]),
                rtcp_feedback: vec![
                    RtcpFeedback::Nack,
                    RtcpFeedback::NackPli,
                    RtcpFeedback::CcmFir,
                    RtcpFeedback::GoogRemb,
                    RtcpFeedback::TransportCC,
                ],
            },
            RtpCodecCapability::Video {
                mime_type: MimeTypeVideo::H265,
                preferred_payload_type: None,
                clock_rate: NonZeroU32::new(90000).unwrap(),
                parameters: RtpCodecParametersParameters::from([
                    ("packetization-mode", 1u32.into()),
                    ("level-asymmetry-allowed", 1u32.into()),
                ]),
                rtcp_feedback: vec![
                    RtcpFeedback::Nack,
                    RtcpFeedback::NackPli,
                    RtcpFeedback::CcmFir,
                    RtcpFeedback::GoogRemb,
                    RtcpFeedback::TransportCC,
                ],
            },
            RtpCodecCapability::Video {
                mime_type: MimeTypeVideo::H265,
                preferred_payload_type: None,
                clock_rate: NonZeroU32::new(90000).unwrap(),
                parameters: RtpCodecParametersParameters::from([
                    ("packetization-mode", 0u32.into()),
                    ("level-asymmetry-allowed", 1u32.into()),
                ]),
                rtcp_feedback: vec![
                    RtcpFeedback::Nack,
                    RtcpFeedback::NackPli,
                    RtcpFeedback::CcmFir,
                    RtcpFeedback::GoogRemb,
                    RtcpFeedback::TransportCC,
                ],
            },
        ],
        header_extensions: vec![
            RtpHeaderExtension {
                kind: Some(MediaKind::Audio),
                uri: RtpHeaderExtensionUri::MID,
                preferred_id: 1,
                preferred_encrypt: false,
                direction: RtpHeaderExtensionDirection::SendRecv,
            },
            RtpHeaderExtension {
                kind: Some(MediaKind::Video),
                uri: RtpHeaderExtensionUri::MID,
                preferred_id: 1,
                preferred_encrypt: false,
                direction: RtpHeaderExtensionDirection::SendRecv,
            },
            RtpHeaderExtension {
                kind: Some(MediaKind::Video),
                uri: RtpHeaderExtensionUri::RtpStreamId,
                preferred_id: 2,
                preferred_encrypt: false,
                direction: RtpHeaderExtensionDirection::RecvOnly,
            },
            RtpHeaderExtension {
                kind: Some(MediaKind::Video),
                uri: RtpHeaderExtensionUri::RepairRtpStreamId,
                preferred_id: 3,
                preferred_encrypt: false,
                direction: RtpHeaderExtensionDirection::RecvOnly,
            },
            RtpHeaderExtension {
                kind: Some(MediaKind::Audio),
                uri: RtpHeaderExtensionUri::AbsSendTime,
                preferred_id: 4,
                preferred_encrypt: false,
                direction: RtpHeaderExtensionDirection::SendRecv,
            },
            RtpHeaderExtension {
                kind: Some(MediaKind::Video),
                uri: RtpHeaderExtensionUri::AbsSendTime,
                preferred_id: 4,
                preferred_encrypt: false,
                direction: RtpHeaderExtensionDirection::SendRecv,
            },
            // NOTE: For audio we just enable transport-wide-cc-01 when receiving media.
            RtpHeaderExtension {
                kind: Some(MediaKind::Audio),
                uri: RtpHeaderExtensionUri::TransportWideCCDraft01,
                preferred_id: 5,
                preferred_encrypt: false,
                direction: RtpHeaderExtensionDirection::RecvOnly,
            },
            RtpHeaderExtension {
                kind: Some(MediaKind::Video),
                uri: RtpHeaderExtensionUri::TransportWideCCDraft01,
                preferred_id: 5,
                preferred_encrypt: false,
                direction: RtpHeaderExtensionDirection::SendRecv,
            },
            // NOTE: Remove this once framemarking draft becomes RFC.
            RtpHeaderExtension {
                kind: Some(MediaKind::Video),
                uri: RtpHeaderExtensionUri::FrameMarkingDraft07,
                preferred_id: 6,
                preferred_encrypt: false,
                direction: RtpHeaderExtensionDirection::SendRecv,
            },
            RtpHeaderExtension {
                kind: Some(MediaKind::Video),
                uri: RtpHeaderExtensionUri::FrameMarking,
                preferred_id: 7,
                preferred_encrypt: false,
                direction: RtpHeaderExtensionDirection::SendRecv,
            },
            RtpHeaderExtension {
                kind: Some(MediaKind::Audio),
                uri: RtpHeaderExtensionUri::AudioLevel,
                preferred_id: 10,
                preferred_encrypt: false,
                direction: RtpHeaderExtensionDirection::SendRecv,
            },
            RtpHeaderExtension {
                kind: Some(MediaKind::Video),
                uri: RtpHeaderExtensionUri::VideoOrientation,
                preferred_id: 11,
                preferred_encrypt: false,
                direction: RtpHeaderExtensionDirection::SendRecv,
            },
            RtpHeaderExtension {
                kind: Some(MediaKind::Video),
                uri: RtpHeaderExtensionUri::TimeOffset,
                preferred_id: 12,
                preferred_encrypt: false,
                direction: RtpHeaderExtensionDirection::SendRecv,
            },
        ],
        fec_mechanisms: vec![],
    }
}
