use atrium_lex::lexicon::*;
use heck::ToPascalCase;
use std::collections::HashMap;

pub(crate) fn find_ref_unions(defs: &HashMap<String, LexUserType>) -> Vec<(String, LexRefUnion)> {
    let mut unions = Vec::new();
    for (key, def) in defs {
        match def {
            LexUserType::Record(record) => {
                let LexRecordRecord::Object(object) = &record.record;
                find_ref_unions_in_object(object, "Record", &mut unions);
            }
            LexUserType::XrpcQuery(query) => {
                if let Some(output) = &query.output {
                    if let Some(schema) = &output.schema {
                        find_ref_unions_in_body_schema(schema, "Output", &mut unions);
                    }
                }
            }
            LexUserType::XrpcProcedure(procedure) => {
                if let Some(input) = &procedure.input {
                    if let Some(schema) = &input.schema {
                        find_ref_unions_in_body_schema(schema, "Input", &mut unions);
                    }
                }
                if let Some(output) = &procedure.output {
                    if let Some(schema) = &output.schema {
                        find_ref_unions_in_body_schema(schema, "Output", &mut unions);
                    }
                }
            }
            LexUserType::XrpcSubscription(subscription) => {
                if let Some(message) = &subscription.message {
                    if let Some(schema) = &message.schema {
                        find_ref_unions_in_subscription_message_schema(
                            schema,
                            "Message",
                            &mut unions,
                        );
                    }
                }
            }
            LexUserType::Array(array) => {
                find_ref_unions_in_array(array, &key.to_pascal_case(), &mut unions);
            }
            LexUserType::Object(object) => {
                find_ref_unions_in_object(object, &key.to_pascal_case(), &mut unions);
            }
            _ => {}
        }
    }
    unions.sort_by_cached_key(|(name, _)| name.clone());
    unions
}

fn find_ref_unions_in_body_schema(
    schema: &LexXrpcBodySchema,
    name: &str,
    unions: &mut Vec<(String, LexRefUnion)>,
) {
    match schema {
        LexXrpcBodySchema::Union(_) => unimplemented!(),
        LexXrpcBodySchema::Object(object) => find_ref_unions_in_object(object, name, unions),
        _ => {}
    }
}

fn find_ref_unions_in_subscription_message_schema(
    schema: &LexXrpcSubscriptionMessageSchema,
    name: &str,
    unions: &mut Vec<(String, LexRefUnion)>,
) {
    match schema {
        LexXrpcSubscriptionMessageSchema::Union(union) => {
            unions.push((name.into(), union.clone()));
        }
        LexXrpcSubscriptionMessageSchema::Object(object) => {
            find_ref_unions_in_object(object, name, unions)
        }
        _ => {}
    }
}

fn find_ref_unions_in_array(array: &LexArray, name: &str, unions: &mut Vec<(String, LexRefUnion)>) {
    if let LexArrayItem::Union(union) = &array.items {
        unions.push((format!("{}Item", name), union.clone()));
    }
}

fn find_ref_unions_in_object(
    object: &LexObject,
    name: &str,
    unions: &mut Vec<(String, LexRefUnion)>,
) {
    for (k, property) in &object.properties {
        match property {
            LexObjectProperty::Union(union) => {
                unions.push((format!("{name}{}Enum", k.to_pascal_case()), union.clone()));
            }
            LexObjectProperty::Array(array) => {
                find_ref_unions_in_array(array, &(name.to_string() + &k.to_pascal_case()), unions);
            }
            _ => {}
        }
    }
}
