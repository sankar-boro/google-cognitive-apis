//! Speech-to-text recognizer module - v1 STT API.
#![allow(clippy::manual_map)]
use crate::api::grpc::google::cloud::speechtotext::v2::{
    speech_client::SpeechClient, streaming_recognize_request::StreamingRequest, RecognizeRequest,
    RecognizeResponse, StreamingRecognitionConfig, StreamingRecognizeRequest,
    StreamingRecognizeResponse,
};
use crate::common::{get_token, new_grpc_channel, new_interceptor, TokenInterceptor};
use crate::errors::Result;
use async_stream::try_stream;
use futures_core::stream::Stream;
use log::*;
use std::result::Result as StdResult;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::codegen::InterceptedService;
use tonic::Response as TonicResponse;
use tonic::Status as TonicStatus;
use tonic::{transport::Channel, Streaming};

const GRPC_API_DOMAIN: &str = "speech.googleapis.com";
const GRPC_API_URL: &str = "https://speech.googleapis.com";

/// Google Speech API recognizer
#[derive(Debug)]
pub struct Recognizer {
    /// internal GRPC speech client
    speech_client: SpeechClient<InterceptedService<Channel, TokenInterceptor>>,

    /// channel for sending audio data
    audio_sender: Option<mpsc::Sender<StreamingRecognizeRequest>>,

    /// channel for streaming audio data into GRPC API
    audio_receiver: Option<mpsc::Receiver<StreamingRecognizeRequest>>,

    /// For channel based streaming this is the internal channel sender
    /// where STT results will be sent. Library client is using respective
    /// receiver to get the results. See example recognizer_streaming for details
    result_sender: Option<mpsc::Sender<StreamingRecognizeResponse>>,
}

impl Recognizer {
    /// Creates new speech recognizer from provided
    /// Google credentials and google speech configuration.
    /// This kind of recognizer can be used for streaming recognition.
    pub async fn create_streaming_recognizer(
        // Google Cloud Platform JSON credentials for project with Speech APIs enabled
        google_credentials: impl AsRef<str>,
        //  Streaming recognition configuration
        config: StreamingRecognitionConfig,
        // Capacity of audio sink (tokio channel used by caller to send audio data).
        // If not provided defaults to 1000.
        buffer_size: Option<usize>,
    ) -> Result<Self> {
        let channel = new_grpc_channel(GRPC_API_DOMAIN, GRPC_API_URL, None).await?;

        let token_header_val = get_token(google_credentials)?;

        let speech_client =
            SpeechClient::with_interceptor(channel, new_interceptor(token_header_val));

        let (audio_sender, audio_receiver) =
            mpsc::channel::<StreamingRecognizeRequest>(buffer_size.unwrap_or(1000));

        let streaming_config = StreamingRecognizeRequest {
            streaming_request: Some(StreamingRequest::StreamingConfig(config)),
            recognizer: "".to_string(),
        };

        audio_sender.send(streaming_config).await?;

        Ok(Recognizer {
            speech_client,
            audio_sender: Some(audio_sender),
            audio_receiver: Some(audio_receiver),
            result_sender: None,
        })
    }

    /// Creates new speech recognizer from provided
    /// Google credentials. This kind of recognizer can be used
    /// for long running recognition.
    pub async fn create_asynchronous_recognizer(
        google_credentials: impl AsRef<str>,
    ) -> Result<Self> {
        let channel = new_grpc_channel(GRPC_API_DOMAIN, GRPC_API_URL, None).await?;

        let token_header_val = get_token(google_credentials)?;

        let speech_client = SpeechClient::with_interceptor(
            channel.clone(),
            new_interceptor(token_header_val.clone()),
        );

        Ok(Recognizer {
            speech_client,
            audio_sender: None,
            audio_receiver: None,
            result_sender: None,
        })
    }

    /// Creates new speech recognizer from provided
    /// Google credentials. This kind of recognizer can be used
    /// for synchronous recognition.
    pub async fn create_synchronous_recognizer(
        google_credentials: impl AsRef<str>,
    ) -> Result<Self> {
        let channel = new_grpc_channel(GRPC_API_DOMAIN, GRPC_API_URL, None).await?;

        let token_header_val = get_token(google_credentials)?;

        let speech_client =
            SpeechClient::with_interceptor(channel, new_interceptor(token_header_val));

        Ok(Recognizer {
            speech_client,
            audio_sender: None,
            audio_receiver: None,
            result_sender: None,
        })
    }

    /// Returns sender than can be used to stream in audio bytes. This method can be called
    /// multiple times to retrieve multiple senders.
    pub fn get_audio_sink(&mut self) -> Option<mpsc::Sender<StreamingRecognizeRequest>> {
        if let Some(audio_sender) = &self.audio_sender {
            Some(audio_sender.clone())
        } else {
            None
        }
    }

    /// Returns sender than can be used to stream in audio bytes. This method will take
    /// the sender out of the option leaving None in its place. No additional sender
    /// can be retrieved from recognizer after this call. When sender is dropped respective
    /// stream will be closed.
    pub fn take_audio_sink(&mut self) -> Option<mpsc::Sender<StreamingRecognizeRequest>> {
        if let Some(audio_sender) = self.audio_sender.take() {
            Some(audio_sender)
        } else {
            None
        }
    }

    /// Drops audio sender so that respective stream can be closed.
    pub fn drop_audio_sink(&mut self) {
        self.audio_sender.take();
    }

    /// Returns receiver that can be used to receive speech-to-text results
    /// used with streaming_recognize function.
    pub fn get_streaming_result_receiver(
        &mut self,
        // buffer size for tokio channel. If not provided defaults to 1000.
        buffer_size: Option<usize>,
    ) -> mpsc::Receiver<StreamingRecognizeResponse> {
        let (result_sender, result_receiver) =
            mpsc::channel::<StreamingRecognizeResponse>(buffer_size.unwrap_or(1000));
        self.result_sender = Some(result_sender);
        result_receiver
    }

    /// Convenience function so that client does not have to create full StreamingRecognizeRequest
    /// and can just pass audio bytes vector instead.
    pub fn streaming_request_from_bytes(audio_bytes: Vec<u8>) -> StreamingRecognizeRequest {
        StreamingRecognizeRequest {
            streaming_request: Some(StreamingRequest::Audio(audio_bytes)),
            recognizer: "".to_string(),
        }
    }

    /// Initiates bidirectional streaming. Returns
    /// asynchronous stream of streaming recognition results
    /// Audio data must be fed into recognizer via channel sender
    /// returned by function get_audio_sink.
    #[allow(unreachable_code)]
    pub async fn streaming_recognize_async_stream(
        &mut self,
    ) -> impl Stream<Item = Result<StreamingRecognizeResponse>> + '_ {
        try_stream! {
                // yank self.audio_receiver so that we can consume it
                if let Some(audio_receiver) = self.audio_receiver.take() {
                    let streaming_recognize_result: StdResult<
                        TonicResponse<Streaming<StreamingRecognizeResponse>>,
                        TonicStatus,
                    > = self.speech_client.streaming_recognize(ReceiverStream::new(audio_receiver)).await;

                    let mut response_stream: Streaming<StreamingRecognizeResponse> =
                        streaming_recognize_result?.into_inner();

                    trace!("streaming_recognize: entering loop");
                    while let Some(streaming_recognize_response) = response_stream.message().await? {
                        yield streaming_recognize_response;
                    }
                    trace!("streaming_recognize: leaving loop");
                }
        }
    }

    /// Initiates bidirectional streaming. This call should be spawned
    /// into separate tokio task. Results can be then retrieved via
    /// channel receiver returned by method get_streaming_result_receiver.
    pub async fn streaming_recognize(&mut self) -> Result<()> {
        // yank self.audio_receiver so that we can consume it
        if let Some(audio_receiver) = self.audio_receiver.take() {
            let streaming_recognize_result: StdResult<
                tonic::Response<Streaming<StreamingRecognizeResponse>>,
                tonic::Status,
            > = self
                .speech_client
                .streaming_recognize(ReceiverStream::new(audio_receiver))
                .await;

            let mut response_stream: Streaming<StreamingRecognizeResponse> =
                streaming_recognize_result?.into_inner();

            while let Some(streaming_recognize_response) = response_stream.message().await? {
                if let Some(result_sender) = &self.result_sender {
                    result_sender.send(streaming_recognize_response).await?;
                }
            }
        }

        Ok(())
    }

    /// Performs synchronous speech recognition.
    pub async fn recognize(&mut self, request: RecognizeRequest) -> Result<RecognizeResponse> {
        let tonic_response: TonicResponse<RecognizeResponse> =
            self.speech_client.recognize(request).await?;
        Ok(tonic_response.into_inner())
    }
}
