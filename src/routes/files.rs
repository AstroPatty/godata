use crate::errors::{GodataError, GodataErrorType};
use crate::handlers;
use crate::project::ProjectManager;
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use tracing::instrument;
use warp::http::StatusCode;
use warp::Filter;
use warp::Reply;
use warp::{http::Response, hyper::Body};

pub(super) fn routes(
    project_manager: Arc<Mutex<ProjectManager>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    project_list(project_manager.clone())
        .or(project_link(project_manager.clone()))
        .or(projects_get(project_manager.clone()))
        .or(projects_path_exists(project_manager.clone()))
        .or(project_generate_path(project_manager.clone()))
        .or(project_remove_file(project_manager.clone()))
        .or(move_file(project_manager.clone()))
}

#[instrument(skip(project_manager))]
fn project_link(
    project_manager: Arc<Mutex<ProjectManager>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("projects" / String / String / "files")
        .and(warp::post())
        .and(warp::query::<HashMap<String, String>>())
        .map(
            move |collection,
                  project_name,
                  mut params: HashMap<String, String>|
                  -> Result<Response<Body>, _> {
                let force = match params.remove("force") {
                    Some(force) => force.parse::<bool>().unwrap(),
                    None => false,
                };
                let ppath = match params.remove("project_path") {
                    Some(project_path) => project_path.to_owned(),
                    None => {
                        tracing::error!("Query missing project_path argument");
                        return Ok::<Response<Body>, Infallible>(
                            warp::reply::with_status(
                                warp::reply::json(&"Missing project_path argument".to_string()),
                                StatusCode::BAD_REQUEST,
                            )
                            .into_response(),
                        );
                    } // invalid request
                };
                let rpath = match params.remove("real_path") {
                    Some(storage_location) => storage_location.to_owned(),
                    None => {
                        tracing::error!("Query missing real_path argument");
                        return Ok(warp::reply::with_status(
                            warp::reply::json(&"Missing real_path argument".to_string()),
                            StatusCode::BAD_REQUEST,
                        )
                        .into_response());
                    } // invalid request
                };

                let type_ = match params.remove("type") {
                    Some(type_) => type_.to_owned(),
                    None => "file".to_owned(),
                };
                if type_ == "file" {
                    handlers::link_file(
                        project_manager.clone(),
                        collection,
                        project_name,
                        ppath,
                        rpath,
                        params,
                        force,
                    )
                } else if type_ == "folder" {
                    let recursive = match params.get("recursive") {
                        Some(recursive) => recursive.parse::<bool>().unwrap(),
                        None => false,
                    };
                    return handlers::link_folder(
                        project_manager.clone(),
                        collection,
                        project_name,
                        ppath,
                        rpath,
                        recursive,
                    );
                } else {
                    tracing::error!("Request included invalid type argument {}", type_);
                    return Ok(warp::reply::with_status(
                        warp::reply::json(&format!("Invalid type argument {}", type_)),
                        StatusCode::BAD_REQUEST,
                    )
                    .into_response()); // invalid request
                }
            },
        )
}

#[instrument(skip(project_manager))]
fn project_list(
    project_manager: Arc<Mutex<ProjectManager>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("projects" / String / String / "list")
        .and(warp::get())
        .and(warp::query::<HashMap<String, String>>())
        .map(
            move |collection, project_name, params: HashMap<String, String>| {
                let show_hidden = match params.get("show_hidden") {
                    Some(show_hidden) => show_hidden.parse::<bool>().unwrap(),
                    None => false,
                };
                match params.get("project_path") {
                    Some(path) => handlers::list_project(
                        project_manager.clone(),
                        collection,
                        project_name,
                        Some(path.to_owned()),
                        show_hidden,
                    ),
                    None => handlers::list_project(
                        project_manager.clone(),
                        collection,
                        project_name,
                        None,
                        show_hidden,
                    ),
                }
            },
        )
}

#[instrument(skip(project_manager))]
fn projects_get(
    project_manager: Arc<Mutex<ProjectManager>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("projects" / String / String / "files")
        .and(warp::get())
        .and(warp::query::<HashMap<String, String>>())
        .map(
            move |collection, project_name, params: HashMap<String, String>| {
                let project_path = params.get("project_path");
                match (params.get("pattern"), project_path) {
                    (None, Some(ppath)) => handlers::get_file(
                        project_manager.clone(),
                        collection,
                        project_name,
                        ppath.to_owned(),
                    ),
                    (Some(pattern), ppath) => handlers::get_files_with_pattern(
                        project_manager.clone(),
                        collection,
                        project_name,
                        ppath.map(|p| p.as_str()),
                        pattern,
                    ),
                    (None, None) => {
                        tracing::error!("Query missing project_path argument");
                        Ok(GodataError::new(
                            GodataErrorType::InvalidPath,
                            "Missing project_path argument".to_string(),
                        )
                        .into_response())
                    }
                }
            },
        )
}

#[instrument(skip(project_manager))]
fn projects_path_exists(
    project_manager: Arc<Mutex<ProjectManager>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("projects" / String / String / "exists")
        .and(warp::get())
        .and(warp::query::<HashMap<String, String>>())
        .map(
            move |collection, project_name, params: HashMap<String, String>| {
                let project_path = match params.get("project_path") {
                    Some(project_path) => project_path.to_owned(),
                    None => {
                        tracing::error!("Query missing project_path argument");
                        return Ok(warp::reply::with_status(
                            warp::reply::json(&"Missing project_path argument".to_string()),
                            StatusCode::BAD_REQUEST,
                        ));
                    } // invalid request
                };
                handlers::path_exists(
                    project_manager.clone(),
                    collection,
                    project_name,
                    project_path,
                )
            },
        )
}

#[instrument(skip(project_manager))]
fn project_generate_path(
    project_manager: Arc<Mutex<ProjectManager>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("projects" / String / String / "generate")
        .and(warp::get())
        .and(warp::query::<HashMap<String, String>>())
        .map(
            move |collection, project_name, params: HashMap<String, String>| {
                let project_path = match params.get("project_path") {
                    Some(project_path) => project_path.to_owned(),
                    None => {
                        tracing::error!("Query missing project_path argument");
                        return Ok(warp::reply::with_status(
                            warp::reply::json(&"Missing project_path argument".to_string()),
                            StatusCode::BAD_REQUEST,
                        ));
                    } // invalid request
                };
                handlers::generate_path(
                    project_manager.clone(),
                    collection,
                    project_name,
                    project_path,
                )
            },
        )
}

#[instrument(skip(project_manager))]
fn project_remove_file(
    project_manager: Arc<Mutex<ProjectManager>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("projects" / String / String / "files")
        .and(warp::delete())
        .and(warp::query::<HashMap<String, String>>())
        .map(
            move |collection, project_name, params: HashMap<String, String>| {
                let project_path = match params.get("project_path") {
                    Some(project_path) => project_path.to_owned(),
                    None => {
                        tracing::error!("Query missing project_path argument");
                        return Ok(warp::reply::with_status(
                            warp::reply::json(&"Missing project_path argument".to_string()),
                            StatusCode::BAD_REQUEST,
                        ));
                    } // invalid request
                };
                handlers::remove_file(
                    project_manager.clone(),
                    collection,
                    project_name,
                    project_path,
                )
            },
        )
}

#[instrument(skip(project_manager))]
fn move_file(
    project_manager: Arc<Mutex<ProjectManager>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("projects" / String / String / "files" / "move")
        .and(warp::post())
        .and(warp::query::<HashMap<String, String>>())
        .map(
            move |collection, project_name, params: HashMap<String, String>| {
                let project_path = match params.get("source_path") {
                    Some(project_path) => project_path.to_owned(),
                    None => {
                        tracing::error!("Query missing project_path argument");
                        return Ok(warp::reply::with_status(
                            warp::reply::json(&"Missing project_path argument".to_string()),
                            StatusCode::BAD_REQUEST,
                        ));
                    } // invalid request
                };
                let new_path = match params.get("destination_path") {
                    Some(new_path) => new_path.to_owned(),
                    None => {
                        tracing::error!("Query missing new_path argument");
                        return Ok(warp::reply::with_status(
                            warp::reply::json(&"Missing new_path argument".to_string()),
                            StatusCode::BAD_REQUEST,
                        ));
                    } // invalid request
                };
                let overwrite = match params.get("overwrite") {
                    Some(overwrite) => overwrite.parse::<bool>().unwrap(),
                    None => false,
                };
                handlers::move_(
                    project_manager.clone(),
                    collection,
                    project_name,
                    project_path,
                    new_path,
                    overwrite,
                )
            },
        )
}
