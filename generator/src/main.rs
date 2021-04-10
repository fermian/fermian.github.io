use std::fs;
use std::time::SystemTime;
use std::fs::File;
use std::io::prelude::*;

extern crate tera;
use tera::Tera;

extern crate minify;
use minify::html::minify;

extern crate pulldown_cmark;
use pulldown_cmark::{
    Parser as MdParser,
    Options as MdOptions,
    Event as MdEvent,
    Tag as MdTag,
    html as MdHtml
};

extern crate glob;
use glob::glob;

extern crate serde;
use serde::Serialize;

fn read_file(filename: &str) -> String {
    let mut file = File::open(filename).expect("unable to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("unable to read file");
    contents
}

fn write_file(filename: &str, contents: String) {
    File::create(filename)
        .expect("unable to open file")
        .write_all(contents.as_bytes())
        .unwrap();
}

fn get_title_md(parser: MdParser) -> String {
    let mut title = String::new();
    let mut in_title = false;
    for event in parser {
        match event {
            MdEvent::Start(MdTag::Heading(1)) => {
                in_title = true;
            }
            MdEvent::Text(text) => {
                if in_title {
                    title = text.to_string();
                }
            }
            MdEvent::End(MdTag::Heading(1)) => {
                if in_title {
                    return title;
                }
            }
            _ => (),
        }
    }
    title
}

#[derive(Serialize)]
struct Post {
    title: String,
    url: String,
    created_time: u64,
}

fn main() {
    let mut tera = Tera::new("templates/*.html").expect("unable to init tera");

    let mut md_options = MdOptions::empty();
    md_options.insert(MdOptions::ENABLE_STRIKETHROUGH);

    // Blog posts

    let mut posts = vec![];

    for entry in glob("blog/*/*.md")
                .expect("unable to glob blog")
                .filter_map(Result::ok) {
        let mut context = tera::Context::new();

        let md_source = read_file(&entry.to_string_lossy());

        let title = get_title_md(MdParser::new(&md_source));
        context.insert("title", &title);

        let parser = MdParser::new_ext(
            &md_source,
            md_options.clone()
        );
        let mut contents = String::new();
        MdHtml::push_html(&mut contents, parser);
        context.insert("contents", &contents);

        let metadata = fs::metadata(entry.clone()).expect("metadata");
        let created_time = metadata
                .created()
                .expect("created_time")
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("duration_since")
                .as_secs();
        context.insert("created_time", &created_time);

        let mut output_name = entry.clone();
        output_name.set_file_name("index.html");
        write_file(
            &output_name.to_string_lossy(),
            minify(&tera
                .render("blog.html", &context)
                .expect("render post"))
        );
        posts.push(Post {
            title: title,
            url: output_name.to_string_lossy().to_string(),
            created_time: created_time,
        });
    }

    // Main page

    {
        let mut main_ctx = tera::Context::new();
        main_ctx.insert("posts", &posts);
        write_file(
            "index.html",
            minify(&tera
                .render_str(&read_file("pages/index.html"), &main_ctx)
                .expect("render pages/index.html"))
        );
    }
}
