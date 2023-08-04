use chrono::{DateTime, Utc};
use core::panic;
use serde_json;
use std::{collections::HashMap, time::SystemTime};
use uuid::Uuid;

use sqlx::{query, PgPool};

pub async fn register_user_db(
    pool: &PgPool,
    token: Uuid,
    name: String,
    nuid: String,
    challenge_strings: &Vec<String>,
    solution: Vec<bool>,
) -> Result<(), sqlx::Error> {
    // Insert the applicant
    let registration_time: DateTime<Utc> = SystemTime::now().into();
    let ser_challenge_strings = match serde_json::to_value(&challenge_strings) {
        Ok(val) => val,
        Err(_) => todo!("Figure out how to handle the serde error properly"),
    };
    let ser_solution = match serde_json::to_value(&solution) {
        Ok(val) => val,
        Err(_) => todo!("Figure out how to handle the serde error properly"),
    };

    // solution set backend q1
    let q1_json_data = r#"
    [
        { "name": "Bella Napoli", "avgScore": 38.6 },
        { "name": "Tenda Asian Fusion", "avgScore": 37.4 },
        { "name": "Red Chopstick", "avgScore": 36.285714285714285 },
        { "name": "El Mixteco", "avgScore": 34.8 },
        { "name": "Bamboo Restaurant", "avgScore": 34.0 }
    ]
    "#;

    let restaurants_q1: Vec<Restaurant> = serde_json::from_str(q1_json_data).unwrap();

    let mut b1_soln: HashMap<String, u64> = HashMap::new();
    for restaurant in restaurants_q1 {
        b1_soln.insert(restaurant.name.clone(), restaurant.avgScore.clone());
    }

    let ser_b1_soln = match serde_json::to_value(&b1_soln) {
        Ok(val) => val,
        Err(_) => todo!("Figure out how to handle the serde error properly"),
    };

    // solution set backend q2
    let q2_json_data = r#"
    [
        {'cuisine': 'American', 'name': 'Wild Asia'},
        {'cuisine': 'American', 'name': 'Manhem Club'},
        {'cuisine': 'American',
         'name': 'The New Starling Athletic Club Of The Bronx'},
        {'cuisine': 'American', 'name': 'Yankee Tavern'},
    ]
    "#;

    let restaurants_q2: Vec<Restaurant> = serde_json::from_str(q2_json_data).unwrap();

    let mut b2_soln: HashMap<String, String> = HashMap::new();
    for restaurant in restaurants_q2 {
        b2_soln.insert(restaurant.cuisine.clone(), restaurant.name.clone());
    }

    let ser_b2_soln = match serde_json::to_value(&b2_soln) {
        Ok(val) => val,
        Err(_) => todo!("Figure out how to handle the serde error properly"),
    };

    query!(
        r#"INSERT INTO applicants (nuid, applicant_name, registration_time, token, challenge_strings, solution, backend_q1_solution, backend_q2_solution)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8);"#,
        nuid,
        name,
        registration_time,
        token,
        ser_challenge_strings,
        ser_solution,
        ser_b1_soln,
        ser_b2_soln,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_applicants_db(
    pool: &PgPool,
    nuids: &[String],
) -> Result<Vec<(String, String, DateTime<Utc>, DateTime<Utc>, bool)>, sqlx::Error> {
    // This is a hack, sqlx doesn't support vector replacement into an IN statement
    let records = query!(
        r#"SELECT DISTINCT ON (nuid) nuid, applicant_name, ok, submission_time, 
        registration_time FROM submissions JOIN applicants using(nuid) where 
        nuid=ANY($1) ORDER BY nuid, submission_time DESC;"#,
        &nuids[..]
    )
    .fetch_all(pool)
    .await?;

    Ok(records
        .iter()
        .map(|record| {
            (
                record.nuid.clone(),
                record.applicant_name.clone(),
                record.registration_time,
                record.submission_time,
                record.ok,
            )
        })
        .collect())
}

pub async fn retreive_token_db(pool: &PgPool, nuid: &String) -> Result<Uuid, sqlx::Error> {
    let record = query!(r#"SELECT token FROM applicants WHERE nuid=$1"#, nuid)
        .fetch_one(pool)
        .await?;

    Ok(record.token)
}

pub async fn retreive_challenge_db(pool: &PgPool, token: Uuid) -> Result<Vec<String>, sqlx::Error> {
    let record = query!(
        r#"SELECT challenge_strings FROM applicants where token=$1"#,
        token
    )
    .fetch_one(pool)
    .await?;

    match serde_json::from_value(record.challenge_strings) {
        Ok(challenge_strings) => Ok(challenge_strings),
        Err(_e) => {
            panic!("challenge strings didn't deserialize properly - this should never happen")
        }
    }
}

pub async fn retreive_soln(pool: &PgPool, token: Uuid) -> Result<(Vec<bool>, String), sqlx::Error> {
    let record = query!(
        r#"SELECT nuid, solution FROM applicants WHERE token=$1"#,
        token
    )
    .fetch_one(pool)
    .await?;

    match serde_json::from_value(record.solution) {
        Ok(soln) => Ok((soln, record.nuid)),
        Err(_e) => panic!("solution didn't deserialize properly - this should never happen"),
    }
}

pub async fn retreive_soln_bq1(
    pool: &PgPool,
    token: Uuid,
) -> Result<(HashMap<String, u64>, String), sqlx::Error> {
    let record = query!(
        r#"SELECT nuid, backend_q1_solution FROM applicants WHERE token=$1"#,
        token
    )
    .fetch_one(pool)
    .await?;

    match serde_json::from_value(record.backend_q1_solution) {
        Ok(soln) => Ok((soln, record.nuid)),
        Err(_e) => panic!("solution didn't deserialize properly - this should never happen"),
    }
}

pub async fn retreive_soln_bq2(
    pool: &PgPool,
    token: Uuid,
) -> Result<(HashMap<String, String>, String), sqlx::Error> {
    let record = query!(
        r#"SELECT nuid, backend_q2_solution FROM applicants WHERE token=$1"#,
        token
    )
    .fetch_one(pool)
    .await?;

    match serde_json::from_value(record.backend_q2_solution) {
        Ok(soln) => Ok((soln, record.nuid)),
        Err(_e) => panic!("solution didn't deserialize properly - this should never happen"),
    }
}

pub async fn write_submission(pool: PgPool, nuid: String, ok: bool) -> Result<(), sqlx::Error> {
    let submission_time: DateTime<Utc> = SystemTime::now().into();

    query!(
        r#"INSERT INTO submissions (nuid, ok, submission_time) VALUES ($1, $2, $3);"#,
        nuid,
        ok,
        submission_time,
    )
    .execute(&pool)
    .await?;

    Ok(())
}
