## Bup: A Background Backup Scheduler

Bup is a simple background backup scheduler designed for backing up services like postgres, redis.

It periodically backs up specified services and dumps the files into the designated folder with a retention period mentioned in configuration.

### Features
- Periodic backups of services
- Configurable retention period
- Simple configuration file
- Services Specfic backup options like formats, methods etc
- Zips the backup files for storage efficiency
- Proper logging of backup operations


### Usage

- Handfull use for someone who runs services locally using docker and needs to have a simple backup solution.

1. Clone and build the crate:
    ```bash
    git clone https://github.com/shawakash/bup
    cd bup

    cargo build --release
    ```

2. Create a configuration file say `bup.toml` in the root directory with the following structure:
    ```toml
        [common]
        backup_dir = "./backup"
        log_level = "info"
        log_dir = "./logs"
        retention_days = 7

        [[services]]
        type = "postgres"
        alias = "main-db"

        [services.schedule]
        interval_seconds = 3600
        timezone = "UTC"
        start_time = "02:00"

        [services.connection]
        service_type = "postgres"
        host = "localhost"
        port = 5432
        username = "akira"
        password = "${LOTTO_DB_PASSWORD}"
        database = "akira"

        [services.backup_options]
        format = "plain"
        schema_only = "false"
        data_only = "false"
    ```

    Include as many services as needed in the configuration file.

3. Run the application:
    ```bash
    cargo run --release --prefix bup --config ./bup.toml
    ```


### Extensions

- Create your own submodule for a specific service with their specific connection config.
- Implement the `BackupService` trait for your service.
- Add more services by extending the `services` section in the configuration file.


### Restoration

Some examples of Service restoration:

- For Postgres, you can use the `pg_restore` command to restore from the backup files.

```bash
docker exec -i postgres psql -U akira -d akira < /path/to/backup/file.sql
```

- For Redis, you can use the `redis-cli` command to restore from the backup files.
  We need to first stop the redis service, then copy the backup file to the redis data directory and start the service again.

```bash
docker stop redis
docker cp backup.rdb redis:/data/dump.rdb
docker start redis
```
