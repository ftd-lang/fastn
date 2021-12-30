extern crate self as fpm;

#[macro_use]
extern crate lazy_static;

#[macro_use]
pub(crate) mod utils;

mod commands;
mod config;
mod dependency;
mod document;
mod font;
mod i18n;
mod library;
mod snapshot;
mod tracker;
mod translation;

pub(crate) use commands::build::process_file;
pub use commands::{
    build::build, diff::diff, mark_upto_date::mark_upto_date, start_tracking::start_tracking,
    status::status, stop_tracking::stop_tracking, sync::sync,
    translation_status::translation_status, update::update,
};
pub use config::Config;
pub(crate) use config::Package;
pub(crate) use dependency::Dependency;
pub(crate) use document::{get_documents, get_file, paths_to_files, Document, File, Static};
pub(crate) use font::Font;
pub(crate) use library::{FPMLibrary, Library};
pub(crate) use snapshot::Snapshot;
pub(crate) use tracker::Track;
pub(crate) use translation::{TranslatedDocument, TranslationData};
pub(crate) use utils::{copy_dir_all, get_timestamp_nanosecond};

fn ftd_html() -> &'static str {
    include_str!("../ftd.html")
}
fn fpm_ftd() -> &'static str {
    include_str!("../ftd/fpm.ftd")
}

fn fpm_lib_ftd() -> &'static str {
    include_str!("../ftd/fpm-lib.ftd")
}

fn with_fallback() -> &'static str {
    include_str!("../with-fallback.html")
}

fn with_message() -> &'static str {
    include_str!("../with-message.html")
}

fn available_languages(config: &fpm::Config) -> fpm::Result<String> {
    let path = config.root.join("FPM/translation/available-languages.ftd");
    Ok(if path.is_file() {
        std::fs::read_to_string(path)?
    } else {
        include_str!("../ftd/translation/available-languages.ftd").to_string()
    })
}

fn original_package_status(config: &fpm::Config) -> fpm::Result<String> {
    let path = config.root.join("FPM/translation/original-status.ftd");
    Ok(if path.is_file() {
        std::fs::read_to_string(path)?
    } else {
        include_str!("../ftd/translation/original-status.ftd").to_string()
    })
}

fn translation_package_status(config: &fpm::Config) -> fpm::Result<String> {
    let path = config.root.join("FPM/translation/translation-status.ftd");
    Ok(if path.is_file() {
        std::fs::read_to_string(path)?
    } else {
        include_str!("../ftd/translation/translation-status.ftd").to_string()
    })
}

fn get_messages(status: &fpm::TranslatedDocument, config: &fpm::Config) -> fpm::Result<String> {
    Ok(match status {
        TranslatedDocument::Missing { .. } => {
            let path = config.root.join("FPM/translation/missing.ftd");
            if path.is_file() {
                std::fs::read_to_string(path)?
            } else {
                include_str!("../ftd/translation/missing.ftd").to_string()
            }
        }
        TranslatedDocument::NeverMarked { .. } => {
            let path = config.root.join("FPM/translation/never-marked.ftd");
            if path.is_file() {
                std::fs::read_to_string(path)?
            } else {
                include_str!("../ftd/translation/never-marked.ftd").to_string()
            }
        }
        TranslatedDocument::Outdated { .. } => {
            let path = config.root.join("FPM/translation/out-of-date.ftd");
            if path.is_file() {
                std::fs::read_to_string(path)?
            } else {
                include_str!("../ftd/translation/out-of-date.ftd").to_string()
            }
        }
        TranslatedDocument::UptoDate { .. } => {
            let path = config.root.join("FPM/translation/upto-date.ftd");
            if path.is_file() {
                std::fs::read_to_string(path)?
            } else {
                include_str!("../ftd/translation/upto-date.ftd").to_string()
            }
        }
    })
}

// fn default_markdown() -> &'static str {
//     include_str!("../ftd/markdown.ftd")
// }

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("HttpError: {}", _0)]
    HttpError(#[from] reqwest::Error),

    #[error("IoError: {}", _0)]
    IoError(#[from] std::io::Error),

    #[error("IoError: {}", _0)]
    ZipError(#[from] zip::result::ZipError),

    #[error("FTDError: {}", _0)]
    FTDError(#[from] ftd::p1::Error),

    #[error("PackageError: {message}")]
    PackageError { message: String },

    #[error("UsageError: {message}")]
    UsageError { message: String },

    #[error("IgnoreError: {}", _0)]
    IgnoreError(#[from] ignore::Error),

    #[error("FromPathBufError: {}", _0)]
    FromPathBufError(#[from] camino::FromPathBufError),
}

pub type Result<T> = std::result::Result<T, Error>;

pub(crate) fn usage_error<T>(message: String) -> Result<T> {
    Err(Error::UsageError { message })
}

#[cfg(test)]
mod tests {

    #[test]
    fn fbt() {
        if fbt_lib::main().is_some() {
            panic!("test failed")
        }
    }
}
