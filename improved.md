Goal

Complete the remaining features of our Tauri-based transcription app:
	1.	Record audio (start_recording, stop_recording).
	2.	Convert the generated .wav files to .txt using whisper-cli.
	3.	Copy the resulting transcription to the clipboard and show a desktop notification.

⸻

Key Paths

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

What to Implement
	1.	Recording Toggle
	•	Re-use the existing global hot-key.
	•	If the hot-key is pressed while recording → stop and proceed to step 2.
	•	If pressed while idle → start a new recording in the folder above.
	2.	Transcription Pipeline
	•	Spawn whisper-cli within ~/external-libraries/whisper.cpp/.
	•	Pass the most-recent .wav file and --output-txt --no-prints.
	•	Wait for the .txt file to appear alongside the .wav.
	3.	Post-Processing
	•	Read the generated .txt file.
	•	Copy its contents to the clipboard.
	•	Emit a desktop notification:
	•	Title: “Transcription ready”
	•	Body: first 100 chars of the text (ellipsize if longer).
	4.	Error Handling
	•	If whisper-cli exits non-zero, show a notification with the stderr lines.
	•	Remove any partially generated files on failure.

⸻

References
	•	Tauri docs (context7): `/tauri-apps/tauri-docs`
	•	Tauri Rust crate: https://docs.rs/tauri/latest/tauri/
	•	Mic recorder plugin: https://github.com/ayangweb/tauri-plugin-mic-recorder

Use these sources for API details, IPC, notifications, and clipboard utilities.

⸻

Keep the code idiomatic, handle edge cases, and keep the user flow snappy.