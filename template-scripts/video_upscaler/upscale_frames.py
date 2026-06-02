def main(input):
    # Read output from the extraction node
    extraction_data = input.get("extract_frames", {})
    raw_dir = extraction_data.get("frame_directory", "")
    frames = extraction_data.get("total_frames", 0)

    print(f"[INFO] Upscaling {frames} frames from {raw_dir} using GPU...")

    return {"upscaled_directory": "/tmp/frames/upscaled/", "status": "complete"}
