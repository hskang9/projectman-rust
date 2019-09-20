use std::env;
use structopt::StructOpt;
extern crate dirs;
use dialoguer::{theme::ColorfulTheme, theme::CustomPromptCharacterTheme, Input, Select};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::process::Command;
extern crate colored;
use colored::*;

#[derive(Serialize, Deserialize, Debug)]
struct Project {
    name: String,
    path: String,
    editor: String,
}

#[derive(StructOpt)]
struct Cli {
    pattern: Option<String>,

    project: Option<String>,
}

fn main() {
    let settings_dir: String = format!(
        "{}/.projectman/settings.json",
        dirs::home_dir().unwrap().display()
    );

    let default = json!({
        "commandToOpen": "code",
        "projects": []
    });

    // Check whether setting file exists
    if !path_exists(settings_dir.clone()) {
        println!(
            "Generating new settings file at {}...",
            settings_dir.clone()
        );
        match fs::create_dir_all(format!(
            "{}/.projectman",
            dirs::home_dir().unwrap().display()
        )) {
            Ok(_) => (),
            Err(why) => panic!("Failed to create dir: {}", why),
        }
        save_settings(default.clone());
    }

    let file_string = match fs::read_to_string(settings_dir.clone()) {
        Err(why) => {
            panic!("Setting file error at {}: {}", settings_dir.red(), why);
        }
        Ok(file) => file,
    };

    let settings_data = serde_json::from_str(&file_string).unwrap();

    let args = Cli::from_args();
    match args.pattern {
        None => open_project(settings_data, args.project),
        Some(ref x) if x == "open" => open_project(settings_data, args.project),
        Some(ref x) if x == "add" || x == "save" => add_project(settings_data),
        Some(ref x) if x == "remove" => remove_project(settings_data),
        Some(ref x) if x == "seteditor" => set_editor(settings_data),
        Some(ref _x) => {
            println!("{}", format!("Command '{}' not found", _x).red());
            help()
        }
    }
}

fn list_projects(settings_data: serde_json::value::Value) -> Vec<String> {
    let mut selections = vec![];
    for i in 0..settings_data["projects"].as_array().unwrap().len() {
        let selection = settings_data["projects"][i]["name"]
            .as_str()
            .unwrap()
            .to_string();
        selections.push(selection.clone());
    }
    selections
}

fn browse(prompt: &str, settings_data: serde_json::value::Value) -> String {
    let selections = list_projects(settings_data.clone());
    if selections.len() == 0 {
        println!("{}", format!("Project does not exist :( Add it using {} or cd till the project folder and type {}",
     "`pm add [projectPath]`".yellow(), 
     "`pm add`".yellow()).red().bold());
        panic!("No project found");
    }

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .default(0)
        .items(&selections[..])
        .interact()
        .unwrap();
    let result = &selections[selection.clone()];

    result.to_string()
}

fn open_project(settings_data: serde_json::value::Value, project: Option<String>) {
    let open_prompt: &str = &format!("{} Select project to open", "?".green());
    match project {
        // if input is none give selections
        None => {
            let result = browse(open_prompt, settings_data.clone());
            open_project(settings_data, Some(result))
        }
        // if input is in the list, open it
        Some(ref x) if project_exists(x.clone(), settings_data.clone()) => {
            let editor = find_project_editor(x.clone(), settings_data.clone());
            if editor != "default" {
                let command = editor;
                let path = find_project_path(project.clone().unwrap(), settings_data.clone());
                open_process(command, path);
            }
            let command = settings_data["commandToOpen"].as_str().unwrap();
            let path = find_project_path(project.clone().unwrap(), settings_data.clone());
            println!(">>> Opening {}...", x.green());
            open_process(command.to_string(), path);
        }
        // if the input is not in the list, call support
        Some(ref _x) => {
            let command = settings_data["commandToOpen"].as_str().unwrap();
            println!(
                "{}\n{}\n{}",
                "Could not open project :(".red().bold(),
                format!(
                    "Are you sure your editor uses command `{}` to open directories from terminal?",
                    command.yellow().bold()
                ),
                format!(
                    "If not, use {} to set Editor/IDE of your choice",
                    "`pm seteditor`".yellow().bold()
                )
            );
        }
    }
}

fn add_project(settings_data: serde_json::value::Value) {
    let mut next_settings = settings_data.clone();
    let path = env::current_dir();
    let hint = env::current_dir().unwrap().display().to_string().split("/").last().unwrap().to_string();
    let theme = CustomPromptCharacterTheme::new(':');
    let project_name: String = Input::with_theme(&theme)
        .with_prompt("Project Name \u{2692}")
        .allow_empty(true)
        .default(hint)
        .interact()
        .unwrap();
    let result = project_name.clone();

    // Check whether the project already exists
    if project_exists(result.clone(), next_settings.clone()) {
        println!("{}", format!("{}", "Project with this name already exists".red().bold()));
        return   
    }
    let new_project: Project = Project {
        name: result.clone(),
        path: path.unwrap().display().to_string(),
        editor: settings_data["commandToOpen"].as_str().unwrap().to_string(),
    };
    let p = serde_json::to_value(new_project).unwrap();
    next_settings["projects"].as_array_mut().unwrap().push(p);

    // Save next settings file
    println!(
        "{}",
        format!("Project {} is successfully added", result.cyan().bold()).green()
    );
    save_settings(next_settings);
}

fn save_settings(settings_data: serde_json::value::Value) {
    let settings_dir: String = format!(
        "{}/.projectman/settings.json",
        dirs::home_dir().unwrap().display()
    );
    let f = serde_json::to_string(&settings_data).unwrap();
    let mut file = File::create(&settings_dir).expect("Unable to write");
    file.write_all(f.as_bytes())
        .expect("Cannot write to a file");
}

fn remove_project(settings_data: serde_json::value::Value) {
    let mut next_settings = settings_data.clone();

    let remove_prompt: &str = &format!("{} Select project to remove", "?".green());
    let result = browse(remove_prompt, settings_data);

    // Remove the project in json file
    next_settings = delete_project_json(next_settings, result.to_string());
    println!(
        "{}",
        format!("Project {} is successfully removed", result.cyan().bold()).green()
    );
    save_settings(next_settings);
}

fn delete_project_json(
    mut settings_data: serde_json::value::Value,
    project: String,
) -> serde_json::value::Value {
    for i in 0..settings_data["projects"].as_array().unwrap().len() {
        let selection = settings_data["projects"][i]["name"]
            .as_str()
            .unwrap()
            .to_string();
        if selection == project {
            settings_data["projects"].as_array_mut().unwrap().remove(i);
            return settings_data;
        }
    }
    panic!("The project to remove does not exist in the settings file".red());
}

fn set_editor(settings_data: serde_json::value::Value) {
    let mut next_settings = settings_data.clone();

    let seteditor_prompt: &str = &format!("{} Select project to set editor", "?".green());
    let result = browse(seteditor_prompt, settings_data);

    let theme = CustomPromptCharacterTheme::new('>');

    let input: String = Input::with_theme(&theme)
        .with_prompt("The command to open your editor")
        .interact()
        .unwrap();

    // Set editor for the project in json file
    next_settings = seteditor_project_json(next_settings, result.to_string(), input);
    println!("{}", "Editor is successfully updated".green());
    save_settings(next_settings);
}

fn seteditor_project_json(
    mut settings_data: serde_json::value::Value,
    project: String,
    editor: String,
) -> serde_json::value::Value {
    for i in 0..settings_data["projects"].as_array().unwrap().len() {
        let selection = settings_data["projects"][i]["name"]
            .as_str()
            .unwrap()
            .to_string();
        if selection == project {
            *settings_data["projects"][i].get_mut("editor").unwrap() = json!(editor);
            return settings_data;
        }
    }
    return settings_data;
}

fn help() {
    print!(
        "\nUsage: pm <command>

Options:
  -V, --version                output the version number
  -h, --help                   output usage information

Commands:
  open|o [projectName]         Open one of your saved projects
  add|save [projectDirectory]  Save current directory as a project
  remove [projectName]         Remove the project
  seteditor [commandToOpen]    Set text editor to use
  edit                         Edit settings.json\n"
    )
}

fn find_project_path(name: String, settings_data: serde_json::value::Value) -> String {
    for i in 0..settings_data["projects"].as_array().unwrap().len() {
        let project = settings_data["projects"][i]["name"].as_str().unwrap();
        let path = settings_data["projects"][i]["path"].as_str().unwrap();
        if project == name {
            return path.to_string();
        }
    }
    panic!("setting file is broken".red());
}

fn find_project_editor(name: String, settings_data: serde_json::value::Value) -> String {
    for i in 0..settings_data["projects"].as_array().unwrap().len() {
        let project = settings_data["projects"][i]["name"].as_str().unwrap();
        let editor = settings_data["projects"][i]["editor"].as_str().unwrap();
        if project == name {
            return editor.to_string();
        }
    }
    return "default".to_string();
}

fn project_exists(name: String, setttings_data: serde_json::value::Value) -> bool {
    for i in 0..setttings_data["projects"].as_array().unwrap().len() {
        let project = setttings_data["projects"][i]["name"].as_str().unwrap();
        if project == name {
            return true;
        }
    }
    false
}

fn open_process(command: String, path: String) {
    Command::new(&command)
        .arg(&path)
        .spawn()
        .expect("Failed to process editor process");
}

fn path_exists(path: String) -> bool {
    match fs::metadata(&path) {
        Ok(_some) => true,
        Err(_) => false,
    }
}
