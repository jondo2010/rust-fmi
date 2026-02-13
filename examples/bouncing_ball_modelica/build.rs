use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let model_path = manifest_dir.join("src/bouncing_ball.mo");
    println!("cargo:rerun-if-changed={}", model_path.display());
    fmi_export::rumoca::write_modelica_to_out_dir("BouncingBall", &model_path)
        .expect("render bouncing_ball.mo");
}
