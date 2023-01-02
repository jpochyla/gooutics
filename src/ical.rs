use anyhow::Result;
use icalendar::{Calendar, Class, Component, Event, EventLike};

use crate::goout::GetSchedules;

pub fn event_calendar(language: &str, schedules: &GetSchedules) -> Result<Calendar> {
    let mut cal = Calendar::new();

    for schedule in &schedules.schedules {
        let Some(venue) = schedule.relationships.venue.as_ref().and_then(|r| schedules.find_venue(r.id)) else {
            continue;
        };
        let Some(event) = schedule.relationships.event.as_ref().and_then(|r| schedules.find_event(r.id)) else {
            continue;
        };

        let locale = event.locales.get(language);
        let summary = locale.map(|l| l.name.as_str()).unwrap_or_default();
        let description = locale.map(|l| l.description.as_str()).unwrap_or_default();

        let cal_event = Event::new()
            .summary(summary)
            .description(description)
            .starts(schedule.attributes.start_at)
            .ends(schedule.attributes.end_at)
            .url(&schedule.url)
            .location(&venue.attributes.address)
            .class(Class::Public)
            .done();

        cal.push(cal_event);
    }

    Ok(cal)
}
