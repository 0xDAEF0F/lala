# transcriber

## required dependencies

- [whisper.cpp](https://github.com/ggerganov/whisper.cpp)

- cli docs: [whisper-cli](https://github.com/ggml-org/whisper.cpp/blob/master/examples/cli/README.md)

Note: transcriber models are under the `models` folder.

```bash
$ cd ~/external-libraries/whisper.cpp
$ ./build/bin/whisper-cli -m models/ggml-medium.bin --no-prints samples/jfk.wav
```

output:
```
[00:00:00.000 --> 00:00:11.000]   And so, my fellow Americans, ask not what your country can do for you; ask what you can do for your country.
```

## References

- Tauri docs (context7): `/tauri-apps/tauri-docs`
- Tauri Rust crate: [docs.rs/tauri](https://docs.rs/tauri/latest/tauri/)
