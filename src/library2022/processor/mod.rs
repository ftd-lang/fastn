pub(crate) mod document;
pub(crate) mod request_data;
pub(crate) mod sitemap;

// pub enum Processor {
//     Toc,
//     GetData,
//     Sitemap,
//     FullSitemap,
//     DocumentReaders,
//     DocumentWriters,
//     UserGroupById,
//     UserGroups,
//     RequestData,
// }
//
// impl std::str::FromStr for Processor {
//     type Err = fpm::Error;
//
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         match s {
//             "toc" => Ok(Self::Toc),
//             "request-data" => Ok(Self::RequestData),
//             "sitemap" => Ok(Self::Sitemap),
//             _ => fpm::usage_error(format!("processor not found {s}")),
//         }
//     }
// }
