use atrium_lex::lexicon::*;
use heck::{ToPascalCase, ToSnakeCase};
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashSet;
use syn::{Path, Result};

pub(crate) struct LexConverter {
    schema_id: String,
}

impl LexConverter {
    pub fn new(schema_id: String) -> Self {
        Self { schema_id }
    }
    pub fn convert(&self, name: &str, def: &LexUserType, is_main: bool) -> Result<TokenStream> {
        let def_name = if is_main {
            String::new()
        } else {
            format!("#{name}")
        };
        let description = format!("`{}{}`", self.schema_id, def_name);
        let user_type = match def {
            LexUserType::Record(record) => self.record(record)?,
            LexUserType::XrpcQuery(query) => self.query(name, query)?,
            LexUserType::XrpcProcedure(procedure) => self.procedure(name, procedure)?,
            LexUserType::XrpcSubscription(subscription) => self.subscription(name, subscription)?,
            LexUserType::Token(token) => self.token(name, token)?,
            LexUserType::Object(object) => {
                self.object(if is_main { "Main" } else { name }, object)?
            }
            LexUserType::String(string) => self.string(name, string)?,
            _ => unimplemented!("{def:?}"),
        };
        Ok(quote! {
            #[doc = #description]
            #user_type
        })
    }
    pub fn ref_unions(&self, ref_unions: &[(String, LexRefUnion)]) -> Result<TokenStream> {
        let mut enums = Vec::new();
        for (name, ref_union) in ref_unions {
            enums.push(self::refs_enum(
                name,
                &ref_union.refs,
                Some(&self.schema_id),
            )?);
        }
        Ok(quote!(#(#enums)*))
    }
    fn record(&self, record: &LexRecord) -> Result<TokenStream> {
        let LexRecordRecord::Object(object) = &record.record;
        self.object("Record", object)
    }
    fn xrpc_parameters(&self, parameters: &LexXrpcParameters) -> Result<TokenStream> {
        let properties = parameters
            .properties
            .iter()
            .map(|(k, v)| {
                let value = match v {
                    LexXrpcParametersProperty::Boolean(boolean) => {
                        LexObjectProperty::Boolean(boolean.clone())
                    }
                    LexXrpcParametersProperty::Integer(integer) => {
                        LexObjectProperty::Integer(integer.clone())
                    }
                    LexXrpcParametersProperty::String(string) => {
                        LexObjectProperty::String(string.clone())
                    }
                    LexXrpcParametersProperty::Unknown(unknown) => {
                        LexObjectProperty::Unknown(unknown.clone())
                    }
                    LexXrpcParametersProperty::Array(primitive_array) => {
                        LexObjectProperty::Array(LexArray {
                            description: primitive_array.description.clone(),
                            items: match &primitive_array.items {
                                LexPrimitiveArrayItem::Boolean(b) => {
                                    LexArrayItem::Boolean(b.clone())
                                }
                                LexPrimitiveArrayItem::Integer(i) => {
                                    LexArrayItem::Integer(i.clone())
                                }
                                LexPrimitiveArrayItem::String(s) => LexArrayItem::String(s.clone()),
                                LexPrimitiveArrayItem::Unknown(u) => {
                                    LexArrayItem::Unknown(u.clone())
                                }
                            },
                            min_length: primitive_array.min_length,
                            max_length: primitive_array.max_length,
                        })
                    }
                };
                (k.clone(), value)
            })
            .collect();
        self.object(
            "Parameters",
            &LexObject {
                description: parameters.description.clone(),
                required: parameters.required.clone(),
                nullable: None,
                properties: Some(properties),
            },
        )
    }
    fn xrpc_body(&self, name: &str, body: &LexXrpcBody) -> Result<TokenStream> {
        let description = self.description(&body.description);
        let schema = if let Some(schema) = &body.schema {
            match schema {
                LexXrpcBodySchema::Ref(r#ref) => {
                    let type_name = format_ident!("{}", name.to_pascal_case());
                    let (description, ref_type) = self.ref_type(r#ref)?;
                    quote! {
                        #description
                        pub type #type_name = #ref_type;
                    }
                }
                LexXrpcBodySchema::Object(object) => self.object(name, object)?,
                _ => unimplemented!("{schema:?}"),
            }
        } else {
            return Ok(quote!());
        };
        Ok(quote! {
            #description
            #schema
        })
    }
    fn xrpc_errors(&self, errors: &Option<Vec<LexXrpcError>>) -> Result<TokenStream> {
        let derives = self::derives()?;
        let variants = errors.as_ref().map_or(Vec::new(), |e| {
            e.iter()
                .map(|error| {
                    let description = self.description(&error.description);
                    let name = format_ident!("{}", error.name.to_pascal_case());
                    quote! {
                        #description
                        #name(Option<String>)
                    }
                })
                .collect()
        });
        Ok(quote! {
            #derives
            #[serde(tag = "error", content = "message")]
            pub enum Error {
                #(#variants),*
            }
        })
    }
    fn query(&self, name: &str, query: &LexXrpcQuery) -> Result<TokenStream> {
        let description = self.description(&query.description);
        let parameters = &query.parameters;
        let output = query.output.as_ref();
        let trait_impl: TokenStream = self.xrpc_trait_query(
            name,
            parameters.is_some(),
            output.map_or(false, |o| o.schema.is_some()),
        )?;
        let params = if let Some(LexXrpcQueryParameter::Params(parameters)) = parameters {
            self.xrpc_parameters(parameters)?
        } else {
            quote!()
        };
        let outputs = if let Some(body) = output {
            self.xrpc_body("Output", body)?
        } else {
            quote!()
        };
        let errors = self.xrpc_errors(&query.errors)?;
        Ok(quote! {
            #description
            #trait_impl
            #params
            #outputs
            #errors
        })
    }
    fn procedure(&self, name: &str, procedure: &LexXrpcProcedure) -> Result<TokenStream> {
        let description = self.description(&procedure.description);
        let input = procedure.input.as_ref();
        let output = procedure.output.as_ref();
        let trait_impl: TokenStream =
            self.xrpc_trait_procedure(name, input, output.map_or(false, |o| o.schema.is_some()))?;
        let inputs = if let Some(body) = input {
            self.xrpc_body("Input", body)?
        } else {
            quote!()
        };
        let outputs = if let Some(body) = output {
            self.xrpc_body("Output", body)?
        } else {
            quote!()
        };
        let errors = self.xrpc_errors(&procedure.errors)?;
        Ok(quote! {
            #description
            #trait_impl
            #inputs
            #outputs
            #errors
        })
    }
    fn xrpc_trait_query(
        &self,
        name: &str,
        has_params: bool,
        has_output: bool,
    ) -> Result<TokenStream> {
        let mut args = vec![quote!(&self)];
        if has_params {
            args.push(quote!(params: Parameters));
        }
        let param_value = if has_params {
            quote!(Some(serde_urlencoded::to_string(&params)?))
        } else {
            quote!(None)
        };
        let nsid = &self.schema_id;
        let xrpc_call = quote! {
            crate::xrpc::XrpcClient::send::<Error>(
                self,
                http::Method::GET,
                #nsid,
                #param_value,
                None,
                None,
            )
            .await?
        };
        self.xrpc_trait_common(name, &xrpc_call, &args, has_output)
    }
    fn xrpc_trait_procedure(
        &self,
        name: &str,
        input: Option<&LexXrpcBody>,
        has_output: bool,
    ) -> Result<TokenStream> {
        let mut args = vec![quote!(&self)];
        if let Some(body) = &input {
            if body.schema.is_some() {
                args.push(quote!(input: Input));
            } else {
                args.push(quote!(input: Vec<u8>));
            }
        }
        let (input_value, encoding) = if let Some(body) = &input {
            let encoding = &body.encoding;
            if body.schema.is_some() {
                (
                    quote!(Some(serde_json::to_vec(&input)?)),
                    quote!(Some(String::from(#encoding))),
                )
            } else {
                (quote!(Some(input)), quote!(Some(String::from(#encoding))))
            }
        } else {
            (quote!(None), quote!(None))
        };
        let nsid = &self.schema_id;
        let xrpc_call = quote! {
            crate::xrpc::XrpcClient::send::<Error>(
                self,
                http::Method::POST,
                #nsid,
                None,
                #input_value,
                #encoding,
            )
            .await?
        };
        self.xrpc_trait_common(name, &xrpc_call, &args, has_output)
    }
    fn xrpc_trait_common(
        &self,
        name: &str,
        xrpc_call: &TokenStream,
        args: &[TokenStream],
        has_output: bool,
    ) -> Result<TokenStream> {
        let trait_name = format_ident!("{}", name.to_pascal_case());
        let method_name = format_ident!("{}", name.to_snake_case());
        let body = if has_output {
            quote! {
                async fn #method_name(#(#args),*) -> Result<Output, Box<dyn std::error::Error>> {
                    let body = #xrpc_call;
                    serde_json::from_slice(&body).map_err(|e| e.into())
                }
            }
        } else {
            quote! {
                async fn #method_name(#(#args),*) -> Result<(), Box<dyn std::error::Error>> {
                    let _ = #xrpc_call;
                    Ok(())
                }
            }
        };
        Ok(quote! {
            #[async_trait::async_trait]
            pub trait #trait_name: crate::xrpc::XrpcClient {
                #body
            }
        })
    }
    fn subscription(&self, name: &str, subscription: &LexXrpcSubscription) -> Result<TokenStream> {
        let description = self.description(&subscription.description);
        let subscription_name = format_ident!("{}", name.to_pascal_case());
        // TODO
        Ok(quote! {
            #description
            pub struct #subscription_name;
        })
    }
    fn token(&self, name: &str, token: &LexToken) -> Result<TokenStream> {
        let description = self.description(&token.description);
        let token_name = format_ident!("{}", name.to_pascal_case());
        // TODO
        Ok(quote! {
            #description
            pub struct #token_name;
        })
    }
    fn object(&self, name: &str, object: &LexObject) -> Result<TokenStream> {
        let description = self.description(&object.description);
        let derives = self::derives()?;
        let struct_name = format_ident!("{}", name.to_pascal_case());
        let required = if let Some(required) = &object.required {
            HashSet::from_iter(required)
        } else {
            HashSet::new()
        };
        let mut fields = Vec::new();
        if let Some(properties) = &object.properties {
            for key in properties.keys().sorted() {
                fields.push(self.object_property(
                    key,
                    &properties[key],
                    required.contains(key),
                    name,
                )?);
            }
        }
        Ok(quote! {
            #description
            #derives
            #[serde(rename_all = "camelCase")]
            pub struct #struct_name {
                #(#fields)*
            }
        })
    }
    fn object_property(
        &self,
        name: &str,
        property: &LexObjectProperty,
        is_required: bool,
        object_name: &str,
    ) -> Result<TokenStream> {
        let (description, field_type) = match property {
            LexObjectProperty::Ref(r#ref) => self.ref_type(r#ref)?,
            LexObjectProperty::Union(union) => self.union_type(
                union,
                format!(
                    "{}{}Enum",
                    object_name.to_pascal_case(),
                    name.to_pascal_case()
                )
                .as_str(),
            )?,
            LexObjectProperty::Bytes(bytes) => self.bytes_type(bytes)?,
            LexObjectProperty::CidLink(cid_link) => self.cid_link_type(cid_link)?,
            LexObjectProperty::Array(array) => {
                let description = self.description(&array.description);
                let (_, item_type) = match &array.items {
                    LexArrayItem::Integer(integer) => self.integer_type(integer)?,
                    LexArrayItem::String(string) => self.string_type(string)?,
                    LexArrayItem::Unknown(unknown) => self.unknown_type(unknown)?,
                    LexArrayItem::CidLink(cid_link) => self.cid_link_type(cid_link)?,
                    LexArrayItem::Ref(r#ref) => self.ref_type(r#ref)?,
                    LexArrayItem::Union(union) => self.union_type(
                        union,
                        format!(
                            "{}{}Item",
                            object_name.to_pascal_case(),
                            name.to_pascal_case()
                        )
                        .as_str(),
                    )?,
                    _ => unimplemented!("{:?}", array.items),
                };
                // TODO: must be determined
                if item_type.is_empty() {
                    return Ok(quote!());
                }
                (description, quote!(Vec<#item_type>))
            }
            LexObjectProperty::Blob(blob) => self.blob_type(blob)?,
            LexObjectProperty::Boolean(boolean) => self.boolean_type(boolean)?,
            LexObjectProperty::Integer(integer) => self.integer_type(integer)?,
            LexObjectProperty::String(string) => self.string_type(string)?,
            LexObjectProperty::Unknown(unknown) => self.unknown_type(unknown)?,
        };
        // TODO: must be determined
        if field_type.is_empty() {
            return Ok(quote!());
        }
        // TODO: other keywords?
        let field_name = format_ident!(
            "{}",
            if name == "type" {
                String::from("r#type")
            } else {
                name.to_snake_case()
            }
        );
        Ok(if is_required {
            quote! {
                #description
                pub #field_name: #field_type,
            }
        } else {
            quote! {
                #description
                #[serde(skip_serializing_if = "Option::is_none")]
                pub #field_name: Option<#field_type>,
            }
        })
    }
    fn string(&self, name: &str, string: &LexString) -> Result<TokenStream> {
        let description = self.description(&string.description);
        let string_name = format_ident!("{}", name.to_pascal_case());
        Ok(quote! {
            #description
            pub type #string_name = String;
        })
    }
    fn ref_type(&self, r#ref: &LexRef) -> Result<(TokenStream, TokenStream)> {
        let description = self.description(&r#ref.description);
        Ok((description, self::resolve_ref(&r#ref.r#ref, "main")?))
    }
    fn union_type(
        &self,
        union: &LexRefUnion,
        enum_name: &str,
    ) -> Result<(TokenStream, TokenStream)> {
        let description = self.description(&union.description);
        let enum_type_name = format_ident!("{}", enum_name);
        // Use `Box` to avoid recursive.
        Ok((description, quote!(Box<#enum_type_name>)))
    }
    fn bytes_type(&self, bytes: &LexBytes) -> Result<(TokenStream, TokenStream)> {
        let description = self.description(&bytes.description);
        // TODO
        Ok((description, quote!()))
    }
    fn cid_link_type(&self, cid_link: &LexCidLink) -> Result<(TokenStream, TokenStream)> {
        let description = self.description(&cid_link.description);
        // TODO
        Ok((description, quote!()))
    }
    fn blob_type(&self, blob: &LexBlob) -> Result<(TokenStream, TokenStream)> {
        let description = self.description(&blob.description);
        Ok((description, quote!(crate::blob::BlobRef)))
    }
    fn boolean_type(&self, boolean: &LexBoolean) -> Result<(TokenStream, TokenStream)> {
        let description = self.description(&boolean.description);
        Ok((description, quote!(bool)))
    }
    fn integer_type(&self, integer: &LexInteger) -> Result<(TokenStream, TokenStream)> {
        let description = self.description(&integer.description);
        // TODO: usize?
        Ok((description, quote!(i32)))
    }
    fn string_type(&self, string: &LexString) -> Result<(TokenStream, TokenStream)> {
        let description = self.description(&string.description);
        // TODO: format, enum?
        Ok((description, quote!(String)))
    }
    fn unknown_type(&self, unknown: &LexUnknown) -> Result<(TokenStream, TokenStream)> {
        let description = self.description(&unknown.description);
        Ok((description, quote!(crate::records::Record)))
    }
    fn description(&self, description: &Option<String>) -> TokenStream {
        if let Some(description) = description {
            quote!(#[doc = #description])
        } else {
            quote!()
        }
    }
}

pub(crate) fn refs_enum(
    name: &str,
    refs: &[String],
    schema_id: Option<&String>,
) -> Result<TokenStream> {
    let is_record = schema_id.is_none();
    let derives = self::derives()?;
    let enum_name = format_ident!("{name}");
    let mut variants = Vec::new();
    for r#ref in refs {
        let default = if is_record { "record" } else { "main" };
        let path = self::resolve_ref(r#ref, default)?;
        let rename = if r#ref.starts_with('#') {
            format!(
                "{}{}",
                schema_id.expect("schema id must be specified"),
                r#ref
            )
        } else {
            r#ref.clone()
        };
        let s = path.to_string().replace(' ', "");
        let mut parts = s
            .strip_prefix("crate::")
            .unwrap_or(&s)
            .split("::")
            .map(str::to_pascal_case)
            .collect_vec();
        if is_record {
            parts.pop();
        }
        let name = format_ident!("{}", parts.join(""));
        variants.push(quote! {
            #[serde(rename = #rename)]
            #name(#path)
        });
    }
    Ok(quote! {
        #derives
        #[serde(tag = "$type")]
        pub enum #enum_name {
            #(#variants),*
        }
    })
}

pub(crate) fn traits_macro(traits: &[String]) -> Result<TokenStream> {
    let mut paths = Vec::new();
    let mut roots = HashSet::new();
    for t in traits {
        let parts = t.split('.').collect_vec();
        roots.insert(format_ident!("{}", parts[0].to_snake_case()));
        let basename = parts.last().unwrap();
        let path = syn::parse_str::<Path>(&format!(
            "{}::{}",
            parts.iter().map(|s| s.to_snake_case()).join("::"),
            basename.to_pascal_case()
        ))?;
        paths.push(quote! {
            impl #path for $type {}
        });
    }
    let roots = roots.into_iter().sorted().collect_vec();
    Ok(quote! {
        #[macro_export]
        macro_rules! impl_traits {
            ($type:ty) => {
                use atrium_api::{#(#roots),*};
                #(#paths)*
            }
        }
    })
}

pub(crate) fn modules(names: &[String]) -> Result<TokenStream> {
    let v = names
        .iter()
        .filter_map(|s| {
            if s != "lib" {
                let m = format_ident!("{s}");
                Some(quote!(pub mod #m;))
            } else {
                None
            }
        })
        .collect_vec();
    Ok(quote!(#(#v)*))
}

fn derives() -> Result<TokenStream> {
    let mut derives = Vec::new();
    for derive in &[
        "serde::Serialize",
        "serde::Deserialize",
        "Debug",
        "Clone",
        "PartialEq",
        "Eq",
    ] {
        derives.push(syn::parse_str::<Path>(derive)?);
    }
    Ok(quote!(#[derive(#(#derives),*)]))
}

fn resolve_ref(r#ref: &str, default: &str) -> Result<TokenStream> {
    let (namespace, def) = r#ref.split_once('#').unwrap_or((r#ref, default));
    let path = syn::parse_str::<Path>(&if namespace.is_empty() {
        def.to_pascal_case()
    } else {
        format!(
            "crate::{}::{}",
            namespace.split('.').map(str::to_snake_case).join("::"),
            def.to_pascal_case()
        )
    })?;
    Ok(quote!(#path))
}
