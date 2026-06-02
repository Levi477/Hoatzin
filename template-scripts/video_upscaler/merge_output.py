def main(input):
    # Fan-In: Read data from BOTH upstream nodes
    audio_data = input.get("extract_audio", {})
    video_data = input.get("upscale_frames", {})

    audio_track = audio_data.get("audio_track", "missing_audio.aac")
    upscaled_dir = video_data.get("upscaled_directory", "missing_dir")

    print(f"[INFO] Merging {upscaled_dir} with {audio_track}...")

    return {
        "final_render": "/exports/final_upscaled_video.mp4",
        "render_status": "Success",
    }
