use rss::Channel;
use std::fs::File;
use std::io::BufReader;

struct MyItem {
    title: String,
    url: String,
}

fn load_channel_from_file(path: &str) -> Channel {
    return Channel::read_from(BufReader::new(File::open(path).unwrap())).unwrap();
}

fn get_items(channel: Channel) -> Vec<MyItem> {
    let mut items: Vec<MyItem> = channel
        .items
        .into_iter()
        .filter(|x| x.title.is_some() && x.enclosure.is_some())
        .map(|x| {
            MyItem {
                title: x.title.unwrap(),
                url: x.enclosure.unwrap().url,
            }
        })
        .collect();
    items.sort_by_cached_key(|it| it.title.clone());
    return items;
}

fn main() {
    let path = "./rss.xml";
    let channel = load_channel_from_file(path);
    println!("{}", channel.title);

    let items = get_items(channel);
    for it in items {
        println!("{} => {}", it.title, it.url);
    }
}
