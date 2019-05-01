use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde_json::Value;

pub struct GithubActivities {
    pub activity_objects: HashMap<String, HashMap<String, GithubObject>>
}

pub trait ParseActivity {
    const TYPE_NAME: &'static str;
    fn parse_id(value: &Value) -> Result<String, failure::Error>;
    fn parse_object(id: &str, value: &Value) -> Result<GithubObject, failure::Error>;
    fn parse_activity(created_at: &DateTime<Utc>, value: &Value) -> Result<Activity, failure::Error>;
}

pub struct RawEvent {
    pub type_name: String,
    pub repo_name: String,
    pub created_at: DateTime<Utc>,
    raw_data: Value,
}

impl RawEvent {
    pub fn new(type_name: &str, repo_name: &str, created_at: DateTime<Utc>, raw_data: &Value) -> RawEvent {
        RawEvent {
            type_name: String::from(type_name),
            repo_name: String::from(repo_name),
            raw_data: raw_data.to_owned(),
            created_at,
        }
    }

    fn parse_id<P: ParseActivity>(&self) -> Result<String, failure::Error> {
        P::parse_id(&self.raw_data)
    }

    fn parse_object<P: ParseActivity>(&self) -> Result<GithubObject, failure::Error> {
        let id = self.parse_id::<P>()?;
        P::parse_object(&id, &self.raw_data)
    }

    fn parse_activity<P: ParseActivity>(&self) -> Result<Activity, failure::Error> {
        P::parse_activity(&self.created_at, &self.raw_data)
    }
}

impl GithubActivities {
    pub fn new() -> GithubActivities {
        GithubActivities {
            activity_objects: HashMap::new()
        }
    }

    fn get_mut(&mut self, repo_name: &str, object_id: &str) -> Option<&mut GithubObject> {
        self.activity_objects
            .entry(String::from(repo_name))
            .or_insert(HashMap::new())
            .get_mut(object_id)
    }

    fn append(&mut self, repo_name: &str, obj: GithubObject) {
        let id = obj.id.to_owned();
        self.activity_objects
            .entry(String::from(repo_name))
            .or_insert(HashMap::new())
            .insert(String::from(id.as_str()), obj);
    }

    pub fn append_activity<P>(&mut self, raw_event: &RawEvent) -> Result<(), failure::Error>
        where P: ParseActivity{
        let repo_name = &raw_event.repo_name;

        let id = raw_event.parse_id::<P>()?;
        let github_obj = match self.get_mut(repo_name, &id) {
            Some(obj) => obj,
            None => {
                let obj = raw_event.parse_object::<P>()?;
                self.append(repo_name, obj);
                self.get_mut(repo_name, &id).unwrap()
            }
        };

        let activity = raw_event.parse_activity::<P>()?;
        github_obj.activities.push(activity);
        Ok(())
    }
}

#[derive(Debug)]
pub enum GithubObjectType {
    Issue,
    PullRequest,
    Commit,
}

impl GithubObjectType {
    pub fn value(&self) -> &str {
        match self {
            GithubObjectType::Issue => "Issue",
            GithubObjectType::PullRequest => "PR",
            GithubObjectType::Commit => "Commit",
        }
    }
}

#[derive(Debug)]
pub struct GithubObject {
    pub id: String,
    pub object_type: GithubObjectType,
    pub link: String,
    pub title: Option<String>,
    pub activities: Vec<Activity>,
}

impl GithubObject {
    pub fn new(id: &str, obj_type: GithubObjectType, link: &str, title: Option<&str>) -> GithubObject {
        GithubObject {
            id: String::from(id),
            object_type: obj_type,
            link: String::from(link),
            title: title.map(|t| t.to_owned()),
            activities: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Activity {
    pub action: String,
    pub body: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Activity {
    pub fn new(action: impl Into<String>, body: Option<&str>, created_at: &DateTime<Utc>) -> Activity {
        Activity {
            action: action.into(),
            body: body.map(|b| String::from(b)),
            created_at: created_at.to_owned(),
        }
    }
}
