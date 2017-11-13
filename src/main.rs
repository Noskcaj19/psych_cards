extern crate colored;
#[macro_use]
extern crate quick_error;
extern crate reqwest;
extern crate select;

use colored::*;

use std::io::{BufRead, BufReader, Read};
use std::fs::File;

use select::document::Document;
use select::predicate::{Class, Name};


quick_error! {
    #[derive(Debug)]
    pub enum ProgramError {
        Io(err: std::io::Error) {
            from()
                description("io error")
                display("I/O error: {}", err)
                cause(err)
        }
        Url(err: reqwest::Error) {
            from()
                description("Url fetch error")
                cause(err)
        }
        Parse {
            description("Error parsing document")
        }
        Arg {
            description("Invalid arguments")
        }
    }
}

#[derive(Debug)]
struct DefinitionLink {
    title: String,
    href: String,
}

#[derive(Debug)]
struct Definition {
    title: String,
    text: String,
}


fn document_from_url(url: &str) -> Result<Document, ProgramError> {
    let mut res = reqwest::get(url)?;
    let mut buf = Vec::new();
    res.read_to_end(&mut buf)?;
    let body = String::from_utf8_lossy(&buf);
    Ok(Document::from(body.to_string().as_str()))
}

fn get_definition_links(term: &str) -> Result<Vec<DefinitionLink>, ProgramError> {
    let doc = document_from_url(&format!(
        "https://www.alleydog.com/search-results.php?q={}",
        term
    ))?;
    let definition_div = doc.find(Class("results"))
        .next()
        .ok_or(ProgramError::Parse)?;

    let mut definitions = Vec::new();
    for def_element in definition_div.find(Name("a")) {
        let title = def_element.text();
        if title.starts_with("are we missing") {
            break;
        }
        let href = match def_element.attr("href") {
            Some(url) => url.to_owned(),
            None => continue,
        };
        definitions.push(DefinitionLink { title, href })
    }
    Ok(definitions)
}


fn get_definition(link: &DefinitionLink) -> Result<Definition, ProgramError> {
    let doc = document_from_url(&link.href)?;
    let article = doc.find(Name("article")).next().ok_or(ProgramError::Parse)?;

    Ok(Definition {
        title: article
            .find(Name("h1"))
            .next()
            .ok_or(ProgramError::Parse)?
            .text()
            .trim()
            .to_owned(),
        text: article
            .find(Name("p"))
            .next()
            .ok_or(ProgramError::Parse)?
            .text()
            .trim()
            .to_owned(),
    })
}

fn display_definition(term: &DefinitionLink) -> Result<(), ProgramError> {
    let def = get_definition(term)?;

    println!("{}:", def.title.bold());
    println!("{}", def.text);
    println!();
    Ok(())
}

fn display_term(term: &str) -> Result<(), ProgramError> {
    let def_links = get_definition_links(term)?
        .into_iter()
        .filter(|x| x.title.contains("Glossary"));

    for def_link in def_links.take(4) {
        display_definition(&def_link)?;
    }
    Ok(())
}


fn run() -> Result<(), ProgramError> {
    let file = std::env::args().nth(1).ok_or(ProgramError::Arg)?;
    let start = std::env::args()
        .nth(2)
        .and_then(|a| a.parse().ok())
        .unwrap_or(1);
    if start == 0 {
        return Err(ProgramError::Arg);
    }
    let input = File::open(file)?;
    let buffer = BufReader::new(input);

    let lines: Vec<_> = buffer
        .lines()
        .skip(start - 1)
        .filter_map(|x| x.ok())
        .collect();

    let max = lines.len() + start - 1;

    for (index, line) in (start..).zip(lines.iter()) {
        println!("{} ({}/{})", line.yellow().italic().underline(), index, max);

        display_term(&line)?;

        println!(">");
        let mut buffer = String::new();
        std::io::stdin().read_line(&mut buffer)?;
    }
    Ok(())
}

fn main() {
    run().expect("Program Error");
}
