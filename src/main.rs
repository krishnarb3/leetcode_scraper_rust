use clap::Parser;
#[macro_use]
extern crate lambda_runtime as lambda;
use graphql_client::{reqwest::post_graphql_blocking as post_graphql, GraphQLQuery};
use lambda::handler_fn;
use log::LevelFilter;
use rand::Rng;
use reqwest::{blocking::Client, header};
use serde_json::Value;
use simple_logger::SimpleLogger;
use std::env;
use tokio::task;

const COOKIE_ENV_KEY: &str = "LEETCODE_SESSION";
const DISCORD_WEBHOOK_URL_KEY: &str = "DISCORD_WEBHOOK_URL_KEY";
const LEETCODE_PROBLEMS_PREFIX: &str = "https://leetcode.com/problems/";
const RUN_MODE: &str = "RUN_MODE";
const AWS_LAMBDA_MODE: &str = "AWS_LAMBDA";
const USER_AGENT: &str = "graphql-rust/0.10.0";

#[tokio::main]
async fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .with_utc_timestamps()
        .init()
        .unwrap();
    let mode = env::var(RUN_MODE);
    if mode.is_ok() && mode.unwrap() == AWS_LAMBDA_MODE {
        let func = handler_fn(lambda_fetch);
        lambda_runtime::run(func).await;
    } else {
        task::spawn_blocking(move || {
            cli_fetch();
        })
        .await;
    }
}

async fn lambda_fetch(req: Args, context: lambda_runtime::Context) -> Result<(), anyhow::Error> {
    log::info!("Handling lambda request");
    let config = aws_config::load_from_env().await;
    leetcode_fetch(req.companies, req.difficulties);
    Ok(())
}

fn cli_fetch() {
    let args: Args = Args::parse();

    let companies = args.companies;
    let difficulties = args.difficulties;
    leetcode_fetch(companies, difficulties)
}

fn leetcode_fetch(companies: Vec<String>, difficulties: Vec<String>) {
    let mut rng = rand::thread_rng();
    let variables = get_company_tag::Variables {
        slug: companies[rng.gen_range(0..companies.len())].to_string(),
    };

    let leetcode_session =
        env::var(COOKIE_ENV_KEY).expect(&*format!("{} env variable not found", COOKIE_ENV_KEY));
    let cookie = format!("LEETCODE_SESSION={:#?}", leetcode_session);
    let mut headers = header::HeaderMap::new();
    headers.insert("Cookie", header::HeaderValue::from_str(&*cookie).unwrap());

    let leetcode_client = Client::builder()
        .user_agent(USER_AGENT)
        .default_headers(headers)
        .build()
        .expect("Unable to create leetcode client");

    let response_body = post_graphql::<getCompanyTag, _>(
        &leetcode_client,
        "https://leetcode.com/graphql",
        variables,
    );
    if response_body.is_err() {
        panic!("{}", response_body.unwrap_err());
    }
    let response_data: get_company_tag::ResponseData =
        response_body.unwrap().data.expect("Missing response data");
    let questions = response_data.company_tag.unwrap().questions;
    let mut result = Vec::new();
    for question in questions {
        let question_fields = question.unwrap();
        if difficulties.contains(&question_fields.difficulty.to_string()) {
            let title_slug = question_fields.title_slug;
            result.push(title_slug.to_string());
        }
    }

    let random_problem_url =
        LEETCODE_PROBLEMS_PREFIX.to_string() + &result[rng.gen_range(0..result.len())];

    let discord_webhook_url = env::var(DISCORD_WEBHOOK_URL_KEY);
    if discord_webhook_url.is_ok() {
        let discord_client = reqwest::blocking::Client::new();
        let body: Value =
            serde_json::from_str(&*format!("{{\"content\":\"{}\"}}", random_problem_url))
                .expect("Error creating json from provided url string");
        let response = discord_client
            .post(discord_webhook_url.unwrap())
            .json(&body)
            .send();
        match response.unwrap().status() {
            reqwest::StatusCode::OK | reqwest::StatusCode::NO_CONTENT => {
                log::info!(
                    "Successfully pushed problem: {} to discord",
                    random_problem_url
                );
            }
            other => {
                panic!(
                    "Error occurred while pushing problem to discord: {:?}",
                    other
                );
            }
        }
    } else {
        log::info!("Discord webhook url is not configured");
        log::info!("{}", random_problem_url);
    }
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/query.graphql",
    response_derives = "Debug"
)]
pub struct getCompanyTag;

// Get arguments from user
#[derive(Parser, Deserialize)]
#[clap(author, version, about, long_about = None)]
pub(crate) struct Args {
    // Companies to choose problem from
    #[clap(short, long, multiple_values = true)]
    companies: Vec<String>,

    // Difficulties to choose problem from
    #[clap(short, long, multiple_values = true)]
    difficulties: Vec<String>,
}
