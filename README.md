> :warning: **This is a snapshot repository for peer review, it does not have the complete history of the project.** 
> 
> The actual project is managed in a private GitLab repository.

# Koru

Koru is an API to manage long-term shared expenses.
The frontend mobile application can be found at [koru_app](https://github.com/Yneluki/koru_app)

## Development

### Basics

The project requires a rust development environment, which can easily be installed using [rustup](https://rustup.rs/)

It also uses git hooks to enforce proper formatting (`cargo fmt`) and usage of [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/#summary)
These can be installed by running:

```shell
./scripts/git-hooks.sh
```

The CI/CD pipeline will fail on incorrect formatting, so it is highly recommended to install the hooks.

### Feature Flags & Requirements

The application supports optional backends that can be enabled through feature flags.

- `postgres`: use PostgreSQL as data store
- `redis-bus`: use Redis as event bus
- `redis-session`: use Redis for session management
- `redis`: enables both `redis-bus` and `redis-session`
- `pushy`: use Pushy for notification (does not do anything by itself, it needs `notification`)
- `notification`: enables notification sending (currently directly enables `pushy` since it is the only implementation)
- `openapi`: enables Swagger UI endpoint & OpenApi docs (available at `/swagger-ui/`)
- `development`: enables `postgres`, `redis`, `notification` and `openapi`
- `production`: enables `postgres`, `redis` and `notification`

By default, all features are disabled and the app will run using an in memory data store, event bus and session store.
Even though they work, it is not recommended to use the in memory implementations, the `production` feature should be preferred for production use.

When running commands locally with the `production` feature enabled, Postgres & Redis will be required, therefore you will need to ensure additional tools are available on your system:

- [Docker](https://www.docker.com/)
- [SQLx CLI](https://crates.io/crates/sqlx-cli) (`cargo install sqlx-cli`)

You can then run local Postgres & Redis containers in the background in order to be able to compile, test and run the application.
```shell
# Start Postgres & Redis containers, apply SQLx migrations
./scripts/init-db.sh
```

Note: a cleanup script is also available `./scripts/stop-db.sh`

### Testing

The app tests can be executed using:
```shell
cargo test
```
It will run the unit & integration tests using the in memory configurations.

To run the integration tests with `production` feature, relying on Postgres & Redis, run:
```shell
KORU_ENV=integration cargo test --features production --no-default-features
```

> **Note:** to enable app logs during tests, add `TEST_LOG=1` environment variable. Logs are in bunyan format, so you may prefer to pipe them through `bunyan`.

> **Note:** It is also possible to run the tests using [nextest](https://github.com/nextest-rs/nextest)
> ```shell
> # Install with
> cargo install --locked cargo-nextest
> # Replace `cargo test` with
> cargo nextest run
> ```

### Running Locally

The app can be executed locally using:
```shell
# buyan piping is optional, but since the logs use bunyan format, it will help making them more readable.
cargo run | bunyan
```

It will run the app using the in memory configurations.

To run the integration tests with `production` feature, relying on Postgres & Redis, run:
```shell
# buyan piping is optional, but since the logs use bunyan format, it will help making them more readable.
KORU_ENV=integration cargo run --features production --no-default-features | buyan
```

### Building

#### Executable

An executable version of the application can be built with:
```shell
cargo build --release --bin koru --features production --no-default-features
```
This will build a fully featured application with support for in memory, Postgres and Redis backends.

Features can be adjusted to support only the ones you need. But it should be noted that if some features are not included, trying to set them up in [configuration](#configuration) will lead to a crash on startup.
If no features are provided, only in memory is supported.

#### Docker Image

A docker image can be built with:
```shell
docker build -t koru:latest .
```
It will build an image with all features, and `prod` configuration (see [Configuration](#configuration)). For other configurations, edit `Dockerfile`.

Note that the build requires SQLx offline data, which can be created with:
```shell
cargo sqlx prepare --merged -- --all-targets --features production --no-default-features
```

Note: if it fails with a warning `no queries found`, try updating sqlx-cli `cargo install sqlx-cli`

### Cargo Make

To simplify the different commands, it is possible to use [cargo-make](https://github.com/sagiegurari/cargo-make), `Makefile.toml` defines the different tasks.
Also not mandatory, it is highly recommended.

> **Note:** Provided `Makefile.toml` relies on [nextest](https://github.com/nextest-rs/nextest) for testing.
> ```shell
> # Install it by running
> cargo install --locked cargo-nextest
> ```

```shell
# Install cargo make 
cargo install --force cargo-make
# install git-hooks, setup redis & postgres if necessary, run fmt, clippy, unit test & integration test
cargo make
# setup redis & postgres if necessary, run fmt, clippy, unit test & integration test
cargo make all-test
# run fmt, clippy, unit test
cargo make unit-test
# setup redis & postgres if necessary, run fmt, clippy & integration test
cargo make integration-test
# setup redis & postgres if necessary
cargo make setup-infra
# generate SQLx offline data
cargo make sqlx-offline
# install git-hooks
cargo make git-hooks
# generate OpenApi file (in openapi folder)
cargo make openapi
# See Makefile.toml for more.
```

## Usage

### Curl

Here are some curl commands showing the use of the API
```shell
# Register
curl -i -H 'Content-Type: application/json' -d '{"name":"rbiland","password":"123","email":"r@r1.com"}' "http://localhost:8000/register"
# Login
curl -i -H 'Content-Type: application/json' -d '{"password":"123","email":"r@r1.com"}' -c cookie "http://localhost:8000/login"
# Create group
curl -i -H 'Content-Type: application/json' -d '{"name":"my group","color":{"red":0,"green":255,"blue":0}}' -b cookie "http://localhost:8000/groups"
# Get groups
curl -i -b cookie "http://localhost:8000/groups"
# Delete group (REPLACE GROUP_ID)
curl -i -b cookie -X DELETE "http://localhost:8000/groups/GROUP_ID"
# Create expense (REPLACE GROUP_ID)
curl -i -H 'Content-Type: application/json' -d '{"description":"my expense", "amount": 12}' -b cookie "http://localhost:8000/groups/GROUP_ID/expenses"
# Update expense (REPLACE GROUP_ID & EXPENSE_ID)
curl -i -H 'Content-Type: application/json' -d '{"description":"my expense 2", "amount": 20}' -b cookie -X PUT "http://localhost:8000/groups/GROUP_ID/expenses/EXPENSE_ID"
# Delete expense (REPLACE GROUP_ID & EXPENSE_ID)
curl -i -b cookie -X DELETE "http://localhost:8000/groups/GROUP_ID/expenses/EXPENSE_ID"
# Get un-settled expenses (REPLACE GROUP_ID)
curl -i -b cookie "http://localhost:8000/groups/GROUP_ID/expenses"
# Generate group token (REPLACE GROUP_ID)
curl -i -b cookie "http://localhost:8000/groups/GROUP_ID/token"
# Join group (needs a second user) (REPLACE GROUP_ID & TOKEN)
curl -i -H 'Content-Type: application/json' -d '{"token":"TOKEN","color":{"red":0,"green":255,"blue":0}}' -b cookie2 "http://localhost:8000/groups/GROUP_ID/members"
# Settle (REPLACE GROUP_ID)
curl -i -b cookie -X POST "http://localhost:8000/groups/GROUP_ID/settlements"
# Get past settlements (REPLACE GROUP_ID)
curl -i -b cookie "http://localhost:8000/groups/GROUP_ID/settlements"
# Get expenses of settlement (REPLACE GROUP_ID & STL_ID)
curl -i -b cookie "http://localhost:8000/groups/GROUP_ID/expenses?settlement_id=STL_ID"
# Change color (REPLACE GROUP_ID)
curl -i -H 'Content-Type: application/json' -d '{"color":{"red":255,"green":255,"blue":255}}' -b cookie -X PATCH "http://localhost:8000/groups/GROUP_ID/members"
# Register device (REPLACE MY_DEVICE_ID)
curl -i -H 'Content-Type: application/json' -d '{"device":"MY_DEVICE_ID"}' -b cookie "http://localhost:8000/devices"
# Remove device
curl -i -b cookie -X DELETE "http://localhost:8000/devices"
```

### OpenAPI / Swagger

Optionally, a Swagger UI endpoint is available with the `openapi` feature (included in `development` and `default`). 
It can be accessed at `http://localhost:8000/swagger-ui/`.

To run an executable that includes the endpoint you can run:

```shell
# Default config includes OpenApi
cargo run
# development config is the same as production but with openapi, so basically a dev build
cargo run --features development
```

To build a docker image that includes OpenApi you can run:

```shell
docker build --build-arg KORU_FEATURES=development -t koru:development .
```

To generate the OpenAPI file (in openapi folder) run:
```shell
cargo run --bin gen-openapi
# Or
cargo make openapi
```

## Configuration

The application is configured using stacked configurations, from yaml files and environment variables.

- `config/default.yml`: base configuration file
- `config/${KORU_ENV}.yml`: environment specific configuration file, based on `KORU_ENV` environment variable. Possible values are `local` (default), `integration`, and `prod`.
- Environment variables starting with `KORU` and using `__` as separators between the different levels. See examples bellow.

The following shows all available configurations, it should be noted that it cannot be used as-is, where `## CHOOSE ONE` is mentioned, only one option should be provided.
Configurations requiring specific feature flags are commented accordingly.

```yaml
api:
  host: localhost
  port: 8000
  session:
    hmac: Very_Long-SecRet!00##123456789-Very_Long-SecRet!00##123456789-Very_Long-SecRet!00##123456789
    duration: 20
    store: ### CHOOSE ONE
      redis: ### --features redis-session
        host: localhost
        port: 6379
      memory:
application:
  auth: ### CHOOSE ONE
    none:
    internal:
  token:
    jwt:
      secret: dEmOSecreT!
  notification:
    pushy:
      url: localhost
      token: YOUR_TOKEN
database: ### CHOOSE ONE
  postgres: ### --features postgres
    host: localhost
    port: 5432
    username: postgres
    password: password
    name: koru
  memory:
event_bus: ### CHOOSE ONE
  redis: ### --features redis-bus
    event_channel: koru_events
    host: localhost
    port: 6379
  memory:
```

Configurations can be overridden using environment variables, for example: 

- to change the JWT secret, you would use `KORU__APPLICATION__TOKEN__JWT__SECRET=whatever`
- to change the Redis event bus channel, you would use `KORU__EVENT_BUS__REDIS__EVENT_CHANNEL=channel`

### All Configurations
| Configuration                          | Explanation                                                                                                                             |
|----------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------|
| `api.host`                             | Application API host                                                                                                                    |
| `api.port`                             | Application API port                                                                                                                    |
| `application.token`                    | Generator to use for generating join group tokens <br/>(Currently only JWT is available)                                                |
| `application.token.jwt.secret`         | Secret for the JWT token generator                                                                                                      |
| `api.session`                          | Session management configuration                                                                                                        |
| `api.session.hmac`                     | HMAC for signing cookies                                                                                                                |
| `api.session.duration`                 | Session duration in Days                                                                                                                |
| `api.session.store`                    | Session store to use <br/>`redis` (requires `--features redis-session`) or `memory`                                                     |
| `api.session.store.redis.host`         | Host for Redis instance                                                                                                                 |
| `api.session.store.redis.port`         | Port for Redis instance                                                                                                                 |
| `api.session.store.memory`             | Use the the in memory session store                                                                                                     |
| `database`                             | Database to use for data storage <br/>`postgres` (requires `--features postgres`) or `memory`                                           |
| `database.postgres.host`               | Host for Postgres instance                                                                                                              |
| `database.postgres.port`               | Port for Postgres instance                                                                                                              |
| `database.postgres.username`           | Username for Postgres instance                                                                                                          |
| `database.postgres.password`           | Password for Postgres instance                                                                                                          |
| `database.postgres.name`               | Postgres database name                                                                                                                  |
| `database.memory`                      | Use the in memory data store                                                                                                            |
| `event_bus`                            | Event bus to use for sending and processing events asynchronously <br/>`redis` (requires `--features redis-bus`) or `memory`            |
| `event_bus.redis.host`                 | Host for Redis instance                                                                                                                 |
| `event_bus.redis.port`                 | Port for Redis instance                                                                                                                 |
| `event_bus.redis.event_channel`        | PubSub channel name for sending generated events                                                                                        |
| `event_bus.memory`                     | Use the in memory event bus                                                                                                             |
| `application.notification`             | Notification service to use for sending user notifications based on events <br/>(Currently only [Pushy](https://pushy.me) is available) |
| `application.notification.pushy.url`   | URL of the Pushy service                                                                                                                |
| `application.notification.pushy.token` | API key for Pushy                                                                                                                       |
| `application.auth`                     | Application auth mechanism to use <br/>`none` or `internal`                                                                             |
| `application.auth.none`                | Disable auth in the application, no password will be needed for registration & login                                                    |
| `application.auth.internal`            | Use the internal store for authentication                                                                                               |

### Environment Defaults
| Configuration                          | Default    | Local     | Integration | Prod                 |
|----------------------------------------|------------|-----------|-------------|----------------------|
| `api.host`                             |            | localhost | localhost   | 0.0.0.0              |
| `api.port`                             | 8000       |           |             |                      |
| `application.token`                    |            |           |             |                      |
| `application.token.jwt.secret`         | fake value |           |             | ENV_VAR              |
| `api.session`                          |            |           |             |                      |
| `api.session.hmac`                     | fake value |           |             | ENV_VAR              |
| `api.session.duration`                 | 20         |           |             |                      |
| `api.session.store`                    |            | memory    | redis       | redis                |
| `api.session.store.redis.host`         |            |           | localhost   | redis                |
| `api.session.store.redis.port`         |            |           | 6379        | 6379                 |
| `api.session.store.memory`             |            |           |             |                      |
| `database`                             |            | memory    | postgres    | postgres             |
| `database.postgres.host`               |            |           | localhost   | postgres             |
| `database.postgres.port`               |            |           | 5432        | 5432                 |
| `database.postgres.username`           |            |           | postgres    | ENV_VAR              |
| `database.postgres.password`           |            |           | password    | ENV_VAR              |
| `database.postgres.name`               |            |           | koru        | koru                 |
| `database.memory`                      |            |           |             |                      |
| `event_bus`                            |            | memory    | redis       | redis                |
| `event_bus.redis.host`                 |            |           | localhost   | redis                |
| `event_bus.redis.port`                 |            |           | 6379        | 6379                 |
| `event_bus.redis.event_channel`        |            |           | koru_events | koru_events          |
| `event_bus.memory`                     |            |           |             |                      |
| `application.notification`             |            |           |             |                      |
| `application.notification.pushy.url`   | localhost  |           |             | https://api.pushy.me |
| `application.notification.pushy.token` | fake value |           |             | ENV_VAR              |
| `application.auth`                     |            | internal  | internal    | internal             |
| `application.auth.none`                |            |           |             |                      |
| `application.auth.internal`            |            |           |             |                      |

