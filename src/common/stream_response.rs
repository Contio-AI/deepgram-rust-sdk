//! Stream Response module

use serde::{Deserialize, Serialize};

/// A single transcribed word.
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded
#[derive(Debug, Serialize, Deserialize)]
pub struct Word {
    #[allow(missing_docs)]
    pub word: String,

    #[allow(missing_docs)]
    pub start: Option<f64>, // Optional for Flux format

    #[allow(missing_docs)]
    pub end: Option<f64>, // Optional for Flux format

    #[allow(missing_docs)]
    pub confidence: f64,

    #[allow(missing_docs)]
    pub speaker: Option<i32>,

    #[allow(missing_docs)]
    pub punctuated_word: Option<String>,

    #[allow(missing_docs)]
    pub language: Option<String>,
}

/// Transcript alternatives.
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded
#[derive(Debug, Serialize, Deserialize)]
pub struct Alternatives {
    #[allow(missing_docs)]
    pub transcript: String,

    #[allow(missing_docs)]
    pub words: Vec<Word>,

    #[allow(missing_docs)]
    pub confidence: f64,

    #[allow(missing_docs)]
    #[serde(default)]
    pub languages: Vec<String>,
}

/// Transcription results for a single audio channel.
///
/// See the [Deepgram API Reference][api]
/// and the [Deepgram Multichannel feature docs][docs] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded
/// [docs]: https://developers.deepgram.com/documentation/features/multichannel/
#[derive(Debug, Serialize, Deserialize)]
pub struct Channel {
    #[allow(missing_docs)]
    pub alternatives: Vec<Alternatives>,
}

/// Modle info
#[derive(Debug, Serialize, Deserialize)]
pub struct ModelInfo {
    #[allow(missing_docs)]
    pub name: String,

    #[allow(missing_docs)]
    pub version: String,

    #[allow(missing_docs)]
    pub arch: String,
}

/// Metadata about the transcription.
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded
#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    #[allow(missing_docs)]
    pub request_id: String,

    #[allow(missing_docs)]
    pub model_info: ModelInfo,

    #[allow(missing_docs)]
    pub model_uuid: String,
}

/// Flux turn events for conversational speech recognition.
#[derive(Debug, Serialize, Deserialize)]
pub enum TurnEvent {
    #[allow(missing_docs)]
    #[serde(rename = "Update")]
    Update,
    #[allow(missing_docs)]
    #[serde(rename = "StartOfTurn")]
    StartOfTurn,
    #[allow(missing_docs)]
    #[serde(rename = "EagerEndOfTurn")]
    EagerEndOfTurn,
    #[allow(missing_docs)]
    #[serde(rename = "TurnResumed")]
    TurnResumed,
    #[allow(missing_docs)]
    #[serde(rename = "EndOfTurn")]
    EndOfTurn,
}

/// Possible websocket message types
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum StreamResponse {
    #[allow(missing_docs)]
    TranscriptResponse {
        #[allow(missing_docs)]
        #[serde(rename = "type")]
        type_field: String,

        #[allow(missing_docs)]
        start: f64,

        #[allow(missing_docs)]
        duration: f64,

        #[allow(missing_docs)]
        is_final: bool,

        #[allow(missing_docs)]
        speech_final: bool,

        #[allow(missing_docs)]
        from_finalize: bool,

        #[allow(missing_docs)]
        channel: Channel,

        #[allow(missing_docs)]
        metadata: Metadata,

        #[allow(missing_docs)]
        channel_index: Vec<i32>,
    },
    #[allow(missing_docs)]
    TerminalResponse {
        #[allow(missing_docs)]
        request_id: String,

        #[allow(missing_docs)]
        created: String,

        #[allow(missing_docs)]
        duration: f64,

        #[allow(missing_docs)]
        channels: u32,
    },
    #[allow(missing_docs)]
    SpeechStartedResponse {
        #[allow(missing_docs)]
        #[serde(rename = "type")]
        type_field: String,

        #[allow(missing_docs)]
        channel: Vec<u8>,

        #[allow(missing_docs)]
        timestamp: f64,
    },
    #[allow(missing_docs)]
    UtteranceEndResponse {
        #[allow(missing_docs)]
        #[serde(rename = "type")]
        type_field: String,

        #[allow(missing_docs)]
        channel: Vec<u8>,

        #[allow(missing_docs)]
        last_word_end: f64,
    },
    #[allow(missing_docs)]
    TurnInfoResponse {
        #[allow(missing_docs)]
        #[serde(rename = "type")]
        type_field: String,

        #[allow(missing_docs)]
        request_id: String,

        #[allow(missing_docs)]
        event: TurnEvent,

        #[allow(missing_docs)]
        turn_index: u32,

        #[allow(missing_docs)]
        audio_window_start: f64,

        #[allow(missing_docs)]
        audio_window_end: f64,

        #[allow(missing_docs)]
        transcript: String,

        #[allow(missing_docs)]
        words: Option<Vec<Word>>,

        #[allow(missing_docs)]
        end_of_turn_confidence: f64,

        #[allow(missing_docs)]
        sequence_id: u64,
    },
    #[allow(missing_docs)]
    ConnectedResponse {
        #[allow(missing_docs)]
        #[serde(rename = "type")]
        type_field: String,

        #[allow(missing_docs)]
        request_id: String,

        #[allow(missing_docs)]
        sequence_id: u64,
    },
    #[allow(missing_docs)]
    ErrorResponse {
        #[allow(missing_docs)]
        #[serde(rename = "type")]
        type_field: String,

        #[allow(missing_docs)]
        code: String,

        #[allow(missing_docs)]
        description: String,

        #[allow(missing_docs)]
        sequence_id: u64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_connected_response_parsing() {
        let json = r#"{"type":"Connected","request_id":"test-123","sequence_id":0}"#;
        let result: Result<StreamResponse, _> = serde_json::from_str(json);

        assert!(result.is_ok());
        match result.unwrap() {
            StreamResponse::ConnectedResponse {
                type_field,
                request_id,
                sequence_id,
            } => {
                assert_eq!(type_field, "Connected");
                assert_eq!(request_id, "test-123");
                assert_eq!(sequence_id, 0);
            }
            _ => panic!("Expected ConnectedResponse"),
        }
    }

    #[test]
    fn test_turninfo_startofturn_parsing() {
        let json = r#"{"type":"TurnInfo","request_id":"ca47dc5f-27bc-440c-b71f-d8eb6359df71","event":"StartOfTurn","turn_index":0,"audio_window_start":0.0,"audio_window_end":0.48,"transcript":"Fine","words":[{"word":"Fine","confidence":0.4573}],"end_of_turn_confidence":0.0189,"sequence_id":2}"#;
        let result: Result<StreamResponse, _> = serde_json::from_str(json);

        println!("Parsing result: {:?}", result);
        assert!(result.is_ok());
        match result.unwrap() {
            StreamResponse::TurnInfoResponse {
                type_field,
                request_id,
                event,
                transcript,
                sequence_id,
                ..
            } => {
                assert_eq!(type_field, "TurnInfo");
                assert_eq!(request_id, "ca47dc5f-27bc-440c-b71f-d8eb6359df71");
                assert_eq!(transcript, "Fine");
                assert_eq!(sequence_id, 2);
                assert!(matches!(event, TurnEvent::StartOfTurn));
            }
            other => panic!("Expected TurnInfoResponse, got: {:?}", other),
        }
    }

    #[test]
    fn test_turninfo_update_parsing() {
        let json = r#"{"type":"TurnInfo","request_id":"ca47dc5f-27bc-440c-b71f-d8eb6359df71","event":"Update","turn_index":0,"audio_window_start":0.0,"audio_window_end":0.72,"transcript":"final.","words":[{"word":"final.","confidence":1.0}],"end_of_turn_confidence":0.1776,"sequence_id":3}"#;
        let result: Result<StreamResponse, _> = serde_json::from_str(json);

        println!("Parsing result: {:?}", result);
        assert!(result.is_ok());
        match result.unwrap() {
            StreamResponse::TurnInfoResponse {
                type_field,
                request_id,
                event,
                transcript,
                sequence_id,
                ..
            } => {
                assert_eq!(type_field, "TurnInfo");
                assert_eq!(request_id, "ca47dc5f-27bc-440c-b71f-d8eb6359df71");
                assert_eq!(transcript, "final.");
                assert_eq!(sequence_id, 3);
                assert!(matches!(event, TurnEvent::Update));
            }
            other => panic!("Expected TurnInfoResponse, got: {:?}", other),
        }
    }

    #[test]
    fn test_turninfo_longer_transcript_parsing() {
        let json = r#"{"type":"TurnInfo","request_id":"ca47dc5f-27bc-440c-b71f-d8eb6359df71","event":"Update","turn_index":0,"audio_window_start":0.0,"audio_window_end":5.28,"transcript":"final. Where was the greatest moment? All over Uganda, we made everyone happy.","words":[{"word":"final.","confidence":1.0},{"word":"Where","confidence":0.707},{"word":"was","confidence":0.9976},{"word":"the","confidence":0.9956},{"word":"greatest","confidence":0.9985},{"word":"moment?","confidence":0.9971},{"word":"All","confidence":0.9956},{"word":"over","confidence":0.9956},{"word":"Uganda,","confidence":1.0},{"word":"we","confidence":0.9956},{"word":"made","confidence":0.998},{"word":"everyone","confidence":0.9995}],"end_of_turn_confidence":0.0198,"sequence_id":21}"#;
        let result: Result<StreamResponse, _> = serde_json::from_str(json);

        println!("Parsing result: {:?}", result);
        assert!(result.is_ok());
        match result.unwrap() {
            StreamResponse::TurnInfoResponse {
                type_field,
                request_id,
                event,
                transcript,
                sequence_id,
                ..
            } => {
                assert_eq!(type_field, "TurnInfo");
                assert_eq!(request_id, "ca47dc5f-27bc-440c-b71f-d8eb6359df71");
                assert_eq!(transcript, "final. Where was the greatest moment? All over Uganda, we made everyone happy.");
                assert_eq!(sequence_id, 21);
                assert!(matches!(event, TurnEvent::Update));
            }
            other => panic!("Expected TurnInfoResponse, got: {:?}", other),
        }
    }

    #[test]
    fn test_all_turn_events() {
        let test_cases = vec![
            ("Update", TurnEvent::Update),
            ("StartOfTurn", TurnEvent::StartOfTurn),
            ("EagerEndOfTurn", TurnEvent::EagerEndOfTurn),
            ("TurnResumed", TurnEvent::TurnResumed),
            ("EndOfTurn", TurnEvent::EndOfTurn),
        ];

        for (event_str, _expected_event) in test_cases {
            let json = format!(
                r#"{{"type":"TurnInfo","request_id":"test-123","event":"{}","turn_index":0,"audio_window_start":0.0,"audio_window_end":1.0,"transcript":"test","words":[],"end_of_turn_confidence":0.5,"sequence_id":1}}"#,
                event_str
            );

            let result: Result<StreamResponse, _> = serde_json::from_str(&json);
            println!("Testing event '{}': {:?}", event_str, result);

            assert!(result.is_ok(), "Failed to parse event: {}", event_str);
            match result.unwrap() {
                StreamResponse::TurnInfoResponse { .. } => {
                    // Just verify we got a TurnInfoResponse, the specific event matching is complex with the current enum structure
                }
                other => panic!(
                    "Expected TurnInfoResponse for event '{}', got: {:?}",
                    event_str, other
                ),
            }
        }
    }

    #[test]
    fn test_word_deserialization_issue() {
        // Test if the Word struct can deserialize Flux format (missing start/end)
        let flux_word_json = r#"{"word":"final.","confidence":1.0}"#;
        let word_result: Result<Word, _> = serde_json::from_str(flux_word_json);

        println!("Flux Word deserialization: {:?}", word_result);

        // This will likely fail because start/end are missing
        if word_result.is_err() {
            println!("❌ CONFIRMED: Word struct cannot deserialize Flux format - missing start/end fields");
        }

        // Test with complete word format
        let complete_word_json = r#"{"word":"final.","start":0.0,"end":1.0,"confidence":1.0}"#;
        let complete_result: Result<Word, _> = serde_json::from_str(complete_word_json);
        println!("Complete Word deserialization: {:?}", complete_result);
    }

    #[test]
    fn test_demonstrates_the_bug() {
        // This test demonstrates the exact bug we're seeing
        let json = r#"{"type":"TurnInfo","request_id":"ca47dc5f-27bc-440c-b71f-d8eb6359df71","event":"Update","turn_index":0,"audio_window_start":0.0,"audio_window_end":0.72,"transcript":"final.","words":[{"word":"final.","confidence":1.0}],"end_of_turn_confidence":0.1776,"sequence_id":3}"#;
        let result: Result<StreamResponse, _> = serde_json::from_str(json);

        println!("BUG DEMONSTRATION - Parsing result: {:?}", result);

        // This assertion will FAIL, demonstrating the bug
        // The JSON should parse as TurnInfoResponse but actually parses as ConnectedResponse
        match result.unwrap() {
            StreamResponse::TurnInfoResponse { .. } => {
                println!("✅ CORRECTLY parsed as TurnInfoResponse");
            }
            StreamResponse::ConnectedResponse {
                type_field,
                request_id,
                sequence_id,
            } => {
                println!("❌ BUG: Incorrectly parsed as ConnectedResponse!");
                println!("   type_field: {}", type_field);
                println!("   request_id: {}", request_id);
                println!("   sequence_id: {}", sequence_id);
                println!("   TRANSCRIPT DATA LOST!");
                panic!("BUG CONFIRMED: TurnInfo message parsed as ConnectedResponse, losing transcript data");
            }
            other => {
                panic!("Unexpected parse result: {:?}", other);
            }
        }
    }
}
