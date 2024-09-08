use soroban_sdk::{contracttype, Env, symbol_short, Vec, Bytes};

use crate::storage_types::{ Project, DataKey };
use crate::utils::u128_to_bytes;

#[derive(Clone)]
#[contracttype]

enum DataKeyAddress {
    Initialized,
    TotalAddress,
    Shares(u32),
    Addresses(u32),
}

pub fn get_project(e: &Env, project_id: Bytes) -> (Project, DataKey) {
    let project_key = DataKey::Project(project_id);
    let project: Project = e.storage().instance().get(&project_key).unwrap();
    (project, project_key)
}

pub fn get_all_projects(e: Env) -> Vec<Project> {
    let project_count: u128 = e
        .storage()
        .instance()
        .get(&symbol_short!("pk"))
        .unwrap_or(0);

    let mut projects: Vec<Project> = Vec::new(&e);

    for id in 1..=project_count {
        let project_id_in_bytes = u128_to_bytes(&e, id);
        let project_key = DataKey::Project(project_id_in_bytes);
        if let Some(project) = e.storage().instance().get(&project_key) {
            projects.push_back(project);
        }
    }

    projects
}