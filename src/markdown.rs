use std::io::Write;
use failure::Error;
use crate::activity::{GithubActivities, GithubObject, Activity};

const MAX_BODY_LENGTH: &'static usize = &30;
const INDENT: &'static [u8] = b"  ";

pub trait Markdown {
    fn write_markdown<W: Write>(&self, writer: &mut W) -> Result<(), Error>;
}

impl Markdown for GithubActivities {
    fn write_markdown<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        for (repo_name, obj_map) in self.activity_objects.iter() {
            let line = format!("## {}\n", repo_name);
            writer.write_all(line.as_bytes())?;

            for obj in obj_map.values() {
                obj.write_markdown(writer)?;
            }
        }
        Ok(())
    }
}

impl Markdown for GithubObject {
    fn write_markdown<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        let title = match &self.title {
            Some(t) => t,
            None => "",
        };

        let line = format!("- [{} [{}]({})] {}\n",
                           self.object_type.value(),
                           self.id,
                           self.link,
                           title);
        writer.write_all(line.as_bytes())?;

        let mut activities = self.activities.to_vec();
        activities.sort_by_key(|a| a.created_at);
        for activity in activities.iter() {
            writer.write_all(INDENT)?;
            activity.write_markdown(writer)?;
        }
        Ok(())
    }
}

impl Markdown for Activity {
    fn write_markdown<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_all(b"- ")?;
        writer.write_all(self.action.as_bytes())?;
        if let Some(b) = &self.body {
            let body = if b.chars().count() <= *MAX_BODY_LENGTH {
                String::from(b.as_ref())
            } else {
                let b_: String = b.chars().take(*MAX_BODY_LENGTH).collect();
                format!("{}...", b_)
            };
            writer.write_all(b": ")?;
            writer.write_all(body.as_bytes())?;
        }
        writer.write_all(b"\n")?;
        Ok(())
    }
}
