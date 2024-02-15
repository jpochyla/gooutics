use anyhow::Context;
use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use reqwest::Client;

use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

fn client() -> Client {
    Client::new()
}

fn load_json<'de, T>(json: &'de str) -> Result<T>
where
    T: Deserialize<'de>,
{
    let mut jd = serde_json::Deserializer::from_str(json);
    let val = serde_path_to_error::deserialize(&mut jd)?;
    Ok(val)
}

fn parse_venue_id(html: &str) -> Option<&str> {
    html.split(r#"https:\u002F\u002Fgoout.net\u002Fvenue\u002F"#)
        .nth(1)?
        .split('"')
        .next()
}

pub async fn get_venue_id(language: &str, short_id: &str) -> Result<String> {
    let html = client()
        .get(format!("https://goout.net/{language}/venue/{short_id}"))
        .send()
        .await?
        .text()
        .await?;
    tracing::debug!(?html);

    let id = parse_venue_id(&html)
        .with_context(|| format!("Failed to parse venue ID for {short_id:?}"))?;
    Ok(id.to_owned())
}

pub async fn get_schedules(language: &str, venue_ids: &str) -> Result<GetSchedules> {
    let json = client()
        .get("https://goout.net/services/entities/v1/schedules")
        .query(&[
            ("venueIds[]", venue_ids),
            ("languages[]", language),
            ("include", "events,venues"),
        ])
        .send()
        .await?
        .text()
        .await?;
    tracing::debug!(?json);

    let json =
        load_json(&json).with_context(|| format!("Failed to parse schedules for {venue_ids:?}"))?;
    Ok(json)
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rel {
    pub id: i64,
    #[serde(rename = "type")]
    pub type_field: String,
}

//
// GetSchedules
//

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSchedules {
    pub schedules: Vec<Schedule>,
    pub included: GetSchedulesInc,
}

impl GetSchedules {
    pub fn find_event(&self, id: i64) -> Option<&Event> {
        self.included.events.iter().find(|event| event.id == id)
    }

    pub fn find_venue(&self, id: i64) -> Option<&Venue> {
        self.included.venues.iter().find(|venue| venue.id == id)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSchedulesInc {
    pub events: Vec<Event>,
    pub schedules: Vec<Schedule>,
    pub venues: Vec<Venue>,
}

//
// Schedule
//

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Schedule {
    pub id: i64,
    pub attributes: ScheduleAttrs,
    pub relationships: ScheduleRels,
    pub locales: HashMap<String, ScheduleLocale>,
    #[serde(rename = "type")]
    pub type_field: String,
    pub url: String,
}

impl Schedule {
    pub fn is_postponed_indefinitely(&self) -> bool {
        self.attributes
            .tags
            .iter()
            .any(|tag| tag == "postponed_indefinitely")
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleAttrs {
    pub state: String,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub has_time: bool,
    pub has_time_end: bool,
    pub doors_time_at: Option<String>,
    pub announced_at: String,
    pub published_at: String,
    pub is_permanent: bool,
    pub external_tickets_url: Option<String>,
    pub external_stream_url: Option<String>,
    pub parsed_at: Option<String>, // Date
    pub tags: Vec<String>,
    pub source_urls: Vec<String>,
    pub ticketing_state: String,
    pub pricing: Option<String>,
    pub updated_at: String,
    pub currency: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleRels {
    pub contacts: Vec<Rel>,
    pub sale: Option<Rel>,
    pub venue: Option<Rel>,
    pub event: Option<Rel>,
    pub parent: Option<Rel>,
    pub parent_inner_schedules: Vec<Rel>,
    pub inner_schedules: Vec<Rel>,
    pub duplicate_schedules: Vec<Rel>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleLocale {
    pub stage: Option<String>,
    pub site_url: String,
}

//
// Event
//

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub id: i64,
    pub attributes: EventAttrs,
    pub relationships: EventRels,
    pub locales: HashMap<String, EventLocale>,
    #[serde(rename = "type")]
    pub type_field: String,
    pub url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventAttrs {
    pub state: String,
    pub main_category: String,
    pub categories: Vec<String>,
    pub keywords: Vec<Value>,
    pub tags: Vec<String>,
    pub tags_manual: Vec<String>,
    pub film_meta: FilmMeta,
    pub exhibition_meta: ExhibitionMeta,
    pub minor_performers: Vec<String>,
    pub recommendation: Option<String>,
    pub schedules_range: Value,
    pub has_time_slots: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilmMeta {
    pub imdb: Value,
    pub csfd: Value,
    pub filmweb: Value,
    pub original_name: Value,
    pub released: Value,
    pub length: Value,
    pub director: Value,
    pub author: Value,
    pub country_isos: Vec<Value>,
    pub rating: Value,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExhibitionMeta {
    pub curator: Value,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventLocale {
    pub name: String,
    pub note: String,
    pub description: String,
    pub meta_description: Option<String>,
    pub meta_title: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventRels {
    pub videos: Vec<Rel>,
    pub images: Vec<Rel>,
    pub performers: Vec<Rel>,
    pub revision_parent: Option<Rel>,
    pub revisions: Vec<Rel>,
}

//
// Venue
//

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Venue {
    pub id: i64,
    pub attributes: VenueAttrs,
    pub locales: HashMap<String, VenueLocale>,
    #[serde(rename = "type")]
    pub type_field: String,
    pub url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VenueAttrs {
    pub state: String,
    pub main_category: String,
    pub categories: Vec<String>,
    pub address: String,
    pub country_iso: String,
    pub latitude: f64,
    pub longitude: f64,
    pub updated_at: String,
    pub email: String,
    pub phone: String,
    pub url_facebook: Option<String>,
    pub source_url: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VenueLocale {
    pub name: String,
    pub description: String,
    pub site_url: String,
    pub meta_description: Option<String>,
    pub meta_title: Option<String>,
}
