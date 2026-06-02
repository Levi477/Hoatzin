def main(input):
    # Extract context from preprocessing
    preprocessed = input.get("preprocess_data", {})
    clean_data = preprocessed.get("clean_path", "")
    records = preprocessed.get("usable_records", 0)

    if not preprocessed.get("features_normalized", False):
        raise ValueError("Cannot train on un-normalized data!")

    print(
        f"[INFO] Initializing model training on {records} records from {clean_data}..."
    )

    return {"model_path": "/models/v1_final.pt", "accuracy": 0.94, "loss": 0.03}
