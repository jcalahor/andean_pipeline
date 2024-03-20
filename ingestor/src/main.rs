use rdkafka::{
    consumer::{BaseConsumer, Consumer},
    ClientConfig, ClientContext, Message,
};
use std::error::Error;
use std::{thread, time::Duration};
use std::str;
use tokio::sync::mpsc;
mod scylladb_db;
mod tick;
use scylladb_db::{SyllaDBHandler};
use tick::Tick;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let mut db_handler = SyllaDBHandler::new();
    db_handler.connect().await;

    let (tx, mut rx) = mpsc::channel(100);

    let consumer: BaseConsumer = ClientConfig::new()
    .set("bootstrap.servers", "kafka:9092")
    //for auth
    /*.set("security.protocol", "SASL_SSL")
    .set("sasl.mechanisms", "PLAIN")
    .set("sasl.username", "<update>")
    .set("sasl.password", "<update>")*/
    .set("group.id", "andean-group")
    .create()
    .expect("invalid consumer config");

    consumer
    .subscribe(&["input_readings"])
    .expect("topic subscribe failed");

    tokio::task::spawn(async move {
        loop {
            println!("consuming...");
            for msg_result in consumer.iter() {
                        let msg = msg_result.unwrap();
                        //let key: &str = msg.key_view().unwrap().unwrap();
                        let value = msg.payload().unwrap();
                        let buff = str::from_utf8(&value).unwrap();
                        
                        let tick: Tick = serde_json::from_slice(&value).expect("failed to deser JSON to Tick");
                        println!(
                            "{:?} value {:?} in offset {:?} from partition {}",
                            tick,
                            buff,
                            msg.offset(),
                            msg.partition()
                        );
                        
                        if let Err(_) = tx.send(tick).await {
                            println!("receiver dropped when sending for insertion {:?}", buff);                         
                        }
                    }
            }
        }
    );

    while let Some(res) = rx.recv().await {
        println!("got = {:?}", res);
        db_handler.insert_tick(res).await?;
    }
    
    Ok(())
}