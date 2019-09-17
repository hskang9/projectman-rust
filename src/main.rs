use structopt::StructOpt;
use std::env;
extern crate dirs;
use serde_json::json;
use serde::{Serialize,Deserialize};
use std::fs;
#[macro_use] extern crate shell;
use dialoguer::{theme::ColorfulTheme, theme::CustomPromptCharacterTheme, Select, Input};
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;

#[derive(Serialize,Deserialize,Debug)]
struct Project {
    name: String,
    path: String,
    editor: String
}

#[derive(StructOpt)]
struct Cli {
    pattern: Option<String>,

    project: Option<String>
}

// Prompt Constant Variables
static OPEN_PROMPT: &str = "? Select project to open";
static REMOVE_PROMPT: &str = "? Select project to remove";
static SETEDITOR_PROMPT: &str = "? Select project to set editor";

    
fn main() {
    let settings_dir: String = format!("{}/.projectman/settings_rust.json", dirs::home_dir().unwrap().display());

    let default = json!({
            "commandToOpen": "code",
            "projects": []
        });
    let settings_data = serde_json::from_str(&fs::read_to_string(settings_dir.clone()).unwrap()).unwrap_or(default); 
    
    let args = Cli::from_args();
    match args.pattern {
        None => open_project(settings_data, args.project),
        Some(ref x) if x == "open" => open_project(settings_data, args.project),
        Some(ref x) if x == "add" || x == "save" => add_project(settings_data),
        Some(ref x) if x == "remove" => remove_project(settings_data),
        Some(ref x) if x == "seteditor" => set_editor(settings_data),
        Some(ref _x) => {println!("Command {} not found", _x); help()}
    }
    
}

fn list_projects(settings_data: serde_json::value::Value) -> Vec<String> {
    let mut selections = vec![];
    for i in 0..settings_data["projects"].as_array().unwrap().len() {
        let selection = settings_data["projects"][i]["name"].as_str().unwrap().to_string();
        selections.push(selection.clone());
    }
    selections
}

fn browse(prompt: &str, settings_data: serde_json::value::Value) -> String {
    let selections = list_projects(settings_data.clone());
    if selections.len() == 0 { panic!("No project found");}

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .default(0)
        .items(&selections[..])
        .interact()
        .unwrap();
    let result = &selections[selection.clone()];
    println!(">>> Opening {}...", result.clone());
    //open_project(settings_data.clone(), Some(result.to_string()));
    result.to_string()
}

fn open_project(settings_data: serde_json::value::Value, project: Option<String>) {
    match project {
        // if input is none give selections
        None => { let result = browse(OPEN_PROMPT, settings_data.clone()); open_project(settings_data, Some(result)) },
        // if input is in the list, open it
        Some(ref x) if check_existence(x.clone(), settings_data.clone())=> {
            let editor = find_project_editor(x.clone(), settings_data.clone());
            if editor != "default" {
                let command = editor;
                let path = find_project_path(project.clone().unwrap(), settings_data.clone());
                cmd!("{} {}", &command, &path).run().unwrap();
            }
            let command = settings_data["commandToOpen"].as_str().unwrap(); 
            let path = find_project_path(project.clone().unwrap(), settings_data.clone());
            cmd!("{} {}", command, &path).run().unwrap();
        },
        // if the input is not in the list, call support
        Some(ref _x) => {
            println!("Project does not exist. Add it using `pm add [projectPath]` or cd till the project folder and type `pm add`");
        }
    }
}

fn add_project(settings_data: serde_json::value::Value) {
    let mut next_settings = settings_data.clone();
    let theme = CustomPromptCharacterTheme::new('\u{2692}');
    let project_name: String = Input::with_theme(&theme)
        .with_prompt("Project name ")
        .interact()
        .unwrap();
    let path = env::current_dir();
    let new_project:Project = Project{ name: project_name, path: path.unwrap().display().to_string(), editor: settings_data["commandToOpen"].as_str().unwrap().to_string()};
    let p = serde_json::to_value(new_project).unwrap();
    next_settings["projects"].as_array_mut().unwrap().push(p);

    // Save next settings file
    println!("{:?}", next_settings);
    save_settings(next_settings);
    
}

fn save_settings(settings_data: serde_json::value::Value) {
    let settings_dir: String = format!("{}/.projectman/settings_rust.json", dirs::home_dir().unwrap().display());
    let f = serde_json::to_string(&settings_data).unwrap();
    let mut file = File::create(&settings_dir).expect("Unable to write"); 
    file.write_all(f.as_bytes()).expect("Cannot write to a file"); 
}

fn remove_project(settings_data: serde_json::value::Value) {
    let mut next_settings = settings_data.clone();

    let result = browse(REMOVE_PROMPT, settings_data);

    // Remove the project in json file
    next_settings = delete_project_json(next_settings, result.to_string());
    println!("{:?}", next_settings);
    save_settings(next_settings);
}

fn delete_project_json(mut settings_data: serde_json::value::Value, project: String) -> serde_json::value::Value {
    for i in 0..settings_data["projects"].as_array().unwrap().len() {
        let selection = settings_data["projects"][i]["name"].as_str().unwrap().to_string();
        if selection == project { settings_data["projects"].as_array_mut().unwrap().remove(i); return settings_data}
    }
    return settings_data;
}

fn set_editor(settings_data: serde_json::value::Value) {
    let mut next_settings = settings_data.clone();
    let result = browse(SETEDITOR_PROMPT, settings_data);

    let theme = CustomPromptCharacterTheme::new('>');

    let input: String = Input::with_theme(&theme)
        .with_prompt("The command to open your editor")
        .interact()
        .unwrap();

    // Set editor for the project in json file
    next_settings = seteditor_project_json(next_settings, result.to_string(), input);
    println!("{:?}", next_settings);
    save_settings(next_settings);
}

fn seteditor_project_json(mut settings_data: serde_json::value::Value, project: String, editor: String) -> serde_json::value::Value {
    for i in 0..settings_data["projects"].as_array().unwrap().len() {
        let selection = settings_data["projects"][i]["name"].as_str().unwrap().to_string();
        if selection == project { *settings_data["projects"][i].get_mut("editor").unwrap() = json!(editor); return settings_data}
    }
    return settings_data;
}

fn help() {
    print!("\nUsage: pm <command>

Options:
  -V, --version                output the version number
  -h, --help                   output usage information

Commands:
  open|o [projectName]         Open one of your saved projects
  add|save [projectDirectory]  Save current directory as a project
  remove [projectName]         Remove the project
  seteditor [commandToOpen]    Set text editor to use
  edit                         Edit settings.json\n")

}

fn find_project_path(name: String, settings_data: serde_json::value::Value) -> String {
    for i in 0..settings_data["projects"].as_array().unwrap().len() {
        let project = settings_data["projects"][i]["name"].as_str().unwrap();
        let path = settings_data["projects"][i]["path"].as_str().unwrap();
        if project == name { println!("{:?}", path); return path.to_string(); }
    }
    panic!("setting file is broken");
    return "Should not execute this".to_string();
}

fn find_project_editor(name: String, settings_data: serde_json::value::Value) -> String {
    for i in 0..settings_data["projects"].as_array().unwrap().len() {
        let project = settings_data["projects"][i]["name"].as_str().unwrap();
        let editor = settings_data["projects"][i]["editor"].as_str().unwrap();
        if project == name { println!("{:?}", editor); return editor.to_string(); }
    }
    return "default".to_string();
}


fn check_existence(name: String, setttings_data: serde_json::value::Value) -> bool {
    for i in 0..setttings_data["projects"].as_array().unwrap().len() {
        let project = setttings_data["projects"][i]["name"].as_str().unwrap();
        if project == name { return true; }
    }
    false
}