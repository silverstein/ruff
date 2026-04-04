use std::fmt;
use std::str::FromStr;

use ruff_python_ast::{PythonVersion, PythonVersionDeserializationError};
use serde::Deserialize;

use crate::metadata::value::RangedValue;

/// A Python version explicitly supported by ty configuration and CLI parsing.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SupportedPythonVersion {
    version: PythonVersion,
    name: &'static str,
}

impl SupportedPythonVersion {
    pub const PY37: Self = Self::new(PythonVersion::PY37, "3.7");
    pub const PY38: Self = Self::new(PythonVersion::PY38, "3.8");
    pub const PY39: Self = Self::new(PythonVersion::PY39, "3.9");
    pub const PY310: Self = Self::new(PythonVersion::PY310, "3.10");
    pub const PY311: Self = Self::new(PythonVersion::PY311, "3.11");
    pub const PY312: Self = Self::new(PythonVersion::PY312, "3.12");
    pub const PY313: Self = Self::new(PythonVersion::PY313, "3.13");
    pub const PY314: Self = Self::new(PythonVersion::PY314, "3.14");
    pub const PY315: Self = Self::new(PythonVersion::PY315, "3.15");
    const VARIANTS: &[Self] = &[
        Self::PY37,
        Self::PY38,
        Self::PY39,
        Self::PY310,
        Self::PY311,
        Self::PY312,
        Self::PY313,
        Self::PY314,
        Self::PY315,
    ];

    const fn new(version: PythonVersion, name: &'static str) -> Self {
        Self { version, name }
    }

    pub const fn as_str(self) -> &'static str {
        self.name
    }

    pub const fn into_inner(self) -> PythonVersion {
        self.version
    }
}

impl fmt::Display for SupportedPythonVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<SupportedPythonVersion> for PythonVersion {
    fn from(value: SupportedPythonVersion) -> Self {
        value.into_inner()
    }
}

impl FromStr for SupportedPythonVersion {
    type Err = SupportedPythonVersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let version = PythonVersion::from_str(s).map_err(SupportedPythonVersionError::Parse)?;

        Self::VARIANTS
            .iter()
            .copied()
            .find(|supported| supported.into_inner() == version)
            .ok_or(SupportedPythonVersionError::Unsupported(version))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SupportedPythonVersionError {
    Parse(PythonVersionDeserializationError),
    Unsupported(PythonVersion),
}

impl fmt::Display for SupportedPythonVersionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(err) => err.fmt(f),
            Self::Unsupported(version) => {
                write!(
                    f,
                    "unsupported value `{version}` for `python-version`; expected one of "
                )?;

                let mut versions = SupportedPythonVersion::VARIANTS.iter().copied();
                if let Some(first) = versions.next() {
                    write!(f, "`{first}`")?;
                }

                for version in versions {
                    write!(f, ", `{version}`")?;
                }

                Ok(())
            }
        }
    }
}

impl std::error::Error for SupportedPythonVersionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Parse(err) => Some(err),
            Self::Unsupported(_) => None,
        }
    }
}

pub(crate) fn deserialize_supported_python_version<'de, D>(
    deserializer: D,
) -> Result<Option<RangedValue<PythonVersion>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(
        Option::<RangedValue<SupportedPythonVersion>>::deserialize(deserializer)?
            .map(|version| version.map_value(PythonVersion::from)),
    )
}

mod serde_impl {
    use super::SupportedPythonVersion;

    impl<'de> serde::Deserialize<'de> for SupportedPythonVersion {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            String::deserialize(deserializer)?
                .parse()
                .map_err(serde::de::Error::custom)
        }
    }

    impl serde::Serialize for SupportedPythonVersion {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.serialize_str(self.as_str())
        }
    }
}

#[cfg(feature = "schemars")]
mod schemars_impl {
    use schemars::{JsonSchema, Schema, SchemaGenerator};
    use serde_json::Value;

    use super::SupportedPythonVersion;

    impl JsonSchema for SupportedPythonVersion {
        fn schema_name() -> std::borrow::Cow<'static, str> {
            std::borrow::Cow::Borrowed("SupportedPythonVersion")
        }

        fn json_schema(_generator: &mut SchemaGenerator) -> Schema {
            let one_of = SupportedPythonVersion::VARIANTS
                .iter()
                .copied()
                .map(|version| {
                    let mut schema = schemars::json_schema!({
                        "type": "string",
                        "const": version.as_str(),
                    });
                    schema.ensure_object().insert(
                        "description".to_string(),
                        Value::String(format!("Python {version}")),
                    );
                    schema.into()
                })
                .collect();

            let mut schema = Schema::default();
            schema
                .ensure_object()
                .insert("oneOf".to_string(), Value::Array(one_of));
            schema
        }
    }
}

#[cfg(feature = "clap")]
mod clap {
    use clap::builder::PossibleValue;

    use super::SupportedPythonVersion;

    impl clap::ValueEnum for SupportedPythonVersion {
        fn value_variants<'a>() -> &'a [Self] {
            SupportedPythonVersion::VARIANTS
        }

        fn to_possible_value(&self) -> Option<PossibleValue> {
            Some(PossibleValue::new(self.as_str()))
        }
    }
}
