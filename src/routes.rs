use warp::Filter;
use crate::{handlers, project};
use crate::project::ProjectManager;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;


pub(crate) fn routes(project_manager: Arc<Mutex<ProjectManager>>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    list_collections()
        .or(list_projects(project_manager.clone()))
        .or(projects_get(project_manager.clone()))
}

fn list_collections() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("collections")
        .and(warp::get())
        .and(warp::query::<HashMap<String, bool>>())
        .map(move |p: HashMap<String, bool>| match p.get("show_hidden") {
            Some(show_hidden) => handlers::list_collections(*show_hidden),
            None => handlers::list_collections(false)
        })
    }

fn list_projects(project_manager: Arc<Mutex<ProjectManager>>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("projects" / String)
        .and(warp::get())
        .and(warp::query::<HashMap<String, bool>>())
        .map(move |collection, p: HashMap<String, bool>| match p.get("show_hidden") {
            Some(show_hidden) => handlers::list_projects(project_manager.clone(), collection, *show_hidden),
            None => handlers::list_projects(project_manager.clone(), collection, false)
        })
        
}

fn projects_get(project_manager: Arc<Mutex<ProjectManager>>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("projects" / String / String)
        .and(warp::get())
        .and(warp::query::<HashMap<String, String>>())
        .map(move |collection, project_name, params: HashMap<String, String>| {
            let show_hidden = match params.get("show_hidden") {
                Some(show_hidden) => show_hidden.parse::<bool>().unwrap(),
                None => false
            };
            match params.get("path") {
                Some(path) => handlers::list_project(project_manager.clone(), collection, project_name, Some(path.to_owned()), show_hidden),
                None => handlers::list_project(project_manager.clone(), collection, project_name, None, show_hidden)
            }
        })

}


fn parse_query_params(params: HashMap<String, String>) -> HashMap<String, String> {
    let mut parsed_params = HashMap::new();
    for (key, value) in params {
        parsed_params.insert(key, value);
    }
    parsed_params
}