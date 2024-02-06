mod files;
mod projects;

use crate::project::ProjectManager;
use std::sync::{Arc, Mutex};
use warp::Filter;

pub fn routes(
    project_manager: Arc<Mutex<ProjectManager>>,
) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    projects::routes(project_manager.clone()).or(files::routes(project_manager.clone()))
}
