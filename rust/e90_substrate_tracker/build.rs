use std::io::Result;

fn main() -> Result<()> {
    let proto_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("proto");

    tonic_prost_build::configure()
        .build_server(true)
        .build_client(false)
        .compile_protos(
            &[
                "cim_common.proto",
                "e90_substrate_tracking.proto",
            ],
            &[proto_dir.to_str().unwrap()],
        )?;

    Ok(())
}

