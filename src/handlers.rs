use crate::project::ProjectManager;
use crate::project::get_collection_names;
use std::sync::{Arc, Mutex};
use std::convert::Infallible;
use warp::http::StatusCode;

pub(crate) fn list_collections(show_hidden: bool) -> Result<impl warp::Reply, Infallible> {
    let collections = get_collection_names(show_hidden);
    Ok(warp::reply::json(&collections.unwrap()))
}

pub(crate) fn list_projects(project_manager: Arc<Mutex<ProjectManager>>, collection: String, show_hidden: bool) -> Result<impl warp::Reply, Infallible> {
    let projects = project_manager.lock().unwrap().get_project_names(collection.clone(), show_hidden);
    match projects {
        Ok(project_list) => Ok(warp::reply::with_status(warp::reply::json(&project_list), StatusCode::OK)),
        Err(_) => Ok(warp::reply::with_status(warp::reply::json(
            &format!("No collection named {collection}")
        ), StatusCode::NOT_FOUND))
    }
}