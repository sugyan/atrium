// This file is generated by atrium-codegen. DO NOT EDIT.
#![doc = "Definitions for the `app.bsky.embed.images` namespace."]
#![doc = "A set of images embedded in some other form of content"]
#[derive(serde :: Serialize, serde :: Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Main {
    pub images: Vec<Image>,
}
#[derive(serde :: Serialize, serde :: Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    pub alt: String,
    pub image: crate::blob::BlobRef,
}
#[derive(serde :: Serialize, serde :: Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub images: Vec<ViewImage>,
}
#[derive(serde :: Serialize, serde :: Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ViewImage {
    pub alt: String,
    pub fullsize: String,
    pub thumb: String,
}
