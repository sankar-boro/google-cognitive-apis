#![allow(clippy::from_over_into)]
#![allow(clippy::manual_unwrap_or)]
#![allow(clippy::manual_map)]
use crate::api::grpc::google::cloud::speechtotext::v2::recognition_config::DecodingConfig as GrpcDecodingConfig;
use crate::api::grpc::google::cloud::speechtotext::v2::{
    ExplicitDecodingConfig, SpeechAdaptation as GrpcSpeechAdaptation,
};
/// This module DOES NOT address all the differences between GRPC v1 and v1p1beta1 proto definitions
/// of speech-to-text API. For now it only defines extended version of SpeechContext struct
/// where boost parameter is added. Also RecognitionConfig using new SpeechContext is defined here.
///
/// RecognitionConfig also provides support for adaptation attribute (SpeechAdaptation) which
/// when used replaces SpeechContext.
/// JSON-to-GRPC struct conversion does not support following attributes:
///     alternative_language_codes
///     diarization_speaker_count
///     enable_speaker_diarization
///     enable_spoken_emojis
///     enable_spoken_punctuation
///     enable_word_confidence
///     diarization_config
///     metadata
/// This can be added but would require implementation of serde deserialization
/// and Into<T> implementation as we currently have in v1 REST API.
// use crate::api::grpc::google::cloud::speechtotext::v2::SpeechContext as GrpcSpeechContext;
use crate::errors::{Error, Result};
use serde::{Deserialize, Serialize};
use std::convert::Into;
// since initial implementation really extends just SpeechContext and defines RecognitionConfig which
// uses it we will reuse all other structures from v1 API
use super::common::{
    default_audio_channel_count,
    default_enable_automatic_punctuation,
    default_enable_separate_recognition_per_channel,
    default_enable_word_time_offsets,
    default_encoding,
    default_language_code,
    default_max_alternatives,
    default_max_speaker_count,
    default_min_speaker_count,
    default_model,
    default_profanity_filter,
    AudioEncoding,
    RecognitionConfigModel,
    // RecognitionMetadata,
};
use crate::api::grpc::google::cloud::speechtotext::v2::custom_class::ClassItem as GrpcClassItem;
use crate::api::grpc::google::cloud::speechtotext::v2::phrase_set::Phrase as GrpcPhrase;
use crate::api::grpc::google::cloud::speechtotext::v2::CustomClass as GrpcCustomClass;
use crate::api::grpc::google::cloud::speechtotext::v2::PhraseSet as GrpcPhraseSet;
use crate::api::grpc::google::cloud::speechtotext::v2::RecognitionConfig as GrpcRecognitionConfig;
use crate::api::grpc::google::cloud::speechtotext::v2::RecognitionFeatures as GrpcRecognitionFeatures;
use crate::api::grpc::google::cloud::speechtotext::v2::SpeakerDiarizationConfig as GrpcSpeakerDiarizationConfig;

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct SpeechContext {
//     phrases: Vec<String>,
//     boost: f32,
// }

// impl Into<GrpcSpeechContext> for SpeechContext {
//     fn into(self) -> GrpcSpeechContext {
//         GrpcSpeechContext {
//             phrases: self.phrases,
//             boost: self.boost,
//         }
//     }
// }

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpeakerDiarizationConfig {
    #[serde(rename = "enableSpeakerDiarization")]
    enable_speaker_diarization: bool,

    #[serde(rename = "minSpeakerCount", default = "default_min_speaker_count")]
    min_speaker_count: i32,

    #[serde(rename = "maxSpeakerCount", default = "default_max_speaker_count")]
    max_speaker_count: i32,

    #[serde(rename = "speakerTag", skip_serializing_if = "Option::is_none")]
    speaker_tag: Option<i32>,
}

#[allow(deprecated)]
impl Into<GrpcSpeakerDiarizationConfig> for SpeakerDiarizationConfig {
    fn into(self) -> GrpcSpeakerDiarizationConfig {
        GrpcSpeakerDiarizationConfig {
            // enable_speaker_diarization: self.enable_speaker_diarization,
            min_speaker_count: self.min_speaker_count,
            max_speaker_count: self.max_speaker_count,
            // speaker_tag: 0, // REST interface will never provide this
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecognitionConfig {
    #[serde(default = "default_encoding")]
    pub encoding: AudioEncoding,

    #[serde(rename = "sampleRateHertz", skip_serializing_if = "Option::is_none")]
    pub sample_rate_hertz: Option<i32>,

    #[serde(rename = "audioChannelCount", default = "default_audio_channel_count")]
    pub audio_channel_count: i32,

    #[serde(
        rename = "enableSeparateRecognitionPerChannel",
        default = "default_enable_separate_recognition_per_channel"
    )]
    pub enable_separate_recognition_per_channel: bool,

    #[serde(rename = "languageCode", default = "default_language_code")]
    pub language_code: String,

    #[serde(rename = "maxAlternatives", default = "default_max_alternatives")]
    pub max_alternatives: i32,

    #[serde(rename = "profanityFilter", default = "default_profanity_filter")]
    pub profanity_filter: bool,

    // #[serde(rename = "speechContexts")]
    // pub speech_contexts: Vec<SpeechContext>,
    #[serde(
        rename = "enableWordTimeOffsets",
        default = "default_enable_word_time_offsets"
    )]
    pub enable_word_time_offsets: bool,

    #[serde(
        rename = "enableAutomaticPunctuation",
        default = "default_enable_automatic_punctuation"
    )]
    pub enable_automatic_punctuation: bool,

    #[serde(rename = "diarizationConfig", skip_serializing_if = "Option::is_none")]
    pub diarization_config: Option<SpeakerDiarizationConfig>,

    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub metadata: Option<RecognitionMetadata>,
    #[serde(default = "default_model")]
    pub model: RecognitionConfigModel,

    #[serde(rename = "useEnhanced", skip_serializing_if = "Option::is_none")]
    pub use_enhanced: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub adaptation: Option<SpeechAdaptation>,
}

#[allow(deprecated)]
impl Into<GrpcRecognitionConfig> for RecognitionConfig {
    fn into(self) -> GrpcRecognitionConfig {
        GrpcRecognitionConfig {
            model: self.model.to_string(),
            language_codes: vec![self.language_code],

            features: Some(GrpcRecognitionFeatures {
                profanity_filter: self.profanity_filter,
                enable_word_time_offsets: self.enable_word_time_offsets,
                enable_automatic_punctuation: self.enable_automatic_punctuation,
                ..Default::default()
            }),

            adaptation: self.adaptation.map(Into::into),

            decoding_config: Some(GrpcDecodingConfig::ExplicitDecodingConfig(
                ExplicitDecodingConfig {
                    encoding: self.encoding as i32,
                    sample_rate_hertz: self.sample_rate_hertz.unwrap_or(8000),
                    audio_channel_count: self.audio_channel_count,
                },
            )),

            ..Default::default()
        }
    }
}
/// See https://cloud.google.com/speech-to-text/docs/reference/rest/v1p1beta1/RecognitionConfig#SpeechAdaptation
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpeechAdaptation {
    #[serde(rename = "phraseSets")]
    pub phrase_sets: Vec<PhraseSet>,

    #[serde(rename = "phraseSetReferences")]
    pub phrase_set_references: Vec<String>,

    #[serde(rename = "customClasses")]
    pub custom_classes: Vec<CustomClass>,
}

impl Into<GrpcSpeechAdaptation> for SpeechAdaptation {
    fn into(self) -> GrpcSpeechAdaptation {
        GrpcSpeechAdaptation {
            ..Default::default()
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PhraseSet {
    pub name: String,
    pub phrases: Vec<Phrase>,
    pub boost: f32,
}

impl Into<GrpcPhraseSet> for PhraseSet {
    fn into(self) -> GrpcPhraseSet {
        GrpcPhraseSet {
            name: self.name,
            phrases: self.phrases.into_iter().map(Into::into).collect(),
            boost: self.boost,
            ..Default::default()
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Phrase {
    pub value: String,
    pub boost: f32,
}

impl Into<GrpcPhrase> for Phrase {
    fn into(self) -> GrpcPhrase {
        GrpcPhrase {
            value: self.value,
            boost: self.boost,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomClass {
    pub name: String,

    #[serde(rename = "customClassId")]
    pub custom_class_id: String,

    pub items: Vec<ClassItem>,
}

impl Into<GrpcCustomClass> for CustomClass {
    fn into(self) -> GrpcCustomClass {
        GrpcCustomClass {
            name: self.name,
            items: self
                .items
                .into_iter()
                .map(|item| GrpcClassItem { value: item.value })
                .collect(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClassItem {
    pub value: String,
}

/// Converts string into RecognitionConfig. Uses serde_path_to_error to get detailed and meaningful parsing errors
pub fn deserialize_recognition_config(json_str: &str) -> Result<RecognitionConfig> {
    let jd = &mut serde_json::Deserializer::from_str(json_str);
    let result: std::result::Result<RecognitionConfig, _> = serde_path_to_error::deserialize(jd);
    match result {
        Ok(config) => Ok(config),
        Err(err) => {
            let err_path = err.path().to_string();
            Err(Error::new(format!(
                "Error when deserializing speech recognition config (v1p1beta1) at path: {}. Full error: {}",
                err_path,
                err
            )))
        }
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    // cargo test -- --show-output test_convert_to_beta_grpc
    #[test]
    fn test_convert_to_beta_grpc() {
        let json_str = r#"
            {
                "encoding": "MULAW",
                "languageCode" : "sv_SE",
                "speechContexts" : [
                    {
                        "phrases" : [
                            "$FULLPHONENUM"
                        ],
                        "boost" : 1
                    }
                ],
                "diarizationConfig": {
                    "enableSpeakerDiarization": false,
                    "minSpeakerCount": 2
                },
                "adaptation": {
                    "phraseSets": [
                        {
                          "name": "phraseSets1",
                          "phrases": [
                            {
                              "value": "PHRASE_XY",
                              "boost": 23.0
                            }
                          ],
                          "boost": 24.0
                        }
                    ],
                    "phraseSetReferences": ["foo", "bar"],
                    "customClasses": [
                        {
                          "name": "customClasses1",
                          "customClassId": "customClasses1ID1",
                          "items": [
                            {
                              "value": "customClassItem1"
                            }
                          ]
                        }
                    ]
                }
            }
            "#;
        let recognition_config = deserialize_recognition_config(json_str).unwrap();
        let recognition_config_grpc: GrpcRecognitionConfig = recognition_config.into();
        // in the listing below speechContexts WILL contain boosts since this is supported
        // in v1p1beta1 GRPC API
        // diarization_config will be none since we do not support it yet for v1p1beta1
        // adaptation element will be present as well
        println!(
            "recognition_config_grpc(v1p1beta1) {:#?}",
            recognition_config_grpc
        );
    }
}
