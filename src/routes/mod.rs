mod projects;
mod files;

use std::sync::{Arc, Mutex};
use crate::project::ProjectManager;
use warp::Filter;

pub fn routes(project_manager: Arc<Mutex<ProjectManager>>) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    projects::routes(project_manager.clone())
        .or(files::routes(project_manager.clone()))
}