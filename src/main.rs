use std::{
  ffi::{c_int, c_ulonglong, CStr},
  os::raw::c_void,
  ptr::{null, null_mut},
};

use anyhow::Context;
use ffmpeg_sys_next::{
  av_find_best_stream, av_frame_alloc, av_get_pix_fmt_name, av_image_alloc, av_image_copy,
  av_packet_alloc, av_packet_unref, av_read_frame, avcodec_alloc_context3, avcodec_find_decoder,
  avcodec_open2, avcodec_parameters_to_context, avcodec_receive_frame, avcodec_send_packet,
  avformat_find_stream_info, avformat_open_input, fclose, fopen, fwrite, AVCodecContext,
  AVFormatContext, AVFrame, AVMediaType, AVPacket, AVERROR, AVERROR_EOF, EAGAIN, FILE,
};

fn main() -> anyhow::Result<()> {
  unsafe {
    let mut format_context: *mut AVFormatContext = null_mut();
    let source_filename = CStr::from_bytes_with_nul_unchecked(b"res/video.mp4\0");
    let output_file = CStr::from_bytes_with_nul_unchecked(b"res/video\0");
    assert!(
      avformat_open_input(
        &mut format_context,
        source_filename.as_ptr(),
        null(),
        null_mut(),
      ) >= 0
    );

    let format_context = &mut *format_context;

    assert!(avformat_find_stream_info(format_context, null_mut()) >= 0);

    let stream_index = av_find_best_stream(
      format_context,
      AVMediaType::AVMEDIA_TYPE_VIDEO,
      -1,
      -1,
      null_mut(),
      0,
    );
    assert!(stream_index >= 0);
    let video_stream =
      &*std::slice::from_raw_parts(format_context.streams, format_context.nb_streams as usize)
        [stream_index as usize];
    let video_codec = avcodec_find_decoder((*video_stream.codecpar).codec_id)
      .as_ref()
      .context("Failed to find video codec")?;
    let video_codec_context = avcodec_alloc_context3(video_codec)
      .as_mut()
      .context("Failed to allocate video codec context")?;
    assert!(avcodec_parameters_to_context(video_codec_context, video_stream.codecpar) >= 0);
    assert!(avcodec_open2(video_codec_context, video_codec, null_mut()) >= 0);

    let file = fopen(
      output_file.as_ptr(),
      CStr::from_bytes_with_nul_unchecked(b"wb\0").as_ptr(),
    )
    .as_mut()
    .context("Failed to open output file")?;

    let mut decoded_image_data = [null_mut::<u8>(); 4];
    let mut decoded_image_line_size = [c_int::from(0); 4];
    let decoded_image_data_size = av_image_alloc(
      decoded_image_data.as_mut_ptr(),
      decoded_image_line_size.as_mut_ptr(),
      video_codec_context.width,
      video_codec_context.height,
      video_codec_context.pix_fmt,
      1,
    );
    assert!(decoded_image_data_size >= 0);

    // av_dump_format(format_context, 0, source_filename.as_ptr(), 0);

    let frame = av_frame_alloc()
      .as_mut()
      .context("Failed to allocate frame")?;
    let packet = av_packet_alloc()
      .as_mut()
      .context("Failed to allocate packet")?;

    while av_read_frame(format_context, packet) >= 0 {
      if packet.stream_index == video_stream.index {
        decode_packet(
          video_codec_context,
          packet,
          frame,
          decoded_image_data.as_ptr(),
          decoded_image_data_size,
          decoded_image_line_size.as_ptr(),
          file,
        );
      }
      av_packet_unref(packet);
    }

    let pix_fmt_name = CStr::from_ptr(av_get_pix_fmt_name(video_codec_context.pix_fmt));

    println!("{:?}", pix_fmt_name);

    decode_packet(
      video_codec_context,
      null(),
      frame,
      decoded_image_data.as_ptr(),
      decoded_image_data_size,
      decoded_image_line_size.as_ptr(),
      file,
    );
    fclose(file);
  }
  Ok(())
}

fn decode_packet(
  video_codec_context: &mut AVCodecContext,
  packet: *const AVPacket,
  frame: &mut AVFrame,
  decoded_image_data: *const *mut u8,
  decoded_image_data_size: c_int,
  decoded_image_line_size: *const c_int,
  file: &mut FILE,
) {
  unsafe {
    assert!(avcodec_send_packet(video_codec_context, packet) >= 0);

    let mut return_code = 0;
    while return_code >= 0 {
      return_code = avcodec_receive_frame(video_codec_context, frame);
      if return_code < 0 {
        if return_code == AVERROR_EOF || return_code == AVERROR(EAGAIN) {
          break;
        }
        panic!("Error during decoding ({})", return_code);
      }

      assert!(
        frame.width == video_codec_context.width
          && frame.height == video_codec_context.height
          && frame.format == video_codec_context.pix_fmt as i32,
      );
    }

    av_image_copy(
      decoded_image_data,
      decoded_image_line_size,
      (frame.data.as_ptr()) as *const *const u8,
      frame.linesize.as_ptr(),
      video_codec_context.pix_fmt,
      video_codec_context.width,
      video_codec_context.height,
    );

    fwrite(
      *decoded_image_data as *const c_void,
      1,
      decoded_image_data_size as c_ulonglong,
      file,
    );
  }
}
