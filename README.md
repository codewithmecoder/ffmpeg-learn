# ffmpeg-learn

ffmpeg-learn

## Setting FFMPEG up

- vcpkg : VCPKG_ROOT
- vcpkg integrate install
- vcpkg install ffmpeg
- llvm for clang : LIBCLANG_PATH

## play raw video

```SHELL
ffplay -f rawvideo -pixel_format yuv420p -video_size 1920x1080 res/video
```
