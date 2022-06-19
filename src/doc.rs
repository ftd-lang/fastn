// TODO: make async
pub async fn parse<'a>(
    name: &str,
    source: &str,
    lib: &'a fpm::Library,
    base_url: &str,
    current_package: Option<&fpm::Package>,
) -> ftd::p1::Result<ftd::p2::Document> {
    let mut s = ftd::interpret(name, source)?;

    let mut packages_under_process = vec![current_package
        .map(|v| v.to_owned())
        .unwrap_or_else(|| lib.config.package.clone())];
    let document;
    loop {
        match s {
            ftd::Interpreter::Done { document: doc } => {
                document = doc;
                break;
            }
            ftd::Interpreter::StuckOnProcessor { state, section } => {
                let value = lib
                    .process(&section, &state.tdoc(&mut Default::default()))
                    .await?;
                s = state.continue_after_processor(&section, value)?;
            }
            ftd::Interpreter::StuckOnImport {
                module,
                state: mut st,
            } => {
                packages_under_process.truncate(st.document_stack.len());
                let source = if module.eq("fpm/time") {
                    st.add_foreign_variable_prefix(module.as_str(), vec![module.to_string()]);
                    packages_under_process.push(
                        packages_under_process
                            .last()
                            .ok_or_else(|| ftd::p1::Error::ParseError {
                                message: "The processing document stack is empty".to_string(),
                                doc_id: "".to_string(),
                                line_number: 0,
                            })?
                            .clone(),
                    );
                    "".to_string()
                } else if module.ends_with("assets") {
                    st.add_foreign_variable_prefix(
                        module.as_str(),
                        vec![format!("{}#files", module)],
                    );

                    packages_under_process.push(
                        packages_under_process
                            .last()
                            .ok_or_else(|| ftd::p1::Error::ParseError {
                                message: "The processing document stack is empty".to_string(),
                                doc_id: "".to_string(),
                                line_number: 0,
                            })?
                            .clone(),
                    );

                    let current_package = packages_under_process.last().ok_or_else(|| {
                        ftd::p1::Error::ParseError {
                            message: "The processing document stack is empty".to_string(),
                            doc_id: "".to_string(),
                            line_number: 0,
                        }
                    })?;

                    if module.starts_with(current_package.name.as_str()) {
                        current_package
                            .get_font_ftd()
                            .unwrap_or_else(|| "".to_string())
                    } else {
                        let mut font_ftd = "".to_string();
                        for (alias, package) in current_package.aliases() {
                            if module.starts_with(alias) {
                                font_ftd = package.get_font_ftd().unwrap_or_else(|| "".to_string());
                                break;
                            }
                        }
                        font_ftd
                    }
                } else {
                    lib.get_with_result(module.as_str(), &mut packages_under_process)
                        .await?
                };
                s = st.continue_after_import(module.as_str(), source.as_str())?;
            }
            ftd::Interpreter::StuckOnForeignVariable { variable, state } => {
                packages_under_process.truncate(state.document_stack.len());
                let current_package =
                    packages_under_process
                        .last()
                        .ok_or_else(|| ftd::p1::Error::ParseError {
                            message: "The processing document stack is empty".to_string(),
                            doc_id: "".to_string(),
                            line_number: 0,
                        })?;
                let value = resolve_foreign_variable(
                    variable.as_str(),
                    name,
                    current_package,
                    lib,
                    base_url,
                )?;
                s = state.continue_after_variable(variable.as_str(), value)?
            }
        }
    }
    Ok(document)
}

// TODO: make async
pub async fn parse2<'a>(
    name: &str,
    source: &str,
    lib: &'a mut fpm::Library2,
    base_url: &str,
) -> ftd::p1::Result<ftd::p2::Document> {
    let mut s = ftd::interpret(name, source)?;

    let document;
    loop {
        match s {
            ftd::Interpreter::Done { document: doc } => {
                document = doc;
                break;
            }
            ftd::Interpreter::StuckOnProcessor { state, section } => {
                let value = lib
                    .process(&section, &state.tdoc(&mut Default::default()))
                    .await?;
                s = state.continue_after_processor(&section, value)?;
            }
            ftd::Interpreter::StuckOnImport {
                module,
                state: mut st,
            } => {
                lib.packages_under_process.truncate(st.document_stack.len());
                let current_package = lib.get_current_package()?.to_owned();
                let source = if module.eq("fpm/time") {
                    st.add_foreign_variable_prefix(module.as_str(), vec![module.to_string()]);
                    lib.push_package_under_process(&current_package).await?;
                    "".to_string()
                } else if module.ends_with("assets") {
                    st.add_foreign_variable_prefix(
                        module.as_str(),
                        vec![format!("{}#files", module)],
                    );

                    if module.starts_with(current_package.name.as_str()) {
                        lib.push_package_under_process(&current_package).await?;
                        lib.get_current_package()?
                            .get_font_ftd()
                            .unwrap_or_else(|| "".to_string())
                    } else {
                        let mut font_ftd = "".to_string();
                        for (alias, package) in current_package.aliases() {
                            if module.starts_with(alias) {
                                lib.push_package_under_process(package).await?;
                                font_ftd = lib
                                    .config
                                    .all_packages
                                    .get(package.name.as_str())
                                    .unwrap()
                                    .get_font_ftd()
                                    .unwrap_or_else(|| "".to_string());
                                break;
                            }
                        }
                        font_ftd
                    }
                } else {
                    lib.get_with_result(module.as_str()).await?
                };
                s = st.continue_after_import(module.as_str(), source.as_str())?;
            }
            ftd::Interpreter::StuckOnForeignVariable { variable, state } => {
                lib.packages_under_process
                    .truncate(state.document_stack.len());
                let value =
                    resolve_foreign_variable2(variable.as_str(), name, lib, base_url).await?;
                s = state.continue_after_variable(variable.as_str(), value)?
            }
        }
    }
    Ok(document)
}

async fn resolve_foreign_variable2(
    variable: &str,
    doc_name: &str,
    lib: &mut fpm::Library2,
    base_url: &str,
) -> ftd::p1::Result<ftd::Value> {
    let package = lib.get_current_package()?.to_owned();
    if let Ok(value) = resolve_ftd_foreign_variable(variable, doc_name) {
        return Ok(value);
    }

    if let Some((package_name, files)) = variable.split_once("/assets#files.") {
        if package.name.eq(package_name) {
            if let Ok(value) = get_assets_value(&package, files, lib, doc_name, base_url).await {
                return Ok(value);
            }
        }
        for (alias, package) in package.aliases() {
            if alias.eq(package_name) {
                if let Ok(value) = get_assets_value(package, files, lib, doc_name, base_url).await {
                    return Ok(value);
                }
            }
        }
    }

    return ftd::e2(format!("{} not found 1", variable).as_str(), doc_name, 0);

    async fn get_assets_value(
        package: &fpm::Package,
        files: &str,
        lib: &mut fpm::Library2,
        doc_name: &str,
        base_url: &str,
    ) -> ftd::p1::Result<ftd::Value> {
        lib.push_package_under_process(package).await?;
        let base_url = base_url.trim_end_matches('/');
        let mut files = files.to_string();
        let light = {
            if let Some(f) = files.strip_suffix(".light") {
                files = f.to_string();
                true
            } else {
                false
            }
        };
        let dark = {
            if light {
                false
            } else if let Some(f) = files.strip_suffix(".dark") {
                files = f.to_string();
                true
            } else {
                false
            }
        };

        match files.rsplit_once(".") {
            Some((file, ext))
                if mime_guess::MimeGuess::from_ext(ext)
                    .first_or_octet_stream()
                    .to_string()
                    .starts_with("image/")
                    && package
                        .resolve_by_file_name(
                            format!("{}.{}", file.replace('.', "/"), ext).as_str(),
                            None,
                        )
                        .await
                        .is_ok() =>
            {
                let light_mode = format!(
                    "{base_url}/-/{}/{}.{}",
                    package.name,
                    file.replace('.', "/"),
                    ext
                );
                if light {
                    return Ok(ftd::Value::String {
                        text: light_mode,
                        source: ftd::TextSource::Header,
                    });
                }
                let dark_mode_path = format!("{}-dark.{}", file.replace('.', "/"), ext);
                let dark_mode = if package
                    .resolve_by_file_name(dark_mode_path.as_str(), None)
                    .await
                    .is_ok()
                {
                    format!(
                        "{base_url}/-/{}/{}-dark.{}",
                        package.name,
                        file.replace('.', "/"),
                        ext
                    )
                } else {
                    let package_root = lib.config.get_root_for_package(package);
                    tokio::fs::copy(
                        package_root.join(format!("{}.{}", file.replace('.', "/"), ext)),
                        dbg!(package_root.join(dark_mode_path)),
                    )
                    .await
                    .map_err(|e| ftd::p1::Error::ParseError {
                        message: e.to_string(),
                        doc_id: lib.document_id.to_string(),
                        line_number: 0,
                    })?;
                    light_mode.clone()
                };

                if dark {
                    return Ok(ftd::Value::String {
                        text: dark_mode,
                        source: ftd::TextSource::Header,
                    });
                }
                Ok(ftd::Value::Record {
                    name: "ftd#image-src".to_string(),
                    fields: std::array::IntoIter::new([
                        (
                            "light".to_string(),
                            ftd::PropertyValue::Value {
                                value: ftd::Value::String {
                                    text: light_mode,
                                    source: ftd::TextSource::Header,
                                },
                            },
                        ),
                        (
                            "dark".to_string(),
                            ftd::PropertyValue::Value {
                                value: ftd::Value::String {
                                    text: dark_mode,
                                    source: ftd::TextSource::Header,
                                },
                            },
                        ),
                    ])
                    .collect(),
                })
            }
            Some((file, ext))
                if package
                    .resolve_by_file_name(
                        format!("{}.{}", file.replace('.', "/"), ext).as_str(),
                        None,
                    )
                    .await
                    .is_ok() =>
            {
                Ok(ftd::Value::String {
                    text: format!("/-/{}/{}.{}", package.name, file.replace('.', "/"), ext),
                    source: ftd::TextSource::Header,
                })
            }
            None if package
                .resolve_by_file_name(files.as_str(), None)
                .await
                .is_ok() =>
            {
                Ok(ftd::Value::String {
                    text: format!("/-/{}/{}", package.name, files),
                    source: ftd::TextSource::Header,
                })
            }
            _ => ftd::e2(format!("{} not found 2", files).as_str(), doc_name, 0),
        }
    }
}

fn resolve_foreign_variable(
    variable: &str,
    doc_name: &str,
    package: &fpm::Package,
    lib: &fpm::Library,
    base_url: &str,
) -> ftd::p1::Result<ftd::Value> {
    if let Ok(value) = resolve_ftd_foreign_variable(variable, doc_name) {
        return Ok(value);
    }

    if let Some((package_name, files)) = variable.split_once("/assets#files.") {
        if package.name.eq(package_name) {
            if let Ok(value) = get_assets_value(package, files, lib, doc_name, base_url) {
                return Ok(value);
            }
        }
        for (alias, package) in package.aliases() {
            if alias.eq(package_name) {
                if let Ok(value) = get_assets_value(package, files, lib, doc_name, base_url) {
                    return Ok(value);
                }
            }
        }
    }

    return ftd::e2(format!("{} not found 1", variable).as_str(), doc_name, 0);

    fn get_assets_value(
        package: &fpm::Package,
        files: &str,
        lib: &fpm::Library,
        doc_name: &str,
        base_url: &str,
    ) -> ftd::p1::Result<ftd::Value> {
        let base_url = base_url.trim_end_matches('/');
        let mut files = files.to_string();
        let path = lib.config.get_root_for_package(package);
        let light = {
            if let Some(f) = files.strip_suffix(".light") {
                files = f.to_string();
                true
            } else {
                false
            }
        };
        let dark = {
            if light {
                false
            } else if let Some(f) = files.strip_suffix(".dark") {
                files = f.to_string();
                true
            } else {
                false
            }
        };

        match files.rsplit_once(".") {
            Some((file, ext))
                if mime_guess::MimeGuess::from_ext(ext)
                    .first_or_octet_stream()
                    .to_string()
                    .starts_with("image/")
                    && path
                        .join(format!("{}.{}", file.replace('.', "/"), ext))
                        .exists() =>
            {
                let light_mode = format!(
                    "{base_url}/-/{}/{}.{}",
                    package.name,
                    file.replace('.', "/"),
                    ext
                );
                if light {
                    return Ok(ftd::Value::String {
                        text: light_mode,
                        source: ftd::TextSource::Header,
                    });
                }
                let dark_mode = if path
                    .join(format!("{}-dark.{}", file.replace('.', "/"), ext))
                    .exists()
                {
                    format!(
                        "{base_url}/-/{}/{}-dark.{}",
                        package.name,
                        file.replace('.', "/"),
                        ext
                    )
                } else {
                    light_mode.clone()
                };

                if dark {
                    return Ok(ftd::Value::String {
                        text: dark_mode,
                        source: ftd::TextSource::Header,
                    });
                }
                Ok(ftd::Value::Record {
                    name: "ftd#image-src".to_string(),
                    fields: std::array::IntoIter::new([
                        (
                            "light".to_string(),
                            ftd::PropertyValue::Value {
                                value: ftd::Value::String {
                                    text: light_mode,
                                    source: ftd::TextSource::Header,
                                },
                            },
                        ),
                        (
                            "dark".to_string(),
                            ftd::PropertyValue::Value {
                                value: ftd::Value::String {
                                    text: dark_mode,
                                    source: ftd::TextSource::Header,
                                },
                            },
                        ),
                    ])
                    .collect(),
                })
            }
            Some((file, ext))
                if path
                    .join(format!("{}.{}", file.replace('.', "/"), ext))
                    .exists() =>
            {
                Ok(ftd::Value::String {
                    text: format!("/-/{}/{}.{}", package.name, file.replace('.', "/"), ext),
                    source: ftd::TextSource::Header,
                })
            }
            None if path.join(&files).exists() => Ok(ftd::Value::String {
                text: format!("/-/{}/{}", package.name, files),
                source: ftd::TextSource::Header,
            }),
            _ => ftd::e2(format!("{} not found 2", files).as_str(), doc_name, 0),
        }
    }
}

// No need to make async since this is pure.
pub fn parse_ftd(
    name: &str,
    source: &str,
    lib: &fpm::FPMLibrary,
) -> ftd::p1::Result<ftd::p2::Document> {
    let mut s = ftd::interpret(name, source)?;
    let document;
    loop {
        match s {
            ftd::Interpreter::Done { document: doc } => {
                document = doc;
                break;
            }
            ftd::Interpreter::StuckOnProcessor { state, section } => {
                let value = lib.process(&section, &state.tdoc(&mut Default::default()))?;
                s = state.continue_after_processor(&section, value)?;
            }
            ftd::Interpreter::StuckOnImport { module, state: st } => {
                let source =
                    lib.get_with_result(module.as_str(), &st.tdoc(&mut Default::default()))?;
                s = st.continue_after_import(module.as_str(), source.as_str())?;
            }
            ftd::Interpreter::StuckOnForeignVariable { variable, state } => {
                let value = resolve_ftd_foreign_variable(variable.as_str(), name)?;
                s = state.continue_after_variable(variable.as_str(), value)?
            }
        }
    }
    Ok(document)
}

fn resolve_ftd_foreign_variable(variable: &str, doc_name: &str) -> ftd::p1::Result<ftd::Value> {
    match variable.strip_prefix("fpm/time#") {
        Some("now-str") => Ok(ftd::Value::String {
            text: std::str::from_utf8(
                std::process::Command::new("date")
                    .output()
                    .expect("failed to execute process")
                    .stdout
                    .as_slice(),
            )
            .unwrap()
            .to_string(),
            source: ftd::TextSource::Header,
        }),
        _ => ftd::e2(format!("{} not found 3", variable).as_str(), doc_name, 0),
    }
}
