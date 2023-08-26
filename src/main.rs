use pest::iterators::Pair;
use std::collections::HashMap;
use std::io::Write;
use std::fs::File;
use pest::iterators::Pairs;
use clap::Parser as ClapParser;
use pest::Parser;
use pest_derive::Parser;
use std::fs;

/// Convert .ptq files to .txt for the Suika2 engine
#[derive(ClapParser, Debug)]
#[command(author = None, version = None, about = None, long_about = None)]
struct Args {
    /// Input directory 
    #[arg(short, long)]
    input: Option<String>,

    /// Output directory 
    #[arg(short, long)]
    output: Option<String>,
}

#[derive(Parser)]
#[grammar = "suika.pest"]
pub struct SuikaParser;

struct IfHandler {
    if_stack: Vec<u32>,
    if_max: u32,
    next_id: u32,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let input_dir = if let Some(input) = args.input { input.to_owned() } else { "./".to_owned() };
    let output_dir = if let Some(output) = args.output { output.to_owned() } else { "./".to_owned() };
    let mut variables = HashMap::<String, u32>::new();
    let mut if_handler = IfHandler { if_stack: vec![], if_max: 0, next_id: 0 };

    process_directory(&input_dir, &input_dir, &output_dir, &mut variables, &mut if_handler)?;

    Ok(())
}

fn process_directory(root_dir: &str, input_dir: &str, output_dir: &str, variables: &mut HashMap::<String, u32>, if_handler: &mut IfHandler) -> std::io::Result<()> {
    let full_output_dir = format!("{}{}", output_dir, input_dir.to_string().strip_prefix(root_dir).unwrap());
    fs::create_dir_all(&full_output_dir)?;
    for path in fs::read_dir(input_dir).unwrap() {
        let pp = path.unwrap().path();
        let filename = pp.file_stem().unwrap().to_str().unwrap();

        if pp.is_file() {
            let extension = pp.extension().unwrap().to_str().unwrap();

            if extension == "ptq" {
                let p = pp.to_str().unwrap().to_owned();
                println!("Parsing: {}/{}.{}", input_dir, filename, extension);
                let data = fs::read_to_string(p)?;
                let res = SuikaParser::parse(Rule::file, &data);
                match res {
                    Ok(pairs) => write_file(input_dir, filename, &full_output_dir, pairs, variables, if_handler)?,
                    Err(e) => panic!("Can't parse {}:\n{:?}", filename, e),
                };
            }
        } else if pp.is_dir() {
            let p = pp.to_str().unwrap().to_owned();
            process_directory(root_dir, &p, output_dir, variables, if_handler)?;
        } else {
            println!("Unknown file {:?}", pp);
        }
    }

    Ok(())
}

fn write_file(input_dir: &str, filename: &str, output: &str, pairs: Pairs<'_, Rule>, variables: &mut HashMap::<String, u32>, if_handler: &mut IfHandler) -> std::io::Result<()> {
    let mut output_file = File::create(format!("{}/{}.txt", output, filename)).expect("Unable to create file");
    for pair in pairs {
        write_pair(input_dir, filename, &mut output_file, pair, variables, if_handler)?;
    }
    Ok(())
}

fn get_variable_name(variables: &mut HashMap::<String, u32>, var_name: &str) -> String {
    let var_type = &var_name[..1];

    if var_type != "$" && var_type != "%" {
        var_type.to_string()
    } else if var_name == "$RAND" {
        var_name.to_string()
    } else if variables.contains_key(var_name) {
        let name = if var_type == "%" {
            let c = ((variables[var_name] + 97) as u8) as char;
            format!("{}", c)
        } else {
            format!("{}", variables[var_name])
        };
        format!("{}{}", var_type, name)
    } else {
        let var_type = &var_name[..1];
        let mut max_val = 0;
        let mut found = false;
        for (name, v) in variables.iter() {
            if *var_type == name[..1] && *v >= max_val {
                max_val = *v;
                found = true;
            }
        }
        if found { max_val += 1; }
        variables.insert(var_name.to_string(), max_val);

        if var_type == "%" {
            if max_val >= 26 {
                panic!("There can only be 26 string variables (starting with %)");
            }

            let c = ((max_val + 97) as u8) as char;
            format!("%{}", c)
        } else {
            format!("${}", max_val)
        }
    }
}

fn write_pair(root_dir: &str, filename: &str, output_file: &mut File, pair: Pair<Rule>, variables: &mut HashMap::<String, u32>, if_handler: &mut IfHandler) -> std::io::Result<()> {
    match pair.as_rule() {
        Rule::variable => {
            let inner: Vec<_> = pair.into_inner().collect();
            let var_name = inner[0].as_str();
            let var_name = get_variable_name(variables, &var_name);
            write!(output_file, "@set {} {} {}\n", var_name, inner[1].as_str(), inner[2].as_str())?;
        },
        Rule::string_variable => {
            let inner: Vec<_> = pair.into_inner().collect();
            let var_name = inner[0].as_str();
            let var_name = get_variable_name(variables, &var_name);
            write!(output_file, "@set {} = {}\n", var_name, parse_variables(inner[1].as_str().to_string(), variables))?;
        },
        Rule::EOI => {},
        Rule::function => {
            let mut inner: Vec<_> = pair.into_inner().collect();
            let func_name = inner.remove(0).as_str();

            if func_name == "say" {
                match inner.len() {
                    1 => write!(output_file, "{}\n", parse_variables(unquote_str(inner[0].as_str()), variables))?,
                    2 => write!(output_file, "*{}*{}\n", parse_variables(unquote_str(inner[0].as_str()), variables), parse_variables(unquote_str(inner[1].as_str()), variables))?,
                    3 => write!(output_file, "*{}*{}*{}\n", parse_variables(unquote_str(inner[0].as_str()), variables), parse_variables(unquote_str(inner[1].as_str()), variables), parse_variables(unquote_str(inner[2].as_str()), variables))?,
                    _ => panic!("Too much arguments given to 'say'."),
                };
            } else if func_name == "using" {
                write!(output_file, "using {}\n", parse_variables(unquote_str(inner[0].as_str()), variables))?
            } else if func_name == "include" {
                let included_filename = unquote_str(inner[0].as_str());
                let included_filename = format!("{}/{}", root_dir, included_filename);
                let data = fs::read_to_string(&included_filename)?;
                let pairs = SuikaParser::parse(Rule::file, &data).unwrap();
                for pair in pairs {
                    write_pair(root_dir, filename, output_file, pair, variables, if_handler)?;
                }
            } else if func_name == "script" {
                write!(output_file, "wms {}\n", unquote_str(inner[0].as_str()))?
            } else if func_name == "skip" {
                let enabled = unquote_str(inner[0].as_str());
                write!(output_file, "skip {}\n", if enabled == "on" { "enabled" } else { "disabled" })?
            } else {
                let oname = match func_name {
                    "bg" | "choose" | "load" | "bgm" | "se" | "ichoose"
                    | "ch" | "chs" | "cha" | "chapter" | "gosub" | "click"
                    | "return" | "goto" | "gui" | "shake" | "video" | "vol"
                    | "anime" | "wait" => format!("@{}", func_name),
                    "label" => {
                        let label_name = inner.remove(0).as_str();
                        format!(":{}", label_name)
                    },
                    _ => panic!("Unknown: {}", func_name),
                };
                let unquote = match func_name {
                    "choose" | "ichoose" | "chapter" => false,
                    _ => true,
                };

                write!(output_file, "{}", oname)?;

                for i in inner {
                    if unquote {
                        write!(output_file, " {}", unquote_str(i.as_str()))?;
                    } else {
                        write!(output_file, " {}", i.as_str())?;
                    }
                }
                write!(output_file, "\n")?;
            }
        },
        Rule::conditional => {
            let mut inner: Vec<_> = pair.into_inner().collect();
            let cond: Vec<_> = inner.remove(0).into_inner().collect();
            let left = get_variable_name(variables, cond[0].as_str());
            let operator = cond[1].as_str();
            let right = get_variable_name(variables, cond[2].as_str());

            if_handler.if_max += 1;
            let next_if = if_handler.if_max;
            if_handler.if_stack.push(next_if);

            let mut has_else = false;
            for if_pair in &inner {
                match if_pair.as_rule() {
                    Rule::else_if | Rule::else_ => {
                        has_else = true;
                    },
                    _ => {},
                };
            }

            write!(output_file, "@if {} {} {} ", left, inverse_condition(operator), right)?;
            if has_else {
                write!(output_file, "NEXT_{}_{}\n", filename, if_handler.next_id)?;
            } else {
                write!(output_file, "END_IF_{}_{}\n", filename, next_if)?;
            }

            let mut next_id = if_handler.next_id;
            if_handler.next_id += 1;

            for if_pair in inner {
                match if_pair.as_rule() {
                    Rule::else_if | Rule::else_ => {
                        write!(output_file, "@goto END_IF_{}_{}\n:NEXT_{}_{}\n", filename, next_if, filename, next_id)?;
                        if_handler.next_id += 1;
                        next_id = if_handler.next_id;
                    },
                    _ => {},
                }
                write_pair(root_dir, filename, output_file, if_pair, variables, if_handler)?;
            }

            write!(output_file, ":END_IF_{}_{}\n", filename, next_if)?;
            if_handler.if_stack.pop();
        },
        Rule::else_if => {
            let mut inner: Vec<_> = pair.into_inner().collect();
            let cond: Vec<_> = inner.remove(0).into_inner().collect();
            let left = get_variable_name(variables, cond[0].as_str());
            let operator = cond[1].as_str();
            let right = cond[2].as_str();

            write!(output_file, "@if {} {} {} NEXT_{}_{}\n", left, inverse_condition(operator), right, filename, if_handler.next_id)?;

            for if_pair in inner {
                write_pair(root_dir, filename, output_file, if_pair, variables, if_handler)?;
            }
        },
        Rule::else_ => {
            let inner: Vec<_> = pair.into_inner().collect();
            for if_pair in inner {
                write_pair(root_dir, filename, output_file, if_pair, variables, if_handler)?;
            }
        },
        _ => println!("{:?}:\n\t{:?}", pair.as_rule(), pair.as_str()),
    };

    Ok(())
}

fn inverse_condition(operator: &str) -> String {
    format!("{}", match operator {
        "==" =>  "!=",
        "!=" => "==",
        "<" =>  ">=",
        ">" =>  "<=",
        "<=" => ">",
        ">=" => "<",
        _ => panic!("Unknown operator '{}'", operator),
    })
}

fn unquote_str(string: &str) -> String {
    let mut s = string.to_string();

    if s.starts_with('"') && s.ends_with('"') {
        s = s.replace("\\\"", "\"");
        s = (&s[1..(s.len() - 1)]).to_string();
    }

    s
}

fn parse_variables(string: String, variables: &mut HashMap::<String, u32>) -> String {
    let mut output_string = String::new();

    let mut current_var_name = String::new();
    let mut index = 0;
    let chars: Vec<_> = string.chars().collect();
    while index < chars.len() {
        let c = chars[index];
        if c == '$' || c == '%' {
            current_var_name.push(c);
            index += 1;
        } else if current_var_name.len() > 0 {
            match c {
                'a'..='z' | 'A'..='Z' | '_' => {
                    current_var_name.push(c);
                    index += 1;
                },
                _ => {
                    let suika_var_name = get_variable_name(variables, &current_var_name);
                    output_string = format!("{}{}", output_string, suika_var_name);
                    current_var_name.clear();
                },
            };
        } else {
            output_string.push(c);
            index += 1;
        }
    }

    if current_var_name.len() > 0 {
        let suika_var_name = get_variable_name(variables, &current_var_name);
        output_string = format!("{}{}", output_string, suika_var_name);
    }

    output_string
}
