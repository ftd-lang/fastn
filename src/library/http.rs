pub async fn processor<'a>(
    section: &ftd::p1::Section,
    doc: &ftd::p2::TDoc<'a>,
) -> ftd::p1::Result<ftd::Value> {
    {
        let method = section
            .header
            .str_with_default(doc.name, section.line_number, "method", "GET")?
            .to_lowercase();

        if method != "get" {
            return ftd::p2::utils::e2(
                format!("only GET method is allowed, found: {}", method),
                doc.name,
                section.line_number,
            );
        }
    }

    let url = match section
        .header
        .string_optional(doc.name, section.line_number, "url")?
    {
        Some(v) => v,
        None => {
            return ftd::p2::utils::e2(
                "'url' key is required when using `$processor$: http`",
                doc.name,
                section.line_number,
            )
        }
    };

    let mut url = match url::Url::parse(url.as_str()) {
        Ok(v) => v,
        Err(e) => {
            return ftd::p2::utils::e2(
                format!("invalid url: {:?}", e),
                doc.name,
                section.line_number,
            )
        }
    };

    for (_, k, v) in section.header.0.iter() {
        if k == "$processor$" || k == "url" || k == "method" {
            continue;
        }
        url.query_pairs_mut().append_pair(k, v);
    }

    let json = match crate::http::http_get(url.as_str()).await {
        Ok(v) => v,
        Err(e) => {
            return ftd::p2::utils::e2(
                format!("HTTP::get failed: {:?}", e),
                doc.name,
                section.line_number,
            )
        }
    };
    doc.from_json(&json, section)
}

// Need to pass the request object also
// From request get the url, get query parameters, get the data from body(form data, post data)
pub fn request_data_processor<'a>(
    section: &ftd::p1::Section,
    doc: &ftd::p2::TDoc<'a>,
    config: &fpm::Config,
) -> ftd::p1::Result<ftd::Value> {
    // TODO: Need to return from query parameters and body as well
    let query = match config.request.as_ref() {
        Some(request) => request.query().map_err(|e| ftd::p1::Error::ParseError {
            message: format!("Not able to parse from query string: {:?}", e),
            doc_id: doc.name.to_string(),
            line_number: section.line_number,
        })?,
        None => {
            return ftd::p2::utils::e2(
                "HttpRequest object should not be null",
                doc.name,
                section.line_number,
            )
        }
    };
    dbg!(&query);
    doc.from_json(&query, section)
}
