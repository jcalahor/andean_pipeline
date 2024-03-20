
use std::error::Error;
use std::sync::Arc;
use scylla::frame::value::Timestamp;
use scylla::prepared_statement::PreparedStatement;
use scylla::statement::Consistency;
use scylla::load_balancing::DefaultPolicy;
use scylla::transport::ExecutionProfile;
use scylla::transport::retry_policy::DefaultRetryPolicy;
use scylla::transport::Compression;
use scylla::IntoTypedRows;
use scylla::{Session, SessionBuilder};
use std::process;
use tokio::sync::Semaphore;
use std::time::Duration;
use chrono::Utc;
use crate::tick::Tick;
use chrono::prelude::*;

const PARALLEL: usize = 2048; // Concurrency,
                              // let all shards work in parallel rather than hit a single
                              // shard at a time.

struct DB {
    session: Arc<Session>,
    ps: PreparedStatement,
    sem: Arc<Semaphore>
}

pub struct SyllaDBHandler {
    host: &'static str,
    dc: &'static str,
    usr: &'static str,
    pwd: &'static str,
    db: Option<DB>
}

impl  SyllaDBHandler {
    pub fn new() -> SyllaDBHandler {
        SyllaDBHandler {
            host: "172.19.0.4",
            dc: "datacenter1",
            usr: "scylla",
            pwd: "scylla",
            db: None
        }
    }

    pub async fn connect(&mut self) -> Result<(), Box<dyn Error>> {
        println!("Connecting to {} ...", self.host);
        let default_policy = DefaultPolicy::builder()
            .prefer_datacenter(self.dc.to_string())
            .token_aware(true)
            .permit_dc_failover(false)
            .build();
    
        let profile = ExecutionProfile::builder()
            .load_balancing_policy(default_policy)
            .build();
    
        let handle = profile.into_handle();
    
        let raw_session: Session = SessionBuilder::new()
            .known_node(self.host)
            .default_execution_profile_handle(handle)
            .compression(Some(Compression::Lz4))
            .user(self.usr, self.pwd)
            .build()
            .await?;

        let session = Arc::new(raw_session);


        println!("!!!!!Connected successfully! Policy: TokenAware(DCAware())");

        let ks = "trading";
        let table = "market_data";

            // Create KS and Table
        let ks_stmt = format!("CREATE KEYSPACE IF NOT EXISTS {} WITH replication = {{'class': 'NetworkTopologyStrategy', '{}': 1}}", ks, self.dc);
        session.query(ks_stmt, &[]).await?;

        let cf_stmt = format!("CREATE TABLE IF NOT EXISTS {}.{} (symbol text, ts timestamp, price float, PRIMARY KEY(symbol, ts)) 
                            WITH default_time_to_live = 2592000 
                            AND compaction = {{'class': 'TimeWindowCompactionStrategy', 'compaction_window_size': 3}}", ks, table);
        session.query(cf_stmt, &[]).await?;

        println!("Keyspace and Table processing is complete");

        // Check for Schema Agreement
        if session
            .await_timed_schema_agreement(Duration::from_secs(5))
            .await?
        {
            println!("Schema is in agreement - Proceeding");
        } else {
            println!("Schema is NOT in agreement - Stop processing");
            process::exit(1);
        }

        // Prepare Statement - use LocalQuorum
        // Always use Prepared Statements whenever possible
        // Prepared Statements are a requirement for TokenAware load balancing
        let stmt = format!(
            "INSERT INTO {}.{} (symbol, ts, price) VALUES (?, ?, ?)",
            ks, table
        );
        let mut ps: PreparedStatement = session.prepare(stmt).await?;
        // LocalQuorum means a majority of replicas need to acknowledge the operation
        // for it to be considered successful
        ps.set_consistency(Consistency::LocalQuorum);

        // Retry policy - the default when not specified
        // Another option would be using 'FalthroughRetryPolicy', which effectively never retries
        // Similarly as the loading balancing policy, it is also possible to implement your own retry policy
        ps.set_retry_policy(Some(Arc::new(DefaultRetryPolicy::new())));

        println!();

        // 1. Spawn a new semaphore
        // 2. Start ingestion
        let sem: Arc<Semaphore> = Arc::new(Semaphore::new(PARALLEL));

        self.db = Some(DB {
            session: session,
            ps: ps,
            sem: sem
        });
    
        Ok(())
     }


     pub async fn insert_tick(&mut self, tick: Tick) -> Result<(), Box<dyn Error>> {

        let session = self.db.as_mut().unwrap().session.clone();
        let ps = self.db.as_mut().unwrap().ps.clone();
        let permit = self.db.as_mut().unwrap().sem.clone().acquire_owned().await;       
        let dt = DateTime::from_timestamp(tick.TimeStamp, 0);
    
        
        tokio::task::spawn(async move {            
            if let Err(_) = session
                .execute(&ps, (&tick.Symbol, dt, &tick.Price))
                .await {
                    println!("error storing tick: {} {}", &tick.Symbol, &tick.TimeStamp)
                }

            let _permit = permit;
        });

        Ok(())
     }





}

