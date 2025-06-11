use std::{fmt, str::FromStr as _};

use axum::{extract::FromRequestParts, http::request::Parts};
use flux_lib::locale::Locale;

use crate::app::error::AppError;

pub struct AppLocale {
    pub locale: Locale,
}

impl fmt::Display for AppLocale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.locale.to_string())
    }
}

impl<S> FromRequestParts<S> for AppLocale
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let locale = parts
            .headers
            .get("locale")
            .and_then(|l| l.to_str().ok())
            .unwrap_or("");

        let locale = Locale::from_str(locale).unwrap_or(Locale::En);

        Ok(AppLocale { locale })
    }
}
