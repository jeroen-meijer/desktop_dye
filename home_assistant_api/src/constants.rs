#![allow(dead_code)]

use const_format::concatcp;

pub const DEFAULT_PORT: u16 = 8123;

pub const BASE_URL: &str = "/api/";

pub const URL_CONFIG: &str = concatcp!(BASE_URL, "config");
pub const URL_EVENTS: &str = concatcp!(BASE_URL, "events");
pub const URL_STATES: &str = concatcp!(BASE_URL, "states");
pub const URL_SERVICES: &str = concatcp!(BASE_URL, "services");
pub const URL_EVENTS_EVENT: &str = concatcp!(BASE_URL, "events/%s");
pub const URL_STATES_ENTITY: &str = concatcp!(BASE_URL, "states/%s");
pub const URL_SERVICES_SERVICE: &str = concatcp!(BASE_URL, "services/%s/%s");

pub const HTTP_HEADER_HA_AUTH: &str = "X-HA-access";
