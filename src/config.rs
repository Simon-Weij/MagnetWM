use std::{collections::HashMap, fs, path::PathBuf};

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::env;

type ConfigPair = HashMap<(String, String), (String, String)>;

#[derive(Deserialize)]
struct RawConfig {
    keybinds: HashMap<String, String>,
}

pub fn parse_config(path: Option<PathBuf>) -> Result<ConfigPair, anyhow::Error> {
    let path = match path {
        Some(value) => value,
        None => config_path()?,
    };

    let config = fs::read_to_string(&path)
        .with_context(|| format!("could not read config file at {}", path.display()))?;

    let raw: RawConfig = toml::from_str(&config).context("could not unmarshal config")?;

    parse_keybinds(&raw.keybinds).context("could not parse keybinds")
}

fn parse_keybinds(keybinds: &HashMap<String, String>) -> Result<ConfigPair> {
    let mut config = HashMap::with_capacity(keybinds.len());

    for (config_key, config_value) in keybinds {
        let Some((modifier, key)) = config_key.split_once('+') else {
            bail!("invalid keybind {config_key:?}, expected exactly 1 '+'");
        };
        if key.contains('+') {
            bail!("invalid keybind {config_key:?}, expected exactly 1 '+'");
        }

        let Some((action, argument)) = config_value.split_once(':') else {
            bail!("invalid action {config_value:?}, expected exactly 1 ':'");
        };
        if argument.contains(':') {
            bail!("invalid action {config_value:?}, expected exactly 1 ':'");
        }

        config.insert(
            (modifier.to_string(), key.to_string()),
            (action.to_string(), argument.to_string()),
        );
    }

    Ok(config)
}

fn config_path() -> Result<PathBuf> {
    match env::var("XDG_CONFIG_HOME") {
        Ok(value) => Ok(PathBuf::from(value).join("magnet").join("config.toml")),
        Err(_) => match env::var("HOME") {
            Ok(value) => Ok(PathBuf::from(value)
                .join(".config")
                .join("magnet")
                .join("config.toml")),
            Err(_) => bail!(
                "neither XDG_CONFIG_HOME nor HOME is set. Either set one of those or use the --config flag"
            ),
        },
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, path::PathBuf};

    use crate::config::{config_path, parse_config, parse_keybinds};
    use serial_test::serial;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn should_parse_config() {
        let mut file = NamedTempFile::new().expect("could not create file");
        writeln!(
            file,
            r#"[keybinds]
            "Mod+Space" = "spawn:kitty""#
        )
        .expect("could not create file");

        let expected = HashMap::from([(
            ("Mod".to_string(), "Space".to_string()),
            ("spawn".to_string(), "kitty".to_string()),
        )]);

        let path = file.path().to_path_buf();

        let got = parse_config(Some(path)).expect("could not parse config");

        assert_eq!(got, expected);
    }

    #[test]
    fn should_parse_keybinds() {
        let keybinds = HashMap::from([("Mod+Space".to_string(), "spawn:kitty".to_string())]);
        let expected = HashMap::from([(
            ("Mod".to_string(), "Space".to_string()),
            ("spawn".to_string(), "kitty".to_string()),
        )]);

        let result = parse_keybinds(&keybinds).expect("could not parse keybinds");
        assert_eq!(result, expected);
    }

    #[test]
    fn should_reject_missing_plus() {
        assert!(
            parse_keybinds(&HashMap::from([(
                "Space".to_string(),
                "spawn:kitty".to_string()
            )]))
            .is_err()
        );
    }

    #[test]
    fn should_reject_multiple_plusses() {
        assert!(
            parse_keybinds(&HashMap::from([(
                "Mod++Space".to_string(),
                "spawn:kitty".to_string()
            )]))
            .is_err()
        );
    }

    #[test]
    fn should_reject_missing_colon() {
        assert!(
            parse_keybinds(&HashMap::from([(
                "Mod+Space".to_string(),
                "spawn".to_string()
            )]))
            .is_err()
        );
    }

    #[test]
    fn should_reject_multiple_colons() {
        assert!(
            parse_keybinds(&HashMap::from([(
                "Mod+Space".to_string(),
                "spawn::kitty".to_string()
            )]))
            .is_err()
        );
    }

    #[test]
    fn should_reject_missing_value() {
        assert!(
            parse_keybinds(&HashMap::from([("Mod+Space".to_string(), String::new())])).is_err()
        );
    }

    #[serial]
    fn should_get_config() -> anyhow::Result<()> {
        unsafe {
            std::env::set_var("XDG_CONFIG_HOME", "/home/alice/.config");

            assert_eq!(
                config_path()?,
                PathBuf::from("/home/alice/.config/magnet/config.toml")
            );

            std::env::remove_var("XDG_CONFIG_HOME");
            std::env::set_var("HOME", "/home/alice");

            assert_eq!(
                config_path()?,
                PathBuf::from("/home/alice/.config/magnet/config.toml")
            );

            std::env::remove_var("XDG_CONFIG_HOME");
            std::env::remove_var("HOME");

            let err = config_path().expect_err("neither XDG_CONFIG_HOME nor HOME is set. Either set one of those or use the --config flag");
            assert!(
                err.to_string()
                    .contains("neither XDG_CONFIG_HOME nor HOME is set")
            );
        }
        Ok(())
    }
}
