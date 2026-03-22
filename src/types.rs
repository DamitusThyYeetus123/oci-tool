use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Descriptor {
    pub mediaType: String,
    pub digest: String,
    pub size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub urls: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifactType: Option<String>,
}
#[derive(Serialize, Deserialize)]
pub struct Manifest {
    pub schemaVersion: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mediaType: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifactType: Option<String>,
    pub config: Descriptor,
    pub layers: Vec<Descriptor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<Descriptor>,
}
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub os: String,
    pub arch: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    pub rootfs: Rootfs,
}
#[derive(Serialize, Deserialize)]
pub struct Rootfs {
    pub diff_ids: Vec<String>,
    pub r#type: String,
}
#[derive(Serialize, Deserialize)]
pub struct ManifestDescriptor {
    pub mediaType: String,
    pub digest: String,
    pub size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub urls: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifactType: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<ManifestPlatform>,
}
#[derive(Serialize, Deserialize)]
pub struct ManifestPlatform {
    pub architecture: String,
    pub os: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
}
#[derive(Serialize, Deserialize)]
pub struct Index {
    pub schemaVersion: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mediaType: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifactType: Option<String>,
    pub manifests: Vec<ManifestDescriptor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<Descriptor>,
}
