use itertools::Itertools;

pub fn processor(
    section: &ftd::p1::Section,
    doc: &ftd::p2::TDoc,
    config: &fpm::Config,
    document_id: &str,
    base_url: &str,
) -> ftd::p1::Result<ftd::Value> {
    let versions =
        futures::executor::block_on(config.get_versions(&config.package)).map_err(|e| {
            ftd::p1::Error::ParseError {
                message: format!("Cant find versions: {:?}", e),
                doc_id: doc.name.to_string(),
                line_number: section.line_number,
            }
        })?;

    let version =
        fpm::Version::parse(document_id, config.package.versioned.as_str()).map_err(|e| {
            ftd::p1::Error::ParseError {
                message: format!("{:?}", e),
                doc_id: doc.name.to_string(),
                line_number: section.line_number,
            }
        })?;

    let doc_id = if let Some(doc) = document_id.split_once('/').map(|(_, v)| v) {
        doc
    } else {
        document_id
    }
    .to_string();

    let base_url = base_url
        .trim_end_matches('/')
        .trim_start_matches('/')
        .to_string();
    let base_url = if !base_url.is_empty() {
        format!("/{base_url}/")
    } else {
        String::from("/")
    };

    let url = match doc_id.as_str().rsplit_once('.') {
        Some(("index", "ftd")) => base_url,
        Some((file_path, "ftd")) | Some((file_path, "md")) => {
            format!("{base_url}{file_path}/")
        }
        Some(_) | None => {
            // Unknown file found, create URL
            format!("{base_url}{file_path}/", file_path = doc_id.as_str())
        }
    };
    let mut found = false;
    if let Some(doc) = versions.get(&fpm::Version::base_(version.base.to_owned())) {
        if doc.iter().map(|v| v.get_id()).any(|x| x == doc_id) {
            found = true;
        }
    }

    let mut version_toc = vec![];
    for key in versions
        .iter()
        .filter(|(v, _)| v.base.eq(&version.base))
        .map(|(v, _)| v)
        .sorted()
    {
        if key.eq(&fpm::Version::base()) {
            continue;
        }
        let doc = versions[key].to_owned();
        if !found {
            if !doc.iter().map(|v| v.get_id()).any(|x| x == doc_id) {
                continue;
            }
            found = true;
        }
        version_toc.push(fpm::library::toc::TocItem {
            id: None,
            title: Some(key.original.to_string()),
            url: Some(format!("{}{}", key.original, url)),
            number: vec![],
            is_heading: version.eq(key),
            is_disabled: false,
            img_src: None,
            font_icon: None,
            children: vec![],
        });
    }

    let toc_items = version_toc
        .iter()
        .map(|item| item.to_toc_item_compat())
        .collect::<Vec<fpm::library::toc::TocItemCompat>>();

    doc.from_json(&toc_items, section)
}
