use serde::{Deserialize, Serialize};
use std::convert::Into;
use std::fmt;

#[allow(non_camel_case_types)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AudioEncoding {
    ENCODING_UNSPECIFIED,
    LINEAR16,
    FLAC,
    MULAW,
    AMR,
    AMR_WB,
    OGG_OPUS,
    SPEEX_WITH_HEADER_BYTE,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RecognitionConfigModel {
    command_and_search,
    phone_call,
    video,
    default,
    latest_long,
    latest_short,
    medical_conversation,
    medical_dictation,
}

impl fmt::Display for RecognitionConfigModel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum InteractionType {
    INTERACTION_TYPE_UNSPECIFIED,
    DISCUSSION,
    PRESENTATION,
    PHONE_CALL,
    VOICEMAIL,
    PROFESSIONALLY_PRODUCED,
    VOICE_SEARCH,
    VOICE_COMMAND,
    DICTATION,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MicrophoneDistance {
    MICROPHONE_DISTANCE_UNSPECIFIED,
    NEARFIELD,
    MIDFIELD,
    FARFIELD,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OriginalMediaType {
    ORIGINAL_MEDIA_TYPE_UNSPECIFIED,
    AUDIO,
    VIDEO,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RecordingDeviceType {
    RECORDING_DEVICE_TYPE_UNSPECIFIED,
    SMARTPHONE,
    PC,
    PHONE_LINE,
    VEHICLE,
    OTHER_OUTDOOR_DEVICE,
    OTHER_INDOOR_DEVICE,
}

pub fn default_language_code() -> String {
    // for convenience we are setting this in google STT implementation
    // based on STT DB config
    "".to_owned()
}

pub fn default_model() -> RecognitionConfigModel {
    RecognitionConfigModel::default
}

pub fn default_encoding() -> AudioEncoding {
    AudioEncoding::LINEAR16
}

pub fn default_audio_channel_count() -> i32 {
    1
}

pub fn default_max_alternatives() -> i32 {
    1
}

pub fn default_profanity_filter() -> bool {
    false
}

pub fn default_enable_word_time_offsets() -> bool {
    false
}

pub fn default_enable_automatic_punctuation() -> bool {
    false
}

pub fn default_enable_separate_recognition_per_channel() -> bool {
    false
}

pub fn default_min_speaker_count() -> i32 {
    2
}

pub fn default_max_speaker_count() -> i32 {
    6
}
