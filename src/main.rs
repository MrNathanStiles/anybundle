use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::File;
use std::io::{BufWriter, Write};

use std::path::PathBuf;

const MODE_DEFINE: u32 = 0;
const MODE_FUNCTION: u32 = 1;
const MODE_ASSIGN: u32 = 2;
const MODE_TICK: u32 = 3;

pub struct BundleError {
    message: String
}
pub struct Bundle {
    processed: HashMap<String, i32>,
    source: PathBuf,
    define: Vec<String>,
    assign: Vec<String>,
    function: Vec<String>,
    tick: Vec<String>,
    mode: u32,
}

impl From<std::io::Error> for BundleError {
    fn from(value: std::io::Error) -> Self {
        Self {
            message: value.to_string()
        }
    }
}
impl From<PathBuf> for BundleError {
    fn from(_value: PathBuf) -> Self {
        Self {
            message: "path could not be converted to string".to_owned(),
        }
    }
}

impl From<&'static str> for BundleError {
    fn from(value: &str) -> Self {
        Self {
            message: value.to_owned(),
        }
    }
}

impl Bundle {
    pub fn new(source: PathBuf) -> Self {
        Self {
            processed: HashMap::new(),
            source,
            define: Vec::new(),
            assign: Vec::new(),
            function: Vec::new(),
            tick: Vec::new(),
            mode: MODE_ASSIGN,
        }
    }

    pub fn bundle(&mut self) -> Result<(), BundleError> {
        self.process(self.source.to_owned())?;
        
        if let Some(file_name) = self.source.file_name() {
            
            let old_filename =  if let Some(old_filename) = file_name.to_str() {
                old_filename
            } else {
                return Err("failed".into());
            };

            let new_filename = format!("built_{}", old_filename);
            let mut new_path = self.source.clone();
            new_path.set_file_name(new_filename);
            
            let file = File::create(&new_path)?;
            let mut writer = BufWriter::new(file);

            for line in &self.define {
                writeln!(writer, "{}", line)?;
            }

            for line in &self.function {
                writeln!(writer, "{}", line)?;
            }

            for line in &self.assign {
                writeln!(writer, "{}", line)?;
            }

            for line in &self.tick {
                writeln!(writer, "{}", line)?;
            }

            Ok(())
        } else {
            Err("failed".into())
        }
    }

    fn process(&mut self, file_path: PathBuf) -> Result<(), BundleError> {

        let absolute_path = std::path::absolute(&file_path)?.into_string()?;

        if self.processed.contains_key(&absolute_path) {
            return Ok(());
        }

        self.processed.insert(absolute_path, 0);
        
        let lines = Self::get_lines(&file_path)?;
        

        for line in &lines {
            let trimmed = line.trim();
            if trimmed.starts_with("#use ") {
                if self.mode == MODE_TICK {
                    return Err("cannot switch mode now".into())
                }
                if let Some(dir) = file_path.parent() {
                    let using = dir.join(&trimmed[5..]);
                    self.process(using)?;
                } else {
                    return Err("cannot get parent".into())
                }
                continue;
            } else if trimmed.starts_with("#define") {
                if self.mode == MODE_TICK {
                    return Err("cannot switch mode now".into())
                }
                self.mode = MODE_DEFINE;
                continue;
            } else if trimmed.starts_with("#assign") {
                if self.mode == MODE_TICK {
                    return Err("cannot switch mode now".into())
                }
                self.mode = MODE_ASSIGN;
                continue;
            } else if trimmed.starts_with("#function") {
                if self.mode == MODE_TICK {
                    return Err("cannot switch mode now".into())
                }
                self.mode = MODE_FUNCTION;
                continue;
            } else if trimmed.starts_with("on_tick") {
                if self.mode == MODE_TICK {
                    return Err("cannot switch mode now".into())
                }
                self.mode = MODE_TICK;
            }
            match self.mode {
                MODE_DEFINE => {
                    self.define.push(line.to_owned());
                }
                MODE_FUNCTION => {
                    self.function.push(line.to_owned());
                }
                MODE_ASSIGN => {
                    self.assign.push(line.to_owned());
                }
                MODE_TICK => {
                    self.tick.push(line.to_owned());
                }
                _ => {
                    return Err("unexpected mode".into());
                }
            }
        }
        Ok(())
    }

    fn get_lines(file_path: &PathBuf) -> Result<Vec<String>, BundleError>  {
        let content = fs::read_to_string(file_path)?;
        Ok(content.lines().map(String::from).collect())
    }
}


fn main() {
    let args: Vec<String> = env::args().collect();
    let file_path = PathBuf::from(&args[1]);
    let mut bundle = Bundle::new(file_path);
    match bundle.bundle() {
        Err(e) => {
            panic!("{}", &e.message)
        }
        Ok(()) => {
            println!("done")
        }
    }
    
    

}
