use anyhow::Result;
use async_trait::async_trait;
use lolraft::process::*;
use redb::{Database, ReadableTable, TableDefinition};
use std::sync::Arc;

mod entry {
    use lolraft::process::Entry;
    pub fn ser(x: Entry) -> Vec<u8> {
        todo!()
    }
    pub fn desr(bin: &[u8]) -> Entry {
        todo!()
    }
}

struct LogStore {
    db: Arc<Database>,
    space: String,
}
impl LogStore {
    fn table_def(&self) -> TableDefinition<u64, Vec<u8>> {
        TableDefinition::new(&self.space)
    }
}
#[async_trait]
impl RaftLogStore for LogStore {
    async fn insert_entry(&self, i: Index, e: Entry) -> Result<()> {
        let tx = self.db.begin_write()?;
        {
            let mut tbl = tx.open_table(self.table_def())?;
            tbl.insert(i, entry::ser(e))?;
        }
        tx.commit()?;
        Ok(())
    }
    async fn delete_entries_before(&self, i: Index) -> Result<()> {
        let tx = self.db.begin_write()?;
        {
            let mut tbl = tx.open_table(self.table_def())?;
            tbl.retain(|k, _| k >= i)?;
        }
        tx.commit()?;
        Ok(())
    }
    async fn get_entry(&self, i: Index) -> Result<Option<Entry>> {
        let tx = self.db.begin_read()?;
        let tbl = tx.open_table(self.table_def())?;
        match tbl.get(i)? {
            Some(bin) => Ok(Some(entry::desr(&bin.value()))),
            None => Ok(None),
        }
    }
    async fn get_head_index(&self) -> Result<Index> {
        let tx = self.db.begin_read()?;
        let tbl = tx.open_table(self.table_def())?;
        let out = tbl.first()?;
        Ok(match out {
            Some((k, v)) => k.value(),
            None => 0,
        })
    }
    async fn get_last_index(&self) -> Result<Index> {
        let tx = self.db.begin_read()?;
        let tbl = tx.open_table(self.table_def())?;
        let out = tbl.last()?;
        Ok(match out {
            Some((k, v)) => k.value(),
            None => 0,
        })
    }
}

mod ballot {
    use lolraft::process::Ballot;
    pub fn ser(x: Ballot) -> Vec<u8> {
        todo!()
    }
    pub fn desr(bin: &[u8]) -> Ballot {
        todo!()
    }
}

struct BallotStore {
    db: Arc<Database>,
    space: String,
}
impl BallotStore {
    fn table_def(&self) -> TableDefinition<(), Vec<u8>> {
        TableDefinition::new(&self.space)
    }
}
#[async_trait]
impl RaftBallotStore for BallotStore {
    async fn save_ballot(&self, ballot: Ballot) -> Result<()> {
        let tx = self.db.begin_write()?;
        {
            let mut tbl = tx.open_table(self.table_def())?;
            tbl.insert((), ballot::ser(ballot))?;
        }
        tx.commit()?;
        Ok(())
    }
    async fn load_ballot(&self) -> Result<Ballot> {
        let tx = self.db.begin_read()?;
        let tbl = tx.open_table(self.table_def())?;
        match tbl.get(())? {
            Some(bin) => Ok(ballot::desr(&bin.value())),
            None => Err(anyhow::anyhow!("No ballot")),
        }
    }
}

pub fn new(db: redb::Database, lane_id: u32) -> (impl RaftLogStore, impl RaftBallotStore) {
    let db = Arc::new(db);
    let log = LogStore {
        space: format!("log-{lane_id}"),
        db: db.clone(),
    };
    let ballot = BallotStore {
        space: format!("ballot-{lane_id}"),
        db: db.clone(),
    };
    (log, ballot)
}
