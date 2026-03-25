mod types;
use crate::types::{
    Config, ContainerConfig, Descriptor, Index, Manifest, ManifestDescriptor, Rootfs,
};
use clap::Parser;
use flate2::Compression;
use flate2::write::GzEncoder;
use sha2::{Digest, Sha256};
use std::fmt::Write;
use std::fs::File;
use std::path::PathBuf;
use std::{fs, io};

struct Layer {
    layer_path: PathBuf,
    image_path: String,
    hash: String,
    size: u64,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to use as the root filesystem
    #[arg(short, long)]
    rootfs: String,
    /// Pair of source and image paths used to define additional layers, e.g.
    /// source_path:image_path
    #[arg(short, long)]
    layer: Vec<String>,
    /// Output directory to create image
    #[arg(short, long, default_value = "result")]
    output: String,
    /// Command to use on container startup
    #[arg(short, long)]
    command: Option<Vec<String>>,
    /// Working Directory to run the container's commands in
    #[arg(short, long)]
    workingdir: Option<String>,
    /// Environment variables to pass to the container, e.g. var=content
    #[arg(short, long)]
    env: Option<Vec<String>>,
    /// Compress layers
    #[arg(long)]
    compress: bool,
}

fn create_layer(
    read_path: &str,
    image_path: &str,
    outpath: &str,
    compress: bool,
) -> Result<Layer, std::io::Error> {
    // Read path and create compressed tarball
    fs::create_dir_all("tmp")?;
    let tarball = File::create("tmp/layer.tar")?;
    let mut tar = tar::Builder::new(tarball);
    tar.follow_symlinks(false);
    tar.sparse(false);
    if (fs::metadata(read_path)?.is_dir()) {
        tar.append_dir_all(format!("./{image_path}"), read_path)?;
    } else {
        tar.append_path_with_name(read_path, format!("./{image_path}"))?;
    }
    let mut layer_size = tar.into_inner()?.metadata()?.len();

    // Hash tarball and move to correct directory
    let mut hasher = Sha256::new();
    let mut file = fs::File::open("tmp/layer.tar")?;

    let hash_contents = io::copy(&mut file, &mut hasher)?;
    let hash_bytes = hasher.finalize();

    let mut hash_str = String::new();
    for &byte in hash_bytes.as_slice() {
        write!(&mut hash_str, "{:02x}", byte).expect("Unable to write");
    }
    fs::create_dir_all(format!("{outpath}/blobs/sha256")).expect("failed to create dir");
    if (compress == true) {
        let compressed = File::create(format!("{outpath}/blobs/sha256/{hash_str}"))?;
        let mut encoder = GzEncoder::new(compressed, Compression::default());
        let mut file = fs::File::open("tmp/layer.tar")?;
        io::copy(&mut file, &mut encoder).expect("failed to move layer");

        layer_size = encoder.finish()?.metadata()?.len();
    }

    // Return Layer
    Ok(Layer {
        layer_path: PathBuf::from(format!("{outpath}/blobs/sha256/{hash_str}")),
        image_path: image_path.to_string(),
        hash: hash_str.clone(),
        size: layer_size,
    })
}

fn make_config(rootfs: &Layer, layers: &Vec<Layer>, containerconf: ContainerConfig) -> Config {
    let mut layer_digests: Vec<String> = Vec::new();
    layer_digests.push(format!("sha256:{0}", rootfs.hash));
    for layer in layers {
        layer_digests.push(format!("sha256:{0}", layer.hash));
    }
    Config {
        os: "linux".to_string(),
        arch: "amd64".to_string(),
        created: None,
        author: None,
        config: Some(containerconf),
        rootfs: Rootfs {
            diff_ids: layer_digests,
            r#type: "layers".to_string(),
        },
    }
}

fn make_manifest(
    rootfs: &Layer,
    layers: &Vec<Layer>,
    config_hash: &String,
    config_size: u64,
) -> Manifest {
    let mut layerDescriptors: Vec<Descriptor> = Vec::new();
    layerDescriptors.push(Descriptor {
        mediaType: "application/vnd.oci.image.layer.v1.tar+gzip".to_string(),
        digest: format!("sha256:{0}", rootfs.hash),
        size: rootfs.size,
        urls: None,
        data: None,
        artifactType: None,
    });
    for layer in layers {
        layerDescriptors.push(Descriptor {
            mediaType: "application/vnd.oci.image.layer.v1.tar+gzip".to_string(),
            digest: format!("sha256:{0}", layer.hash),
            size: layer.size,
            urls: None,
            data: None,
            artifactType: None,
        })
    }
    Manifest {
        schemaVersion: 2,
        mediaType: Some("application/vnd.oci.image.manifest.v1+json".to_string()),
        config: Descriptor {
            mediaType: "application/vnd.oci.image.config.v1+json".to_string(),
            size: config_size,
            digest: format!("sha256:{config_hash}"),
            artifactType: None,
            data: None,
            urls: None,
        },
        layers: layerDescriptors,
        subject: None,
        artifactType: None,
    }
}

fn make_index(manifests: Vec<ManifestDescriptor>) -> Index {
    Index {
        schemaVersion: 2,
        mediaType: Some("application/vnd.oci.image.index.v1+json".to_string()),
        artifactType: None,
        manifests,
        subject: None,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let root = create_layer(cli.rootfs.as_str(), "/", cli.output.as_str(), cli.compress)?;
    let mut layers: Vec<Layer> = Vec::new();
    for path in cli.layer {
        let layer = create_layer(
            path.split_once(":").unwrap().0,
            path.split_once(":").unwrap().1,
            cli.output.as_str(),
            cli.compress,
        )?;
        layers.push(layer);
    }
    println!("Finished creating layers, generating config..");
    let config = make_config(
        &root,
        &layers,
        ContainerConfig {
            User: None,
            Env: cli.env,
            Entrypoint: None,
            Cmd: cli.command,
            WorkingDir: cli.workingdir,
        },
    );
    println!("Config generated");
    fs::write("tmp/config.json", serde_json::to_string_pretty(&config)?)?;
    println!("Config written successfully");
    let mut hasher = Sha256::new();
    let mut file = fs::File::open("tmp/config.json")?;

    let hash_contents = io::copy(&mut file, &mut hasher)?;
    let hash_bytes = hasher.finalize();

    let mut config_hash = String::new();
    for &byte in hash_bytes.as_slice() {
        write!(&mut config_hash, "{:02x}", byte).expect("Unable to write");
    }

    fs::copy(
        "tmp/config.json",
        format!("{0}/blobs/sha256/{1}", cli.output, config_hash),
    )?;

    let manifest = make_manifest(
        &root,
        &layers,
        &config_hash,
        fs::metadata(format!("{0}/blobs/sha256/{1}", cli.output, config_hash))?.len(),
    );

    fs::write(
        "tmp/manifest.json",
        serde_json::to_string_pretty(&manifest)?,
    )?;
    let mut hasher = Sha256::new();
    let mut file = fs::File::open("tmp/manifest.json")?;

    let hash_contents = io::copy(&mut file, &mut hasher)?;
    let hash_bytes = hasher.finalize();

    let mut manifest_hash = String::new();
    for &byte in hash_bytes.as_slice() {
        write!(&mut manifest_hash, "{:02x}", byte).expect("Unable to write");
    }

    fs::copy(
        "tmp/manifest.json",
        format!("{0}/blobs/sha256/{1}", cli.output, manifest_hash),
    )?;

    let mut manifests: Vec<ManifestDescriptor> = Vec::new();
    manifests.push(ManifestDescriptor {
        mediaType: "application/vnd.oci.image.manifest.v1+json".to_string(),
        digest: format!("sha256:{0}", manifest_hash),
        size: fs::metadata(format!("{0}/blobs/sha256/{1}", cli.output, manifest_hash))?.len(),
        urls: None,
        data: None,
        artifactType: None,
        platform: None,
    });
    let index = make_index(manifests);
    fs::write(
        format!("{0}/index.json", cli.output),
        serde_json::to_string_pretty(&index)?,
    )?;
    fs::remove_dir_all("tmp")?;
    Ok(())
}
