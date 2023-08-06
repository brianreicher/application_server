use rand::{
    seq::{IteratorRandom, SliceRandom},
    Rng,
};
use sqlx::PgPool;

use uuid::Uuid;

use crate::{
    db::{self},
    endpoints::errors::ModelError,
};

use super::types::{Applicant, Color};

use strum::{EnumIter, IntoEnumIterator};

use rand_pcg::Pcg64;
use rand_seeder::Seeder;

pub async fn get_applicants(
    pool: PgPool,
    applicants: &[String],
) -> Result<Vec<Applicant>, ModelError> {
    match db::transactions::get_applicants_db(&pool, applicants).await {
        Ok(vec) => Ok(vec
            .iter()
            .map(|(nuid, name, reg_time, sub_time, ok)| {
                let time_to_completion = match sub_time.signed_duration_since(*reg_time).to_std() {
                    Ok(d) => d,
                    Err(_) => std::time::Duration::ZERO,
                };
                Applicant {
                    nuid: nuid.clone(),
                    name: name.clone(),
                    time_to_completion,
                    ok: *ok,
                }
            })
            .collect()),
        Err(_) => Err(ModelError::SqlError),
    }
}
pub async fn register_user(
    pool: PgPool,
    name: String,
    nuid: String,
) -> Result<(Uuid, Vec<String>), ModelError> {
    let token = Uuid::new_v4();
    let (challenge_strings, solution) = generate_challenge(
        &nuid,
        100,
        vec![
            String::from(""),
            Color::Red.to_string(),
            Color::Orange.to_string(),
            Color::Yellow.to_string(),
            Color::Green.to_string(),
            Color::Blue.to_string(),
            Color::Violet.to_string(),
        ],
    );

    match db::transactions::register_user_db(&pool, token, name, nuid, &challenge_strings, solution)
        .await
    {
        Ok(()) => Ok((token, challenge_strings)),
        // there's a bunch of different ways that this can fail, I should probably
        // handle the error -
        Err(_e) => Err(ModelError::DuplicateUser),
    }
}

pub async fn retreive_token(pool: PgPool, nuid: &String) -> Result<Uuid, ModelError> {
    match db::transactions::retreive_token_db(&pool, nuid).await {
        Ok(token) => Ok(token),
        Err(_) => Err(ModelError::NoUserFound),
    }
}

pub async fn retreive_challenge(pool: &PgPool, token: Uuid) -> Result<Vec<String>, ModelError> {
    match db::transactions::retreive_challenge_db(pool, token).await {
        Ok(challenge) => Ok(challenge),
        Err(_) => Err(ModelError::NoUserFound),
    }
}

pub async fn check_solution(
    pool: PgPool,
    token: Uuid,
    given_soln: &Vec<String>,
) -> Result<bool, ModelError> {
    // Check if the solution is correct - write the row to the solutions table
    match db::transactions::retreive_soln(&pool, token).await {
        Ok((soln, nuid)) => {
            let ok = soln == *given_soln;
            if let Err(_e) = db::transactions::write_submission(pool, nuid, ok).await {
                return Err(ModelError::SqlError);
            }
            Ok(ok)
        }
        Err(_) => Err(ModelError::NoUserFound),
    }
}

#[derive(EnumIter, Debug)]
enum EditType {
    Insertion,
    Deletion,
    Substitution,
}

fn generate_challenge(
    nuid: &str,
    n_random: usize,
    mandatory_cases: Vec<String>,
) -> (Vec<String>, Vec<String>) {
    let mut rng: Pcg64 = Seeder::from(nuid).make_rng();
    let random_cases: Vec<String> = (0..n_random)
        .map(|_| {
            let color = Color::iter().choose(&mut rng).unwrap().to_string();
            let len = color.len();
            let random_count = rng.gen_range(0..=len);
            if random_count == 0 {
                return color;
            }
            match EditType::iter().choose(&mut rng).unwrap() {
                EditType::Deletion => color.chars().skip(random_count).collect(),
                EditType::Insertion => {
                    let alphabet: Vec<char> = ('a'..='z').collect();
                    let mut color_chars: Vec<char> = color.chars().collect();
                    let random_chars = alphabet
                        .choose_multiple(&mut rng, random_count)
                        .cloned()
                        .collect::<Vec<char>>();
                    let random_indices = (0..random_count)
                        .map(|_| rng.gen_range(0..=color_chars.len()))
                        .collect::<Vec<usize>>();
                    for (index, random_char) in random_indices.into_iter().zip(random_chars) {
                        color_chars.insert(index, random_char);
                    }
                    color_chars.into_iter().collect()
                }
                EditType::Substitution => {
                    let changed_indices: Vec<_> =
                        (0..random_count).map(|_| rng.gen_range(0..len)).collect();
                    let alphabet: Vec<char> = ('a'..='z').collect();
                    let mut color_chars: Vec<char> = color.chars().collect();

                    for index in changed_indices {
                        let original_char = color_chars[index];
                        let mut new_char;
                        loop {
                            new_char = *alphabet.choose(&mut rng).unwrap();
                            if new_char != original_char {
                                break;
                            }
                        }
                        color_chars[index] = new_char;
                    }
                    color_chars.into_iter().collect()
                }
            }
        })
        .collect();

    let mut all_cases = mandatory_cases;
    all_cases.extend(random_cases);

    let answers: Vec<String> = all_cases
        .iter()
        .filter(|case| one_edit_away(case))
        .cloned()
        .collect();

    (all_cases, answers)
}

fn n_edits_away(str1: &str, str2: &str, n: isize) -> bool {
    if (str1.len() as isize - str2.len() as isize).abs() > n {
        return false;
    }

    let (shorter, longer) = if str1.len() > str2.len() {
        (str2, str1)
    } else {
        (str1, str2)
    };

    let mut short_pointer = 0;
    let mut long_pointer = 0;
    let mut edit_count = 0;

    while short_pointer < shorter.len() && long_pointer < longer.len() {
        if shorter.chars().nth(short_pointer) != longer.chars().nth(long_pointer) {
            edit_count += 1;
            if edit_count > n {
                return false;
            }
            if shorter.len() == longer.len() {
                short_pointer += 1;
            }
        } else {
            short_pointer += 1;
        }
        long_pointer += 1;
    }
    edit_count <= n
}

fn one_edit_away(str: &str) -> bool {
    Color::iter().any(|color| n_edits_away(str, color.to_string().as_str(), 1))
}

#[cfg(test)]
mod tests {

    use super::generate_challenge;
    use super::one_edit_away;
    use super::Color;

    #[test]
    fn test_generate_challenge() {
        let mandatory_cases: Vec<String> = vec![
            String::from(""),
            Color::Red.to_string(),
            Color::Orange.to_string(),
            Color::Yellow.to_string(),
            Color::Green.to_string(),
            Color::Blue.to_string(),
            Color::Violet.to_string(),
        ];
        let n_mandatory = mandatory_cases.len();
        let n_random = 10;
        let (cases, answers) =
            generate_challenge(&String::from("001234567"), n_random, mandatory_cases);

        assert_eq!(cases.len(), n_mandatory + n_random);

        assert!(answers.iter().all(|answer| one_edit_away(answer)));
    }

    #[test]
    fn test_one_edit_away_example() {
        assert!(one_edit_away("red"));
        assert!(one_edit_away("lue"));
        assert!(!one_edit_away("ooran"));
        assert!(!one_edit_away("abc"));
        assert!(one_edit_away("greene"));
    }
}
