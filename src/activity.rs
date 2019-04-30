use std::collections::HashMap;

use chrono::{DateTime, Utc};

pub struct GithubActivities {
    pub activity_objects: HashMap<String, HashMap<String, GithubObject>>
}

impl GithubActivities {
    pub fn new() -> GithubActivities {
        GithubActivities {
            activity_objects: HashMap::new()
        }
    }

    pub fn get_mut(&mut self, repo_name: &str, object_id: &str) -> Option<&mut GithubObject> {
        self.activity_objects
            .entry(String::from(repo_name))
            .or_insert(HashMap::new())
            .get_mut(object_id)
    }

    pub fn append(&mut self, repo_name: &str, obj: GithubObject) -> &mut GithubObject {
        let id = obj.id.to_owned();
        self.activity_objects
            .entry(String::from(repo_name))
            .or_insert(HashMap::new())
            .insert(String::from(id.as_str()), obj);

        self.activity_objects
            .get_mut(repo_name)
            .unwrap()
            .get_mut(&id)
            .unwrap()
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
