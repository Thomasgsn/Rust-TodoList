use core::time;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::process::Command;
use std::thread;

fn clear_console() {
    if cfg!(target_os = "windows") {
        Command::new("cmd").args(&["/C", "cls"]).status().unwrap();
    } else {
        Command::new("clear").status().unwrap();
    }
}

struct Item {
    text: String,
    status: Status,
    person: Option<String>,
}

#[derive(Clone, Copy)]
enum Status {
    Pending,
    Finished,
    Stopped,
    Completed,
    ToDo,
}

impl Status {
    fn symbol(&self) -> char {
        match self {
            Status::Pending => 'O',
            Status::Finished => '✓',
            Status::Stopped => '!',
            Status::Completed => '#',
            Status::ToDo => '.',
        }
    }

    fn from_str(s: &str) -> Self {
        match s {
            "✓" => Status::Finished,
            "O" => Status::Pending,
            "!" => Status::Stopped,
            "#" => Status::Completed,
            "." => Status::ToDo,
            _ => Status::Pending,
        }
    }

    fn to_str(&self) -> &'static str {
        match self {
            Status::Pending => "O",
            Status::Finished => "✓",
            Status::Stopped => "!",
            Status::Completed => "#",
            Status::ToDo => ".",
        }
    }

    fn color(&self) -> String {
        match self {
            Status::Pending => "\x1b[93m".to_string(),
            Status::Completed => "\x1b[92m".to_string(),
            Status::Finished => "\x1b[94m".to_string(),
            Status::Stopped => "\x1b[91m".to_string(),
            Status::ToDo => "\x1b[90m".to_string(),
        }
    }
}

impl Item {
    fn check_task(&mut self, to: &str) {
        match to {
            "pending" => self.status = Status::Pending,
            "stop" => self.status = Status::Stopped,
            "complete" => self.status = Status::Completed,
            "finish" => self.status = Status::Finished,
            "todo" => self.status = Status::ToDo,
            _ => {}
        }
    }

    fn display(&self, i: usize, start: usize) {
        let sufixe = "\x1b[0m";

        let person_str = match &self.person {
            Some(p) if !p.is_empty() => format!(" (for: {})", p),
            _ => "".to_string(),
        };

        println!(
            "{}{}. [{}] {}{}{}",
            self.status.color(),
            start + i + 1,
            self.status.symbol(),
            self.text,
            person_str,
            sufixe
        );
    }
}

fn load_list(filename: &str) -> Vec<Item> {
    let mut list = Vec::new();

    if let Ok(file) = File::open(filename) {
        let reader = BufReader::new(file);

        for line in reader.lines() {
            if let Ok(line) = line {
                let parts: Vec<&str> = line.splitn(3, '|').collect();
                if parts.len() == 3 {
                    let status = Status::from_str(parts[0]);
                    let text = parts[1].to_string();
                    let person_raw = parts[2].trim().to_string();
                    let person = if person_raw.is_empty() {
                        None
                    } else {
                        Some(person_raw)
                    };
                    list.push(Item {
                        text,
                        status,
                        person,
                    });
                }
            }
        }
    }

    list
}

fn save_list(filename: &str, list: &Vec<Item>) {
    let mut file = File::create(filename).expect("Cannot create file");

    for item in list {
        writeln!(
            file,
            "{}|{}|{}",
            item.status.to_str(),
            item.text,
            item.person.clone().unwrap_or_default()
        )
        .unwrap();
    }
}

fn get_user_input(text: String) -> String {
    print!("{text}");
    io::stdout().flush().unwrap();

    let mut choice = String::new();
    io::stdin().read_line(&mut choice).unwrap();
    let choice = choice.trim();

    choice.to_string()
}

fn intro() {
    clear_console();
    println!("Welcome back boss !");
    thread::sleep(time::Duration::from_millis(750));
}
fn outro() {
    clear_console();
    println!("Salam boss !");
    thread::sleep(time::Duration::from_millis(1000));
    clear_console();
}

fn main() {
    intro();

    let filename = "todo.txt";
    let mut list = load_list(filename);
    let mut current_page = 0;
    const ITEMS_PER_PAGE: usize = 9;

    loop {
        clear_console();

        println!("\n========== LIST ==========\n");
        let start = current_page * ITEMS_PER_PAGE;
        let end = (start + ITEMS_PER_PAGE).min(list.len());
        if start >= list.len() {
            println!("(No task to display)");
        } else {
            for (i, item) in list[start..end].iter().enumerate() {
                item.display(i, start);
            }
        }
        println!("\n==============================\n");

        if current_page > 0 {
            println!(" a : Previous page");
        }
        if end < list.len() {
            println!(" e : Next page");
        }

        if current_page > 0 || end < list.len() {
            print!("\n");
        }

        println!(" 1 : Add a task");

        if list.len() > 0 {
            println!(" 2 : Change status of a task");
            println!(" 3 : Remove a task");
            println!(" 4 : Edit text of a task");
            println!(" 5 : Edit person of a task");
        }

        println!(" q : Quit");

        let choice = get_user_input(String::from("\nChoice : "));

        match choice.as_str() {
            "1" => {
                let item_text = get_user_input("Task name: ".to_string());
                let item_person = get_user_input("For who: ".to_string());

                let item = Item {
                    text: item_text.trim().to_string(),
                    status: Status::ToDo,
                    person: {
                        let p = item_person.trim().to_string();
                        if p.is_empty() { None } else { Some(p) }
                    },
                };

                list.push(item);
                save_list(filename, &list);
            }

            "2" => {
                let index = get_user_input("Which task (id): ".to_string());

                if let Ok(i) = index.trim().parse::<usize>() {
                    if let Some(item) = list.get_mut(i - 1) {
                        let new_status = get_user_input(
                            "Set new status (pending / stop / complete / finish / todo): "
                                .to_string(),
                        );

                        item.check_task(&new_status.trim().to_lowercase());
                        save_list(filename, &list);
                    }
                }
            }

            "3" => {
                let index = get_user_input("Which task (id): ".to_string());
                if let Ok(i) = index.trim().parse::<usize>() {
                    if i > 0 && i <= list.len() {
                        list.remove(i - 1);
                        save_list(filename, &list);
                    }
                }
            }

            "4" => {
                let index = get_user_input("Which task (id): ".to_string());
                if let Ok(i) = index.trim().parse::<usize>() {
                    if let Some(item) = list.get_mut(i - 1) {
                        let new_text = get_user_input("Rename to: ".to_string());
                        item.text = new_text.trim().to_string();
                        save_list(filename, &list);
                    }
                }
            }

            "5" => {
                let index = get_user_input("Which task (id): ".to_string());
                if let Ok(i) = index.trim().parse::<usize>() {
                    if let Some(item) = list.get_mut(i - 1) {
                        let new_person =
                            get_user_input("Enter new person (empty to clear): ".to_string());
                        item.person = if new_person.trim().is_empty() {
                            None
                        } else {
                            Some(new_person)
                        };
                        save_list(filename, &list);
                    }
                }
            }

            "a" => {
                if current_page > 0 {
                    current_page -= 1;
                }
            }

            "e" => {
                if (current_page + 1) * ITEMS_PER_PAGE < list.len() {
                    current_page += 1;
                }
            }

            "q" | "quit" | "exit" => {
                outro();
                break;
            }

            _ => {
                println!("Invalid option!");
                thread::sleep(time::Duration::from_millis(1000));
            }
        }
    }
}
