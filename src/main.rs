#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    // Parser tests
    #[test]
    fn empty_command() {
        assert_eq!(::parse_user_input("  \t  "), ::Command::Empty);
    }

    #[test]
    fn illegal_command() {
        assert!(is_illegal_command(&::parse_user_input("delete")));
        assert!(is_illegal_command(&::parse_user_input("add kyle to kyle"))); // kyle dept missing
        assert!(is_illegal_command(&::parse_user_input("list foo"))); // foo dept does not exist
    }

    #[test]
    fn empty_company() {
        let cmd = ::parse_user_input("list");
        assert_eq!(::react(&mut company_factory(), cmd), String::new());
    }

    #[test]
    fn company_one_employee() {
        let cmd = ::parse_user_input("add Sam to HR");
        let mut comp = company_factory();
        ::react(&mut comp, cmd); // Ignore the string here, we only want to check that the state
                                 // for the Company comp has changed
        let cmd = ::parse_user_input("list");
        assert_eq!(::react(&mut comp, cmd), "HR\n\tSam\n\n".to_string());
    }

    #[test]
    fn company_many_employees() {
        let mut comp = company_factory();
        for cmd in vec![::parse_user_input("add Sam to HR"),
                         ::parse_user_input("add Kyle to Finance"),
                         ::parse_user_input("add Annie to Finance"),
                         ::parse_user_input("add Bobby to Sales")] {
            ::react(&mut comp, cmd);
        }
        let cmd = ::parse_user_input("list");
        assert_eq!(::react(&mut comp, cmd), "Finance\n\tAnnie\n\tKyle\n\nHR\n\tSam\n\nSales\n\tBobby\n\n".to_string());
        let cmd = ::parse_user_input("list Finance");
        assert_eq!(::react(&mut comp, cmd), "Annie\nKyle\n".to_string());
    }

    fn company_factory() -> ::Company {
        ::Company  { name : String::from("Testers, Inc."), employees : BTreeMap::new() }
    }

    fn is_illegal_command(cmd : &::Command) -> bool {
        match cmd {
            ::Command::Illegal(_) => true,
            _ => false
        }
    }
}

#[macro_use]
extern crate custom_derive;
#[macro_use]
extern crate enum_derive;

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord)]
struct Employee {
    first_name : String
}

impl std::fmt::Display for Employee {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.first_name)
    }
}
use std::collections::{BTreeMap, BTreeSet};
type Employees = BTreeMap<Department, BTreeSet<Employee>>;
struct Company {
    name : String,
    employees : Employees
}


custom_derive! {
    #[derive(Debug, EnumFromStr, PartialEq, Eq)]
    enum Department {
        Accounting,
        CustomerService,
        Marketing,
        HR,
        Sales,
        IT,
        QA,
        Finance
    }
}

// By default, the Ord trait for enums is defined by top-to-bottom declaration of its variants.
// We really do not want this in this case. Instead, order variants by the variant name.
// https://doc.rust-lang.org/std/cmp/trait.Ord.html#derivable
use std::cmp::Ordering;
impl Ord for Department {
    fn cmp(&self, other: &Department) -> Ordering {
        format!("{:?}", self).cmp(&format!("{:?}", other))
    }
}

impl PartialOrd for Department {
    fn partial_cmp(&self, other: &Department) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, PartialEq)]
enum Command {
    Empty,
    Quit,
    Add(Employee, Department),
    List(Department),
    ListAll,
    Illegal(&'static str) // String contains the reason why the parse failed
}

fn dept_from_string(token : &str) -> Option<Department> {
    match token.parse::<Department>() {
        Ok(dept) => Some(dept),
        Err(_) => None
    }
}

fn add_cmd(mut it : std::str::SplitWhitespace) -> Command {
    match it.next() {
       None => Command::Illegal("You must provide an employee name."),
       Some(token) => {
           let empl = Employee { first_name : token.to_string() };
           match it.next() {
               None => Command::Illegal("Add command syntax : add <NAME> to <DEPT>"),
               Some("to") => {
                   match it.next() {
                       None => Command::Illegal("You must provide a department the employee belongs to."),
                       Some(token) => {
                           if let None = it.next() {
                               if let Some(dept) = dept_from_string(token) {
                                   return Command::Add(empl, dept);
                               }
                               return Command::Illegal("Department not found.");
                           }
                           Command::Illegal("Found too many tokens for the add command")
                       }
                   }
               },
               _ => Command::Illegal("Expected 'to' separator.")
           }
       }
    }
}

// The accepted commands are the following:
//
// Add <NAME> to <DEPT>
// List <DEPT>
// List
// Quit
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
                    match token.parse() {
                        Ok(dept) =>  Command::List(dept), // rustc can infer type here because
                                                          // the only possible type is valid
                        Err(_) => Command::Illegal("Department not found.")
                    }
                }
            }
        },
        Some("quit") => Command::Quit,
        None => Command::Empty,
        _ => Command::Illegal("Unknown command.")
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

use std::collections::btree_map::Entry;
fn add_employee(comp : &mut Company, emp : Employee, dept : Department) {
    match comp.employees.entry(dept) {
        Entry::Vacant(e) => {
            let mut new_value = BTreeSet::new();
            new_value.insert(emp);
            e.insert(new_value);
        }, // we must use surrounding brackets here
        Entry::Occupied(mut e) => {
            e.get_mut().insert(emp);
        }
    }
}

fn list_for_department(comp : &Company, dept : &Department) -> String {
    match comp.employees.get(&dept) {
        Some(v) => {
            let mut res = String::new();
            for emp in v.iter() {
                res.push_str(&format!("{}\n", emp));
            }
            res
        },
        None => "There are no employees assigned to this department.".to_string()
    }
}

fn list_all(comp : &Company) -> String {
    let mut res = String::new();
    for (dept, emps) in &comp.employees {
        res.push_str(&format!("{:?}\n", dept));
        for emp in emps.iter() {
            res.push_str(&format!("\t{}\n", emp));
        }
        res.push_str("\n");
    }
    res
}

fn react(comp : &mut Company, cmd :Command) -> String {
    use Command::*;
    match cmd {
        Add(emp, dept) => {
            let res = format!("Successfully added {} into {:?}.", emp, dept);
            add_employee(comp, emp, dept); // move (into the Company struct) happens here
            res
        },
        List(dept) => list_for_department(comp, &dept),
        ListAll => list_all(&comp),
        Illegal(msg) => msg.to_string(),
        Empty => String::new(),
        _ => panic!("Unexpected command.")
    }
}

fn main() {
    let mut company = Company  { name : String::from("Giggle, Inc."), employees : BTreeMap::new() };
    loop {
        let next_cmd = wait_for_command();
        if let Command::Quit = next_cmd {
            std::process::exit(0);
        } else {
            println!("{}", react(&mut company, next_cmd));
        }
    }
}
