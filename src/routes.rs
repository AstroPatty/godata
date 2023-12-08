use warp::Filter;
use crate::{handlers, project};
use crate::project::ProjectManager;
use std::sync::{Arc, Mutex};


pub(crate) fn routes(project_manager: Arc<Mutex<ProjectManager>>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    list_collections()
        .or(list_projects(project_manager.clone()))
        .or(list_project())
}

fn list_collections() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("collections")
        .and(warp::get())
        .map(|| warp::reply::json(&handlers::list_collections()))
    }

fn list_projects(project_manager: Arc<Mutex<ProjectManager>>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("projects" / String)
        .and(warp::get())
        .map(move |collection| warp::reply::json(&handlers::list_projects(project_manager.clone(), collection)))
}

fn list_project() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("projects" / String / String)
        .and(warp::get())
        .map(|collection, project| format!("list project ${project} in collection ${collection}"))
}