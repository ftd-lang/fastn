#[derive(Debug, Clone)]
pub enum File {
    Ftd(Document),
    Static(Static),
    Markdown(Document),
}

impl File {
    pub fn get_id(&self) -> String {
        match self {
            Self::Ftd(a) => a.id.clone(),
            Self::Static(a) => a.id.clone(),
            Self::Markdown(a) => a.id.clone(),
        }
    }
    pub fn get_base_path(&self) -> String {
        match self {
            Self::Ftd(a) => a.parent_path.to_string(),
            Self::Static(a) => a.base_path.to_string(),
            Self::Markdown(a) => a.parent_path.to_string(),
        }
    }
    pub fn get_full_path(&self) -> camino::Utf8PathBuf {
        let (id, base_path) = match self {
            Self::Ftd(a) => (a.id.to_string(), a.parent_path.to_string()),
            Self::Static(a) => (a.id.to_string(), a.base_path.to_string()),
            Self::Markdown(a) => (a.id.to_string(), a.parent_path.to_string()),
        };
        camino::Utf8PathBuf::from(base_path).join(id)
    }
}

#[derive(Debug, Clone)]
pub struct Document {
    pub id: String,
    pub content: String,
    pub parent_path: String,
    pub depth: usize,
}

#[derive(Debug, Clone)]
pub struct Static {
    pub id: String,
    pub base_path: String,
    pub depth: usize,
}

pub(crate) async fn get_root_documents(config: &fpm::Config) -> fpm::Result<Vec<fpm::File>> {
    fpm::get_documents(&ignore::WalkBuilder::new("./"), config).await
}

pub(crate) async fn get_documents(
    ignore_paths: &ignore::WalkBuilder,
    config: &fpm::Config,
) -> fpm::Result<Vec<fpm::File>> {
    let mut ignore_paths = ignore_paths.clone();
    ignore_paths.overrides(package_ignores()?); // unwrap ok because this we know can never fail
    ignore_paths.standard_filters(true);
    ignore_paths.overrides(config.ignored.clone());
    let all_files = ignore_paths
        .build()
        .into_iter()
        .flatten()
        .map(|x| x.into_path())
        .collect::<Vec<std::path::PathBuf>>();
    let mut documents = fpm::paths_to_files(all_files, &config.root).await?;
    documents.sort_by_key(|v| v.get_id());

    Ok(documents)
}

pub(crate) async fn paths_to_files(
    files: Vec<std::path::PathBuf>,
    base_path: &camino::Utf8Path,
) -> fpm::Result<Vec<fpm::File>> {
    Ok(futures::future::join_all(
        files
            .into_iter()
            .map(|x| {
                let base = base_path.to_path_buf();
                tokio::spawn(async move { fpm::get_file(&x, &base).await })
            })
            .collect::<Vec<tokio::task::JoinHandle<fpm::Result<fpm::File>>>>(),
    )
    .await
    .into_iter()
    .flatten()
    .flatten()
    .collect::<Vec<fpm::File>>())
}

pub fn package_ignores() -> Result<ignore::overrides::Override, ignore::Error> {
    let mut overrides = ignore::overrides::OverrideBuilder::new("./");
    overrides.add("!.history")?;
    overrides.add("!.packages")?;
    overrides.add("!.tracks")?;
    overrides.add("!FPM")?;
    overrides.add("!rust-toolchain")?;
    overrides.add("!.build")?;
    overrides.build()
}

pub(crate) async fn get_file(
    doc_path: &std::path::Path,
    base_path: &camino::Utf8Path,
) -> fpm::Result<File> {
    if doc_path.is_dir() {
        return Err(fpm::Error::UsageError {
            message: format!("{:?} should be a file", doc_path),
        });
    }

    let doc_path_str = doc_path.to_str().unwrap();

    let id = match std::fs::canonicalize(doc_path)?
        .to_str()
        .unwrap()
        .rsplit_once(format!("{}/", base_path).as_str())
    {
        Some((_, id)) => id.to_string(),
        None => {
            return Err(fpm::Error::UsageError {
                message: format!("{:?} should be a file", doc_path),
            });
        }
    };

    Ok(match id.rsplit_once(".") {
        Some((_, "ftd")) => File::Ftd(Document {
            id: id.to_string(),
            content: tokio::fs::read_to_string(&doc_path).await?,
            parent_path: base_path.to_string(),
            depth: doc_path_str.split('/').count() - 1,
        }),
        Some((doc_name, "md")) => File::Markdown(Document {
            id: if doc_name == "README"
                && !(std::path::Path::new("./index.ftd").exists()
                    || std::path::Path::new("./index.md").exists())
            {
                "index.md".to_string()
            } else {
                id.to_string()
            },
            content: tokio::fs::read_to_string(&doc_path).await?,
            parent_path: base_path.to_string(),
            depth: doc_path_str.split('/').count() - 1,
        }),
        _ => File::Static(Static {
            id: id.to_string(),
            base_path: base_path.to_string(),
            depth: doc_path_str.split('/').count() - 1,
        }),
    })
}
