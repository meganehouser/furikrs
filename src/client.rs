use std::collections::HashSet;
use std::convert::TryFrom;

use chrono::{DateTime, Utc};
use failure::{Error, err_msg, format_err};
use github_rs::client::{Executor, Github};
use serde_json::Value;

use crate::activity::GithubActivities;
use crate::json::{as_datetime, as_str};

const DEFAULT_EVENT_TYPES: &'static [&'static str] = &[
    "IssuesEvent",
    "PullRequestEvent",
    "PullRequestReviewCommentEvent",
    "IssueCommentEvent",
    "CommitCommentEvent",
];

pub struct ActivityClient {
    user_name: String,
    access_token: String,
    include_event_types: HashSet<String>,
}

impl ActivityClient {
    pub fn new(user_name: &str, access_token: &str) -> ActivityClient {
        ActivityClient {
            user_name: user_name.to_string(),
            access_token: access_token.to_owned(),
            include_event_types: DEFAULT_EVENT_TYPES
                .iter()
                .map(|s| s.to_string())
                .collect(),
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

        let events: Vec<&Value> = json.as_array()
            .ok_or_else(|| err_msg("invalid format json"))?
            .iter()
            .filter(|evnt| self.include_event_types.contains(as_str(&evnt["type"]).unwrap()))
            .filter(|evnt| {
                let created_at = as_datetime(&evnt["created_at"]).unwrap();
                (from <= &created_at && &created_at <= to)
            }).collect();

        Ok(GithubActivities::try_from(events.as_slice())?)
    }
}
