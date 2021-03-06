// This is a contrived example intended to illustrate actix-web features.
// *Imagine* that you have a process that involves 3 steps.  The steps here
// are dumb in that they do nothing other than call an
// httpbin endpoint that returns the json that was posted to it.  The intent
// here is to illustrate how to chain these steps together as futures and return
// a final result in a response.
//
// Actix-web features illustrated here include:
//     1. handling json input param
//     2. validating user-submitted parameters using the 'validator' crate
//     2. actix-web client features:
//           - POSTing json body
//     3. chaining futures into a single response used by an async endpoint

use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::io;
use std::env; 

use actix_web::{
    client::Client,
    error::ErrorBadRequest,
    web::{self, BytesMut},
    App, Error, HttpResponse, HttpServer,
};
use futures::StreamExt;
use validator::Validate;
use validator_derive::Validate;

#[derive(Debug, Validate, Deserialize, Serialize)]
struct SomeData {
    #[validate(length(min = "1", max = "1000000"))]
    id: String,
    #[validate(length(min = "1", max = "100"))]
    name: String,
}

#[derive(Debug, Deserialize)]
struct HttpBinResponse {
    args: HashMap<String, String>,
    data: String,
    files: HashMap<String, String>,
    form: HashMap<String, String>,
    headers: HashMap<String, String>,
    json: SomeData,
    origin: String,
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct News {
    day: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Team {
    team_abbreviation: String,
    active_area: String,
    join_year: u32,
}

/// validate data, post json to httpbin, get it back in the response body, return deserialized
async fn step_x(data: SomeData, client: &Client) -> Result<SomeData, Error> {
    // validate data
    data.validate().map_err(ErrorBadRequest)?;

    let mut res = client
        .post("https://httpbin.org/post")
        .send_json(&data)
        .await
        .map_err(Error::from)?; // <- convert SendRequestError to an Error

    let mut body = BytesMut::new();
    while let Some(chunk) = res.next().await {
        body.extend_from_slice(&chunk?);
    }

    let body: HttpBinResponse = serde_json::from_slice(&body).unwrap();
    Ok(body.json)
}

async fn create_something(
    some_data: web::Json<SomeData>,
    client: web::Data<Client>,
) -> Result<HttpResponse, Error> {
    let some_data_2 = step_x(some_data.into_inner(), &client).await?;
    let some_data_3 = step_x(some_data_2, &client).await?;
    let d = step_x(some_data_3, &client).await?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&d).unwrap()))
}

async fn todays_shami_momo(
    _client: web::Data<Client>,
) -> Result<HttpResponse, Error> {
    let news = News { day: "today".to_string(), content: "Shamiko is going to go on date with Momo.".to_string() };

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&news)?))
}

async fn all_teams(
    _client: web::Data<Client>,
) -> Result<HttpResponse, Error> {
    let mut res: Vec<Team> = Vec::new();

    let t1 = Team { team_abbreviation: "鹿島".to_string(),
                    active_area: "茨城県".to_string(),
                    join_year: 1991 };
    let t2 = Team { team_abbreviation: "浦和".to_string(),
                    active_area: "埼玉県".to_string(),
                    join_year: 1991 };
    let t3 = Team { team_abbreviation: "水戸".to_string(),
                    active_area: "茨城県".to_string(),
                    join_year: 2000 };

    res.push(t1);
    res.push(t2);
    res.push(t3);

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&res)?))
}

async fn teams_j1(
    _client: web::Data<Client>,
) -> Result<HttpResponse, Error> {
    let mut res: Vec<Team> = Vec::new();

    let t1 = Team { team_abbreviation: "鹿島".to_string(),
                    active_area: "茨城県".to_string(),
                    join_year: 1991 };
    let t2 = Team { team_abbreviation: "浦和".to_string(),
                    active_area: "埼玉県".to_string(),
                    join_year: 1991 };

    res.push(t1);
    res.push(t2);

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&res)?))
}

async fn teams_j2(
    _client: web::Data<Client>,
) -> Result<HttpResponse, Error> {
    let mut res: Vec<Team> = Vec::new();

    let t3 = Team { team_abbreviation: "水戸".to_string(),
                    active_area: "茨城県".to_string(),
                    join_year: 2000 };

    res.push(t3);

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&res)?))
}

#[actix_rt::main]
async fn main() -> io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    // let endpoint = "127.0.0.1:8080";
    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a number");

    // println!("Starting server at: {:?}", endpoint);
    HttpServer::new(|| {
        App::new()
            .data(Client::default())
            .service(web::resource("/something").route(web::post().to(create_something)))
            .service(web::resource("/shami_momo").route(web::get().to(todays_shami_momo)))

            .service(web::resource("/api/v0/teams").route(web::get().to(all_teams)))
            .service(web::resource("/api/v0/teams/j1").route(web::get().to(teams_j1)))
            .service(web::resource("/api/v0/teams/j2").route(web::get().to(teams_j2)))
    })
    //.bind(endpoint)?
    .bind(("0.0.0.0", port))? 
    .run()
    .await
}
