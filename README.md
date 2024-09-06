# Cornerstone Cafe Signup Website

## Background

The Cornerstone Cafe is an annual fundraiser for the small Christian school I
attended for grades 1-12. The fundraiser is a cafe-style booth at a local fair
serving burgers and fries. Each year, well over 100 workers sign up to work one
or more of the 250 shifts available. 

As you may expect, organizing this effort is quite the undertaking. Before I
created this website this was done quite manually, with the schedule laid out in
a spreadsheet and signups done in-person or via email. After being recruited one
year to write a utility to create PDFs for each worker using the spreadsheets, I
thought it would be an interesting project to build a website to facilitate the
schedule creation and signup for all the volunteers at the Cafe.

## Using

Please feel free to use this for your own fundraisers. See the 
[considerations](#considerations) listed below for some things you might want
to change.

### Installation

#### Docker compose

A simple compose file is included in the repository. This will set up docker
containers for the website and database, as well as an otel-collector and jaeger
to collect and display metrics and traces. To start it all, run the following
command:

```docker compose up -d```

The included docker-compose file expects a [configuration file](#configuration)
for the website to be provided at `docker.config.toml`.

#### Alternatives

You can build and run the website without Docker, but it will take some
considerations. 

* You'll need to have a Postgres database for the website
  to store its data.
* You'll need to have the `public` directory adjacent to the executable
  so it can find the static files stored there.
* Build dependencies:
  * [Rust + Cargo](https://www.rust-lang.org/tools/install)
  * [Tailwind](https://tailwindcss.com/docs/installation)
* [Configuration](#configuration)

### Configuration

Configuration is done through the `config.toml` file which the website expects
to find adjacent to the executable. The available options are described below:
```toml
[website] # Required section
base_url = "localhost" # Used in the creation of links in emails
port = 3000 # Optional port number on which to host the website. Defaults to 3000
session_key = "..." # Optional key used to encrypt sessions. Should be at least 64 characters
otel_endpoint = "grpc://localhost:4317" # Optional endpoint for an otel collector
timezone = "America/Los_Angeles" # Optional timezone name to use in display of timestamps

[admin] # Details used in signatures of emails. All required
name = "Your Name"
email = "your_email@example.com"
phone = "555-555-5555"

[database] # Required section
database_url = "postgres://localhost:5432" # Postgres database url

[ssl] # Optional. If missing, website will run in http mode
cert = "./certs/cert.pem" # Path to cert file
key = "./certs/key.pem" # Path to key file

[email] # Optional. Without it, no emails will be sent
address = "your_email@example.com" # Sending email address
password = "Y0uR_Pas%worD#" # Password for sending email address
server = "smtp.example.com" # SMTP server to be used to send emails
```

### Bootstrapping a user

When the database is first set up, no users are created, so you'll be unable
to do anything with the website. To create your first user, use 
[psql](https://www.postgresql.org/docs/current/app-psql.html) to get into the
database and create an invitation for yourself to create an account.

If you're using the included docker compose file: 

```
  docker compose exec -U cafe postgres psql
```

Once in psql:

```
  INSERT INTO admin_invite (email) VALUES ({your email}) RETURNING id;
```

Copy the id you received and go to `{your domain}/account/create/{id}` to finish
setting up your account.


## Considerations

Currently several places in the website are quite specific to the Cornerstone
Cafe. The plan is to eventually get the website to a point where it can all
be configured in the database or config.toml, but that is not the case yet.
If you'd like to use this website, here are some of the files that should be
changed:

### Templates 

* index.html (title tag)
* navigation.html (page title)
* email/messages/* (several references, probably worth writing entirely custom
  messages)

### Source Code

 * accounts/invite.rs (email message)
  
