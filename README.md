# 📬 Newsletter

#### Self hosted Email Newsletter written in Rust, only for adding _email subscription_ page to your blog - nothing more, nothing less

<div align="center">
  <h3>Powered by</h3>
  <div style="display: flex; justify-content: center;">
  <a href="https://neon.tech/">
  <img width="200px" src="https://i.imgur.com/ZO6E4tQ.png" alt="NeonDB Logo">
  </a>
  &nbsp;
  &nbsp;
  <a href="https://github.com/meltred/meltcd">
  <img width="200px" src="https://i.imgur.com/S3kHtNI.png" alt="MeltCD Logo"> 
  </a>
  </div>
</div> 

### Demo

> It is being used in my [blog](https://kunalsin9h.com/blog) page

![image](https://github.com/KunalSin9h/newsletter/assets/82411321/ac151b2c-9baf-4f6d-a667-c3fd24796c15)


### Requirements

- [Rust](https://www.rust-lang.org/tools/install)
- [Postmark Account](https://postmarkapp.com/) (for email client)
- [PostgreSQL](https://www.postgresql.org/download/)
- [Redis](https://redis.io/download) (or anther redis-api compatible cache, in my case [DragonFlyDB](https://www.dragonflydb.io/))
- [Docker](https://docs.docker.com/get-docker/)
- [Docker Compose](https://docs.docker.com/compose/install/)
- [Gnu Make](https://www.gnu.org/software/make/) (optional)

### Setup

> [!IMPORTANT]
> If you want to run the project using docker, then see these instructions [here](./Docker-Setup.md)

```bash
# clone the project
git clone https://github.com/kunalsin9h/newsletter.git

# change directory
cd newsletter

# install required dependencies

# install sqlx cli
cargo install sqlx-cli --no-default-features --features postgres

# install livejq cli
cargo install livejq

# run database

make postgres_up # or run the script in ./scripts/init_db.sh
make redis_up    # or run the script in ./scripts/init_redis.sh


# run the app
cargo run | livejq
```

The application will be served on the specified port on the `./configuration/base.yaml` file. (default: `5000`)

# API Docs

> [!NOTE]
> The default username is `admin` and the default password is `admin`
> You must change the password from the admin dashboard

You have to update the `email_client` section in the `./configuration/local.yaml` in order to
use the email client

1. Add a new subscriber

To add a new subscriber send a `x-www-form-urlencoded` **POST** request to `http://localhost:5000/subscription` with `name` and `email` fields.

Then the subscriber will receive a confirmation email, which will contain a link to confirm the subscription.

2.  ... TODO...

# Story
![image](https://github.com/KunalSin9h/newsletter/assets/82411321/f982d3d3-3b04-455a-8491-4c6f76568e80)

The project was build while reading above book, **_From Zero to Production in Rust_**.

I have completed the project to use it on my own blog ([here](https://kunalsin9h.com/blog))

Do you thinking, how simple the project idea is?

_I recommend you to take a look at the source and test, good luck!_
