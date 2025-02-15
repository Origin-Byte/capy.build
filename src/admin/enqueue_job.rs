use crate::db;
use crate::schema::{background_jobs, crates};
use crate::worker::jobs;
use anyhow::Result;
use crates_io_worker::BackgroundJob;
use diesel::dsl::exists;
use diesel::prelude::*;
use secrecy::{ExposeSecret, SecretString};

#[derive(clap::Parser, Debug)]
#[command(
    name = "enqueue-job",
    about = "Add a job to the background worker queue",
    rename_all = "snake_case"
)]
pub enum Command {
    UpdateDownloads,
    DumpDb {
        #[arg(env = "READ_ONLY_REPLICA_URL")]
        database_url: SecretString,
        #[arg(default_value = "db-dump.tar.gz")]
        target_name: String,
    },
    DailyDbMaintenance,
    SquashIndex,
    NormalizeIndex {
        #[arg(long = "dry-run")]
        dry_run: bool,
    },
    CheckTyposquat {
        #[arg()]
        name: String,
    },
    SyncAdmins {
        /// Force a sync even if one is already in progress
        #[arg(long)]
        force: bool,
    },
}

pub fn run(command: Command) -> Result<()> {
    let conn = &mut db::oneoff_connection()?;
    println!("Enqueueing background job: {command:?}");

    match command {
        Command::UpdateDownloads => {
            let count: i64 = background_jobs::table
                .filter(background_jobs::job_type.eq(jobs::UpdateDownloads::JOB_NAME))
                .count()
                .get_result(conn)?;

            if count > 0 {
                println!(
                    "Did not enqueue {}, existing job already in progress",
                    jobs::UpdateDownloads::JOB_NAME
                );
            } else {
                jobs::UpdateDownloads.enqueue(conn)?;
            }
        }
        Command::DumpDb {
            database_url,
            target_name,
        } => {
            jobs::DumpDb::new(database_url.expose_secret(), target_name).enqueue(conn)?;
        }
        Command::SyncAdmins { force } => {
            if !force {
                // By default, we don't want to enqueue a sync if one is already
                // in progress. If a sync fails due to e.g. an expired pinned
                // certificate we don't want to keep adding new jobs to the
                // queue, since the existing job will be retried until it
                // succeeds.

                let query = background_jobs::table
                    .filter(background_jobs::job_type.eq(jobs::SyncAdmins::JOB_NAME));

                if diesel::select(exists(query)).get_result(conn)? {
                    info!(
                        "Did not enqueue {}, existing job already in progress",
                        jobs::SyncAdmins::JOB_NAME
                    );
                    return Ok(());
                }
            }

            jobs::SyncAdmins.enqueue(conn)?;
        }
        Command::DailyDbMaintenance => {
            jobs::DailyDbMaintenance.enqueue(conn)?;
        }
        Command::SquashIndex => {
            jobs::SquashIndex.enqueue(conn)?;
        }
        Command::NormalizeIndex { dry_run } => {
            jobs::NormalizeIndex::new(dry_run).enqueue(conn)?;
        }
        Command::CheckTyposquat { name } => {
            // The job will fail if the crate doesn't actually exist, so let's check that up front.
            if crates::table
                .filter(crates::name.eq(&name))
                .count()
                .get_result::<i64>(conn)?
                == 0
            {
                anyhow::bail!(
                    "cannot enqueue a typosquat check for a crate that doesn't exist: {name}"
                );
            }

            jobs::CheckTyposquat::new(&name).enqueue(conn)?;
        }
    };

    Ok(())
}
