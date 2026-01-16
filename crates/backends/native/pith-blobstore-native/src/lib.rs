//! Native blob storage implementation.

use rhizome_pith_blobstore::{BlobStore, Container, Error, ObjectMeta};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// In-memory blob storage.
#[derive(Debug, Default)]
pub struct MemoryBlobStore {
    containers: RwLock<HashMap<String, Arc<MemoryContainer>>>,
}

impl MemoryBlobStore {
    /// Create a new empty blob store.
    pub fn new() -> Self {
        Self {
            containers: RwLock::new(HashMap::new()),
        }
    }
}

impl BlobStore for MemoryBlobStore {
    type Container = MemoryContainer;

    async fn create_container(&self, name: &str) -> Result<(), Error> {
        let mut containers = self
            .containers
            .write()
            .map_err(|e| Error::Store(e.to_string()))?;
        if containers.contains_key(name) {
            return Err(Error::ContainerExists(name.to_string()));
        }
        containers.insert(name.to_string(), Arc::new(MemoryContainer::new()));
        Ok(())
    }

    async fn delete_container(&self, name: &str) -> Result<(), Error> {
        let mut containers = self
            .containers
            .write()
            .map_err(|e| Error::Store(e.to_string()))?;
        containers
            .remove(name)
            .ok_or_else(|| Error::ContainerNotFound(name.to_string()))?;
        Ok(())
    }

    async fn container(&self, name: &str) -> Result<Self::Container, Error> {
        let containers = self
            .containers
            .read()
            .map_err(|e| Error::Store(e.to_string()))?;
        containers
            .get(name)
            .map(|c| MemoryContainer {
                objects: c.objects.clone(),
            })
            .ok_or_else(|| Error::ContainerNotFound(name.to_string()))
    }

    async fn container_exists(&self, name: &str) -> Result<bool, Error> {
        let containers = self
            .containers
            .read()
            .map_err(|e| Error::Store(e.to_string()))?;
        Ok(containers.contains_key(name))
    }

    async fn list_containers(&self) -> Result<Vec<String>, Error> {
        let containers = self
            .containers
            .read()
            .map_err(|e| Error::Store(e.to_string()))?;
        Ok(containers.keys().cloned().collect())
    }
}

/// Object data with metadata.
#[derive(Debug, Clone)]
struct StoredObject {
    data: Vec<u8>,
    created_at: u64,
}

/// In-memory container.
#[derive(Debug, Default)]
pub struct MemoryContainer {
    objects: Arc<RwLock<HashMap<String, StoredObject>>>,
}

impl MemoryContainer {
    fn new() -> Self {
        Self {
            objects: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }
}

impl Container for MemoryContainer {
    async fn get(&self, name: &str) -> Result<Vec<u8>, Error> {
        let objects = self
            .objects
            .read()
            .map_err(|e| Error::Store(e.to_string()))?;
        objects
            .get(name)
            .map(|o| o.data.clone())
            .ok_or_else(|| Error::ObjectNotFound(name.to_string()))
    }

    async fn put(&self, name: &str, data: &[u8]) -> Result<(), Error> {
        let mut objects = self
            .objects
            .write()
            .map_err(|e| Error::Store(e.to_string()))?;
        objects.insert(
            name.to_string(),
            StoredObject {
                data: data.to_vec(),
                created_at: Self::now(),
            },
        );
        Ok(())
    }

    async fn delete(&self, name: &str) -> Result<(), Error> {
        let mut objects = self
            .objects
            .write()
            .map_err(|e| Error::Store(e.to_string()))?;
        objects
            .remove(name)
            .ok_or_else(|| Error::ObjectNotFound(name.to_string()))?;
        Ok(())
    }

    async fn exists(&self, name: &str) -> Result<bool, Error> {
        let objects = self
            .objects
            .read()
            .map_err(|e| Error::Store(e.to_string()))?;
        Ok(objects.contains_key(name))
    }

    async fn list(&self) -> Result<Vec<ObjectMeta>, Error> {
        let objects = self
            .objects
            .read()
            .map_err(|e| Error::Store(e.to_string()))?;
        Ok(objects
            .iter()
            .map(|(name, obj)| ObjectMeta {
                name: name.clone(),
                size: obj.data.len() as u64,
                created_at: Some(obj.created_at),
            })
            .collect())
    }

    async fn metadata(&self, name: &str) -> Result<ObjectMeta, Error> {
        let objects = self
            .objects
            .read()
            .map_err(|e| Error::Store(e.to_string()))?;
        objects
            .get(name)
            .map(|obj| ObjectMeta {
                name: name.to_string(),
                size: obj.data.len() as u64,
                created_at: Some(obj.created_at),
            })
            .ok_or_else(|| Error::ObjectNotFound(name.to_string()))
    }

    async fn copy(&self, src: &str, dst: &str) -> Result<(), Error> {
        let mut objects = self
            .objects
            .write()
            .map_err(|e| Error::Store(e.to_string()))?;
        let src_obj = objects
            .get(src)
            .ok_or_else(|| Error::ObjectNotFound(src.to_string()))?
            .clone();
        objects.insert(
            dst.to_string(),
            StoredObject {
                data: src_obj.data,
                created_at: Self::now(),
            },
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn container_lifecycle() {
        let store = MemoryBlobStore::new();

        store.create_container("test").await.unwrap();
        assert!(store.container_exists("test").await.unwrap());

        let names = store.list_containers().await.unwrap();
        assert_eq!(names.len(), 1);

        store.delete_container("test").await.unwrap();
        assert!(!store.container_exists("test").await.unwrap());
    }

    #[tokio::test]
    async fn object_operations() {
        let store = MemoryBlobStore::new();
        store.create_container("bucket").await.unwrap();
        let container = store.container("bucket").await.unwrap();

        container.put("file.txt", b"hello world").await.unwrap();
        assert!(container.exists("file.txt").await.unwrap());
        assert_eq!(container.get("file.txt").await.unwrap(), b"hello world");

        let meta = container.metadata("file.txt").await.unwrap();
        assert_eq!(meta.size, 11);

        container.delete("file.txt").await.unwrap();
        assert!(!container.exists("file.txt").await.unwrap());
    }

    #[tokio::test]
    async fn list_and_copy() {
        let store = MemoryBlobStore::new();
        store.create_container("bucket").await.unwrap();
        let container = store.container("bucket").await.unwrap();

        container.put("a.txt", b"aaa").await.unwrap();
        container.put("b.txt", b"bbb").await.unwrap();

        let objects = container.list().await.unwrap();
        assert_eq!(objects.len(), 2);

        container.copy("a.txt", "c.txt").await.unwrap();
        assert_eq!(container.get("c.txt").await.unwrap(), b"aaa");
    }
}
