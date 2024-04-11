use crate::project::get_collection_names;
use crate::project::ProjectManager;
use warp::reply::Reply;
use warp::{http::Response, hyper::Body};

use serde::Serialize;
use std::collections::HashMap;
use std::convert::Infallible;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::instrument;
use warp::http::StatusCode;
use warp::reply::WithStatus;

#[instrument(name = "handlers.get_version", level = "info")]
pub(crate) fn get_version() -> Result<impl warp::Reply, Infallible> {
    Ok(warp::reply::with_status(
        warp::reply::json(&env!("CARGO_PKG_VERSION").to_string()),
        StatusCode::OK,
    ))
}
#[instrument(
    name = "handlers.list_collections",
    level = "info",
    fields(
        show_hidden = %show_hidden
    )
)]
pub(crate) fn list_collections(show_hidden: bool) -> Result<impl warp::Reply, Infallible> {
    let collections = get_collection_names(show_hidden);
    Ok(warp::reply::json(&collections.unwrap()))
}

#[
instrument(
    name = "handlers.list_projects",
    level = "info",
    skip(project_manager),
    fields(
        collection = %collection,
        show_hidden = %show_hidden
    )
)
]
pub(crate) fn list_projects(
    project_manager: Arc<Mutex<ProjectManager>>,
    collection: String,
    show_hidden: bool,
) -> Result<impl warp::Reply, Infallible> {
    let projects = project_manager
        .lock()
        .unwrap()
        .get_project_names(collection.clone(), show_hidden);
    match projects {
        Ok(project_list) => Ok(warp::reply::json(&project_list).into_response()),
        Err(e) => Ok(e.into_response()),
    }
}

#[instrument(
    name = "handlers.load_project",
    level = "info",
    skip(project_manager),
    fields(
        project_name = %project_name,
        collection = %collection
    )
)]
pub(crate) fn load_project(
    project_manager: Arc<Mutex<ProjectManager>>,
    collection: String,
    project_name: String,
) -> Result<impl warp::Reply, Infallible> {
    // Preload a project into memory. The idea is that in typical use, we want the "load_project" command on the Python side to be effective instant,
    // so we load the project into memory in a separate thread. By the time the user actually tries to USE the project, it should be loaded.
    // This really only matters for large projects, but it's a nice feature to have.
    let project_names = project_manager
        .lock()
        .unwrap()
        .get_project_names(collection.clone(), true);
    match project_names {
        Ok(project_list) => {
            if !project_list.contains(&project_name) {
                tracing::error!("No project named {project_name} in collection {collection}");
                return Ok(warp::reply::with_status(
                    warp::reply::json(&format!(
                        "No project named {project_name} in collection {collection}"
                    )),
                    StatusCode::NOT_FOUND,
                )
                .into_response());
            }
        }
        Err(e) => {
            tracing::error!("No collection named {collection}");
            return Ok(e.into_response());
        }
    }
    let message = format!("Sucessfully loaded project {collection}/{project_name}");
    tracing::info!("Loading project {project_name} in collection {collection}");
    tokio::task::spawn(async move {
        let _ = project_manager
            .lock()
            .unwrap()
            .load_project(&project_name, &collection);
    });
    Ok(warp::reply::with_status(warp::reply::json(&message), StatusCode::OK).into_response())
}

#[instrument(
    name = "handlers.drop_project",
    level = "info",
    skip(project_manager),
    fields(
        project_name = %project_name,
        collection = %collection
    )
)]
pub(crate) fn drop_project(
    project_manager: Arc<Mutex<ProjectManager>>,
    collection: String,
    project_name: String,
) -> Result<impl warp::Reply, Infallible> {
    let project = project_manager
        .lock()
        .unwrap()
        .drop_project(&project_name, &collection);
    match project {
        Ok(_) => {
            tracing::info!("Project {project_name} dropped.");
            Ok(warp::reply::with_status(
                warp::reply::json(&format!("Project {project_name} dropped.")),
                StatusCode::OK,
            )
            .into_response())
        }
        Err(e) => Ok(e.into_response()),
    }
}

#[instrument(
    name = "handlers.list_project",
    level = "info",
    skip(project_manager),
    fields(
        collection = %collection,
        project_name = %project_name,
        project_path = format!("{:?}", project_path),
        show_hidden = %_show_hidden
    )
)]
pub(crate) fn list_project(
    project_manager: Arc<Mutex<ProjectManager>>,
    collection: String,
    project_name: String,
    project_path: Option<String>,
    _show_hidden: bool,
) -> Result<impl warp::Reply, Infallible> {
    let project = project_manager
        .lock()
        .unwrap()
        .load_project(&project_name, &collection);
    match project {
        Ok(project) => {
            let project = project.lock().unwrap();
            let result = project.list(project_path);
            match result {
                Ok(list) => Ok(warp::reply::json(&list).into_response()),
                Err(e) => Ok(e.into_response()),
            }
        }
        Err(e) => Ok(e.into_response()),
    }
}

#[instrument(
    name = "handlers.create_project",
    level = "info",
    skip(project_manager),
    fields(
        collection = %collection,
        project_name = %project_name,
        force = %force,
        storage_location = format!("{:?}", storage_location)
    )
)]
pub(crate) fn create_project(
    project_manager: Arc<Mutex<ProjectManager>>,
    collection: String,
    project_name: String,
    force: bool,
    storage_location: Option<String>,
) -> Result<impl warp::Reply, Infallible> {
    let project = project_manager.lock().unwrap().create_project(
        &project_name,
        &collection,
        force,
        storage_location,
    );
    match project {
        Ok(_) => Ok(warp::reply::with_status(
            warp::reply::json(&format!(
                "Project {project_name} created in collection {collection}"
            )),
            StatusCode::CREATED,
        )
        .into_response()),
        Err(e) => Ok(e.into_response()),
    }
}

#[instrument(
    name = "handlers.delete_project",
    level = "info",
    skip(project_manager),
    fields(
        collection = %collection,
        project_name = %project_name,
        force = %force
    )
)]
pub(crate) fn delete_project(
    project_manager: Arc<Mutex<ProjectManager>>,
    collection: String,
    project_name: String,
    force: bool,
) -> Result<impl warp::Reply, Infallible> {
    let project = project_manager
        .lock()
        .unwrap()
        .delete_project(&project_name, &collection, force);
    match project {
        Ok(_) => Ok(warp::reply::with_status(
            warp::reply::json(&format!(
                "Project {project_name} deleted from collection {collection}"
            )),
            StatusCode::OK,
        )
        .into_response()),
        Err(e) => Ok(e.into_response()),
    }
}

#[derive(Serialize)]
struct LinkResponse {
    message: String,
    removed: Vec<String>,
}

#[instrument(
    name = "handlers.link_file",
    level = "info",
    skip(project_manager),
    fields(
        collection = %collection,
        project_name = %project_name,
        project_path = %project_path,
        file_path = %file_path,
        force = %force
    )
)]
pub(crate) fn link_file(
    project_manager: Arc<Mutex<ProjectManager>>,
    collection: String,
    project_name: String,
    project_path: String,
    file_path: String,
    metadata: HashMap<String, String>,
    force: bool,
) -> Result<Response<Body>, Infallible> {
    let project = project_manager
        .lock()
        .unwrap()
        .load_project(&project_name, &collection);

    match project {
        Err(e) => return Ok(e.into_response()),
        Ok(project) => {
            let parsed_file_path = PathBuf::from(&file_path);
            let result =
                project
                    .lock()
                    .unwrap()
                    .add_file(&project_path, parsed_file_path, metadata, force);

            match result {
                Ok(previous_paths) => {
                    let output: LinkResponse = LinkResponse {
                        message: format!("File {file_path} linked to {project_path} in project {project_name} in collection {collection}"),
                        removed: previous_paths.unwrap_or(Vec::new()),
                    };

                    return Ok(warp::reply::with_status(
                        warp::reply::json(&output),
                        StatusCode::CREATED,
                    )
                    .into_response());
                }
                Err(e) => Ok(e.into_response()),
            }
        }
    }
}

#[instrument(
    name = "handlers.link_folder",
    level = "info",
    skip(project_manager),
    fields(
        collection = %collection,
        project_name = %project_name,
        project_path = %project_path,
        folder_path = %folder_path,
        recursive = %recursive
    )
)]
pub(crate) fn link_folder(
    project_manager: Arc<Mutex<ProjectManager>>,
    collection: String,
    project_name: String,
    project_path: String,
    folder_path: String,
    recursive: bool,
) -> Result<Response<Body>, Infallible> {
    let project = project_manager
        .lock()
        .unwrap()
        .load_project(&project_name, &collection);
    match project {
        Ok(project) => {
            let parsed_folder_path = PathBuf::from(&folder_path);
            let result =
                project
                    .lock()
                    .unwrap()
                    .add_folder(&project_path, parsed_folder_path, recursive);
            match result {
                Ok(_) => {
                    let out = LinkResponse {
                        message: format!("Folder {folder_path} linked to {project_path} in project {project_name} in collection {collection}"),
                        removed: Vec::new(),
                    };
                    return Ok(warp::reply::with_status(
                        warp::reply::json(&out),
                        StatusCode::CREATED,
                    )
                    .into_response());
                }

                Err(e) => {
                    return Ok(e.into_response());
                }
            };
        }
        Err(e) => Ok(e.into_response()),
    }
}

#[instrument(
    name = "handlers.get_file",
    level = "info",
    skip(project_manager),
    fields(
        collection = %collection,
        project_name = %project_name,
        project_path = %project_path
    )
)]
pub(crate) fn get_file(
    project_manager: Arc<Mutex<ProjectManager>>,
    collection: String,
    project_name: String,
    project_path: String,
) -> Result<WithStatus<warp::reply::Json>, Infallible> {
    let project = project_manager
        .lock()
        .unwrap()
        .load_project(&project_name, &collection);
    if project.is_ok() {
        let project = project.unwrap();
        let result = project.lock().unwrap().get_file(&project_path);
        match result {
            Ok(file) => {
                return Ok(warp::reply::with_status(
                    warp::reply::json(&file),
                    StatusCode::OK,
                ))
            }

            Err(_) => {
                return Ok(warp::reply::with_status(
                    warp::reply::json(&format!("File {project_path} does not exist!")),
                    StatusCode::NOT_FOUND,
                ))
            }
        }
    }
    Ok(warp::reply::with_status(
        warp::reply::json(&format!(
            "No project named {project_name} in collection {collection}"
        )),
        StatusCode::NOT_FOUND,
    ))
}

#[instrument(
    name = "handlers.generate_path",
    level = "info",
    skip(project_manager),
    fields(
        collection = %collection,
        project_name = %project_name,
        project_path = %project_path
    )
)]
pub(crate) fn generate_path(
    project_manager: Arc<Mutex<ProjectManager>>,
    collection: String,
    project_name: String,
    project_path: String,
) -> Result<WithStatus<warp::reply::Json>, Infallible> {
    let project = project_manager
        .lock()
        .unwrap()
        .load_project(&project_name, &collection);
    if project.is_ok() {
        let project = project.unwrap();
        let result = project.lock().unwrap().generate_path(&project_path);
        match result {
            Ok(path) => {
                return Ok(warp::reply::with_status(
                    warp::reply::json(&path),
                    StatusCode::OK,
                ))
            }

            Err(_) => {
                return Ok(warp::reply::with_status(
                    warp::reply::json(&"Uncaught error generating path!".to_string()),
                    StatusCode::INTERNAL_SERVER_ERROR,
                ))
            }
        }
    };

    Ok(warp::reply::with_status(
        warp::reply::json(&format!(
            "No project named {project_name} in collection {collection}"
        )),
        StatusCode::NOT_FOUND,
    ))
}

pub(crate) fn path_exists(
    project_manager: Arc<Mutex<ProjectManager>>,
    collection: String,
    project_name: String,
    project_path: String,
) -> Result<WithStatus<warp::reply::Json>, Infallible> {
    let project = project_manager
        .lock()
        .unwrap()
        .load_project(&project_name, &collection);
    if project.is_ok() {
        let project = project.unwrap();
        let result = project.lock().unwrap().exists(project_path);
        if result {
            return Ok(warp::reply::with_status(
                warp::reply::json(&true),
                StatusCode::OK,
            ));
        } else {
            return Ok(warp::reply::with_status(
                warp::reply::json(&false),
                StatusCode::OK,
            ));
        }
    }
    Ok(warp::reply::with_status(
        warp::reply::json(&format!(
            "No project named {project_name} in collection {collection}"
        )),
        StatusCode::NOT_FOUND,
    ))
}

#[instrument(
    name = "handlers.move_",
    level = "info",
    skip(project_manager),
    fields(
        collection = %collection,
        project_name = %project_name,
        project_path = %project_path,
        new_project_path = %new_project_path,
        overwrite = %overwrite
    )
)]
pub(crate) fn move_(
    project_manager: Arc<Mutex<ProjectManager>>,
    collection: String,
    project_name: String,
    project_path: String,
    new_project_path: String,
    overwrite: bool,
) -> Result<WithStatus<warp::reply::Json>, Infallible> {
    let project = project_manager
        .lock()
        .unwrap()
        .load_project(&project_name, &collection);
    if project.is_ok() {
        let project = project.unwrap();
        let result = project
            .lock()
            .unwrap()
            .move_(&project_path, &new_project_path, overwrite);
        match result {
            Ok(v) => {
                return Ok(warp::reply::with_status(
                    warp::reply::json(
                        &LinkResponse {
                            message: format!("File {project_path} moved to {new_project_path} in project {project_name} in collection {collection}"),
                            removed: v.unwrap_or(Vec::new()),
                        }
                    ),
                    StatusCode::OK,
                ))
            }

            Err(_) => {
                return Ok(warp::reply::with_status(
                    warp::reply::json(&format!("File {project_path} does not exist!")),
                    StatusCode::NOT_FOUND,
                ))
            }
        }
    }
    Ok(warp::reply::with_status(
        warp::reply::json(&format!(
            "No project named {project_name} in collection {collection}"
        )),
        StatusCode::NOT_FOUND,
    ))
}

#[instrument(
    name = "handlers.remove_file",
    level = "info",
    skip(project_manager),
    fields(
        collection = %collection,
        project_name = %project_name,
        project_path = %project_path
    )
)]
pub(crate) fn remove_file(
    project_manager: Arc<Mutex<ProjectManager>>,
    collection: String,
    project_name: String,
    project_path: String,
) -> Result<WithStatus<warp::reply::Json>, Infallible> {
    let project = project_manager
        .lock()
        .unwrap()
        .load_project(&project_name, &collection);
    if project.is_ok() {
        let project = project.unwrap();
        let result = project.lock().unwrap().remove_file(&project_path);
        match result {
            Ok(v) => {
                return Ok(warp::reply::with_status(
                    warp::reply::json(&v),
                    StatusCode::OK,
                ))
            }

            Err(_) => {
                return Ok(warp::reply::with_status(
                    warp::reply::json(&format!("File {project_path} does not exist!")),
                    StatusCode::NOT_FOUND,
                ))
            }
        }
    }
    Ok(warp::reply::with_status(
        warp::reply::json(&format!(
            "No project named {project_name} in collection {collection}"
        )),
        StatusCode::NOT_FOUND,
    ))
}

#[instrument(
    name = "handlers.export_project_tree",
    level = "info",
    skip(project_manager),
    fields(
        collection = %collection,
        project_name = %project_name,
        output_path = %output_path
    )
)
]
pub(crate) fn export_project_tree(
    project_manager: Arc<Mutex<ProjectManager>>,
    collection: String,
    project_name: String,
    output_path: String,
) -> Result<WithStatus<warp::reply::Json>, Infallible> {
    let result = project_manager.lock().unwrap().export_project(
        &project_name,
        &collection,
        PathBuf::from(&output_path),
    );
    match result {
        Ok(_) => Ok(warp::reply::with_status(
            warp::reply::json(&format!(
                "tree for project {project_name} in collection {collection} exported"
            )),
            StatusCode::OK,
        )),
        Err(e) => Ok(warp::reply::with_status(
            warp::reply::json(&e.to_string()),
            StatusCode::CONFLICT,
        )),
    }
}

#[instrument(
    name = "handlers.import_project_tree",
    level = "info",
    skip(project_manager),
    fields(
        collection = %collection,
        project_name = %project_name,
        input_path = %input_path
    )
)
]
pub(crate) fn import_project_tree(
    project_manager: Arc<Mutex<ProjectManager>>,
    collection: String,
    project_name: String,
    input_path: String,
) -> Result<WithStatus<warp::reply::Json>, Infallible> {
    let storage_path = PathBuf::from(&input_path);
    let result = project_manager.lock().unwrap().import_project(
        &project_name,
        &collection,
        "local",
        storage_path,
    );
    match result {
        Ok(_p) => Ok(warp::reply::with_status(
            warp::reply::json(&format!(
                "tree for project {project_name} in collection {collection} imported"
            )),
            StatusCode::OK,
        )),
        Err(e) => Ok(warp::reply::with_status(
            warp::reply::json(&e.to_string()),
            StatusCode::CONFLICT,
        )),
    }
}
