use structopt::StructOpt;
use std::env;
use serde_json::json;
use std::fs;
#[macro_use] extern crate shell;

#[derive(StructOpt)]
struct Cli {
    pattern: Option<String>,
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
        Some(ref x) if x == "open" => open_project(settings_data),
        Some(ref x) if x == "add" || x == "save" => add_project(settings_data),
        Some(ref x) if x == "remove" => remove_project(settings_data),
        Some(ref x) if x == "seteditor" => set_editor(settings_data),
        Some(ref x) => help(x.to_string())
    }
    
}

fn browse(settings_data: serde_json::value::Value) {
    let path = env::current_dir();

    for i in 0..settings_data["projects"].as_array().unwrap().len() {
        println!("{:?}", settings_data["projects"][i]);
    }
}

fn open_project(settings_data: serde_json::value::Value) {
    let command = settings_data["commandToOpen"].as_str().unwrap(); 
    cmd!("{}", command).run().unwrap();
}

fn add_project(settings_data: serde_json::value::Value) {

}

fn remove_project(settings_data: serde_json::value::Value) {

}

fn set_editor(settings_data: serde_json::value::Value) {

}

fn help(x: String) {
    println!("Command {} not found", x);
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
