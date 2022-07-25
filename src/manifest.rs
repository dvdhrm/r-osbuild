//! Manifest Format
//!
//! This module implements a strongly-typed Rust representation of the
//! osbuild manifest format. Additionally, it provides serialization and
//! deserialization support to and from JSON.
//!
//! Note that there are multiple versions of the format. They are provided
//! as distinct formats, but share sub-parts of their structures. A special
//! parser allows detecting the format automatically and returning the
//! correct format.

/// Manifest1 Definition
///
/// This type represents the root node of an osbuild manifest v1. It contains
/// a single pipeline and sources.
#[derive(Debug, Default, Eq, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest1 {
    #[serde(default)]
    pub pipeline: Pipeline1,

    #[serde(default)]
    pub sources: Object<Object<Json>>,

    #[serde(default, flatten)]
    object_marker: ObjectMarker,
}

/// Pipeline1 Definition
///
/// This represents a single pipeline of the manifest v1. A pipeline has a set
/// of stages that are executed in order to build it. Optionally, an assembler
/// can be specified, which operates on the output of the final stage and
/// produces the resulting artifact.
///
/// Additionally, a build pipeline can be specified, which is a pipeline by
/// itself and defines the environment the stages of the embedding pipeline are
/// run in. This can be stacked arbitrarily deep.
#[derive(Debug, Default, Eq, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Pipeline1 {
    #[serde(default)]
    pub assembler: Option<Assembler1>,

    #[serde(default)]
    pub build: Option<Box<Build1>>,

    #[serde(default)]
    pub stages: Array<Stage1>,

    #[serde(default, flatten)]
    object_marker: ObjectMarker,
}

/// Assembler1 Definition
///
/// The manifest v1 assemblers are quite similar to the stages, but are limited
/// to one assembler per pipeline. They operate on the output of the final
/// stage and produces the resulting artifact of the pipeline.
#[derive(Debug, Default, Eq, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Assembler1 {
    pub name: String,

    #[serde(default)]
    pub options: Object<Json>,

    #[serde(default, flatten)]
    object_marker: ObjectMarker,
}

/// Build1 Definition
///
/// The build-pipeline definition defines the build environment for another
/// pipeline. It simply combines another pipeline with a runner designator. The
/// runner defines the execution helper used to run stages in the specified
/// build environment.
#[derive(Debug, Default, Eq, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Build1 {
    pub pipeline: Pipeline1,

    pub runner: String,

    #[serde(default, flatten)]
    object_marker: ObjectMarker,
}

/// Stage1 Definition
///
/// The individual stages of a pipeline are defined by this type. They have an
/// associated name to specify the stage-type to pick. Additionally, the option
/// object contains arbitrary options that are passed to the stage.
#[derive(Debug, Default, Eq, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Stage1 {
    pub name: String,

    #[serde(default)]
    pub options: Object<Json>,

    #[serde(default, flatten)]
    object_marker: ObjectMarker,
}

/// JSON Object Mapping
///
/// This type is an alias used to represent JSON objects. It is a simple
/// convenience helper that shows how these objects are represented. Note
/// that keys must be strings to be valid JSON. Hence, only the target
/// value type must be provided.
pub type Object<VALUE> = std::collections::BTreeMap<String, VALUE>;

/// JSON Array Mapping
///
/// This type is an alias used to represent arrays of JSON data of a given
/// type. It is a convenience helper and guarantee on how arrays are
/// represented in the mapping.
pub type Array<VALUE> = Vec<VALUE>;

/// Inner Json Payload
///
/// The `Json` type is used to carry JSON data inside other deserialized
/// objects. In several cases, the manifest format allows for arbitrary
/// inner configuration, as long as it is valid JSON. We represent such
/// cases with this type.
///
/// Ideally, it would be based on the `serde_json::value::RawValue` type.
/// Unfortunately, that type is very much broken in upstream serde_json for
/// many years. Hence, we direct it to `serde_json::value::Value` for now,
/// but allow for future changes to pick an alternative.
pub type Json = serde_json::value::Value;

// Marker for Object Types
//
// The default implementations of serde-derive for maps allow constructing maps
// from serde `Seq`. In case of JSON, this means that a JSON-array can be
// parsed into a rust structure. This is convenient at times, but unintentional
// for strongly-typed parsers. To circumvent this we add an empty, unused
// member of type `ObjectMarker` to our structures and annotate them as
// `flatten`. This causes serde-derive to skip the `Seq` deserializer, as it
// cannot support it in combination with `flatten`.
//
// This is a bit of a hacky workaround for serde-derive. The proper fix would
// be to manually derive `Deserialize` and only provide the importers that we
// actually want. Alternatively, serde-derive would allow us to select the
// types we want to derive, rather than guessing them itself. However, this is
// by far not the only issue with serde we face, so we keep this workaround
// and avoid spending too much time on it.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
struct ObjectMarker {}

#[cfg(test)]
mod tests {
    use super::*;

    // Verify Manifest1 Type
    #[test]
    fn verify_manifest1_type() {
        // Empty instances are allowed, but must be objects.
        assert_eq! {
            serde_json::from_str::<'_, Manifest1>(r#"{}"#).unwrap(),
            Default::default(),
        }
        assert! {
            serde_json::from_str::<'_, Manifest1>(r#"[]"#).unwrap_err().is_data(),
        }

        // Unknown fields are not allowed.
        assert! {
            serde_json::from_str::<'_, Manifest1>(r#"{"foo":"bar"}"#).unwrap_err().is_data(),
        }

        // Pipelines can be embedded directly in object notation.
        assert_eq! {
            serde_json::from_str::<'_, Manifest1>(
                r#"{
                    "pipeline": {}
                }"#
            ).unwrap(),
            Manifest1 {
                pipeline: Pipeline1{ ..Default::default() },
                ..Default::default()
            },
        }

        // Sources are two-layered objects with arbitrary JSON inside.
        assert_eq! {
            serde_json::from_str::<'_, Manifest1>(
                r#"{
                    "sources": {
                        "foo": {
                            "a": 71,
                            "b": "foo"
                        },
                        "bar": {
                            "a": 0,
                            "b": "bar"
                        }
                    }
                }"#
            ).unwrap(),
            Manifest1 {
                sources: Object::from([
                    ("foo".to_owned(), Object::from([
                        ("a".to_owned(), Json::from(71)),
                        ("b".to_owned(), Json::from("foo")),
                    ])),
                    ("bar".to_owned(), Object::from([
                        ("a".to_owned(), Json::from(0)),
                        ("b".to_owned(), Json::from("bar")),
                    ])),
                ]),
                ..Default::default()
            },
        }
    }

    // Verify Pipeline1 Type
    #[test]
    fn verify_pipeline1_type() {
        // Empty instances are allowed, but must be objects.
        assert_eq! {
            serde_json::from_str::<'_, Pipeline1>(r#"{}"#).unwrap(),
            Default::default(),
        }
        assert! {
            serde_json::from_str::<'_, Pipeline1>(r#"[]"#).unwrap_err().is_data(),
        }

        // Unknown fields are not allowed.
        assert! {
            serde_json::from_str::<'_, Pipeline1>(r#"{"foo":"bar"}"#).unwrap_err().is_data(),
        }

        // Nested configurations can be provided via object notation.
        assert_eq! {
            serde_json::from_str::<'_, Pipeline1>(
                r#"{
                    "assembler": {
                        "name": "foobar"
                    },
                    "stages": [
                        { "name": "foobar" }
                    ]
                }"#
            ).unwrap(),
            Pipeline1 {
                assembler: Some(Assembler1 {
                    name: "foobar".to_owned(),
                    ..Default::default()
                }),
                stages: Array::from([
                    Stage1 {
                        name: "foobar".to_owned(),
                        ..Default::default()
                    },
                ]),
                ..Default::default()
            },
        }

        // Nested build pipelines require extra boxes for recursion.
        assert_eq! {
            serde_json::from_str::<'_, Pipeline1>(
                r#"{
                    "build": {
                        "pipeline": {},
                        "runner": "foobar"
                    }
                }"#
            ).unwrap(),
            Pipeline1 {
                build: Some(Box::new(Build1 {
                    pipeline: Pipeline1 { ..Default::default() },
                    runner: "foobar".to_owned(),
                    ..Default::default()
                })),
                ..Default::default()
            },
        }
    }

    // Verify Assembler1 Type
    #[test]
    fn verify_assembler1_type() {
        // Empty instances are not allowed.
        assert! {
            serde_json::from_str::<'_, Assembler1>(r#"{}"#).unwrap_err().is_data(),
        }
        assert! {
            serde_json::from_str::<'_, Assembler1>(r#"[]"#).unwrap_err().is_data(),
        }

        // Unknown fields are not allowed.
        assert! {
            serde_json::from_str::<'_, Assembler1>(r#"{"foo":"bar"}"#).unwrap_err().is_data(),
        }

        // Instances with just a name are valid, but must be objects.
        assert_eq! {
            serde_json::from_str::<'_, Assembler1>(r#"{"name":"foobar"}"#).unwrap(),
            Assembler1 {
                name: "foobar".to_owned(),
                ..Default::default()
            },
        }
        assert! {
            serde_json::from_str::<'_, Assembler1>(r#"["foobar"]"#).unwrap_err().is_data(),
        }

        // Additional options take arbitrary JSON in object notation.
        assert_eq! {
            serde_json::from_str::<'_, Assembler1>(
                r#"{
                    "name": "foobar",
                    "options": {
                        "foo": 0,
                        "bar": 71
                    }
                }"#
            ).unwrap(),
            Assembler1 {
                name: "foobar".to_owned(),
                options: Object::from([
                    ("foo".to_owned(), Json::from(0)),
                    ("bar".to_owned(), Json::from(71)),
                ]),
                ..Default::default()
            },
        }
        assert! {
            serde_json::from_str::<'_, Assembler1>(
                r#"{
                    "name": "foobar",
                    "options": [0, 71]
                }"#
            ).unwrap_err().is_data(),
        }
    }

    // Verify Build1 Type
    #[test]
    fn verify_build1_type() {
        // Empty instances are not allowed.
        assert! {
            serde_json::from_str::<'_, Build1>(r#"{}"#).unwrap_err().is_data(),
        }
        assert! {
            serde_json::from_str::<'_, Build1>(r#"[]"#).unwrap_err().is_data(),
        }

        // Unknown fields are not allowed.
        assert! {
            serde_json::from_str::<'_, Build1>(r#"{"foo":"bar"}"#).unwrap_err().is_data(),
        }

        // Instances require a valid pipeline and runner in object notation.
        assert_eq! {
            serde_json::from_str::<'_, Build1>(
                r#"{
                    "pipeline": {},
                    "runner": "foobar"
                }"#
            ).unwrap(),
            Build1 {
                runner: "foobar".to_owned(),
                ..Default::default()
            },
        }
        assert! {
            serde_json::from_str::<'_, Build1>(
                r#"[{}, "foobar"]"#
            ).unwrap_err().is_data(),
        }

        // Additional pipeline configuration goes into `pipeline`.
        assert_eq! {
            serde_json::from_str::<'_, Build1>(
                r#"{
                    "pipeline": {
                        "stages": [
                            { "name": "foobar" }
                        ]
                    },
                    "runner": "foobar"
                }"#
            ).unwrap(),
            Build1 {
                pipeline: Pipeline1 {
                    stages: Array::from([
                        Stage1 { name: "foobar".to_owned(), ..Default::default() },
                    ]),
                    ..Default::default()
                },
                runner: "foobar".to_owned(),
                ..Default::default()
            },
        }
    }

    // Verify Stage1 Type
    #[test]
    fn verify_stage1_type() {
        // Empty instances are not allowed.
        assert! {
            serde_json::from_str::<'_, Stage1>(r#"{}"#).unwrap_err().is_data(),
        }
        assert! {
            serde_json::from_str::<'_, Stage1>(r#"[]"#).unwrap_err().is_data(),
        }

        // Unknown fields are not allowed.
        assert! {
            serde_json::from_str::<'_, Stage1>(r#"{"foo":"bar"}"#).unwrap_err().is_data(),
        }

        // Instances with just a name are valid, but must be objects.
        assert_eq! {
            serde_json::from_str::<'_, Stage1>(r#"{"name":"foobar"}"#).unwrap(),
            Stage1 {
                name: "foobar".to_owned(),
                ..Default::default()
            },
        }
        assert! {
            serde_json::from_str::<'_, Stage1>(r#"["foobar"]"#).unwrap_err().is_data(),
        }

        // Additional options take arbitrary JSON in object notation.
        assert_eq! {
            serde_json::from_str::<'_, Stage1>(
                r#"{
                    "name": "foobar",
                    "options": {
                        "foo": 0,
                        "bar": 71
                    }
                }"#
            ).unwrap(),
            Stage1 {
                name: "foobar".to_owned(),
                options: Object::from([
                    ("foo".to_owned(), Json::from(0)),
                    ("bar".to_owned(), Json::from(71)),
                ]),
                ..Default::default()
            },
        }
        assert! {
            serde_json::from_str::<'_, Stage1>(
                r#"{
                    "name": "foobar",
                    "options": [0, 71]
                }"#
            ).unwrap_err().is_data(),
        }
    }
}
