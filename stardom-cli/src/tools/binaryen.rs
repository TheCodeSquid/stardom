use std::fmt;

use anyhow::{bail, Result};
use camino::Utf8Path;
use serde::de::{self, Deserialize, Deserializer, Unexpected};
use tokio::process::Command;

use super::{Target, Tool, Tools};

const TOOLS: Tools = Tools {
    name: "binaryen",
    version: "116",
    url: |version, Target { os, arch }| {
        let target = match (os, arch) {
            ("windows", "x86_64") => "x86_64-windows",
            ("macos", "x86_64") => "x86_64-macos",
            ("macos", "aarch64") => "arm64-macos",
            ("linux", "x86_64") => "x86_64-linux",
            _ => bail!("unable to download binaryen for {os} {arch}"),
        };
        Ok(format!("https://github.com/WebAssembly/binaryen/releases/download/version_{version}/binaryen-version_{version}-{target}.tar.gz"))
    },
    tools: &["wasm-opt"],
};

const WASM_OPT: Tool = TOOLS.tool("wasm-opt");

pub async fn wasm_opt(level: OptLevel, input: &Utf8Path, output: &Utf8Path) -> Result<Command> {
    WASM_OPT.command().await.map(|mut cmd| {
        cmd.args([
            input.as_str(),
            &format!("-O{}", level),
            &format!("--output={}", output),
            "--quiet",
        ]);
        cmd
    })
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum OptLevel {
    #[default]
    Default,
    Zero,
    One,
    Two,
    Three,
    Four,
    S,
    Z,
}

impl OptLevel {
    pub fn to_char(&self) -> char {
        match self {
            Self::Zero => '0',
            Self::One => '1',
            Self::Two => '2',
            Self::Three => '3',
            Self::Four => '4',
            Self::S | Self::Default => 's',
            Self::Z => 'z',
        }
    }
}

impl fmt::Display for OptLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

impl<'de> Deserialize<'de> for OptLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Clone, Copy)]
        struct Visitor;
        impl<'de> de::Visitor<'de> for Visitor {
            type Value = OptLevel;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a wasm-opt optimization level")
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if v {
                    Ok(OptLevel::default())
                } else {
                    Ok(OptLevel::Zero)
                }
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match v {
                    "0" => Ok(OptLevel::Zero),
                    "1" => Ok(OptLevel::One),
                    "2" => Ok(OptLevel::Two),
                    "3" => Ok(OptLevel::Three),
                    "4" => Ok(OptLevel::Four),
                    "s" => Ok(OptLevel::S),
                    "z" => Ok(OptLevel::Z),
                    _ => Err(de::Error::invalid_value(Unexpected::Str(v), &self)),
                }
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}
