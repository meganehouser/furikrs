use std::convert::TryFrom;

use chrono::{DateTime, Utc};
use failure::{self, bail, err_msg};
use serde_json::Value;

use crate::activity::{ GithubActivities, GithubObject, Activity, GithubObjectType };

impl TryFrom<&[&Value]> for GithubActivities {
    type Error = failure::Error;

    fn try_from(events: &[&Value]) -> Result<Self, Self::Error> {
        let mut github_activities = GithubActivities::new();

        for event in events {
            append_activity(&event, &mut github_activities)?;
        }

        Ok(github_activities)
    }
}

fn append_activity(value: &Value, github_activities: &mut GithubActivities) -> Result<(), failure::Error> {
    let repo_name = as_str(&value["repo"]["name"])?;

    match as_str(&value["type"])? {
        "IssuesEvent" => {
            let id = &format!("#{}", as_u64(&value["payload"]["issue"]["number"])?);
            let obj = match github_activities.get_mut(repo_name, id) {
                Some(obj) => obj,
                None => {
                    github_activities.append(repo_name,
                                             GithubObject::new(
                                                 id,
                                                 GithubObjectType::Issue,
                                                 as_str(&value["payload"]["issue"]["html_url"])?,
                                                 Some(as_str(&value["payload"]["issue"]["title"])?),
                                             ),
                    )
                }
            };
            obj.activities.push(
                Activity::new(
                    as_str(&value["payload"]["action"])?,
                    None,
                    &as_datetime(&value["created_at"])?,
                )
            );
        }
        "IssueCommentEvent" => {
            let id = &format!("#{}", as_u64(&value["payload"]["issue"]["number"])?);
            let obj = match github_activities.get_mut(repo_name, id) {
                Some(obj) => obj,
                None => {
                    github_activities.append(repo_name,
                                             GithubObject::new(
                                                 id,
                                                 GithubObjectType::Issue,
                                                 as_str(&value["payload"]["comment"]["html_url"])?,
                                                 Some(as_str(&value["payload"]["issue"]["title"])?),
                                             ),
                    )
                }
            };
            obj.activities.push(
                Activity::new(
                    &format!("Comment {}", as_str(&value["payload"]["action"])?),
                    Some(as_str(&value["payload"]["comment"]["body"])?),
                    &as_datetime(&value["created_at"])?,
                )
            );
        }
        "PullRequestEvent" => {
            let id = &format!("#{}", as_u64(&value["payload"]["pull_request"]["number"])?);
            let obj = match github_activities.get_mut(repo_name, id) {
                Some(obj) => obj,
                None => {
                    github_activities.append(repo_name,
                                             GithubObject::new(
                                                 id,
                                                 GithubObjectType::PullRequest,
                                                 as_str(&value["payload"]["pull_request"]["html_url"])?,
                                                 Some(as_str(&value["payload"]["pull_request"]["title"])?),
                                             ),
                    )
                }
            };
            obj.activities.push(
                Activity::new(
                    as_str(&value["payload"]["action"])?,
                    None,
                    &as_datetime(&value["created_at"])?,
                )
            );
        }
        "PullRequestReviewCommentEvent" => {
            let id = &format!("#{}", as_u64(&value["payload"]["pull_request"]["number"])?);
            let obj = match github_activities.get_mut(repo_name, id) {
                Some(obj) => obj,
                None => {
                    github_activities.append(repo_name,
                                             GithubObject::new(
                                                 id,
                                                 GithubObjectType::PullRequest,
                                                 as_str(&value["payload"]["comment"]["html_url"])?,
                                                 Some(as_str(&value["payload"]["pull_request"]["title"])?),
                                             )
                    )
                }
            };
            obj.activities.push(
                Activity::new(
                    &format!("Comment {}", as_str(&value["payload"]["action"])?),
                    Some(as_str(&value["payload"]["comment"]["body"])?),
                    &as_datetime(&value["created_at"])?,
                )
            );
        }
        "CommitCommentEvent" => {
            let id = &as_str(&value["payload"]["comment"]["commit_id"])?[..6];
            let obj = match  github_activities.get_mut(repo_name, id) {
                Some(obj) => obj,
                None => {
                    github_activities.append(repo_name, GithubObject::new(
                        id,
                        GithubObjectType::Commit,
                        as_str(&value["payload"]["comment"]["html_url"])?,
                        None,
                    ))
                }
            };
            obj.activities.push(
                Activity::new(
                    &format!("Comment {}", as_str(&value["payload"]["action"])?),
                    Some(as_str(&value["payload"]["comment"]["body"])?),
                    &as_datetime(&value["created_at"])?,
                )
            );
        }
        _ => bail!("invalid activity type")
    }
    Ok(())
}

pub fn as_str(json: &Value) -> Result<&str, failure::Error> {
    Ok(json.as_str().ok_or_else(|| err_msg("invalid format json"))?)
}

pub fn as_u64(json: &Value) -> Result<u64, failure::Error> {
    Ok(json.as_u64().ok_or_else(|| err_msg("invalid format json"))?)
}

pub fn as_datetime(json: &Value) -> Result<DateTime<Utc>, failure::Error> {
    let datetime_str = as_str(json)?;
    let datetime = DateTime::parse_from_rfc3339(datetime_str)?;
    Ok(datetime.with_timezone(&Utc))
}
