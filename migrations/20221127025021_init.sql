-- Add migration script here
CREATE USER "generate_admin" WITH PASSWORD 'generate_tech_app';

CREATE TABLE IF NOT EXISTS applicants (
    nuid varchar PRIMARY KEY,
    applicant_name varchar NOT NULL,
    registration_time timestamp with time zone NOT NULL,
    token uuid UNIQUE NOT NULL,
    challenge_string varchar NOT NULL,
    solution json NOT NULL,
    backend_q1_solution json NOT NULL,
    backend_q2_solution json NOT NULL
);

CREATE TABLE IF NOT EXISTS submissions (
    submission_id serial PRIMARY KEY,
    -- solution_id integer NOT NULL REFERENCES problems (solution_id),
    nuid varchar NOT NULL REFERENCES applicants (nuid),
    ok boolean NOT NULL,
    submission_time timestamp with time zone NOT NULL
);

