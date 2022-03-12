use std::{env, fmt::{Display, Formatter}, path::Path, fs};

use chrono::{DateTime, Local, FixedOffset};
use directories::BaseDirs;
use regex::Regex;

#[derive(Debug)]
enum TaskError {
    NotFound,
}

impl Display for TaskError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Task not found")
    }
}

struct Task {
    text: String,
    created: DateTime<FixedOffset>,
    completed: Option<DateTime<FixedOffset>>,
}

impl Task {
    fn from_string(string: String) -> Task {
        Task{text: string, created: DateTime::from(Local::now()) , completed: None}
    }
}

impl Display for Task {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.completed.is_none() {
            true => write!(f, "{}", self.text),
            false => write!(f, "X {}", self.text),
        }
    }
}

struct Tasks {
    tasks: Vec<Task>
}

trait TaskSelector {
    fn matches(&self, tasks: &Tasks, index: usize) -> bool;
}

struct AllSelector {}

struct PatternSelector {
    pattern: Regex
}

struct RangeSelector {
    from: usize,
    to: usize,
}

struct IndexSelector {
    index: usize
}

impl TaskSelector for PatternSelector {
    fn matches(&self, tasks: &Tasks, index: usize) -> bool {
        if let Some(content) = &tasks.tasks.get(index) {
            return self.pattern.is_match(&content.text)
        }
        false
    }
}

impl TaskSelector for IndexSelector {
    fn matches(&self, _tasks: &Tasks, index: usize) -> bool {
        index == self.index
    }
}

impl TaskSelector for RangeSelector {
    fn matches(&self, _tasks: &Tasks, index: usize) -> bool {
        index >= self.from && index <= self.to
    }
}

impl TaskSelector for AllSelector {
    fn matches(&self, _tasks: &Tasks, _index: usize) -> bool {
        true
    }
}

enum EmptyBehaviour {
    SelectLast,
    SelectAll,
}

enum DoneHandling {
    Show,
    Hide,
}

fn selector_from_string(string: &String, empty: EmptyBehaviour) -> Box<dyn TaskSelector> {
    if string == "" {
        match empty {
            EmptyBehaviour::SelectLast => return Box::new(IndexSelector{index: 0}),
            EmptyBehaviour::SelectAll => return Box::new(AllSelector{})
        }
    }
    if let Ok(index) = string.parse::<u32>() {
        return Box::new(IndexSelector{index: index as usize - 1});
    }
    if let Some((a, b)) = string.split_once("-") {
        if let (Ok(a), Ok(b)) = (a.parse::<u32>(), b.parse::<u32>()) {
            return Box::new(RangeSelector{from:(a as usize - 1), to:(b as usize - 1)});
        }
    }
    Box::new(PatternSelector{pattern: Regex::new(string).expect("Invalid regex")})
}

enum TaskFileError {
    NotFound,
    MissingColumn,
    ParseColmn,
    WriteColumn,
}

impl Display for TaskFileError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            TaskFileError::NotFound => "Task file not found",
            TaskFileError::MissingColumn => "Task file missing column",
            TaskFileError::ParseColmn => "Failed to parse task",
            TaskFileError::WriteColumn => "Failed to write task",
        })
    }
}

const TIME_FORMAT: &str = "%+";
//const TIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S";

impl Tasks {
    fn load(&mut self, path: &Path) -> Result<(), TaskFileError> {
        for result in csv::Reader::from_path(path).map_err(|_| TaskFileError::NotFound)?.records() {
            if let Ok(record) = result {
                if record.len() < 3 {
                    return Err(TaskFileError::MissingColumn);
                }
                self.tasks.push(Task{
                    text: record[0].to_string(),
                    created: DateTime::parse_from_str(&record[1], TIME_FORMAT).map_err(|_| TaskFileError::ParseColmn)?,
                    completed: DateTime::parse_from_str(&record[2], TIME_FORMAT).ok(),
                })
            }
        }
        Ok(())
    }

    fn save(self, path: &Path) -> Result<(), TaskFileError> {
        if let Ok(mut writer) = csv::Writer::from_path(path) {
            writer.write_record(["text", "created", "completed"])
                .map_err(|_| TaskFileError::WriteColumn)?;
            for task in self.tasks {
                writer.write_record([
                    task.text,
                    task.created.format(TIME_FORMAT).to_string(),
                    match task.completed {
                        Some(time) => time.format(TIME_FORMAT).to_string(),
                        _ => "".to_string(),
                    }
                ]).map_err(|_| TaskFileError::WriteColumn)?;
            }
            writer.flush().map_err(|_| TaskFileError::WriteColumn)?;
        } else {
            return Err(TaskFileError::NotFound);
        }
        Ok(())
    }
    
    fn print_task(&self, task: usize) {
        println!("{} {}", task + 1, self.tasks[task]);
    }

    fn status(&self) {
        println!("Tasks:");
        for (num, task) in self.tasks.iter().enumerate() {
            if task.completed.is_none() {
                self.print_task(num);
            }
        }
    }

    fn select(&self, selector: &(impl TaskSelector + ?Sized), done: DoneHandling) -> Vec<usize> {
        let mut selected: Vec<usize> = vec![];
        for (task_num, task) in self.tasks.iter().enumerate() {
            if match done {
                        DoneHandling::Show => true,
                        DoneHandling::Hide => task.completed.is_none(),
                    } {
                if selector.matches(&self, task_num) {
                    selected.push(task_num);
                }
            }
        }
        selected
    }

    fn create(&mut self, task: Task) {
        println!("Created new task: {}", task);
        self.tasks.insert(0, task)
    }

    fn work_on(&mut self, task: usize) -> Result<(),TaskError> {
        if let Some(content) = self.tasks.get(task) {
            println!("working on {}!", content);
            let working = self.tasks.remove(task);
            self.tasks.insert(0, working);
            Ok(())
        } else {
            Err(TaskError::NotFound)
        }
    }

    fn complete(&mut self, num: usize) -> Result<(),TaskError> {
        if self.tasks.get(num).is_some() {
            let mut task = self.tasks.remove(num);
            println!("completed {}!", task);
            task.completed = Some(DateTime::from(Local::now()));
            self.tasks.push(task);
            Ok(())
        } else {
            Err(TaskError::NotFound)
        }
    }
}

fn main() {
    let user_dir = BaseDirs::new().unwrap().data_local_dir().join("td-todo");
    if !user_dir.exists() {
        fs::create_dir_all(&user_dir).expect("Couldn't create application folder");
    }
    let tasks_file = user_dir.clone().join("tasks.csv");
    let mut tasks = Tasks{tasks:vec![]};
    if let Err(error) = tasks.load(&tasks_file) {
        println!("Error loading tasks: {}", error);
    }
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);
    match args.first() {
        Some(action) => {
            let rest = &args[1..].join(" ");
            match action.as_str() {
                "done" => {
                    tasks.select(
                        &*selector_from_string(rest, EmptyBehaviour::SelectLast),
                        DoneHandling::Hide)
                        .iter()
                        .for_each(|t| tasks.complete(*t)
                        .unwrap());
                    tasks.status();
                }
                "do" => {
                    match tasks.select(
                            &*selector_from_string(rest, EmptyBehaviour::SelectLast),
                            DoneHandling::Hide)
                            .first() {
                        Some(task) => {
                            if let Err(error) = tasks.work_on(*task) {
                                println!("Error doing task: {error}")
                            }
                        },
                        None => println!("Task not found"),
                    }
                }
                "show" => tasks.select(
                        &*selector_from_string(rest, EmptyBehaviour::SelectAll),
                        DoneHandling::Show)
                        .iter().for_each(|t| tasks.print_task(*t)),
                _ => {
                    for text in args.join(" ").split(",") {
                        tasks.create(Task::from_string(text.to_string()));
                    }
                    tasks.status();
                }
            }
        }
        None => tasks.status()
    }
    if let Err(error) = tasks.save(&tasks_file) {
        println!("Error saving tasks: {}", error)
    }
}
