use structopt::StructOpt;
use std::env;
use serde_json::json;
use std::fs;
#[macro_use] extern crate shell;
use dialoguer::{theme::ColorfulTheme, theme::CustomPromptCharacterTheme, Select, Input};

#[derive(StructOpt)]
struct Cli {
    pattern: Option<String>,

    project: Option<String>
}

struct Project {
    name: String,
    path: String,
    editor: String
}

fn main() {
    let default = json!({
            "commandToOpen": "code",
            "projects": []
        });
    let settings_dir = format!("{}/.projectman/settings.json", env::home_dir().unwrap().display());
    let settings_data = serde_json::from_str(&fs::read_to_string(settings_dir).unwrap()).unwrap_or(default); 
    
    let args = Cli::from_args();
    match args.pattern {
        None => browse(settings_data),
        Some(ref x) if x == "open" => open_project(settings_data, args.project),
        Some(ref x) if x == "add" || x == "save" => add_project(settings_data),
        Some(ref x) if x == "remove" => remove_project(settings_data),
        Some(ref x) if x == "seteditor" => set_editor(settings_data),
        Some(ref _x) => {println!("Command {} not found", _x); help()}
    }
    
}

fn browse(settings_data: serde_json::value::Value) {
    let mut selections = vec![];
    for i in 0..settings_data["projects"].as_array().unwrap().len() {
        let selection = settings_data["projects"][i]["name"].as_str().unwrap();
        selections.push(selection.clone());
    }

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("? Select project to open")
        .default(0)
        .items(&selections[..])
        .interact()
        .unwrap();
    let result = selections[selection.clone()];
    println!("Opening {}...", result.clone());
    open_project(settings_data.clone(), Some(result.to_string()));
}

fn open_project(settings_data: serde_json::value::Value, project: Option<String>) {
    match project {
        // if input is none give selections
        None => browse(settings_data),
        // if input is in the list, open it
        Some(ref x) if check_existence(x.clone(), settings_data.clone())=> {
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
}

fn remove_project(settings_data: serde_json::value::Value) {

}

fn set_editor(settings_data: serde_json::value::Value) { 

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


fn check_existence(name: String, setttings_data: serde_json::value::Value) -> bool {
    for i in 0..setttings_data["projects"].as_array().unwrap().len() {
        let project = setttings_data["projects"][i]["name"].as_str().unwrap();
        if project == name { println!("{:?}", true); return true; }
    }
    false
}