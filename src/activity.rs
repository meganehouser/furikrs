use chrono::{DateTime, Utc};
use failure::{bail, Error, err_msg, format_err};
use github_rs::client::{Executor, Github};
use serde_json::Value;

use std::collections::HashSet;

const MAX_BODY_LENGTH: &'static usize = &30;

const DEFAULT_EVENT_TYPES: &'static [&'static str] = &[
    "IssuesEvent",
    "PullRequestEvent",
    "PullRequestReviewCommentEvent",
    "IssueCommentEvent",
    "CommitCommentEvent",
];

#[derive(Debug)]
pub struct Activity {
    activity_type: String,
    action: String,
    title: Option<String>,
    body: Option<String>,
    link: String,
    id: String,
    created_at: DateTime<Utc>,
}

impl Activity {
    fn new(type_: &str, action: &str, title: Option<&str>, body: Option<&str>, link: &str, id: &str, created_at: &DateTime<Utc>) -> Activity {
        Activity {
            activity_type: String::from(type_),
            title: title.map(|t| String::from(t)),
            action: String::from(action),
            body: body.map(|b| String::from(b)),
            link: String::from(link),
            id: String::from(id),
            created_at: created_at.to_owned(),
        }
    }

    pub fn to_markdown(&self) -> String {
        let body = match &self.body {
            None => String::from(""),
            Some(b) => {
                if b.chars().count() <= *MAX_BODY_LENGTH {
                    String::from(b.as_ref())
                } else {
                    let b_: String = b.chars().take(*MAX_BODY_LENGTH).collect();
                    format!("{}...", b_)
                }
            }
        };


        let title = self.title
            .as_ref()
            .map(|s| format!(" :{}", s.as_str()))
            .unwrap_or_else(|| String::from(""));

        // activity_type-action-[id: title](link): body ...
        format!("{}-{}-[{}{}]({})]: {}",
            self.activity_type,
            self.action,
            self.id,
            title,
            self.link,
            body)
    }
}

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
                   -> Result<Vec<Activity>, Error> {
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

        let events = json.as_array()
            .ok_or_else(|| err_msg("invalid format json"))?
            .iter()
            .filter(|evnt| self.include_event_types.contains(evnt["type"].as_str().unwrap()))
            .map(|evnt| object_to_activity(&evnt).unwrap())
            .filter(|evnt| from <= &evnt.created_at && &evnt.created_at <= to)
            .collect();


        Ok(events)
    }
}

fn object_to_activity(object: &Value) -> Result<Activity, Error> {
    match object["type"].as_str().unwrap() {
        "IssuesEvent" => {
            Ok(Activity::new(
                "Issue",
                as_str(&object["payload"]["action"])?,
                Some(as_str(&object["payload"]["issue"]["title"])?),
                None,
                as_str(&object["payload"]["issue"]["html_url"])?,
                &format!("#{}", as_u64(&object["payload"]["issue"]["number"])?),
                &as_datetime(&object["created_at"])?
            ))
        }
        "PullRequestEvent" => {
            Ok(Activity::new(
                "PullRequest",
                as_str(&object["payload"]["action"])?,
                Some(as_str(&object["payload"]["pull_request"]["title"])?),
                None,
                as_str(&object["payload"]["pull_request"]["html_url"])?,
                &format!("#{}", as_u64(&object["payload"]["pull_request"]["number"])?),
                &as_datetime(&object["created_at"])?
            ))
        }
        "PullRequestReviewCommentEvent" => {
            Ok(Activity::new(
                "PullRequestReview",
                as_str(&object["payload"]["action"])?,
                Some(as_str(&object["payload"]["pull_request"]["title"])?),
                Some(as_str(&object["payload"]["comment"]["body"])?),
                as_str(&object["payload"]["comment"]["html_url"])?,
                &format!("#{}", as_u64(&object["payload"]["pull_request"]["number"])?),
                &as_datetime(&object["created_at"])?
            ))
        }
        "IssueCommentEvent" => {
            Ok(Activity::new(
                "IssueComment",
                as_str(&object["payload"]["action"])?,
                Some(as_str(&object["payload"]["issue"]["title"])?),
                Some(as_str(&object["payload"]["comment"]["body"])?),
                as_str(&object["payload"]["comment"]["html_url"])?,
                &format!("#{}", as_u64(&object["payload"]["issue"]["number"])?),
                &as_datetime(&object["created_at"])?
            ))
        }
        "CommitCommentEvent" => {
            Ok(Activity::new(
                "CommitComment",
                as_str(&object["payload"]["action"])?,
                None,
                Some(as_str(&object["payload"]["comment"]["body"])?),
                as_str(&object["payload"]["comment"]["html_url"])?,
                &as_str(&object["payload"]["comment"]["commit_id"])?[..7],
                &as_datetime(&object["created_at"])?
            ))
        }
        _ => bail!("invalid activity type")
    }
}

fn as_str(json: &Value) -> Result<&str, Error> {
    Ok(json.as_str().ok_or_else(|| err_msg("invalid format json"))?)
}

fn as_u64(json: &Value) -> Result<u64, Error> {
    Ok(json.as_u64().ok_or_else(|| err_msg("invalid format json"))?)
}

fn as_datetime(json: &Value) -> Result<DateTime<Utc>, Error> {
    let datetime_str = as_str(json)?;
    let datetime = DateTime::parse_from_rfc3339(datetime_str)?;
    Ok(datetime.with_timezone(&Utc))
}
