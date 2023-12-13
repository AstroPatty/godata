use warp::Filter;
use crate::{handlers, project};
use crate::project::ProjectManager;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use warp::http::StatusCode;


pub(crate) fn routes(project_manager: Arc<Mutex<ProjectManager>>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    list_collections()
        .or(list_projects(project_manager.clone()))
        .or(project_list(project_manager.clone()))
        .or(create_project(project_manager.clone()))
        .or(delete_project(project_manager.clone()))
        .or(project_link(project_manager.clone()))
        .or(projects_get(project_manager.clone()))
        .or(projects_path_exists(project_manager.clone()))
        .or(project_generate_path(project_manager.clone()))
        .or(project_remove_file(project_manager.clone()))
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

fn create_project(project_manager: Arc<Mutex<ProjectManager>>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("create" / String / String)
        .and(warp::post())
        .and(warp::query::<HashMap<String, String>>())
        .map(move |collection, project_name, params: HashMap<String, String>| {
            let force = match params.get("force") {
                Some(force) => force.parse::<bool>().unwrap(),
                None => false
            };
            let storage_location = match params.get("storage_location") {
                Some(storage_location) => Some(storage_location.to_owned()),
                None => None
            };
            handlers::create_project(project_manager.clone(), collection, project_name, force, storage_location)
        })
}

fn delete_project(project_manager: Arc<Mutex<ProjectManager>>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("projects" / String / String)
        .and(warp::delete())
        .and(warp::query::<HashMap<String, String>>())
        .map(move |collection, project_name, params: HashMap<String, String>| {
            let force = match params.get("force") {
                Some(force) => force.parse::<bool>().unwrap(),
                None => false
            };
            handlers::delete_project(project_manager.clone(), collection, project_name, force)
        })
}

fn project_link(project_manager: Arc<Mutex<ProjectManager>>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("projects" / String / String / "files")
        .and(warp::post())
        .and(warp::query::<HashMap<String, String>>())
        .map(move |collection, project_name, params: HashMap<String, String>| {
            let force = match params.get("force") {
                Some(force) => force.parse::<bool>().unwrap(),
                None => false
            };
            let ppath = match params.get("project_path") {
                Some(project_path) => project_path.to_owned(),
                None => return Ok(warp::reply::with_status(warp::reply::json(&format!("Missing project_path argument")), StatusCode::BAD_REQUEST))     // invalid request
            };
            let rpath = match params.get("real_path") {
                Some(storage_location) => storage_location.to_owned(),
                None => return Ok(warp::reply::with_status(warp::reply::json(&format!("Missing real_path argument")), StatusCode::BAD_REQUEST))     // invalid request
            };
            
            let type_ = match params.get("type") {
                Some(type_) => type_.to_owned(),
                None => "file".to_owned()
            };
            if type_ == "file" {
                return handlers::link_file(project_manager.clone(), collection, project_name, ppath, rpath, force)
            }
            else if type_ == "folder" {
                return handlers::link_folder(project_manager.clone(), collection, project_name, ppath, rpath, force)
            }
            else {
                return Ok(warp::reply::with_status(warp::reply::json(&format!("Invalid type argument {}", type_)), StatusCode::BAD_REQUEST))     // invalid request
            }
        })
}

fn project_list(project_manager: Arc<Mutex<ProjectManager>>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("projects" / String / String / "list")
        .and(warp::get())
        .and(warp::query::<HashMap<String, String>>())
        .map(move |collection, project_name, params: HashMap<String, String>| {
            let show_hidden = match params.get("show_hidden") {
                Some(show_hidden) => show_hidden.parse::<bool>().unwrap(),
                None => false
            };
            match params.get("project_path") {
                Some(path) => handlers::list_project(project_manager.clone(), collection, project_name, Some(path.to_owned()), show_hidden),
                None => handlers::list_project(project_manager.clone(), collection, project_name, None, show_hidden)
            }
        })
}

fn projects_get(project_manager: Arc<Mutex<ProjectManager>>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("projects" / String / String / "files")
        .and(warp::get())
        .and(warp::query::<HashMap<String, String>>())
        .map(move |collection, project_name, params: HashMap<String, String>| {
            let project_path = match params.get("project_path") {
                Some(project_path) => project_path.to_owned(),
                None => return Ok(warp::reply::with_status(
                    warp::reply::json(&format!("Missing project_path argument")), 
                    StatusCode::BAD_REQUEST))     // invalid request
            };
            handlers::get_file(project_manager.clone(), collection, project_name, project_path)
        })
}

fn projects_path_exists(project_manager: Arc<Mutex<ProjectManager>>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("projects" / String / String / "exists")
        .and(warp::get())
        .and(warp::query::<HashMap<String, String>>())
        .map(move |collection, project_name, params: HashMap<String, String>| {
            let project_path = match params.get("project_path") {
                Some(project_path) => project_path.to_owned(),
                None => return Ok(warp::reply::with_status(warp::reply::json(&format!("Missing project_path argument")), StatusCode::BAD_REQUEST))     // invalid request
            };
            handlers::path_exists(project_manager.clone(), collection, project_name, project_path)
        })
}

fn project_generate_path(project_manager: Arc<Mutex<ProjectManager>>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("projects" / String / String / "generate")
        .and(warp::get())
        .and(warp::query::<HashMap<String, String>>())
        .map(move |collection, project_name, params: HashMap<String, String>| {
            let project_path = match params.get("project_path") {
                Some(project_path) => project_path.to_owned(),
                None => return Ok(warp::reply::with_status(
                    warp::reply::json(&format!("Missing project_path argument")), 
                    StatusCode::BAD_REQUEST))     // invalid request
            };
            handlers::generate_path(project_manager.clone(), collection, project_name, project_path)
        })

    }
    
fn project_remove_file(project_manager: Arc<Mutex<ProjectManager>>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("projects" / String / String / "files")
        .and(warp::delete())
        .and(warp::query::<HashMap<String, String>>())
        .map(move |collection, project_name, params: HashMap<String, String>| {
            let project_path = match params.get("project_path") {
                Some(project_path) => project_path.to_owned(),
                None => return Ok(warp::reply::with_status(
                    warp::reply::json(&format!("Missing project_path argument")),
                    StatusCode::BAD_REQUEST))     // invalid request
            };
            handlers::remove_file(project_manager.clone(), collection, project_name, project_path)
        })
}