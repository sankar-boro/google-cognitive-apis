use google_cognitive_apis::api::grpc::google::cloud::speechtotext::v2::{
    explicit_decoding_config::AudioEncoding, recognition_config::DecodingConfig,
    ExplicitDecodingConfig, RecognitionConfig, RecognitionFeatures, SpeechAdaptation,
    StreamingRecognitionConfig, StreamingRecognitionFeatures, TranslationConfig,
};
use google_cognitive_apis::speechtotext::recognizer_v2::Recognizer;

use log::*;
use std::fs::{self, File};
use std::io::Read;
use std::{env, vec};

#[tokio::main]
async fn main() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();
    info!("streaming recognizer example");

    let credentials = fs::read_to_string("/tmp/cred.json").unwrap();
    let streaming_config = StreamingRecognitionConfig {
        config: Some(RecognitionConfig {
            // enable_separate_recognition_per_channel: false,
            // language_code: "en-US".to_string(),
            // speech_contexts: vec![],
            // metadata: None,
            // use_enhanced: false,
            model: "".to_string(),
            language_codes: vec![],
            features: Some(RecognitionFeatures {
                profanity_filter: false,
                enable_word_time_offsets: false,
                enable_word_confidence: false,
                enable_automatic_punctuation: false,
                enable_spoken_punctuation: false,
                enable_spoken_emojis: false,
                multi_channel_mode: 2,
                diarization_config: None,
                max_alternatives: 1,
            }),
            adaptation: Some(SpeechAdaptation {
                phrase_sets: vec![],
                custom_classes: vec![],
            }),
            transcript_normalization: None,
            translation_config: Some(TranslationConfig {
                target_language: "es-ES".to_string(),
            }),
            decoding_config: Some(DecodingConfig::ExplicitDecodingConfig(
                ExplicitDecodingConfig {
                    encoding: AudioEncoding::Linear16 as i32,
                    sample_rate_hertz: 8000,
                    audio_channel_count: 1,
                },
            )),
        }),
        config_mask: None,
        streaming_features: Some(StreamingRecognitionFeatures {
            enable_voice_activity_events: false,
            interim_results: true,
            voice_activity_timeout: None,
        }),
    };

    let mut recognizer =
        Recognizer::create_streaming_recognizer(credentials, streaming_config, None)
            .await
            .unwrap();

    // Make sure to use take_audio_sink, not get_audio_sink here! get_audio_sink is cloning the sender
    // contained in recognizer client whereas take_audio_sink will take the sender/sink out of the wrapping option.
    // Thus once tokio task pushing the audio data into Google Speech-to-text API will push all the data, sender will be
    // dropped signaling no more data will be sent. Only then Speech-to-text API will stream back the final
    // response (with is_final attribute set to true).
    // If get_audio_sink is used instead following error occurs ( give it a try:-) ):
    // Audio Timeout Error: Long duration elapsed without audio. Audio should be sent close to real time.
    // See also method drop_audio_sink
    let audio_sender = recognizer.take_audio_sink().unwrap();

    let mut result_receiver = recognizer.get_streaming_result_receiver(None);

    tokio::spawn(async move {
        let recognition_result = recognizer.streaming_recognize().await;

        match recognition_result {
            Err(err) => error!("streaming_recognize error {:?}", err),
            Ok(_) => info!("streaming_recognize ok!"),
        }
    });

    tokio::spawn(async move {
        let mut file = File::open("/tmp/hello_rust_8.wav").unwrap();
        let chunk_size = 1024;

        loop {
            let mut chunk = Vec::with_capacity(chunk_size);
            let n = file
                .by_ref()
                .take(chunk_size as u64)
                .read_to_end(&mut chunk)
                .unwrap();
            if n == 0 {
                break;
            }

            let streaming_request = Recognizer::streaming_request_from_bytes(chunk);

            audio_sender.send(streaming_request).await.unwrap();

            if n < chunk_size {
                break;
            }
        }
    });

    while let Some(reco_result) = result_receiver.recv().await {
        info!("recognition result {:?}", reco_result);
    }
}
