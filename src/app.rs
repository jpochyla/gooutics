use anyhow::{Error, Result};
use axum::{
    extract::Path,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

use crate::{goout, ical};

pub fn create_router() -> Router {
    Router::new()
        .route("/:language/:name/:short_id/events", get(get_events))
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
}

async fn get_events(
    Path((language, _name, short_id)): Path<(String, String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let venue_id = goout::get_venue_id(&language, &short_id).await?;
    let schedules = goout::get_schedules(&language, &venue_id).await?;
    let calendar = ical::event_calendar(&language, &schedules)?;
    Ok((
        [(header::CONTENT_TYPE, "text/calendar")],
        calendar.to_string(),
    ))
}

struct AppError(Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()).into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
