use std::sync::Arc;
use std::time::Duration;
use reqwest::{Client};
use bounded_join_set::JoinSet;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let names = compile_names("usernames.txt")?;
    println!("Checking {} usernames.", names.len());
    let num_sucesses = check_group(&names, 5).await?;

    if num_sucesses == 0 {
        println!("No usernames were available :(");
    } else {
        println!("{} username(s) were available.", num_sucesses);
    }
    Ok(())
}
async fn check_username(client: &Arc<Client>, name: &str) -> Result<bool, reqwest::Error> {
    let name = name.trim();
    if name.len() < 3 || name.len() > 16 {
        return Ok(false);
    }

    let res = client.get("https://api.hystale.com/")
        .query(&[("username", name)])
        .send()
        .await?;
    if res.status().is_success() { // returns 200 if name is available
        return Ok(true);
    }
    Ok(false)
}

async fn check_group(names: &Vec<String>, limit: usize) -> Result<usize, reqwest::Error> {
    let mut available_count: usize = 0;
    let client = Arc::new(Client::new());
    let mut set = JoinSet::new(limit);

    for name in names.iter() {
        let n = name.clone();
        let client_clone = Arc::clone(&client);

        set.spawn(async move {
            let res = check_username(&client_clone, &n).await;
            (n, res)
        });
    }

    while let Some(res) = set.join_next().await {
        match res {
            Ok((name, available)) => {
                if available? {
                    println!("Username '{}' is available.", name);
                    available_count += 1;
                }
            }
            Err(e) => {
                eprintln!("Task failed: {}", e);
            }
        }
    }
    Ok(available_count)
}

fn compile_names(file_path: &str) -> Result<Vec<String>, std::io::Error> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut names = Vec::new();

    for line in reader.lines() {
        let name = line?;
        names.push(name);
    }
    Ok(names)
}
