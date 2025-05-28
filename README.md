# Key Paths

Recorded .wav files	~/Library/Application Support/com.lala.app/tauri-plugin-mic-recorder/
Whisper CLI repo	   ~/external-libraries/whisper.cpp/
Whisper CLI binary	~/external-libraries/whisper.cpp/build/bin/whisper-cli
Default model	      ~/external-libraries/whisper.cpp/models/ggml-base.en.bin

```
$ cd ~/external-libraries/whisper.cpp
$ ./build/bin/whisper-cli -m models/ggml-medium.bin --no-prints samples/jfk.wav

[00:00:00.000 --> 00:00:11.000]   And so, my fellow Americans, ask not what your country can do for you; ask what you can do for your country.
```

References
	•	Tauri docs (context7): `/tauri-apps/tauri-docs`
	•	Tauri Rust crate: https://docs.rs/tauri/latest/tauri/
