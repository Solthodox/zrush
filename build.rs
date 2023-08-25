fn main() {
    tonic_build::configure()
        .build_server(true)
        .out_dir("src/grpc")
        .compile(&["proto/node.proto"], &["proto/"])
        .expect("Tonic: failed to compile from proto");
}
