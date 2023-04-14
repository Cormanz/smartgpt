use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    //pub kind: String,
    //pub url: Url,
    //pub queries: Queries,
    //pub context: Context,
    //pub search_information: SearchInformation,
    pub items: Vec<Item>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Url {
    #[serde(rename = "type")]
    pub type_field: String,
    pub template: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Queries {
    pub request: Vec<Request>,
    pub next_page: Vec<NextPage>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub title: String,
    pub total_results: String,
    pub search_terms: String,
    pub count: i64,
    pub start_index: i64,
    pub input_encoding: String,
    pub output_encoding: String,
    pub safe: String,
    pub cx: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NextPage {
    pub title: String,
    pub total_results: String,
    pub search_terms: String,
    pub count: i64,
    pub start_index: i64,
    pub input_encoding: String,
    pub output_encoding: String,
    pub safe: String,
    pub cx: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Context {
    pub title: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchInformation {
    pub search_time: f64,
    pub formatted_search_time: String,
    pub total_results: String,
    pub formatted_total_results: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    //pub kind: String,
    pub title: String,
    //pub html_title: String,
    pub link: String,
    //pub display_link: String,
    pub snippet: String,
    //pub html_snippet: String,
    //pub cache_id: Option<String>,
    //pub formatted_url: String,
    //pub html_formatted_url: String,
    //pub pagemap: Pagemap,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pagemap {
    #[serde(rename = "cse_thumbnail")]
    #[serde(default)]
    pub cse_thumbnail: Vec<CseThumbnail>,
    pub metatags: Vec<Metatag>,
    #[serde(rename = "cse_image")]
    #[serde(default)]
    pub cse_image: Vec<CseImage>,
    #[serde(default)]
    pub videoobject: Vec<Videoobject>,
    #[serde(default)]
    pub speakablespecification: Vec<Speakablespecification>,
    pub listitem: Option<Vec<Listitem>>,
    #[serde(default)]
    pub imageobject: Vec<Imageobject>,
    pub review: Option<Vec<Review>>,
    #[serde(default)]
    pub person: Vec<Person>,
    #[serde(default)]
    pub aggregaterating: Vec<Aggregaterating>,
    pub rating: Option<Vec<Rating>>,
    #[serde(default)]
    pub individualproduct: Vec<Individualproduct>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CseThumbnail {
    pub src: String,
    pub width: String,
    pub height: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metatag {
    #[serde(rename = "og:image")]
    pub og_image: Option<String>,
    #[serde(rename = "og:type")]
    pub og_type: Option<String>,
    #[serde(rename = "article:published_time")]
    pub article_published_time: Option<String>,
    #[serde(rename = "twitter:card")]
    pub twitter_card: Option<String>,
    #[serde(rename = "twitter:title")]
    pub twitter_title: Option<String>,
    #[serde(rename = "og:image:width")]
    pub og_image_width: Option<String>,
    #[serde(rename = "og:site_name")]
    pub og_site_name: Option<String>,
    pub author: Option<String>,
    #[serde(rename = "og:title")]
    pub og_title: String,
    #[serde(rename = "og:image:height")]
    pub og_image_height: Option<String>,
    #[serde(rename = "og:updated_time")]
    pub og_updated_time: Option<String>,
    #[serde(rename = "msapplication-tileimage")]
    pub msapplication_tileimage: Option<String>,
    #[serde(rename = "og:description")]
    pub og_description: String,
    #[serde(rename = "twitter:image")]
    pub twitter_image: Option<String>,
    #[serde(rename = "pinterest-rich-pin")]
    pub pinterest_rich_pin: Option<String>,
    #[serde(rename = "fb:app_id")]
    pub fb_app_id: Option<String>,
    #[serde(rename = "article:modified_time")]
    pub article_modified_time: Option<String>,
    pub viewport: String,
    #[serde(rename = "twitter:description")]
    pub twitter_description: Option<String>,
    #[serde(rename = "og:locale")]
    pub og_locale: Option<String>,
    #[serde(rename = "og:url")]
    pub og_url: Option<String>,
    #[serde(rename = "msapplication-square70x70logo")]
    pub msapplication_square70x70logo: Option<String>,
    #[serde(rename = "msapplication-wide310x150logo")]
    pub msapplication_wide310x150logo: Option<String>,
    #[serde(rename = "twitter:site")]
    pub twitter_site: Option<String>,
    #[serde(rename = "msapplication-square310x310logo")]
    pub msapplication_square310x310logo: Option<String>,
    #[serde(rename = "parsely-section")]
    pub parsely_section: Option<String>,
    #[serde(rename = "msapplication-tilecolor")]
    pub msapplication_tilecolor: Option<String>,
    #[serde(rename = "article:section")]
    pub article_section: Option<String>,
    #[serde(rename = "msapplication-square150x150logo")]
    pub msapplication_square150x150logo: Option<String>,
    #[serde(rename = "fb:pages")]
    pub fb_pages: Option<String>,
    #[serde(rename = "article:author")]
    pub article_author: Option<String>,
    #[serde(rename = "parsely-tags")]
    pub parsely_tags: Option<String>,
    #[serde(rename = "article:opinion")]
    pub article_opinion: Option<String>,
    #[serde(rename = "onespot:page-type")]
    pub onespot_page_type: Option<String>,
    #[serde(rename = "gmi:topics")]
    pub gmi_topics: Option<String>,
    #[serde(rename = "google-signin-client_id")]
    pub google_signin_client_id: Option<String>,
    #[serde(rename = "be:sdk")]
    pub be_sdk: Option<String>,
    #[serde(rename = "be:norm_url")]
    pub be_norm_url: Option<String>,
    #[serde(rename = "branch:deeplink:$deeplink_path")]
    pub branch_deeplink_deeplink_path: Option<String>,
    #[serde(rename = "be:capsule_url")]
    pub be_capsule_url: Option<String>,
    #[serde(rename = "be:api_dt")]
    pub be_api_dt: Option<String>,
    #[serde(rename = "be:mod_dt")]
    pub be_mod_dt: Option<String>,
    #[serde(rename = "tp:preferredruntimes")]
    pub tp_preferredruntimes: Option<String>,
    #[serde(rename = "be:timer")]
    pub be_timer: Option<String>,
    #[serde(rename = "tp:initialize")]
    pub tp_initialize: Option<String>,
    #[serde(rename = "be:orig_url")]
    pub be_orig_url: Option<String>,
    #[serde(rename = "be:messages")]
    pub be_messages: Option<String>,
    #[serde(rename = "theme-color")]
    pub theme_color: Option<String>,
    #[serde(rename = "twitter:label1")]
    pub twitter_label1: Option<String>,
    #[serde(rename = "twitter:label2")]
    pub twitter_label2: Option<String>,
    #[serde(rename = "slick:category")]
    pub slick_category: Option<String>,
    #[serde(rename = "twitter:creator")]
    pub twitter_creator: Option<String>,
    #[serde(rename = "twitter:data1")]
    pub twitter_data1: Option<String>,
    #[serde(rename = "twitter:data2")]
    pub twitter_data2: Option<String>,
    #[serde(rename = "slick:featured_image")]
    pub slick_featured_image: Option<String>,
    #[serde(rename = "slick:wpversion")]
    pub slick_wpversion: Option<String>,
    #[serde(rename = "slick:group")]
    pub slick_group: Option<String>,
    #[serde(rename = "slick:wppostid")]
    pub slick_wppostid: Option<String>,
    #[serde(rename = "sailthru.tags")]
    pub sailthru_tags: Option<String>,
    #[serde(rename = "sailthru.excerpt")]
    pub sailthru_excerpt: Option<String>,
    #[serde(rename = "sailthru.contenttype")]
    pub sailthru_contenttype: Option<String>,
    pub title: Option<String>,
    #[serde(rename = "article:publisher")]
    pub article_publisher: Option<String>,
    #[serde(rename = "next-head-count")]
    pub next_head_count: Option<String>,
    #[serde(rename = "msapplication-tap-highlight")]
    pub msapplication_tap_highlight: Option<String>,
    #[serde(rename = "sailthru.socialtitle")]
    pub sailthru_socialtitle: Option<String>,
    #[serde(rename = "sailthru.date")]
    pub sailthru_date: Option<String>,
    pub thumbnail: Option<String>,
    #[serde(rename = "x-ua-compatible")]
    pub x_ua_compatible: Option<String>,
    pub m1: Option<String>,
    pub m2: Option<String>,
    #[serde(rename = "auto-publish")]
    pub auto_publish: Option<String>,
    #[serde(rename = "sailthru.image.thumb")]
    pub sailthru_image_thumb: Option<String>,
    #[serde(rename = "sailthru.image.full")]
    pub sailthru_image_full: Option<String>,
    #[serde(rename = "og:date")]
    pub og_date: Option<String>,
    #[serde(rename = "article-type")]
    pub article_type: Option<String>,
    #[serde(rename = "article-tags")]
    pub article_tags: Option<String>,
    #[serde(rename = "article-author")]
    pub article_author2: Option<String>,
    pub pagename: Option<String>,
    pub contentid: Option<String>,
    #[serde(rename = "pin:description")]
    pub pin_description: Option<String>,
    pub handheldfriendly: Option<String>,
    #[serde(rename = "ps-account")]
    pub ps_account: Option<String>,
    #[serde(rename = "ps-language")]
    pub ps_language: Option<String>,
    pub mobileoptimized: Option<String>,
    #[serde(rename = "ps-country")]
    pub ps_country: Option<String>,
    #[serde(rename = "pin:id")]
    pub pin_id: Option<String>,
    #[serde(rename = "og:image:type")]
    pub og_image_type: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CseImage {
    pub src: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Videoobject {
    pub contenturl: String,
    pub uploaddate: String,
    pub name: String,
    pub description: String,
    pub thumbnailurl: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Speakablespecification {
    pub cssselector: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Listitem {
    pub item: Option<String>,
    pub name: String,
    pub position: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Imageobject {
    pub image: String,
    pub thumbnail: String,
    pub caption: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Review {
    pub name: Option<String>,
    pub datecreated: String,
    pub reviewbody: String,
    pub datepublished: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Person {
    pub name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Aggregaterating {
    pub ratingvalue: String,
    pub reviewcount: String,
    pub bestrating: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rating {
    pub ratingvalue: String,
    pub bestrating: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Individualproduct {
    pub name: String,
}
