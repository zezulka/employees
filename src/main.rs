#[macro_use]
extern crate custom_derive;
#[macro_use]
extern crate enum_derive;

#[derive(Debug, Eq, PartialEq, Hash)]
struct Employee {
    first_name : String
}

use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::hash_map::Entry;
type Company = HashMap<Department, HashSet<Employee>>;

custom_derive! {
    #[derive(Debug, EnumFromStr, PartialEq, Eq, Hash)]
    enum Department {
        Accounting, Marketing, CustomerService, HR, Sales, IT, QA, Finance
    }
}

enum Command {
    Empty,
    Add(Employee, Department),
    List(Department),
    ListAll,
    Illegal(String) // contains message containing the reason
}

fn dept_from_string(token : &str) -> Option<Department> {
    match token.parse::<Department>() {
        Ok(dept) => Some(dept),
        Err(_) => None
    }
}

fn add_cmd(mut it : std::str::SplitWhitespace) -> Command {
    match it.next() {
       None => Command::Illegal("You must provide an employee name.".to_string()),
       Some(token) => {
           let empl = Employee { first_name : token.to_string() };
           match it.next() {
               None => Command::Illegal("Add command syntax : add <NAME> to <DEPT>".to_string()),
               Some("to") => {
                   match it.next() {
                       None => Command::Illegal("You must provide a department the employee belongs to.".to_string()),
                       Some(token) => {
                           if let None = it.next() {
                               if let Some(dept) = dept_from_string(token) {
                                   return Command::Add(empl, dept);
                               }
                               return Command::Illegal("Department not found.".to_string());
                           }
                           Command::Illegal("Found too many tokens for the add command".to_string())
                       }
                   }
               },
               _ => Command::Illegal("Expected 'to' separator.".to_string())
           }
       }
    }
}

// The accepted commands are the following:
//
// Add <NAME> to <DEPT>
// List <DEPT>
// List
//
// Spaces between tokens can be of arbitrary positive size.
fn parse_user_input(input : &str) -> Command {
    let mut it = input.split_whitespace();
    match it.next() {
        Some("add") => add_cmd(it),
        Some("list") => {
            match it.next() {
                None => Command::ListAll,
                Some(token) => {
                    let dept : Department = token.parse().unwrap();
                    Command::List(dept)
                }
            }
        },
        None => Command::Empty,
        _ => Command::Illegal("Unknown command.".to_string())
    }
}

// This is a blocking method. Waits for a valid user command (until user inputs a newline
// and submits a valid command).
fn wait_for_command() -> Command {
    use std::io;
    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input)
            .expect("Failed to read user input.");
        input.trim();
        if input.is_empty() {
            continue;
        }
        use Command::*;
        let ui = parse_user_input(&input);
        if let Illegal(msg) = ui {
            println!("{}", msg);
            continue;
        }
        return ui;
    }
}

fn add_employee(comp : &mut Company, emp : Employee, dept : Department) {
    match comp.entry(dept) {
        Entry::Vacant(e) => {
            let mut new_value = HashSet::new();
            new_value.insert(emp);
            e.insert(new_value);
        }, // we must use surrounding brackets here
        Entry::Occupied(mut e) => {
            e.get_mut().insert(emp);
        }
    }
}

fn list_for_department(comp : &Company, dept : &Department) {
    match comp.get(&dept) {
        Some(v) => {
            for emp in v.iter() {
                println!("{:?}", emp)
            }
        },
        None => println!("There are no employees assigned to this department.")
    }
}

fn list_all(comp : &Company) {
    for (key, emps) in comp.iter() {
        println!("{:?}", key);
        for emp in emps.iter() {
            println!("\t{:?}", emp)
        }
    }
}

fn main() {
    let mut company : Company = HashMap::new();
    loop {
        use Command::*;
        match wait_for_command() {
            Add(emp, dept) => add_employee(&mut company, emp, dept),
            List(dept) => list_for_department(&company, &dept),
            ListAll => list_all(&company),
            Illegal(msg) => println!("{}", msg),
            Empty => ()
        }
    }
}
