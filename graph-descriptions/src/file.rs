use error::Error;
use uuid::Uuid;
use graph_description::File;
use serde_json::Value;
use std::convert::TryFrom;
use node::NodeT;

#[derive(Debug, Clone)]
pub enum FileState {
    Created,
    Deleted,
    Existing,
}

impl TryFrom<u32> for FileState {
    type Error = Error;

    fn try_from(i: u32) -> Result<FileState, Error> {
        match i {
            1 => Ok(FileState::Created),
            2 => Ok(FileState::Deleted),
            3 => Ok(FileState::Existing),
            _ => Err(Error::InvalidProcessState(i))
        }
    }
}

impl From<FileState> for u32 {
    fn from(p: FileState) -> u32 {
        match p {
            FileState::Created => 1,
            FileState::Deleted => 2,
            FileState::Existing => 3,
        }
    }
}


impl File {
    pub fn new(asset_id: impl Into<Option<String>>,
               hostname: impl Into<Option<String>>,
               state: FileState,
               timestamp: u64,
               file_name: String,
               file_path: String,
               file_extension: String,
               file_mime_type: String,
               file_size: u64,
               file_version: String,
               file_description: String,
               file_product: String,
               file_company: String,
               file_directory: String,
               file_inode: u64,
               file_hard_links: u64,
               md5_hash: String,
               sha1_hash: String,
               sha256_hash: String,
    ) -> File {
        let asset_id = asset_id.into();
        let hostname = hostname.into();

        if asset_id.is_none() && hostname.is_none() {
            panic!("AssetID or Hostname must be provided for ProcessOutboundConnection");
        }

        let mut fd = File {
            node_key: Uuid::new_v4().to_string(),
            asset_id: asset_id.into(),
            hostname: hostname.into(),
            state: state.clone().into(),
            created_timestamp: 0,
            deleted_timestamp: 0,
            last_seen_timestamp: 0,
            file_name,
            file_path,
            file_extension,
            file_mime_type,
            file_size,
            file_version,
            file_description,
            file_product,
            file_company,
            file_directory,
            file_inode,
            file_hard_links,
            md5_hash,
            sha1_hash,
            sha256_hash,
        };

        match state {
            FileState::Created => fd.created_timestamp = timestamp,
            FileState::Existing => fd.last_seen_timestamp = timestamp,
            FileState::Deleted => fd.deleted_timestamp = timestamp,
        }

        fd
    }

    pub fn into_json(self) -> Value {
        let asset_id = self.asset_id.as_ref().unwrap();
        let mut j = json!({
            "node_key": self.node_key,
            "asset_id": asset_id,
            "dgraph.type": "File"
        });

        if !self.file_name.is_empty() {
            j["file_name"] = Value::from(self.file_name);
        }

        if !self.file_path.is_empty() {
            j["file_path"] = Value::from(self.file_path);
        }

        if !self.file_extension.is_empty() {
            j["file_extension"] = Value::from(self.file_extension);
        }

        if !self.file_mime_type.is_empty() {
            j["file_mime_type"] = Value::from(self.file_mime_type);
        }

        if self.file_size != 0 {
            j["file_size"] = Value::from(self.file_size);
        }

        if !self.file_version.is_empty() {
            j["file_version"] = Value::from(self.file_version);
        }

        if !self.file_description.is_empty() {
            j["file_description"] = Value::from(self.file_description);
        }

        if !self.file_product.is_empty() {
            j["file_product"] = Value::from(self.file_product);
        }

        if !self.file_company.is_empty() {
            j["file_company"] = Value::from(self.file_company);
        }

        if !self.file_directory.is_empty() {
            j["file_directory"] = Value::from(self.file_directory);
        }

        if self.file_inode != 0 {
            j["file_inode"] = Value::from(self.file_inode);
        }

        if self.file_hard_links != 0 {
            j["file_hard_links"] = Value::from(self.file_hard_links);
        }

        if !self.md5_hash.is_empty() {
            j["md5_hash"] = Value::from(self.md5_hash);
        }

        if !self.sha1_hash.is_empty() {
            j["sha1_hash"] = Value::from(self.sha1_hash);
        }

        if !self.sha256_hash.is_empty() {
            j["sha256_hash"] = Value::from(self.sha256_hash);
        }

        if self.created_timestamp != 0 {
            j["created_time"] = self.created_timestamp.into()
        }

        if self.deleted_timestamp != 0 {
            j["deleted_timestamp"] = self.deleted_timestamp.into()
        }
        if self.last_seen_timestamp != 0 {
            j["last_seen_timestamp"] = self.last_seen_timestamp.into()
        }

        j
    }

    pub fn timestamp(&self) -> u64 {
        match FileState::try_from(self.state).unwrap() {
            FileState::Created => self.created_timestamp,
            FileState::Deleted => self.deleted_timestamp,
            FileState::Existing => self.last_seen_timestamp,
        }
    }
}

impl NodeT for File {
    fn get_asset_id(&self) -> Option<&str> {
        self.asset_id.as_ref().map(String::as_str)
    }

    fn set_asset_id(&mut self, asset_id: impl Into<String>) {
        self.asset_id = Some(asset_id.into());
    }

    fn get_node_key(&self) -> &str {
        self.node_key.as_str()
    }

    fn set_node_key(&mut self, node_key: impl Into<String>) {
        self.node_key = node_key.into()
    }

    fn merge(&mut self, other: &Self) -> bool {
        if self.node_key != other.node_key {
            warn!("Attempted to merge two file nodes with different keys. Dropping merge.");
            return false;
        }

        let mut merged = false;

        if self.asset_id.is_none() && other.asset_id.is_some() {
            merged = true;
            self.asset_id = other.asset_id.clone();
        }

        if self.hostname.is_none() && other.hostname.is_some() {
            merged = true;
            self.hostname = other.hostname.clone();
        }

        if self.file_name.is_empty() && !other.file_name.is_empty() {
            merged = true;
            self.file_name = other.file_name.clone();
        }

        if self.file_path.is_empty() && !other.file_path.is_empty() {
            merged = true;
            self.file_path = other.file_path.clone();
        }

        if self.file_extension.is_empty() && !other.file_extension.is_empty() {
            merged = true;
            self.file_extension = other.file_extension.clone();
        }

        if self.file_mime_type.is_empty() && !other.file_mime_type.is_empty() {
            merged = true;
            self.file_mime_type = other.file_mime_type.clone();
        }

        if self.file_size == 0 && other.file_size != 0 {
            merged = true;
            self.file_size = other.file_size;
        }

        if self.file_version.is_empty() && !other.file_version.is_empty() {
            merged = true;
            self.file_version = other.file_version.clone();
        }

        if self.file_description.is_empty() && !other.file_description.is_empty() {
            merged = true;
            self.file_description = other.file_description.clone();
        }

        if self.file_product.is_empty() && !other.file_product.is_empty() {
            merged = true;
            self.file_product = other.file_product.clone();
        }

        if self.file_company.is_empty() && !other.file_company.is_empty() {
            merged = true;
            self.file_company = other.file_company.clone();
        }

        if self.file_directory.is_empty() && !other.file_directory.is_empty() {
            merged = true;
            self.file_directory = other.file_directory.clone();
        }

        if self.file_inode == 0 && !other.file_inode != 0 {
            merged = true;
            self.file_inode = other.file_inode;
        }

        if self.file_hard_links == 0 && !other.file_hard_links != 0 {
            merged = true;
            self.file_hard_links = other.file_hard_links;
        }

        if self.md5_hash.is_empty() && !other.md5_hash.is_empty() {
            merged = true;
            self.md5_hash = other.md5_hash.clone();
        }

        if self.sha1_hash.is_empty() && !other.sha1_hash.is_empty() {
            merged = true;
            self.sha1_hash = other.sha1_hash.clone();
        }

        if self.sha256_hash.is_empty() && !other.sha256_hash.is_empty() {
            merged = true;
            self.sha256_hash = other.sha256_hash.clone();
        }

        if self.created_timestamp == 0 {
            merged = true;
            self.created_timestamp = other.created_timestamp;
        }
        if self.deleted_timestamp == 0 {
            merged = true;
            self.deleted_timestamp = other.deleted_timestamp;
        }
        if self.last_seen_timestamp == 0 {
            merged = true;
            self.last_seen_timestamp = other.last_seen_timestamp;
        }

        merged
    }

    fn merge_into(&mut self, other: Self) -> bool {
        if self.node_key != other.node_key {
            warn!("Attempted to merge two file nodes with different keys. Dropping merge.");
            return false;
        }

        let mut merged = false;

        if self.asset_id.is_none() && other.asset_id.is_some() {
            merged = true;
            self.asset_id = other.asset_id;
        }

        if self.hostname.is_none() && other.hostname.is_some() {
            merged = true;
            self.hostname = other.hostname;
        }

        if self.file_name.is_empty() && !other.file_name.is_empty() {
            merged = true;
            self.file_name = other.file_name;
        }

        if self.file_path.is_empty() && !other.file_path.is_empty() {
            merged = true;
            self.file_path = other.file_path;
        }

        if self.file_extension.is_empty() && !other.file_extension.is_empty() {
            merged = true;
            self.file_extension = other.file_extension;
        }

        if self.file_mime_type.is_empty() && !other.file_mime_type.is_empty() {
            merged = true;
            self.file_mime_type = other.file_mime_type;
        }

        if self.file_size == 0 && other.file_size != 0 {
            merged = true;
            self.file_size = other.file_size;
        }

        if self.file_version.is_empty() && !other.file_version.is_empty() {
            merged = true;
            self.file_version = other.file_version;
        }

        if self.file_description.is_empty() && !other.file_description.is_empty() {
            merged = true;
            self.file_description = other.file_description;
        }

        if self.file_product.is_empty() && !other.file_product.is_empty() {
            merged = true;
            self.file_product = other.file_product;
        }

        if self.file_company.is_empty() && !other.file_company.is_empty() {
            merged = true;
            self.file_company = other.file_company;
        }

        if self.file_directory.is_empty() && !other.file_directory.is_empty() {
            merged = true;
            self.file_directory = other.file_directory;
        }

        if self.file_inode == 0 && !other.file_inode != 0 {
            merged = true;
            self.file_inode = other.file_inode;
        }

        if self.file_hard_links == 0 && !other.file_hard_links != 0 {
            merged = true;
            self.file_hard_links = other.file_hard_links;
        }

        if self.md5_hash.is_empty() && !other.md5_hash.is_empty() {
            merged = true;
            self.md5_hash = other.md5_hash;
        }

        if self.sha1_hash.is_empty() && !other.sha1_hash.is_empty() {
            merged = true;
            self.sha1_hash = other.sha1_hash;
        }

        if self.sha256_hash.is_empty() && !other.sha256_hash.is_empty() {
            merged = true;
            self.sha256_hash = other.sha256_hash;
        }

        if self.created_timestamp == 0 {
            merged = true;
            self.created_timestamp = other.created_timestamp;
        }
        if self.deleted_timestamp == 0 {
            merged = true;
            self.deleted_timestamp = other.deleted_timestamp;
        }
        if self.last_seen_timestamp == 0 {
            merged = true;
            self.last_seen_timestamp = other.last_seen_timestamp;
        }

        merged
    }
}