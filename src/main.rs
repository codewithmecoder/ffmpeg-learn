use std::{
  ffi::CStr,
  ptr::{null, null_mut},
};

use ffmpeg_sys_next::{
  av_find_best_stream, avformat_find_stream_info, avformat_open_input, AVFormatContext, AVMediaType,
};

fn main() {
  unsafe {
    let mut format_context: *mut AVFormatContext = null_mut();
    let source_filename = CStr::from_bytes_with_nul_unchecked(b"res/video.mp4\0");
    assert!(
      avformat_open_input(
        &mut format_context,
        source_filename.as_ptr(),
        null(),
        null_mut(),
      ) >= 0
    );

    assert!(avformat_find_stream_info(format_context, null_mut()) >= 0);
    assert!(
      av_find_best_stream(
        format_context,
        AVMediaType::AVMEDIA_TYPE_VIDEO,
        -1,
        -1,
        null_mut(),
        0
      ) >= 0
    );
  }
}
