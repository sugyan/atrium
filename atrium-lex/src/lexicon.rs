use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::HashMap;

// primitives

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct LexBoolean {
    pub description: Option<String>,
    pub default: Option<bool>,
    pub r#const: Option<bool>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct LexInteger {
    pub description: Option<String>,
    pub default: Option<i64>,
    pub minimum: Option<i64>,
    pub maximum: Option<i64>,
    pub r#enum: Option<Vec<i64>>,
    pub r#const: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
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
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LexString {
    pub description: Option<String>,
    pub format: Option<LexStringFormat>,
    pub default: Option<String>,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub min_graphemes: Option<usize>,
    pub max_graphemes: Option<usize>,
    pub r#enum: Option<Vec<String>>,
    pub r#const: Option<String>,
    pub known_values: Option<Vec<String>>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct LexUnknown {
    pub description: Option<String>,
}

// ipld types

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LexBytes {
    pub description: Option<String>,
    pub max_length: Option<usize>,
    pub min_length: Option<usize>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct LexCidLink {
    pub description: Option<String>,
}

// references

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct LexRef {
    pub description: Option<String>,
    pub r#ref: String,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct LexRefUnion {
    pub description: Option<String>,
    pub refs: Vec<String>,
    pub closed: Option<bool>,
}

// blobs

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LexBlob {
    pub description: Option<String>,
    pub accept: Option<Vec<String>>,
    pub max_size: Option<usize>,
}

// complex types

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
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
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LexArray {
    pub description: Option<String>,
    pub items: LexArrayItem,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum LexPrimitiveArrayItem {
    // lexPrimitive
    Boolean(LexBoolean),
    Integer(LexInteger),
    String(LexString),
    Unknown(LexUnknown),
}
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LexPrimitiveArray {
    pub description: Option<String>,
    pub items: LexPrimitiveArrayItem,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct LexToken {
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
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
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct LexObject {
    pub description: Option<String>,
    pub required: Option<Vec<String>>,
    pub nullable: Option<Vec<String>>,
    pub properties: Option<HashMap<String, LexObjectProperty>>,
}

// xrpc

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
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
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct LexXrpcParameters {
    pub description: Option<String>,
    pub required: Option<Vec<String>>,
    pub properties: HashMap<String, LexXrpcParametersProperty>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum LexXrpcBodySchema {
    // lexRefVariant
    Ref(LexRef),
    Union(LexRefUnion),
    // lexObject
    Object(LexObject),
}
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct LexXrpcBody {
    pub description: Option<String>,
    pub encoding: String,
    pub schema: Option<LexXrpcBodySchema>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum LexXrpcSubscriptionMessageSchema {
    // lexRefVariant
    Ref(LexRef),
    Union(LexRefUnion),
    // lexObject
    Object(LexObject),
}
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct LexXrpcSubscriptionMessage {
    pub description: Option<String>,
    pub schema: Option<LexXrpcSubscriptionMessageSchema>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct LexXrpcError {
    pub description: Option<String>,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum LexXrpcQueryParameter {
    Params(LexXrpcParameters),
}
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct LexXrpcQuery {
    pub description: Option<String>,
    pub parameters: Option<LexXrpcQueryParameter>,
    pub output: Option<LexXrpcBody>,
    pub errors: Option<Vec<LexXrpcError>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum LexXrpcProcedureParameter {
    Params(LexXrpcParameters),
}
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct LexXrpcProcedure {
    pub description: Option<String>,
    pub parameters: Option<LexXrpcProcedureParameter>,
    pub input: Option<LexXrpcBody>,
    pub output: Option<LexXrpcBody>,
    pub errors: Option<Vec<LexXrpcError>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum LexXrpcSubscriptionParameter {
    Params(LexXrpcParameters),
}
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct LexXrpcSubscription {
    pub description: Option<String>,
    pub parameters: Option<LexXrpcSubscriptionParameter>,
    pub message: Option<LexXrpcSubscriptionMessage>,
    pub infos: Option<Vec<LexXrpcError>>,
    pub errors: Option<Vec<LexXrpcError>>,
}

// database

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum LexRecordRecord {
    Object(LexObject),
}
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct LexRecord {
    pub description: Option<String>,
    pub key: Option<String>,
    pub record: LexRecordRecord,
}

// core

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
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
