use crate::commands::command::CommandAction;
use crate::errors::ShellError;
use crate::object::{Primitive, Value};
use crate::parser::Spanned;
use crate::prelude::*;
use std::path::{Path, PathBuf};

pub fn enter(args: CommandArgs) -> Result<OutputStream, ShellError> {
    let path = match args.nth(0) {
        None => return Err(ShellError::maybe_labeled_error(
            "open requires a path or url",
            "missing path",
            args.name_span,
        )),
        Some(p) => p,
    };

    let span = args.name_span;

    let cwd = args
        .env()
        .lock()
        .unwrap()
        .front()
        .unwrap()
        .path()
        .to_path_buf();

    let mut full_path = PathBuf::from(cwd);

    let (file_extension, contents) = match path.item() {
        Value::Primitive(Primitive::String(s)) => {
            if s.starts_with("http:") || s.starts_with("https:") {
                let response = reqwest::get(s);
                match response {
                    Ok(mut r) => match r.text() {
                        Ok(s) => {
                            let fname = r
                                .url()
                                .path_segments()
                                .and_then(|segments| segments.last())
                                .and_then(|name| if name.is_empty() { None } else { Some(name) })
                                .and_then(|name| {
                                    PathBuf::from(name)
                                        .extension()
                                        .map(|name| name.to_string_lossy().to_string())
                                });
                            (fname, s)
                        }
                        Err(_) => {
                            return Err(ShellError::labeled_error(
                                "Web page contents corrupt",
                                "received garbled data",
                                args.expect_nth(0)?.span,
                            ));
                        }
                    },
                    Err(_) => {
                        return Err(ShellError::labeled_error(
                            "URL could not be opened",
                            "url not found",
                            args.expect_nth(0)?.span,
                        ));
                    }
                }
            } else {
                full_path.push(Path::new(&s));
                match std::fs::read_to_string(&full_path) {
                    Ok(s) => (
                        full_path
                            .extension()
                            .map(|name| name.to_string_lossy().to_string()),
                        s,
                    ),
                    Err(_) => {
                        return Err(ShellError::labeled_error(
                            "File cound not be opened",
                            "file not found",
                            args.expect_nth(0)?.span,
                        ));
                    }
                }
            }
        }
        _ => {
            return Err(ShellError::labeled_error(
                "Expected string value for filename",
                "expected filename",
                args.expect_nth(0)?.span,
            ));
        }
    };

    let mut stream = VecDeque::new();

    let file_extension = if args.has("raw") {
        None
    } else if args.has("json") {
        Some("json".to_string())
    } else if args.has("xml") {
        Some("xml".to_string())
    } else if args.has("ini") {
        Some("ini".to_string())
    } else if args.has("yaml") {
        Some("yaml".to_string())
    } else if args.has("toml") {
        Some("toml".to_string())
    } else {
        if let Some(ref named_args) = args.args.named {
            for named in named_args.iter() {
                return Err(ShellError::labeled_error(
                    "Unknown flag for enter",
                    "unknown flag",
                    named.1.span.clone(),
                ));
            }
            file_extension
        } else {
            file_extension
        }
    };

    match file_extension {
        Some(x) if x == "toml" => {
            stream.push_back(ReturnValue::Action(CommandAction::Enter(
                crate::commands::from_toml::from_toml_string_to_value(contents).map_err(
                    move |_| {
                        ShellError::maybe_labeled_error(
                            "Could not load as TOML",
                            "could not load as TOML",
                            span,
                        )
                    },
                )?,
            )));
        }
        Some(x) if x == "json" => {
            stream.push_back(ReturnValue::Action(CommandAction::Enter(
                crate::commands::from_json::from_json_string_to_value(contents).map_err(
                    move |_| {
                        ShellError::maybe_labeled_error(
                            "Could not load as JSON",
                            "could not load as JSON",
                            span,
                        )
                    },
                )?,
            )));
        }
        Some(x) if x == "xml" => {
            stream.push_back(ReturnValue::Action(CommandAction::Enter(
                crate::commands::from_xml::from_xml_string_to_value(contents).map_err(
                    move |_| {
                        ShellError::maybe_labeled_error(
                            "Could not load as XML",
                            "could not load as XML",
                            span,
                        )
                    },
                )?,
            )));
        }
        Some(x) if x == "ini" => {
            stream.push_back(ReturnValue::Action(CommandAction::Enter(
                crate::commands::from_ini::from_ini_string_to_value(contents).map_err(
                    move |_| {
                        ShellError::maybe_labeled_error(
                            "Could not load as INI",
                            "could not load as INI",
                            span,
                        )
                    },
                )?,
            )));
        }
        Some(x) if x == "yml" => {
            stream.push_back(ReturnValue::Action(CommandAction::Enter(
                crate::commands::from_yaml::from_yaml_string_to_value(contents).map_err(
                    move |_| {
                        ShellError::maybe_labeled_error(
                            "Could not load as YAML",
                            "could not load as YAML",
                            span,
                        )
                    },
                )?,
            )));
        }
        Some(x) if x == "yaml" => {
            stream.push_back(ReturnValue::Action(CommandAction::Enter(
                crate::commands::from_yaml::from_yaml_string_to_value(contents).map_err(
                    move |_| {
                        ShellError::maybe_labeled_error(
                            "Could not load as YAML",
                            "could not load as YAML",
                            span,
                        )
                    },
                )?,
            )));
        }
        _ => {
            stream.push_back(ReturnValue::Action(CommandAction::Enter(Value::Primitive(
                Primitive::String(contents),
            ))));
        }
    }

    Ok(stream.boxed())
}
