use std::convert::TryFrom;

use chrono::{DateTime, Utc};
use failure::{self, err_msg};
use serde_json::Value;

use crate::activity::{ GithubActivities, GithubObject, Activity, GithubObjectType };

trait ParseJsonValue {
    const TYPE_NAME: &'static str;
    fn parse_id(value: &Value) -> Result<String, failure::Error>;
    fn parse_object(id: &str, value: &Value) -> Result<GithubObject, failure::Error>;
    fn parse_activity(created_at: &DateTime<Utc>, value: &Value) -> Result<Activity, failure::Error>;
}

pub struct RawEvent {
    pub created_at: DateTime<Utc>,
    raw_data: Value,
}

impl RawEvent {
    fn parse_repo_name(&self) -> Result<&str, failure::Error> {
        Ok(as_str(&self.raw_data["repo"]["name"])?)
    }

    fn parse_id<P: ParseJsonValue>(&self) -> Result<String, failure::Error> {
        P::parse_id(&self.raw_data)
    }

    fn parse_object<P: ParseJsonValue>(&self) -> Result<GithubObject, failure::Error> {
        let id = self.parse_id::<P>()?;
        P::parse_object(&id, &self.raw_data)
    }

    fn parse_activity<P: ParseJsonValue>(&self) -> Result<Activity, failure::Error> {
        P::parse_activity(&self.created_at, &self.raw_data)
    }
}

impl TryFrom<&Value> for RawEvent {
    type Error = failure::Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let created_at = as_datetime(&value["created_at"])?;
        Ok(RawEvent {
            raw_data: value.to_owned(),
            created_at,
        })
    }
}

impl TryFrom<&[RawEvent]> for GithubActivities {
    type Error = failure::Error;

    fn try_from(raw_events: &[RawEvent]) -> Result<Self, Self::Error> {
        let mut github_activities = GithubActivities::new();

        for event in raw_events {
            let event_type = as_str(&event.raw_data["type"])?;
            match event_type {
                type_name if type_name == IssuesEvent::TYPE_NAME => {
                    append_activity::<IssuesEvent>(event, &mut github_activities)?;
                },
                type_name if type_name == IssueCommentEvent::TYPE_NAME => {
                    append_activity::<IssueCommentEvent>(event, &mut github_activities)?;
                }
                type_name if type_name == PullRequestEvent::TYPE_NAME => {
                    append_activity::<PullRequestEvent>(event, &mut github_activities)?;
                },
                type_name if type_name == PullRequestReviewCommentEvent::TYPE_NAME => {
                    append_activity::<PullRequestReviewCommentEvent>(event, &mut github_activities)?;
                },
                type_name if type_name == CommitCommentEvent::TYPE_NAME => {
                    append_activity::<CommitCommentEvent>(event, &mut github_activities)?;
                },
                _ => (),
            }
        }

        Ok(github_activities)
    }
}

fn append_activity<P>(raw_event: &RawEvent, github_activities: &mut GithubActivities) -> Result<(), failure::Error>
    where P: ParseJsonValue {
    let repo_name = raw_event.parse_repo_name()?;

    let id = raw_event.parse_id::<P>()?;
    let github_obj = match github_activities.get_mut(repo_name, &id) {
        Some(obj) => obj,
        None => {
            let obj = raw_event.parse_object::<P>()?;
            github_activities.append(repo_name, obj);
            github_activities.get_mut(repo_name, &id).unwrap()
        }
    };

    let activity = raw_event.parse_activity::<P>()?;
    github_obj.activities.push(activity);
    Ok(())
}


struct IssuesEvent {}

impl ParseJsonValue for IssuesEvent {
    const TYPE_NAME: &'static str = "IssuesEvent";

    fn parse_id(value: &Value) -> Result<String, failure::Error> {
        let issue_no = as_u64(&value["payload"]["issue"]["number"])?;
        Ok(format!("#{}", issue_no))
    }

    fn parse_object(id: &str, value: &Value) -> Result<GithubObject, failure::Error> {
        let html_link = as_str(&value["payload"]["issue"]["html_url"])?;
        let issue_title = as_str(&value["payload"]["issue"]["title"])?;
        Ok(GithubObject::new(id, GithubObjectType::Issue, html_link, Some(issue_title)))
    }
    fn parse_activity(created_at: &DateTime<Utc>, value: &Value) -> Result<Activity, failure::Error> {
        let action = as_str(&value["payload"]["action"])?;
        Ok(Activity::new(action, None, created_at))
    }
}

struct IssueCommentEvent {}

impl ParseJsonValue for IssueCommentEvent {
    const TYPE_NAME: &'static str = "IssueCommentEvent";

    fn parse_id(value: &Value) -> Result<String, failure::Error> {
        let issue_no = as_u64(&value["payload"]["issue"]["number"])?;
        Ok(format!("#{}", issue_no))
    }

    fn parse_object(id: &str, value: &Value) -> Result<GithubObject, failure::Error> {
        let html_link = as_str(&value["payload"]["issue"]["html_url"])?;
        let issue_title = as_str(&value["payload"]["issue"]["title"])?;
        Ok(GithubObject::new(id, GithubObjectType::Issue, html_link, Some(issue_title)))
    }
    fn parse_activity(created_at: &DateTime<Utc>, value: &Value) -> Result<Activity, failure::Error> {
        let action = format!("Comment {}", as_str(&value["payload"]["action"])?);
        let body =  as_str(&value["payload"]["comment"]["body"])?;
        Ok(Activity::new(action, Some(&body), created_at))
    }
}

struct PullRequestEvent {}

impl ParseJsonValue for PullRequestEvent {
    const TYPE_NAME: &'static str = "PullRequestEvent";

    fn parse_id(value: &Value) -> Result<String, failure::Error> {
        let pr_no = as_u64(&value["payload"]["pull_request"]["number"])?;
        Ok(format!("#{}", pr_no))
    }

    fn parse_object(id: &str, value: &Value) -> Result<GithubObject, failure::Error> {
        let html_link = as_str(&value["payload"]["pull_request"]["html_url"])?;
        let pr_title = as_str(&value["payload"]["pull_request"]["title"])?;
        Ok(GithubObject::new(id, GithubObjectType::PullRequest, html_link, Some(pr_title)))
    }
    fn parse_activity(created_at: &DateTime<Utc>, value: &Value) -> Result<Activity, failure::Error> {
        let action = as_str(&value["payload"]["action"])?;
        Ok(Activity::new(action, None, created_at))
    }
}

struct PullRequestReviewCommentEvent {}

impl ParseJsonValue for PullRequestReviewCommentEvent {
    const TYPE_NAME: &'static str = "PullRequestReviewCommentEvent";

    fn parse_id(value: &Value) -> Result<String, failure::Error> {
        let pr_no = as_u64(&value["payload"]["pull_request"]["number"])?;
        Ok(format!("#{}", pr_no))
    }

    fn parse_object(id: &str, value: &Value) -> Result<GithubObject, failure::Error> {
        let html_link = as_str(&value["payload"]["pull_request"]["html_url"])?;
        let pr_title = as_str(&value["payload"]["pull_request"]["title"])?;
        Ok(GithubObject::new(id, GithubObjectType::PullRequest, html_link, Some(pr_title)))
    }
    fn parse_activity(created_at: &DateTime<Utc>, value: &Value) -> Result<Activity, failure::Error> {
        let action = format!("Comment {}", as_str(&value["payload"]["action"])?);
        let body =  as_str(&value["payload"]["comment"]["body"])?;
        Ok(Activity::new(action, Some(&body), created_at))
    }
}

struct CommitCommentEvent {}

impl ParseJsonValue for CommitCommentEvent {
    const TYPE_NAME: &'static str = "CommitCommentEvent";

    fn parse_id(value: &Value) -> Result<String, failure::Error> {
        let commit_id = String::from(&as_str(&value["payload"]["comment"]["commit_id"])?[..6]);
        Ok(commit_id)
    }

    fn parse_object(id: &str, value: &Value) -> Result<GithubObject, failure::Error> {
        let html_link = as_str(&value["payload"]["commit"]["html_url"])?;
        Ok(GithubObject::new(id, GithubObjectType::Commit, html_link, None))
    }
    fn parse_activity(created_at: &DateTime<Utc>, value: &Value) -> Result<Activity, failure::Error> {
        let action = format!("Comment {}", as_str(&value["payload"]["action"])?);
        let body = as_str(&value["payload"]["comment"]["body"])?;
        Ok(Activity::new(action, Some(&body), created_at))
    }
}

fn as_str(json: &Value) -> Result<&str, failure::Error> {
    Ok(json.as_str().ok_or_else(|| err_msg("invalid format json"))?)
}

fn as_u64(json: &Value) -> Result<u64, failure::Error> {
    Ok(json.as_u64().ok_or_else(|| err_msg("invalid format json"))?)
}

fn as_datetime(json: &Value) -> Result<DateTime<Utc>, failure::Error> {
    let datetime_str = as_str(json)?;
    let datetime = DateTime::parse_from_rfc3339(datetime_str)?;
    Ok(datetime.with_timezone(&Utc))
}
