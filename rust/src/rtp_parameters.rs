use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The RTP capabilities define what mediasoup or an endpoint can receive at media level.
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RtpCapabilities {
    // TODO: Does this need to be optional or can be an empty vec?
    /// Supported media and RTX codecs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codecs: Option<Vec<RtpCodecCapability>>,
    // TODO: Does this need to be optional or can be an empty vec?
    /// Supported RTP header extensions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header_extensions: Option<Vec<RtpHeaderExtension>>,
    // TODO: Does this need to be optional or can be an empty vec?
    // TODO: Enum instead of string?
    /// Supported FEC mechanisms.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fec_mechanisms: Option<Vec<String>>,
}

/// Media kind
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaKind {
    Audio,
    Video,
}

// TODO: supportedRtpCapabilities.ts file and generally update TypeScript references
/// Provides information on the capabilities of a codec within the RTP capabilities. The list of
/// media codecs supported by mediasoup and their settings is defined in the
/// supportedRtpCapabilities.ts file.
///
/// Exactly one RtpCodecCapability will be present for each supported combination of parameters that
/// requires a distinct value of preferredPayloadType. For example:
///
/// - Multiple H264 codecs, each with their own distinct 'packetization-mode' and 'profile-level-id'
///   values.
/// - Multiple VP9 codecs, each with their own distinct 'profile-id' value.
///
/// RtpCodecCapability entries in the mediaCodecs array of RouterOptions do not require
/// preferredPayloadType field (if unset, mediasoup will choose a random one). If given, make sure
/// it's in the 96-127 range.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RtpCodecCapability {
    /// Media kind
    pub kind: MediaKind,
    // TODO: Enum?
    /// The codec MIME media type/subtype (e.g. 'audio/opus', 'video/VP8').
    pub mime_type: String,
    /// The preferred RTP payload type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_payload_type: Option<u32>,
    /// Codec clock rate expressed in Hertz.
    pub clock_rate: u32,
    /// The number of channels supported (e.g. two for stereo). Just for audio.
    /// Default 1.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channels: Option<u8>,
    // TODO: Not sure if this hashmap is a correct type
    /// Codec specific parameters. Some parameters (such as 'packetization-mode' and
    /// 'profile-level-id' in H264 or 'profile-id' in VP9) are critical for codec matching.
    pub parameters: HashMap<String, String>,
    // TODO: Does this need to be optional or can be an empty vec?
    /// Transport layer and codec-specific feedback messages for this codec.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rtcp_feedback: Option<Vec<RtcpFeedback>>,
}

/// Direction of RTP header extension.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RtpHeaderExtensionDirection {
    // TODO: Serialization of all of these variants should be lowercase if we ever need it
    SendRecv,
    SendOnly,
    RecvOnly,
    Inactive,
}

// TODO: supportedRtpCapabilities.ts file and generally update TypeScript references
/// Provides information relating to supported header extensions. The list of RTP header extensions
/// supported by mediasoup is defined in the supportedRtpCapabilities.ts file.
///
/// mediasoup does not currently support encrypted RTP header extensions. The direction field is
/// just present in mediasoup RTP capabilities (retrieved via router.rtpCapabilities or
/// mediasoup.getSupportedRtpCapabilities()). It's ignored if present in endpoints' RTP
/// capabilities.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RtpHeaderExtension {
    // TODO: TypeScript version makes this field both optional and possible to set to "",
    //  check if "" is actually needed
    /// Media kind. If `None`, it's valid for all kinds.
    /// Default any media kind.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<MediaKind>,
    /// The URI of the RTP header extension, as defined in RFC 5285.
    pub uri: String,
    /// The preferred numeric identifier that goes in the RTP packet. Must be unique.
    pub preferred_id: u16,
    /// If true, it is preferred that the value in the header be encrypted as per RFC 6904.
    /// Default false.
    pub preferred_encrypt: bool,
    /// If 'sendrecv', mediasoup supports sending and receiving this RTP extension. 'sendonly' means
    /// that mediasoup can send (but not receive) it. 'recvonly' means that mediasoup can receive
    /// (but not send) it.
    pub direction: RtpHeaderExtensionDirection,
}

/// The RTP send parameters describe a media stream received by mediasoup from
/// an endpoint through its corresponding mediasoup Producer. These parameters
/// may include a mid value that the mediasoup transport will use to match
/// received RTP packets based on their MID RTP extension value.
///
/// mediasoup allows RTP send parameters with a single encoding and with multiple
/// encodings (simulcast). In the latter case, each entry in the encodings array
/// must include a ssrc field or a rid field (the RID RTP extension value). Check
/// the Simulcast and SVC sections for more information.
///
/// The RTP receive parameters describe a media stream as sent by mediasoup to
/// an endpoint through its corresponding mediasoup Consumer. The mid value is
/// unset (mediasoup does not include the MID RTP extension into RTP packets
/// being sent to endpoints).
///
/// There is a single entry in the encodings array (even if the corresponding
/// producer uses simulcast). The consumer sends a single and continuous RTP
/// stream to the endpoint and spatial/temporal layer selection is possible via
/// consumer.setPreferredLayers().
///
/// As an exception, previous bullet is not true when consuming a stream over a
/// PipeTransport, in which all RTP streams from the associated producer are
/// forwarded verbatim through the consumer.
///
/// The RTP receive parameters will always have their ssrc values randomly
/// generated for all of its  encodings (and optional rtx: { ssrc: XXXX } if the
/// endpoint supports RTX), regardless of the original RTP send parameters in
/// the associated producer. This applies even if the producer's encodings have
/// rid set.
#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RtpParameters {
    /// The MID RTP extension value as defined in the BUNDLE specification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mid: Option<String>,
    /// Media and RTX codecs in use.
    pub codecs: Vec<RtpCodecParameters>,
    // TODO: Does this need to be optional or can be an empty vec?
    /// RTP header extensions in use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header_extensions: Option<Vec<RtpHeaderExtensionParameters>>,
    // TODO: Does this need to be optional or can be an empty vec?
    /// Transmitted RTP streams and their settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encodings: Option<Vec<RtpEncodingParameters>>,
    /// Parameters used for RTCP.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rtcp: Option<RtcpParameters>,
}

// TODO: supportedRtpCapabilities.ts file and generally update TypeScript references
/// Provides information on codec settings within the RTP parameters. The list
/// of media codecs supported by mediasoup and their settings is defined in the
/// supportedRtpCapabilities.ts file.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RtpCodecParameters {
    // TODO: Enum?
    /// The codec MIME media type/subtype (e.g. 'audio/opus', 'video/VP8').
    pub mime_type: String,
    /// The value that goes in the RTP Payload Type Field. Must be unique.
    pub payload_type: u8,
    /// Codec clock rate expressed in Hertz.
    pub clock_rate: u32,
    /// The number of channels supported (e.g. two for stereo). Just for audio.
    /// Default 1.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channels: Option<u8>,
    // TODO: Not sure if this hashmap is a correct type
    /// Codec-specific parameters available for signaling. Some parameters (such as
    /// 'packetization-mode' and 'profile-level-id' in H264 or 'profile-id' in VP9) are critical for
    /// codec matching.
    pub parameters: HashMap<String, String>,
    // TODO: Does this need to be optional or can be an empty vec?
    /// Transport layer and codec-specific feedback messages for this codec.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rtcp_feedback: Option<Vec<RtcpFeedback>>,
}

// TODO: supportedRtpCapabilities.ts file and generally update TypeScript references
/// Provides information on RTCP feedback messages for a specific codec. Those messages can be
/// transport layer feedback messages or codec-specific feedback messages. The list of RTCP
/// feedbacks supported by mediasoup is defined in the supportedRtpCapabilities.ts file.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct RtcpFeedback {
    // TODO: Enum?
    /// RTCP feedback type.
    pub r#type: String,
    /// RTCP feedback parameter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter: Option<String>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct RtpEncodingParametersRtx {
    ssrc: u32,
}

/// Provides information relating to an encoding, which represents a media RTP
/// stream and its associated RTX stream (if any).
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RtpEncodingParameters {
    /// The media SSRC.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssrc: Option<u32>,
    /// The RID RTP extension value. Must be unique.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rid: Option<String>,
    /// Codec payload type this encoding affects. If unset, first media codec is chosen.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codec_payload_type: Option<u8>,
    /// RTX stream information. It must contain a numeric ssrc field indicating the RTX SSRC.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rtx: Option<RtpEncodingParametersRtx>,
    /// It indicates whether discontinuous RTP transmission will be used. Useful for audio (if the
    /// codec supports it) and for video screen sharing (when static content is being transmitted,
    /// this option disables the RTP inactivity checks in mediasoup).
    /// Default false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dtx: Option<bool>,
    // TODO: Maybe enum?
    /// Number of spatial and temporal layers in the RTP stream (e.g. 'L1T3'). See webrtc-svc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scalability_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale_resolution_down_by: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_bitrate: Option<u32>,
}

// TODO: supportedRtpCapabilities.ts file and generally update TypeScript references
/// Defines a RTP header extension within the RTP parameters. The list of RTP
/// header extensions supported by mediasoup is defined in the
/// supportedRtpCapabilities.ts file.
///
/// mediasoup does not currently support encrypted RTP header extensions and no
/// parameters are currently considered.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct RtpHeaderExtensionParameters {
    /// The URI of the RTP header extension, as defined in RFC 5285.
    pub uri: String,
    /// The numeric identifier that goes in the RTP packet. Must be unique.
    pub id: u16,
    /// If true, the value in the header is encrypted as per RFC 6904. Default false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypt: Option<bool>,
    // TODO: Not sure if this hashmap is a correct type
    /// Configuration parameters for the header extension.
    pub parameters: HashMap<String, String>,
}

/// Provides information on RTCP settings within the RTP parameters.
///
/// If no cname is given in a producer's RTP parameters, the mediasoup transport
/// will choose a random one that will be used into RTCP SDES messages sent to
/// all its associated consumers.
///
/// mediasoup assumes reducedSize to always be true.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RtcpParameters {
    /// The Canonical Name (CNAME) used by RTCP (e.g. in SDES messages).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cname: Option<String>,
    /// Whether reduced size RTCP RFC 5506 is configured (if true) or compound RTCP
    /// as specified in RFC 3550 (if false). Default true.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduced_size: Option<bool>,
    /// Whether RTCP-mux is used. Default true.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mux: Option<bool>,
}