use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// primitives

#[derive(Debug, Serialize, Deserialize)]
pub struct LexBoolean {
    pub desctiption: Option<String>,
    pub default: Option<bool>,
    #[serde(rename = "const")]
    pub const_value: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LexInteger {
    pub desctiption: Option<String>,
    pub default: Option<i64>,
    pub minimum: Option<i64>,
    pub maximum: Option<i64>,
    #[serde(rename = "enum")]
    pub enum_value: Option<Vec<i64>>,
    #[serde(rename = "const")]
    pub const_value: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LexStringFormat {
    Datetime,
    Uri,
    AtUri,
    Did,
    Handle,
    AtIdentifier,
    Nsid,
    Cid,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LexString {
    pub desctiption: Option<String>,
    pub format: Option<LexStringFormat>,
    pub default: Option<String>,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub min_graphemes: Option<usize>,
    pub max_graphemes: Option<usize>,
    #[serde(rename = "enum")]
    pub enum_value: Option<Vec<String>>,
    #[serde(rename = "const")]
    pub const_value: Option<String>,
    pub known_values: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LexUnknown {
    pub desctiption: Option<String>,
}

// ipld types

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LexBytes {
    pub desctiption: Option<String>,
    pub max_length: Option<usize>,
    pub min_length: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LexCidLink {
    pub desctiption: Option<String>,
}

// references

#[derive(Debug, Serialize, Deserialize)]
pub struct LexRef {
    pub desctiption: Option<String>,
    #[serde(rename = "ref")]
    pub ref_value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LexRefUnion {
    pub desctiption: Option<String>,
    pub refs: Vec<String>,
    pub closed: Option<bool>,
}

// blobs

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LexBlob {
    pub desctiption: Option<String>,
    pub accept: Option<Vec<String>>,
    pub max_size: Option<usize>,
}

// complex types

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum LexArrayItem {
    // lexPrimitive
    Boolean(LexBoolean),
    Integer(LexInteger),
    String(LexString),
    Unknown(LexUnknown),
    // lexIpldType
    Bytes(LexBytes),
    CidLink(LexCidLink),
    // lexBlob
    Blob(LexBlob),
    // lexRefVariant
    Ref(LexRef),
    Union(LexRefUnion),
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LexArray {
    pub desctiption: Option<String>,
    pub items: LexArrayItem,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum LexPrimitiveArrayItem {
    // lexPrimitive
    Boolean(LexBoolean),
    Integer(LexInteger),
    String(LexString),
    Unknown(LexUnknown),
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LexPrimitiveArray {
    pub desctiption: Option<String>,
    pub items: LexPrimitiveArrayItem,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LexToken {
    pub desctiption: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum LexObjectProperty {
    // lexRefVariant
    Ref(LexRef),
    Union(LexRefUnion),
    // lexIpldType
    Bytes(LexBytes),
    CidLink(LexCidLink),
    // lexArray
    Array(LexArray),
    // lexBlob
    Blob(LexBlob),
    // lexPrimitive
    Boolean(LexBoolean),
    Integer(LexInteger),
    String(LexString),
    Unknown(LexUnknown),
}
#[derive(Debug, Serialize, Deserialize)]
pub struct LexObject {
    pub desctiption: Option<String>,
    pub required: Option<Vec<String>>,
    pub nullable: Option<Vec<String>>,
    pub properties: Option<HashMap<String, LexObjectProperty>>,
}

// xrpc

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum LexXrpcParametersProperty {
    // lexPrimitive
    Boolean(LexBoolean),
    Integer(LexInteger),
    String(LexString),
    Unknown(LexUnknown),
    // lexPrimitiveArray
    Array(LexPrimitiveArray),
}
#[derive(Debug, Serialize, Deserialize)]
pub struct LexXrpcParameters {
    pub desctiption: Option<String>,
    pub required: Option<Vec<String>>,
    pub properties: HashMap<String, LexXrpcParametersProperty>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum LexXrpcBodySchema {
    // lexRefVariant
    Ref(LexRef),
    Union(LexRefUnion),
    // lexObject
    Object(LexObject),
}
#[derive(Debug, Serialize, Deserialize)]
pub struct LexXrpcBody {
    pub desctiption: Option<String>,
    pub encoding: String,
    pub schema: Option<LexXrpcBodySchema>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum LexXrpcSubscriptionMessageSchema {
    // lexRefVariant
    Ref(LexRef),
    Union(LexRefUnion),
    // lexObject
    Object(LexObject),
}
#[derive(Debug, Serialize, Deserialize)]
pub struct LexXrpcSubscriptionMessage {
    pub desctiption: Option<String>,
    pub schema: Option<LexXrpcSubscriptionMessageSchema>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LexXrpcError {
    pub desctiption: Option<String>,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LexXrpcQuery {
    pub desctiption: Option<String>,
    pub parameters: Option<LexXrpcParameters>,
    pub output: Option<LexXrpcBody>,
    pub errors: Option<Vec<LexXrpcError>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LexXrpcProcedure {
    pub desctiption: Option<String>,
    pub parameters: Option<LexXrpcParameters>,
    pub input: Option<LexXrpcBody>,
    pub output: Option<LexXrpcBody>,
    pub errors: Option<Vec<LexXrpcError>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LexXrpcSubscription {
    pub desctiption: Option<String>,
    pub parameters: Option<LexXrpcParameters>,
    pub message: Option<LexXrpcSubscriptionMessage>,
    pub infos: Option<Vec<LexXrpcError>>,
    pub errors: Option<Vec<LexXrpcError>>,
}

// database

#[derive(Debug, Serialize, Deserialize)]
pub struct LexRecord {
    pub desctiption: Option<String>,
    pub key: Option<String>,
    pub record: LexObject,
}

// core

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum LexUserType {
    // lexRecord
    Record(LexRecord),
    // lexXrpcQuery
    #[serde(rename = "query")]
    XrpcQuery(LexXrpcQuery),
    // lexXrpcProcedure
    #[serde(rename = "procedure")]
    XrpcProcedure(LexXrpcProcedure),
    // lexXrpcSubscription
    #[serde(rename = "subscription")]
    XrpcSubscription(LexXrpcSubscription),
    // lexBlob
    Blob(LexBlob),
    // lexArray
    Array(LexArray),
    // lexToken
    Token(LexToken),
    // lexObject
    Object(LexObject),
    // lexBoolean,
    Boolean(LexBoolean),
    // lexInteger,
    Integer(LexInteger),
    // lexString,
    String(LexString),
    // lexBytes
    Bytes(LexBytes),
    // lexCidLink
    CidLink(LexCidLink),
    // lexUnknown
    Unknown(LexUnknown),
}
