mod task;
use crate::task::Task;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;

type ParseError = &'static str;

#[derive(Debug)]
enum Operation {
    Add,
    Remove,
    Complete,
    List,
}

impl FromStr for Operation {
    type Err = ParseError;
    fn from_str(filter: &str) -> Result<Self, Self::Err> {
        match filter {
            "add" => Ok(Operation::Add),
            "remove" => Ok(Operation::Remove),
            "complete" => Ok(Operation::Complete),
            "list" => Ok(Operation::List),
            _ => Err("Could not parse filter"),
        }
    }
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
enum Filter {
    None,
    Pending,
    Completed,
}

impl FromStr for Filter {
    type Err = ParseError;
    fn from_str(filter: &str) -> Result<Self, Self::Err> {
        match filter {
            "none" => Ok(Filter::None),
            "" => Ok(Filter::None),
            "pending" => Ok(Filter::Pending),
            "completed" => Ok(Filter::Completed),
            _ => Err("Could not parse filter"),
        }
    }
}

impl fmt::Display for Filter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(StructOpt, Debug)]
#[structopt(rename_all = "kebab-case", name = "Task-cli", about = "Task-cli usage")]
struct Opt {
    /// Operation (-o, --operation) add, remove, complete, list
    #[structopt(short, long, rename_all = "lower")]
    operation: Operation,

    /// ID (int) of task for completion or removal
    #[structopt(
        short,
        long,
        required_if("operation", "remove"),
        required_if("operation", "complete")
    )]
    id: Option<usize>,

    /// Description (-d --description) of Task item
    #[structopt(short, long, required_if("operation", "add"))]
    description: Option<String>,

    /// Listing filter for tasks: none (default), pending, completed
    #[structopt(
        short,
        long,
        required_if("operation", "list"),
        default_value = "none",
        rename_all = "lower"
    )]
    filter: Filter,

    /// Path to Task file otherwise defaults to Task.json in the binary's directory
    #[structopt(short, long, parse(from_os_str))]
    json: Option<PathBuf>,
}

fn deserialize_tasks(path: PathBuf) -> Result<BTreeMap<usize, Task>, Box<dyn Error>> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)?;
    match serde_json::from_reader(file) {
        Ok(tasks) => Ok(vec_to_map(tasks)),
        Err(e) if e.is_eof() => Ok(BTreeMap::new()),
        Err(e) => panic!("An error occurred: {}", e),
    }
}

fn vec_to_map(tasks: Vec<Task>) -> BTreeMap<usize, Task> {
    let mut map = BTreeMap::new();
    let mut i = 1;
    for t in tasks {
        map.insert(i, t);
        i += 1;
    }
    return map;
}

fn serialize_tasks(path: PathBuf, tasks: Vec<Task>) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    match serde_json::to_writer(file, &tasks) {
        Ok(()) => Ok(()),
        Err(e) => panic!("An error occurred: {}", e),
    }
}

fn add_task(description: String, mut tasks: BTreeMap<usize, Task>, path: PathBuf) -> () {
    let key = tasks.len() + 1;
    tasks.insert(key, Task::new(description));
    serialize_tasks(path, tasks.values().cloned().collect()).expect("Failed to write to file");
}

fn remove_task(id: usize, mut tasks: BTreeMap<usize, Task>, path: PathBuf) -> () {
    tasks.remove(&id);
    serialize_tasks(path, tasks.values().cloned().collect()).expect("Failed to write to file");
}

fn complete_task(id: usize, mut tasks: BTreeMap<usize, Task>, path: PathBuf) -> () {
    let mut task = tasks.get(&id).expect("Task not found").clone();
    task.complete();
    tasks.insert(id, task);
    serialize_tasks(path, tasks.values().cloned().collect()).expect("Failed to write to file");
}

fn print_tasks(tasks: BTreeMap<usize, Task>, filter: Filter) -> () {
    let header = format!("Task (filter: {})", filter.to_string().to_lowercase());
    let line_break = (0..header.len()).map(|_| "─").collect::<String>();
    println!("{}", header);
    println!("{}", line_break);

    tasks
        .iter()
        .filter(|(_id, task)| match filter {
            Filter::Completed => task.status == task::COMPLETED,
            Filter::Pending => task.status == task::PENDING,
            Filter::None => true,
        })
        .for_each(|(id, task)| {
            println!("{} - {} [{}]", id, task.description, task.status);
        });
}

fn main() {
    let opt = Opt::from_args();
    let operation = opt.operation;
    let default_path = dirs::document_dir().unwrap().with_file_name("todo.json");
    let path = opt.json.unwrap_or_else(|| default_path);
    let tasks = deserialize_tasks(path.clone()).expect("Error deserializing tasks");
    match operation {
        Operation::Add => add_task(opt.description.expect("Description missing."), tasks, path),
        Operation::Remove => remove_task(opt.id.expect("Task id missing"), tasks, path),
        Operation::Complete => complete_task(opt.id.expect("Task id missing"), tasks, path),
        Operation::List => print_tasks(tasks, opt.filter),
    }
}
