# Key Paths

Purpose	            Path
Recorded .wav files	~/Library/Application Support/com.lala.app/tauri-plugin-mic-recorder/
Whisper CLI repo	   ~/external-libraries/whisper.cpp/
Whisper CLI binary	~/external-libraries/whisper.cpp/build/bin/whisper-cli
Default model	      ~/external-libraries/whisper.cpp/models/ggml-base.en.bin

Important: Always execute whisper-cli from inside the whisper.cpp directory so relative paths to models/ resolve correctly.

⸻

Whisper-CLI Cheat-Sheet

Minimal command that works today (leave the model flag implicit, silence all logs, and output .txt beside the audio):

./build/bin/whisper-cli \
  --output-txt \
  --no-prints \
  ~/Library/Application\ Support/com.lala.app/tauri-plugin-mic-recorder/20250524134906.wav

Feel free to expose additional flags (e.g. --threads, --language, etc.) through the UI later.

⸻

References
	•	Tauri docs (context7): `/tauri-apps/tauri-docs`
	•	Tauri Rust crate: https://docs.rs/tauri/latest/tauri/
	•	Mic recorder plugin: https://github.com/ayangweb/tauri-plugin-mic-recorder

Use these sources for API details, IPC, notifications, and clipboard utilities.

Keep the code idiomatic, handle edge cases, and keep the user flow snappy.