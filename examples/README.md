## Running Examples

### Setting Env Vars

```sh
export FILENAME=./examples/audio/bueller.wav
```

### Running the examples

```sh
cargo run --example prerecorded_from_url
```

```sh
cargo run --example simple_stream
```

```sh
cargo run --example callback
```

```sh
cargo run --example make_prerecorded_request_builder
```

```sh
cargo run --example microphone_stream
```

```sh
cargo run --example text_to_speech_to_file
```

```sh
cargo run --example text_to_speech_to_stream
```

```sh
cargo run --example flux_stream
```

### Flux Streaming Example

The `flux_stream` example demonstrates Deepgram's Flux model for conversational speech recognition:

- **Model-integrated turn detection** - No separate VAD needed
- **Turn events** - StartOfTurn, EagerEndOfTurn, TurnResumed, EndOfTurn
- **Configurable thresholds** - eot_threshold, eager_eot_threshold, eot_timeout_ms
- **Color-coded confidence** - Visual feedback for word-level confidence scores
- **Real-time processing** - Optimized for voice agent applications

Flux automatically uses the `/v2/listen` endpoint and provides enhanced conversational features compared to traditional streaming models.
