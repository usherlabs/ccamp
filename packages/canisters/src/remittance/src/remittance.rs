// define all major types and their implementation here
use lib;
use std::collections::HashMap;

pub type Store = HashMap<(String, lib::Chain, String), lib::DataModel>;
