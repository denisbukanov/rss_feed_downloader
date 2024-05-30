use std::io::Write;
use reqwest;
use std::error::Error;
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt;
use inquire::{list_option::ListOption, MultiSelect};
use rss::Channel;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::BufReader;
use std::slice::Iter;

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
            url: self.url.clone()
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

async fn download_file(title: &str, url: &str) -> Result<(), Box<dyn Error>> {
    let response = reqwest::get(url).await?;
    let total_size = response
      .content_length().ok_or("Failed to get content_length")?;
    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;
    let mut dst_file = tokio::fs::File::create("./exmaple.bin").await?;
    let mut last_progress: f64 = 0.0;
    println!("Downloading: {}", title);
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
    // let response = reqwest::get(url).await?;
    // let fut: () = reqwest::get(url);
    // let response = fut.await?;
    // println!("{:?}", response);

#[tokio::main]
async fn main() {
    let response = load_channel_from_web("https://kino.pub/podcast/get/15248/DhHmAoi4Ks3DkmXQdJXKIcjIyg4Gx8jCLZhU6TSn7D73sTdMKKagQGyP4VFT5kut").await;
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
    let _ = download_file(&items[_selected_items[0]].title, &items[_selected_items[0]].url).await;
    // println!("{}", channel.title);
    // for it in items.iter() {
    // println!("{} => {}", it.title, it.url);
    // }
    // pintln!("Answer: {:?}", answer); },
}
