use std::f32::consts::E;
use std::str::Split;
use std::io::Result;

pub(crate) fn parse_command(cmd_str: &str) -> Result<GodataCommand> {
    // Commands are in the form of:
    // "P/M:COMMAND:ARG1:ARG2:...:ARGN"
    // Where P/M is either P for project or M for management
    // COMMAND is the command to be executed
    // ARG1, ARG2, ..., ARGN are the arguments for the command

    // Split the command string into its components
    let mut cmd_parts = cmd_str.split("::");
    let cmd_type = cmd_parts.next().unwrap();
    if cmd_type.len() != 1 {
        panic!("Invalid command type {}", cmd_type);
    }
    let cmd_type = cmd_type.chars().next().unwrap();

    match cmd_type {
        'M' => {
            let cmd_ = ManagementCommand::parse(cmd_parts)?;
            Ok(GodataCommand::Management(cmd_))
        },
        'P' => {
            let cmd_ = ProjectCommand::parse(cmd_parts)?;
            Ok(GodataCommand::Project(cmd_))
        },
        _ => {
            panic!("Invalid command type `{}`", cmd_type);
        }
    }
}

#[derive(Debug)]
pub(crate) enum GodataCommand{
    Management(ManagementCommand),
    Project(ProjectCommand)
}

#[derive(Debug)]
pub(crate) enum ProjectCommand // Commands for updating individual project
{
    AddFile(String, String),
    GetFile(String, String),
    AddFolder(String, String),
    RemoveFile(String),
    Exists(String), 
    GeneratePath(String),
    List(Option<String>)
}

impl ProjectCommand {
    pub(crate) fn parse(mut cmd: Split<&str>) -> Result<ProjectCommand> {
        let cmd_name = cmd.next().unwrap();
        let arguments = cmd.collect::<Vec<&str>>();
        match cmd_name {
            "AddFile" => {
                if arguments.len() != 2 {
                    return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid number of arguments for AddFile"))
                }
                let project_path = arguments[0];
                let file_path = arguments[1];
                let cmd = ProjectCommand::AddFile(project_path.to_string(), file_path.to_string());
                Ok(cmd)
            }
            "GetFile" => {
                if arguments.len() != 1 {
                    return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid number of arguments for GetFile"))
                }
                let project_path = arguments[0];
                let cmd = ProjectCommand::GetFile(project_path.to_string(), "".to_string());
                Ok(cmd)
            }
            "AddFolder" => {
                if arguments.len() != 2 {
                    return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid number of arguments for AddFolder"))
                }
                
                let project_path = arguments[0];
                let folder_path = arguments[1];
                let cmd = ProjectCommand::AddFolder(project_path.to_string(), folder_path.to_string());
                Ok(cmd)
            }
            "RemoveFile" => {
                if arguments.len() != 1 {
                    return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid number of arguments for RemoveFile"))
                }
 
                let project_path = arguments[0];
                let cmd = ProjectCommand::RemoveFile(project_path.to_string());
                Ok(cmd)
            }
            "Exists" => {
                if arguments.len() != 1 {
                    return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid number of arguments for Exists"))
                }

                let project_path = arguments[0];
                let cmd = ProjectCommand::Exists(project_path.to_string());
                Ok(cmd)
            }
            "GeneratePath" => {
                if arguments.len() != 1 {
                    return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid number of arguments for GeneratePath"))
                }

                let project_path = arguments[0];
                let cmd = ProjectCommand::GeneratePath(project_path.to_string());
                Ok(cmd)
            }
            "List" => {
                let project_path;
                if arguments.len() != 1 {
                    project_path = None;
                }
                else {
                    project_path = Some(arguments[0].to_string());
                }
                let cmd = ProjectCommand::List(project_path);
                Ok(cmd)
            }
            _ => {
                panic!("Invalid command `{}`", cmd_name);
            }
        }
    }
}

#[derive(Debug)]
pub(crate) enum ManagementCommand // Commands project management
{
    CreateProject(String),
    DeleteProject(String),
    LoadProject(String, String),
    ListProjects(String),
    ListCollections,
}

impl ManagementCommand {
    pub(crate) fn parse(mut cmd: Split<&str>) -> Result<ManagementCommand> {
        let cmd_name = cmd.next().unwrap();
        let arguments = cmd.collect::<Vec<&str>>();
        match cmd_name {
            "CreateProject" => {
                if arguments.len() != 1 {
                    return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid number of arguments for CreateProject"))
                }
                let project_name = arguments[0];
                let cmd = ManagementCommand::CreateProject(project_name.to_string());
                Ok(cmd)
            }
            "DeleteProject" => {
                if arguments.len() != 1 {
                    return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid number of arguments for DeleteProject"))
                }
                let project_name = arguments[0];
                let cmd = ManagementCommand::DeleteProject(project_name.to_string());
                Ok(cmd)
            }
            "LoadProject" => {
                let collection;
                if arguments.len() == 1 {
                    collection = "default".to_string();
                }
                else if arguments.len() == 2 {
                    collection = arguments[1].to_string();
                }
                else {
                    return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid number of arguments for LoadProject"))
                }
                let project_name = arguments[0];
                let cmd = ManagementCommand::LoadProject(project_name.to_string(), collection);
                Ok(cmd)
            }
            "ListProjects" => {
                if arguments.len() == 0 {
                    Ok(ManagementCommand::ListProjects("default".to_string()))
                }
                else if arguments.len() == 1 {
                    let project_name = arguments[0];
                    Ok(ManagementCommand::ListProjects(project_name.to_string()))
                }
                else {
                    return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid number of arguments for ListProjects"))
                }
            }
            "ListCollections" => {
                let cmd = ManagementCommand::ListCollections;
                Ok(cmd)
            }
            _ => {
                panic!("Invalid command `{}`", cmd_name);
            }
        }
    }
}