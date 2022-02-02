use clap::Parser;
use graphql_client::{reqwest::post_graphql_blocking as post_graphql, GraphQLQuery};
use reqwest::blocking::Client;
use reqwest::header;
use std::env;

const USER_AGENT: &str = "graphql-rust/0.10.0";
const COOKIE_ENV_KEY: &str = "LEETCODE_SESSION";

fn main() -> Result<(), anyhow::Error> {
    let args: Args = Args::parse();

    let companies = &args.companies[0];
    let difficulties = args.difficulties;

    let variables = get_company_tag::Variables {
        slug: companies.to_string(),
    };

    let cookie = match env::var(COOKIE_ENV_KEY) {
        Ok(cookie_value) => format!("LEETCODE_SESSION={:#?}", cookie_value),
        Err(_) => panic!("{:#?} env variable not found", COOKIE_ENV_KEY),
    };
    let mut headers = header::HeaderMap::new();
    headers.insert("Cookie", header::HeaderValue::from_str(&*cookie).unwrap());

    let client = Client::builder()
        .user_agent(USER_AGENT)
        .default_headers(headers)
        .build()?;

    let response_body =
        post_graphql::<getCompanyTag, _>(&client, "https://leetcode.com/graphql", variables);
    if response_body.is_err() {
        panic!("{:#?}", response_body.unwrap_err());
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
        println!("{:#?}", result);
    }
    Ok(())
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/query.graphql",
    response_derives = "Debug"
)]
pub struct getCompanyTag;

// Get arguments from user
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub(crate) struct Args {
    // Companies to choose problem from
    #[clap(short, long)]
    companies: Vec<String>,

    // Difficulties to choose problem from
    #[clap(short, long)]
    difficulties: Vec<String>,
}
