// This file is generated by atprs-codegen. Do not edit.
//! Definitions for the `app.bsky.embed.external` namespace.

// app.bsky.embed.external
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Main {
    pub external: External,
}

// app.bsky.embed.external#external
#[derive(serde::Serialize, serde::Deserialize)]
pub struct External {
    pub description: String,
    // pub thumb: ...,
    pub title: String,
    pub uri: String,
}

// app.bsky.embed.external#view
#[derive(serde::Serialize, serde::Deserialize)]
pub struct View {
    pub external: ViewExternal,
}

// app.bsky.embed.external#viewExternal
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ViewExternal {
    pub description: String,
    pub thumb: Option<String>,
    pub title: String,
    pub uri: String,
}
