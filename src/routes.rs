use warp::Filter;
use crate::{handlers, project};
use crate::project::ProjectManager;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;


pub(crate) fn routes(project_manager: Arc<Mutex<ProjectManager>>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    list_collections()
        .or(list_projects(project_manager.clone()))
        .or(list_project())
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

fn list_project() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("projects" / String / String)
        .and(warp::get())
        .map(|collection, project| format!("list project ${project} in collection ${collection}"))
}