use std::collections::BTreeMap;

use rss::{extension::ExtensionMap, *};
use serde::{Deserialize, Serialize};

use crate::{traits::Saveable, values::IDENT};

/// Serde impled version of rss::Channel
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct PseudoChannel {
    pub title: String,
    pub link: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copyright: Option<String>,
    #[serde(rename = "managingEditor")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub managing_editor: Option<String>,
    #[serde(rename = "webMaster")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webmaster: Option<String>,
    #[serde(rename = "pubDate")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pub_date: Option<String>,
    #[serde(rename = "lastBuildDate")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_build_date: Option<String>,
    #[serde(rename = "category")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub categories: Option<Vec<PseudoCategory>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cloud: Option<PseudoCloud>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<PseudoImage>,
    #[serde(rename = "textInput")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_input: Option<PseudoTextInput>,
    #[serde(rename = "skipHours")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_hours: Option<Vec<String>>,
    #[serde(rename = "skipDays")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_days: Option<Vec<String>>,
    pub items: Option<Vec<PseudoItem>>,
    // pub extensions: ExtensionMap,
    // pub itunes_ext: Option<ITunesChannelExtension>,
    // pub dublin_core_ext: Option<DublinCoreExtension>,
    // pub syndication_ext: Option<SyndicationExtension>,
    // pub namespaces: BTreeMap<String, String>,
}

impl PseudoChannel {
    /// Set self with items
    pub fn with_items(self, items: Vec<PseudoItem>) -> Self {
        Self {
            items: Some(items),
            ..self
        }
    }
}

impl Saveable for PseudoChannel {
    fn get_rss(self) -> Channel {
        self.into()
    }
}

impl From<PseudoChannel> for Channel {
    fn from(val: PseudoChannel) -> Self {
        Channel {
            title: val.title,
            link: val.link,
            description: val.description,
            language: val.language,
            copyright: val.copyright,
            managing_editor: val.managing_editor,
            webmaster: val.webmaster,
            pub_date: val.pub_date,
            last_build_date: val.last_build_date,
            categories: val
                .categories
                .unwrap_or_default()
                .into_iter()
                .map(PseudoCategory::into)
                .collect(),
            generator: Some(val.generator.map_or(IDENT.get().unwrap().to_string(), |s| {
                format!("{} with {s}", IDENT.get().unwrap())
            })),
            docs: val.docs,
            cloud: val.cloud.map(PseudoCloud::into),
            rating: val.rating,
            ttl: val.ttl,
            image: val.image.map(PseudoImage::into),
            text_input: val.text_input.map(PseudoTextInput::into),
            skip_hours: val.skip_hours.unwrap_or_default(),
            skip_days: val.skip_days.unwrap_or_default(),
            items: val
                .items
                .unwrap_or_default()
                .into_iter()
                .map(PseudoItem::into)
                .collect(),
            extensions: BTreeMap::default(),
            itunes_ext: None,
            dublin_core_ext: None,
            syndication_ext: None,
            namespaces: BTreeMap::default(),
        }
    }
}

/// A vector of PseudoItem for saving as json
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct PseudoItemCache(pub Vec<PseudoItem>);

impl Saveable for PseudoItemCache {}

/// Serde impled version of rss::Item
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct PseudoItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(rename = "category")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub categories: Option<Vec<PseudoCategory>>,
    pub comments: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enclosure: Option<PseudoEnclosure>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guid: Option<PseudoGuid>,
    #[serde(rename = "pubDate")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pub_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<PseudoSource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    // pub extensions: ExtensionMap,
    // pub itunes_ext: Option<ITunesItemExtension>,
    // pub dublin_core_ext: Option<DublinCoreExtension>
}

impl PartialEq for PseudoItem {
    fn eq(&self, other: &Self) -> bool {
        (self.link.is_some() && self.link == other.link)
            || (self.title.is_some() && self.title == other.title)
    }
}

impl From<PseudoItem> for Item {
    fn from(val: PseudoItem) -> Self {
        Item {
            title: val.title,
            link: val.link,
            description: val.description,
            author: val.author,
            categories: val
                .categories
                .unwrap_or_default()
                .into_iter()
                .map(PseudoCategory::into)
                .collect(),
            comments: val.comments,
            enclosure: val.enclosure.map(PseudoEnclosure::into),
            guid: val.guid.map(PseudoGuid::into),
            pub_date: val.pub_date,
            source: val.source.map(PseudoSource::into),
            content: val.content,
            extensions: ExtensionMap::default(),
            itunes_ext: None,
            dublin_core_ext: None,
        }
    }
}

/// Serde impled version of rss::Category
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PseudoCategory {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
}

impl From<PseudoCategory> for Category {
    fn from(val: PseudoCategory) -> Self {
        Category {
            name: val.name,
            domain: val.domain,
        }
    }
}

/// Serde impled version of rss::Enclosure
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PseudoEnclosure {
    pub url: String,
    pub length: String,
    #[serde(rename = "type")]
    pub mime_type: String,
}

impl From<PseudoEnclosure> for Enclosure {
    fn from(val: PseudoEnclosure) -> Self {
        Enclosure {
            url: val.url,
            length: val.length,
            mime_type: val.mime_type,
        }
    }
}

/// Serde impled version of rss::Guid
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PseudoGuid {
    pub value: String,
    pub permalink: bool,
}

impl From<PseudoGuid> for Guid {
    fn from(val: PseudoGuid) -> Self {
        Guid {
            value: val.value,
            permalink: val.permalink,
        }
    }
}

/// Serde impled version of rss::Source
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PseudoSource {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

impl From<PseudoSource> for Source {
    fn from(val: PseudoSource) -> Self {
        Source {
            url: val.url,
            title: val.title,
        }
    }
}

/// Serde impled version of rss::Cloud
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PseudoCloud {
    pub domain: String,
    pub port: String,
    pub path: String,
    #[serde(rename = "registerProcedure")]
    pub register_procedure: String,
    pub protocol: String,
}

impl From<PseudoCloud> for Cloud {
    fn from(val: PseudoCloud) -> Self {
        Cloud {
            domain: val.domain,
            port: val.port,
            path: val.path,
            register_procedure: val.register_procedure,
            protocol: val.protocol,
        }
    }
}

/// Serde impled version of rss::Image
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PseudoImage {
    pub url: String,
    pub title: String,
    pub link: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl From<PseudoImage> for Image {
    fn from(val: PseudoImage) -> Self {
        Image {
            url: val.url,
            title: val.title,
            link: val.link,
            width: val.width,
            height: val.height,
            description: val.description,
        }
    }
}

/// Serde impled version of rss::TextInput
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PseudoTextInput {
    pub title: String,
    pub description: String,
    pub name: String,
    pub link: String,
}

impl From<PseudoTextInput> for TextInput {
    fn from(val: PseudoTextInput) -> Self {
        TextInput {
            title: val.title,
            description: val.description,
            name: val.name,
            link: val.link,
        }
    }
}
