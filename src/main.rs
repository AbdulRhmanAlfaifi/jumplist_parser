use clap::{Arg, ArgAction, ArgMatches, Command};
use glob::glob;
use jumplist_parser::errors::JumplistParserError;
use jumplist_parser::JumplistParser;
use jumplist_parser::_Normalize;
use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};

enum OutputFormat {
    JSON,
    JSONL,
    CSV,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> OutputFormat {
        match s {
            "json" => OutputFormat::JSON,
            "jsonl" => OutputFormat::JSONL,
            "csv" => OutputFormat::CSV,
            _ => OutputFormat::CSV,
        }
    }
}

fn parse_cli_args() -> ArgMatches {
    Command::new("jumplist_parser")
        .version(env!("CARGO_PKG_VERSION"))
        .author(clap::crate_authors!())
        .help_template("\
{before-help}

Created By: {author}
Version: v{version}
Reference: https://u0041.co/posts/articals/jumplist-files-artifacts/

{about}

{usage-heading} {usage}

{all-args}{after-help}
")
        .about("Windows Jumplist Files Parser")
        .arg(
            Arg::new("path")
                .short('p')
                .long("path")
                .value_name("PATH")
                .help("Path(s) to Jumplist files to be parsed - accepts glob (defaults to 'AutomaticDestinations' & 'CustomDestinations' for all users)")
                .action(ArgAction::Append)
                .required(false),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("The file path to write the output to")
                .default_value("stdout")
        )
        .arg(
            Arg::new("output-format")
                .long("output-format")
                .value_parser(["csv", "jsonl", "json"])
                .default_value("csv")
                .help("Output format")
        )
        .arg(
            Arg::new("no-headers")
                .long("no-headers")
                .help("Don't print headers when using CSV as the output format")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("normalize")
                .long("normalize")
                .help("Normalize the result to the most important fields")
                .action(ArgAction::SetTrue)
        )
        .get_matches()
}

fn output_data_csv(parsed: JumplistParser) -> String {
    let app_id = &parsed.app_id.clone().unwrap_or_default().to_owned();
    let app_name = &parsed.app_name.clone().unwrap_or_default().to_owned();
    let data = parsed.normalize();
    let mut records: Vec<String> = vec![];
    for i in 0..data.len() {
        let row = data[i].to_owned();
        records.push(format!(
            "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"",
            app_id,
            app_name,
            parsed.r#type,
            row.get("target_full_path").unwrap(),
            row.get("command_line_arguments")
                .unwrap()
                .replace("\"", "\"\""),
            row.get("name_string").unwrap(),
            row.get("target_modification_time").unwrap(),
            row.get("target_access_time").unwrap(),
            row.get("target_creation_time").unwrap(),
            row.get("target_size").unwrap(),
            row.get("target_hostname").unwrap(),
        ));
    }
    // println!("Records: {:?}", records);
    records.join("\n")
}

fn main() {
    let args = parse_cli_args();
    let output_format = OutputFormat::from_str(args.get_one::<String>("output-format").unwrap());
    let output_to = args.get_one::<String>("output").unwrap().clone();
    let normalize = args.get_flag("normalize");
    let mut output: Box<dyn Write> = match output_to.as_str() {
        "stdout" => Box::new(io::stdout()),
        _ => Box::new(File::create(output_to).unwrap()),
    };

    if args.get_flag("no-headers") == false {
        match output_format {
            OutputFormat::CSV => {
                output.write(r#""app_id","app_name","type","target_full_path","command_line_arguments","name_string","target_modification_time","target_access_time","target_creation_time","target_size","target_hostname""#.as_bytes()).expect("Error Writing Data !");
                output.write(b"\n").expect("Error Writing Data !");
            }
            _ => {}
        };
    }

    #[cfg(target_os = "windows")]
    // Default paths for Jumplist files
    let mut jumplist_paths = vec![
        r"C:\Users\*\AppData\Roaming\Microsoft\Windows\Recent\AutomaticDestinations\*ms",
        r"C:\Users\*\AppData\Roaming\Microsoft\Windows\Recent\CustomDestinations\*ms",
    ];

    #[cfg(target_os = "linux")]
    // Default paths for Jumplist files while using WSL
    let mut jumplist_paths = vec![
        r"/mnt/c/Users/*/AppData/Roaming/Microsoft/Windows/Recent/AutomaticDestinations/*ms",
        r"/mnt/c/Users/*/AppData/Roaming/Microsoft/Windows/Recent/CustomDestinations/*ms",
    ];

    if let Some(paths) = args.get_many::<String>("path") {
        jumplist_paths = paths.map(|s| s.as_str()).collect::<Vec<&str>>();
    }

    #[derive(Debug, Serialize)]
    #[serde(untagged)]
    enum JsonRecord {
        Raw(JumplistParser),
        Normalize(Vec<HashMap<String, String>>),
    }
    let mut json_list = vec![];
    for dir in jumplist_paths {
        for entry in glob(dir).expect("Failed to read glob pattern") {
            match entry {
                Ok(path) => {
                    let full_path = path.as_path().to_str().unwrap();
                    match JumplistParser::from_path(full_path) {
                        Ok(parsed) => match output_format {
                            OutputFormat::JSONL => {
                                let json_data;
                                if normalize {
                                    let mut normalized = parsed.normalize();
                                    normalized.iter_mut().for_each(|e| {
                                        e.insert(
                                            "app_id".to_string(),
                                            parsed.app_id.clone().unwrap_or_default(),
                                        );
                                        e.insert(
                                            "app_name".to_string(),
                                            parsed.app_name.clone().unwrap_or_default(),
                                        );
                                    });
                                    json_data = serde_json::to_string(&normalized).unwrap();
                                } else {
                                    json_data = serde_json::to_string(&parsed).unwrap();
                                }
                                output
                                    .write(json_data.as_bytes())
                                    .expect("Error Writing Data !");
                                output.write(b"\n").expect("Error Writing Data !");
                                let _ = output.flush();
                            }
                            OutputFormat::JSON => {
                                if normalize {
                                    json_list.push(JsonRecord::Normalize(parsed.normalize()));
                                } else {
                                    json_list.push(JsonRecord::Raw(parsed));
                                }
                            }
                            OutputFormat::CSV => {
                                if parsed.normalize().len() > 0 {
                                    output
                                        .write(output_data_csv(parsed).as_bytes())
                                        .expect("Error Writing Data !");
                                    output.write(b"\n").expect("Error Writing Data !");
                                    let _ = output.flush();
                                }
                            }
                        },
                        Err(e) => match e {
                            JumplistParserError::NoDestList(s, l, f) => {
                                //get the size of the file in full_path
                                let file_size = std::fs::metadata(full_path)
                                    .expect("Unable to get file size")
                                    .len();
                                eprintln!(
                                    "Error parsing the file '{}', Size: {}, {}:{} : structure incorrect: {:?}",
                                    full_path,
                                    file_size,
                                    l,
                                    f,
                                    s
                                );
                            }
                            _ => {
                                eprintln!(
                                    "Did not parse '{}' correctly. ERROR : '{}'",
                                    full_path, e
                                );
                            }
                        },
                    };
                }
                Err(e) => eprintln!("{:?}", e),
            }
        }
    }
    if let OutputFormat::JSON = output_format {
        let json_data = serde_json::to_string(&json_list).unwrap();
        output
            .write(json_data.as_bytes())
            .expect("Error Writing Data !");
    }
}
