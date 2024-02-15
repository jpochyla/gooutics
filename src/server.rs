use anyhow::{Error, Result};
use axum::{
    extract::Path,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use icalendar::Calendar;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

use crate::{goout, ical};

pub fn create_router() -> Router {
    Router::new()
        .route("/", get(handle_index))
        .route("/:language/:name/:short_id/events/", get(handle_get_events))
        .route("/:language/:name/:short_id/events", get(handle_get_events))
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
}

async fn handle_index() -> Result<impl IntoResponse, AppError> {
    Ok("ok")
}

async fn handle_get_events(
    Path((language, _name, short_id)): Path<(String, String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let calendar = get_events(&language, &short_id).await?;
    Ok((
        [(header::CONTENT_TYPE, "text/calendar")],
        calendar.to_string(),
    ))
}

pub async fn get_events(language: &str, short_id: &str) -> Result<Calendar> {
    let venue_id = goout::get_venue_id(language, short_id).await?;
    let schedules = goout::get_schedules(language, &venue_id).await?;
    let calendar = ical::event_calendar(language, &schedules)?;
    Ok(calendar)
}

struct AppError(Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()).into_response()
    }
}

impl<E> From<E> for AppError
where
    Error: From<E>,
{
    fn from(err: E) -> Self {
        Self(Error::from(err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_punctum_events() {
        let lang = "en";
        let short_id = "vzkpbb";
        let venue_id = goout::get_venue_id(lang, short_id).await.unwrap();
        dbg!(&venue_id);
        let schedules = goout::get_schedules(lang, &venue_id).await.unwrap();
        dbg!(&schedules);
        let calendar = ical::event_calendar(lang, &schedules).unwrap();
        dbg!(&calendar);
    }
}
