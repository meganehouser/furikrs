use std::convert::TryFrom;

use chrono::{DateTime, Utc};
use failure::{self, err_msg};
use serde_json::Value;

use crate::activity::{ GithubActivities, GithubObject, Activity, GithubObjectType };

trait ParseJsonValue {
    fn parse_id(value: &Value) -> Result<String, failure::Error>;
    fn parse_object(id: &str, value: &Value) -> Result<GithubObject, failure::Error>;
    fn parse_activity(created_at: &DateTime<Utc>, value: &Value) -> Result<Activity, failure::Error>;
}

impl TryFrom<(&[Value], &DateTime<Utc>, &DateTime<Utc>)> for GithubActivities {
    type Error = failure::Error;

    fn try_from(init: (&[Value], &DateTime<Utc>, &DateTime<Utc>)) -> Result<Self, Self::Error> {
        let (events, from, to) = init;

        let mut github_activities = GithubActivities::new();

        for event in events {
            let created_at = as_datetime(&event["created_at"])?;
            if !(from <= &created_at && &created_at <= to) {
                continue;
            }

            let event_type = as_str(&event["type"])?;
            match event_type {
                "IssuesEvent" => {
                    append_activity::<IssuesEvent>(&created_at, &event, &mut github_activities)?;
                },
                "IssueCommentEvent" => {
                    append_activity::<IssueCommentEvent>(&created_at, &event, &mut github_activities)?;
                }
                "PullRequestEvent" => {
                    append_activity::<PullRequestEvent>(&created_at, &event, &mut github_activities)?;
                },
                "PullRequestReviewCommentEvent" => {
                    append_activity::<PullRequestReviewCommentEvent>(&created_at, &event, &mut github_activities)?;
                },
                "CommitCommentEvent" => {
                    append_activity::<CommitCommentEvent>(&created_at, &event, &mut github_activities)?;
                },
                _ => (),
            }
        }

        Ok(github_activities)
    }
}

fn append_activity<P>(created_at: &DateTime<Utc>, value: &Value, github_activities: &mut GithubActivities) -> Result<(), failure::Error>
    where P: ParseJsonValue {
    let repo_name = as_str(&value["repo"]["name"])?;

    let id = P::parse_id(value)?;
    let github_obj = match github_activities.get_mut(repo_name, &id) {
        Some(obj) => obj,
        None => {
            let obj = P::parse_object(&id, value)?;
            github_activities.append(repo_name, obj);
            github_activities.get_mut(repo_name, &id).unwrap()
        }
    };

    let activity = P::parse_activity(created_at, value)?;
    github_obj.activities.push(activity);
    Ok(())
}


struct IssuesEvent {}

impl ParseJsonValue for IssuesEvent {
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
