use clap;
use clap::Parser;
use futures_util::StreamExt;
use inquire::{list_option::ListOption, MultiSelect};
use reqwest;
use rss::Channel;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;
use std::slice::Iter;
use tokio::io::AsyncWriteExt;

#[derive(clap::Parser, Debug)]
struct Args {
    #[arg(required = true)]
    feed_url: String,

    #[arg(last = true, required = false, default_value = "./")]
    destination: String,
}

#[derive(Debug)]
struct MyItem {
    title: String,
    url: String,
}

impl Display for MyItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.title)
    }
}

impl Clone for MyItem {
    fn clone(&self) -> MyItem {
        MyItem {
            title: self.title.clone(),
            url: self.url.clone(),
        }
    }
}

#[allow(dead_code)]
fn load_channel_from_file(path: &str) -> Channel {
    Channel::read_from(BufReader::new(File::open(path).unwrap())).unwrap()
}

async fn load_channel_from_web(url: &str) -> Result<Channel, Box<dyn Error>> {
    let response = reqwest::get(url).await?;
    let content = response.bytes().await?;
    Ok(Channel::read_from(&content[..])?)
}

fn get_items(channel: Channel) -> Vec<MyItem> {
    let mut items: Vec<MyItem> = channel
        .items
        .into_iter()
        .filter(|x| x.title.is_some() && x.enclosure.is_some())
        .map(|x| MyItem {
            title: x.title.unwrap(),
            url: x.enclosure.unwrap().url,
        })
        .collect();
    items.sort_by_cached_key(|it| it.title.clone());
    items
}

fn print_selected(selected: Iter<ListOption<MyItem>>) {
    println!("You have selected: ");
    for item in selected {
        println!("\t{}) {}", item.index, item.value.title);
    }
}

fn process_selected(selected: Vec<ListOption<MyItem>>) -> Vec<usize> {
    print_selected(selected.iter());
    selected.into_iter().map(|x| x.index).collect()
}

#[allow(dead_code)]
fn do_download(item: MyItem) {
    println!("{}", item.url);
}

fn get_ext(url: &str) -> String {
    match url.split_once("?") {
        Some((path, _)) => match path.rsplit_once(".") {
            Some((_, ext)) => ext.to_string(),
            None => String::new(),
        },
        None => String::new(),
    }
}

async fn download_file(
    title: &str,
    url: &str,
    destination_dir: &str,
) -> Result<(), Box<dyn Error>> {
    let response = reqwest::get(url).await?;
    let total_size = response
        .content_length()
        .ok_or("Failed to get content_length")?;
    println!("Content len: {}", total_size);
    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;
    let mut last_progress: f64 = 0.0;
    let dst_file_path = Path::new(destination_dir).join(format!("{}.{}", title, get_ext(url)));
    let mut dst_file = tokio::fs::File::create(&dst_file_path).await?;

    println!("Downloading: {} to {}", title, dst_file_path.display());
    print!("Downloaded:  0%");
    let _ = std::io::stdout().flush();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        dst_file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;
        let progress = (downloaded as f64 / total_size as f64) * 100.0;
        if progress - last_progress > 10.0 {
            print!("\x08\x08\x08{}%", progress as u32);
            let _ = std::io::stdout().flush();
            last_progress = progress;
        }
    }
    println!("");
    Ok(())
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    println!("{:?}", args);
    let response = load_channel_from_web(&args.feed_url).await;
    if response.is_err() {
        println!("Error while loading data");
        return;
    }
    let channel = response.unwrap();
    let items = get_items(channel);

    let answer = MultiSelect::new("Boop-beep", items.clone()).raw_prompt();
    let _selected_items: Vec<usize> = match answer {
        Ok(selected) => process_selected(selected),
        _ => [].to_vec(),
    };
    for idx in _selected_items {
        let item = &items[idx];
        let _ = download_file(&item.title, &item.url, &args.destination).await;
    }
}
