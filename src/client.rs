use std::convert::TryFrom;

use chrono::{DateTime, Utc};
use failure::{Error, err_msg, format_err};
use github_rs::client::{Executor, Github};
use serde_json::Value;

use crate::activity::GithubActivities;
use crate::json::RawEvent;

pub struct ActivityClient {
    user_name: String,
    access_token: String,
}

impl ActivityClient {
    pub fn new(user_name: &str, access_token: &str) -> ActivityClient {
        ActivityClient {
            user_name: user_name.to_string(),
            access_token: access_token.to_owned(),
        }
    }

    pub fn collect(&self, from: &DateTime<Utc>, to: &DateTime<Utc>, include_private: bool)
                   -> Result<GithubActivities, Error> {
        let client = Github::new(&self.access_token)
            .map_err(|e| format_err!("{}", e))?;

        let events_func =
            client.get()
                .users()
                .username(&self.user_name)
                .events();

        let (_, _, option_json) = if include_private {
            events_func
                .execute::<Value>()
                .map_err(|e| format_err!("{}", e))?
        } else {
            events_func
                .public()
                .execute::<Value>()
                .map_err(|e| format_err!("{}", e))?
        };

        let json = option_json.ok_or_else(|| err_msg("not found json"))?;
        debug!("{}", json);

        let raw_events: Vec<RawEvent> = json.as_array()
            .ok_or_else(|| err_msg("invalid format json"))?
            .into_iter()
            .map(|value| RawEvent::try_from(value).unwrap())
            .filter(|raw_event| from <= &raw_event.created_at && &raw_event.created_at <= to)
            .collect();

        GithubActivities::try_from(raw_events.as_slice())
    }
}
