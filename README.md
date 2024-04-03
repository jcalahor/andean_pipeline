# Andean Pipeline

Ingest data comming from a Kafka Topic and push (at high speed) it to ScyllaBD using the Tokio Crate

## Architecture


![andean_pipelin](https://github.com/jcalahor/andean_pipeline/assets/7434088/bb81a4be-55e7-4383-a2cb-d69c6058146c)


As the diagram shows the data is produced by the Data Pusher, the type of data is Pricing info of few stocks. 

The pusher runs on it's own docker container.

The Kafka component  also run in a separate container, not much to talk here.

The pipeline worker is a Rust application that runs in other container that contains the logic to consume the data comming from Kafka and then forwarding this data to an insertor module which will insert it to the ScyllaBD

By taking advantage of the asynch capabilities of the Tokio crate the communication between the consumer and the insertor tasks are relative trivial.

For more info on Tokio: https://tokio.rs/ 

For more info on ScyllaDB: https://www.scylladb.com/

## Data Format

At the moment the format supported (that travels in the wire) is standard JSON, however for long term better options can be considered like Protobuf



## Run Locally

Clone the project

```bash
  git clone https://github.com/jcalahor/andean_pipeline.git
```

Go to the project directory

```bash
  cd andean_pipeline
```

Build the development enviroment

```bash
  docker compose build --no-cache
```

Start the containers

```bash
  docker compose up -d
```

Double check if all containers are up, if not re run docker compose up, sometimes kafka won't start right away. This is a known bug that will need to be researched

```bash
  docker ps
CONTAINER ID   IMAGE                             COMMAND                  CREATED       STATUS                        PORTS                                                                   NAMES
2f486bd317be   andean_pipeline-andean_producer   "python3"                6 days ago    Up 2 minutes                                                                                          andean_producer
c69d0e371607   andean_pipeline-andean_pipeline   "bash"                   6 days ago    Up 2 minutes                                                                                          andean_pipeline
19dd335b5754   scylladb/scylla:5.2.2             "/docker-entrypoint.…"   12 days ago   Up About a minute (healthy)   22/tcp, 7000-7001/tcp, 9042/tcp, 9160/tcp, 9180/tcp, 10000/tcp          scylladb-02
4fc004e2e8ee   scylladb/scylla:5.2.2             "/docker-entrypoint.…"   12 days ago   Up 2 minutes (healthy)        22/tcp, 7000-7001/tcp, 9042/tcp, 9160/tcp, 9180/tcp, 10000/tcp          scylladb-01
062d38d55c5e   wurstmeister/kafka                "start-kafka.sh"         3 weeks ago   Up 48 seconds                 0.0.0.0:9092->9092/tcp, :::9092->9092/tcp                               kafka
e1994feff9a1   wurstmeister/zookeeper            "/bin/sh -c '/usr/sb…"   3 weeks ago   Up 2 minutes                  22/tcp, 2888/tcp, 3888/tcp, 0.0.0.0:2181->2181/tcp, :::2181->2181/tcp   zookeeper
```

Launch the kafka consumer. When running the first time, the program will be built as 'cargo run' will compile the porject first

```bash
  $ docker exec -it andean_pipeline bash
  root@andean_pipeline:/usr/src/andean_pipeline# cargo run --bin ingestor
      Finished dev [unoptimized + debuginfo] target(s) in 1.15s
      Running `target/debug/ingestor`
  Connecting to 172.19.0.4 ...
  !!!!!Connected successfully! Policy: TokenAware(DCAware())
  Keyspace and Table processing is complete
  Schema is in agreement - Proceeding

  consuming...

```

Enter into the producer terminal and run it
```bash

  $ docker exec -it andean_producer bash
  root@andean_producer:/usr/src/andean_producer# python producer.py 
  0
  2000
  4000
  root@andean_producer:/usr/src/andean_producer# 
```

Explore the results in ScyllaDB
```bash
  $docker exec -it scylladb-01 cqlsh

  Warning: cqlshrc config files were found at both the old location (/root/.cqlshrc) and                 the new location (/root/.cassandra/cqlshrc), the old config file will not be migrated to the new                 location, and the new location will be used for now.  You should manually                 consolidate the config files at the new location and remove the old file.
  Connected to  at 172.19.0.4:9042.
  [cqlsh 5.0.1 | Cassandra 3.0.8 | CQL spec 3.3.1 | Native protocol v4]
  Use HELP for help.
  cqlsh> select count(*) from trading.market_data;

  count
  -------
  89944

  (1 rows)
  cqlsh> 

```


## Next Steps

The present project is still on experimental mode and the following items are still pending to be completed
- Investigate why some records are being lost not all 90K are being ingested
- Improve log support, log in a flat flat not just print!
- Current producer in Python is very slow we need to build another one in Rust
- Test in multicore environments as right now the system has only been tested in 1 single core
- Support of alternate protocol formats for serialization like Protobuf



## Contributions
 Contributions are not only welcome but encourage :)
 
