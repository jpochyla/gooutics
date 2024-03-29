use anyhow::Result;
use icalendar::{Calendar, Class, Component, Event, EventLike};

use crate::goout::GetSchedules;

pub fn event_calendar(language: &str, schedules: &GetSchedules) -> Result<Calendar> {
    let mut cal = Calendar::new();

    if let Some(venue) = schedules
        .included
        .venues
        .first()
        .and_then(|v| v.locales.get(language).map(|l| l.name.as_str()))
    {
        cal.name(venue);
    }

    for schedule in &schedules.schedules {
        if schedule.is_postponed_indefinitely() {
            continue;
        }

        let Some(venue) = schedule
            .relationships
            .venue
            .as_ref()
            .and_then(|r| schedules.find_venue(r.id))
        else {
            continue;
        };
        let Some(event) = schedule
            .relationships
            .event
            .as_ref()
            .and_then(|r| schedules.find_event(r.id))
        else {
            continue;
        };

        let loc = event.locales.get(language);
        let summary = loc.map(|l| l.name.as_str()).unwrap_or_default();
        let description = loc
            .map(|l| l.description.as_str())
            .map(markdown::to_html)
            .unwrap_or_default();

        let url = schedule
            .locales
            .get(language)
            .map(|l| l.site_url.as_str())
            .unwrap_or_default();

        // Prepend name to the address.
        let address = venue
            .locales
            .get(language)
            .map(|l| {
                let name = &l.name;
                let address = &venue.attributes.address;
                format!("{name}\n{address}")
            })
            .unwrap_or_else(|| venue.attributes.address.clone());

        let cal_event = Event::new()
            .url(url)
            .summary(summary)
            .description(&description)
            .starts(schedule.attributes.start_at)
            .ends(schedule.attributes.end_at)
            .location(&address)
            .class(Class::Public)
            .done();

        cal.push(cal_event);
    }

    Ok(cal)
}
