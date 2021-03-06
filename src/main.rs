// These are for the Iron web framework
extern crate iron;
extern crate staticfile;
extern crate router;
extern crate mount;
extern crate handlebars_iron as hbs;
extern crate urlencoded;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate xdg_basedir;
extern crate getopts;
extern crate toml;

mod arff;
mod visu;

use getopts::Options;
use std::env;
use std::fs::File;
use std::path;
use std::str::FromStr;
use std::io::Read;

#[derive(Deserialize)]
struct Config {
    port: Option<u16>,
}

fn get_default_port() -> u16 {
    let default = 8080;

    let path = match xdg_basedir::get_config_home() {
        Ok(path) => path,
        Err(_) => return default,
    };
    let mut file = match File::open(path.join("varfrc")) {
        Ok(file) => file,
        Err(_) => return default,
    };
    let mut content = String::new();
    match file.read_to_string(&mut content) {
        Err(_) => return default,
        Ok(_) => (),
    }

    let table: Config = match toml::de::from_str(&content) {
        Ok(table) => table,
        _ => return default,
    };

    let port = table.port.unwrap_or(default);

    port
}

fn get_data_dir() -> &'static str {
    match option_env!("VARF_HOME") {
        Some(path) => path,
        // None => "/usr/share/varf",
        // Actually, default to the local data
        None => "data",
    }
}

struct Params {
    filename: String,
    datadir: String,
    port: u16,

    open_browser: bool,
}

fn read_params() -> Result<Params, String> {
    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();
    opts.optflag("h", "help", "Prints this help message.");
    opts.optopt("p", "", "Sets the port to listen to.", "PORT");
    opts.optopt("d",
                "",
                &format!("Sets the directory where varf files are installed. Defaults to {}",
                         get_data_dir()),
                "VARF_HOME");
    opts.optflag("o", "open", "Open the page in the browser");

    let mut matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!("Error: {}", f),
    };

    if matches.opt_present("h") {
        return Err(opts.usage("Usage: varf [OPTIONS] FILENAME"));
    }

    if matches.free.is_empty() {
        println!("Error: no filename given!");
        return Err(opts.usage("Usage: varf [OPTIONS] FILENAME"));
    }

    let filename = matches.free.remove(0);

    let port = match matches.opt_str("p") {
        None => get_default_port(),
        Some(p_str) => u16::from_str(&p_str).unwrap(),
    };
    let datadir = match matches.opt_str("d") {
        None => get_data_dir().to_string(),
        Some(datadir) => datadir,
    };
    let open_browser = matches.opt_present("o");

    Ok(Params {
        filename: filename,
        datadir: datadir,
        port: port,
        open_browser: open_browser,
    })
}

fn main() {

    let params = match read_params() {
        Err(e) => {
            println!("{}", e);
            return;
        }
        Ok(params) => params,
    };

    let content = arff::ArffContent::new(path::Path::new(&params.filename));

    visu::serve_result(&params.datadir, params.port, &content, params.open_browser);
}
