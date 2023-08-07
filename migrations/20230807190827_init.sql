-- Add migration script here
CREATE TABLE IF NOT EXISTS applicants (
    nuid varchar PRIMARY KEY,
    applicant_name varchar NOT NULL,
    registration_time timestamp with time zone NOT NULL,
    token uuid UNIQUE NOT NULL,
    challenge json NOT NULL,
    solution json NOT NULL
);

CREATE TABLE IF NOT EXISTS submissions (
    submission_id serial PRIMARY KEY,
    nuid varchar NOT NULL REFERENCES applicants (nuid),
    ok boolean NOT NULL,
    submission_time timestamp with time zone NOT NULL
);