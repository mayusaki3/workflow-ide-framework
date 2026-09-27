use crate::{
    project::ProjectContext,
    project_io::atomic_write,
    project_resource::{ApplicationJournalData, ResourceReference, ResourceScope},
    resource_operation::{ResourceOperationKind, ResourceOperationPlan},
};
use serde::{Deserialize,Serialize};
use std::{fs,io,path::PathBuf};

pub const RESOURCE_JOURNAL_FORMAT_VERSION:u32=1;

#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
pub struct ResourceOperationJournal {
    pub format_version:u32,
    pub operation_id:String,
    pub kind:StoredOperationKind,
    pub items:Vec<StoredJournalItem>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub application:Option<StoredApplicationJournalData>,
}
#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="lowercase")]
pub enum StoredOperationKind{Import,Rename,Move,Delete,Replace}
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
pub struct StoredJournalItem{
    #[serde(default,skip_serializing_if="Option::is_none")] pub before:Option<StoredReference>,
    #[serde(default,skip_serializing_if="Option::is_none")] pub after:Option<StoredReference>,
}
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
pub struct StoredReference{pub scope:String,pub path:PathBuf}
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
pub struct StoredApplicationJournalData{pub format:String,pub data:Vec<u8>}

impl ResourceOperationJournal{
    pub fn from_plan(plan:&ResourceOperationPlan,application:Option<ApplicationJournalData>)->Self{
        Self{format_version:RESOURCE_JOURNAL_FORMAT_VERSION,operation_id:plan.operation_id.clone(),kind:plan.kind.into(),
            items:plan.items.iter().map(|i|StoredJournalItem{before:i.before.as_ref().map(Into::into),after:i.after.as_ref().map(Into::into)}).collect(),
            application:application.map(|a|StoredApplicationJournalData{format:a.format,data:a.data})}
    }
    pub fn save(&self,context:&ProjectContext)->io::Result<()>{
        let text=toml::to_string_pretty(self).map_err(io::Error::other)?;
        atomic_write(&journal_path(context),text.as_bytes())
    }
    pub fn load(context:&ProjectContext)->io::Result<Option<Self>>{
        let path=journal_path(context); if !path.exists(){return Ok(None);}
        let text=fs::read_to_string(path)?;
        let journal:Self=toml::from_str(&text).map_err(io::Error::other)?;
        if journal.format_version!=RESOURCE_JOURNAL_FORMAT_VERSION{return Err(io::Error::new(io::ErrorKind::InvalidData,"unsupported resource journal format"));}
        Ok(Some(journal))
    }
    pub fn remove(context:&ProjectContext)->io::Result<()>{
        let path=journal_path(context); if path.exists(){fs::remove_file(path)?;} Ok(())
    }
}
impl From<ResourceOperationKind> for StoredOperationKind{fn from(v:ResourceOperationKind)->Self{match v{ResourceOperationKind::Import=>Self::Import,ResourceOperationKind::Rename=>Self::Rename,ResourceOperationKind::Move=>Self::Move,ResourceOperationKind::Delete=>Self::Delete,ResourceOperationKind::Replace=>Self::Replace}}}
impl From<&ResourceReference> for StoredReference{fn from(v:&ResourceReference)->Self{Self{scope:match v.scope{ResourceScope::Application=>"application",ResourceScope::Project=>"project",ResourceScope::External=>"external"}.into(),path:v.path.clone()}}}
pub fn journal_path(context:&ProjectContext)->PathBuf{context.framework_directory().join("pending_resource_operation.toml")}
