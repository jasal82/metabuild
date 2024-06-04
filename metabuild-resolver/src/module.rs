use indexmap::IndexMap;
use resolvo::VersionSet;
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version(pub semver::Version);

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}

impl Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VersionVisitor;

        impl<'de> Visitor<'de> for VersionVisitor {
            type Value = Version;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid version string")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                semver::Version::parse(value)
                    .map(Version)
                    .map_err(de::Error::custom)
                /*SemverVersion::parse(value)
                .map(Version)
                .map_err(de::Error::custom)*/
            }
        }

        deserializer.deserialize_str(VersionVisitor)
    }
}

impl From<semver::Version> for Version {
    fn from(value: semver::Version) -> Self {
        Self(value)
    }
}

impl FromStr for Version {
    type Err = semver::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(semver::Version::parse(s)?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VersionReq(pub semver::VersionReq);

impl VersionReq {
    pub fn matches(&self, version: &Version) -> bool {
        self.0.matches(&version.0)
    }
}

impl Display for VersionReq {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}

impl Serialize for VersionReq {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for VersionReq {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VersionReqVisitor;

        impl<'de> Visitor<'de> for VersionReqVisitor {
            type Value = VersionReq;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid version string")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                semver::VersionReq::parse(value)
                    .map(VersionReq)
                    .map_err(de::Error::custom)
                /*SemverVersion::parse(value)
                .map(Version)
                .map_err(de::Error::custom)*/
            }
        }

        deserializer.deserialize_str(VersionReqVisitor)
    }
}

impl VersionSet for VersionReq {
    type V = Version;

    fn contains(&self, version: &Self::V) -> bool {
        self.matches(&version)
    }
}

impl From<semver::VersionReq> for VersionReq {
    fn from(value: semver::VersionReq) -> Self {
        Self(value)
    }
}

impl FromStr for VersionReq {
    type Err = semver::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(semver::VersionReq::parse(s)?))
    }
}

pub struct Module {
    pub name: String,
    pub version: Version,
    pub dependencies: IndexMap<String, VersionReq>,
}

impl Module {
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.into(),
            version: version.parse().unwrap(),
            dependencies: IndexMap::new(),
        }
    }

    pub fn add_dependency(mut self, name: &str, range: &str) -> Self {
        self.dependencies
            .insert(name.to_string(), range.parse().unwrap());
        self
    }
}
