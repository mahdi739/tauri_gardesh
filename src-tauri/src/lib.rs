use std::{collections::HashMap, env, path::PathBuf};

use dotenv::dotenv;
use reqwest::{header, Client};
use serde::Serialize;
use serde_json::Value;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
  format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  dotenv().ok();

  tauri::Builder::default()
    .plugin(tauri_plugin_shell::init())
    .invoke_handler(tauri::generate_handler![greet])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

#[derive(Debug, Serialize)]
struct Message {
  role: String,
  content: String,
}
#[derive(Debug, Serialize)]
struct CompletionRequest {
  model: String,
  messages: Vec<Message>,
  temperature: u32,
  response_format: HashMap<String, String>,
}

async fn answer(search_text: impl AsRef<str>) {
  let api_key = env::var("GROQ_API_KEY").expect("GROQ_API_KEY must be set");
  let search_text = search_text.as_ref();
  let client = Client::new();
  let messages = vec![Message {
    role: "user".to_string(),
    content: format!(r#"Translate this text to English:\n{search_text}"#),
  }];
  println!("\n\n🔴🔴🔴🔴🔴🔴🔴  Message:\n{messages:#?}\n🔴🔴🔴🔴🔴🔴\n\n");
  let request = CompletionRequest {
    temperature: 0,
    response_format: HashMap::new(),
    model: "mixtral-8x7b-32768".to_string(),
    messages,
  };
  let json_response = client
    .post("https://api.groq.com/openai/v1/chat/completions")
    .header(header::AUTHORIZATION, format!("Bearer {}", api_key))
    .json(&request)
    .send()
    .await
    .unwrap()
    .json::<Value>()
    .await
    .unwrap();
  let translated_search_text = json_response["choices"][0]["message"]["content"].as_str().unwrap();
  use vec_embed_store::{EmbeddingEngineOptions, EmbeddingsDb, SimilaritySearch, TextChunk};
  let embedding_engine_options = EmbeddingEngineOptions {
    // model_name: BGESmallENV15, // see https://docs.rs/fastembed/latest/fastembed/enum.EmbeddingModel.html
    cache_dir: PathBuf::from("cache"),
    show_download_progress: true,
    ..Default::default()
  };
  // Create a new instance of EmbeddingsDb
  let embed_db = EmbeddingsDb::new("fastembed-2", embedding_engine_options).await.unwrap();

  // Define a text for similarity search
  // let search_text = &input[1..].iter().cloned().collect::<String>();

  // Perform a similarity search
  let search_results = embed_db
    .get_similar_to(translated_search_text)
    .limit(2)
    .threshold(0.8)
    .execute()
    .await
    .unwrap();

  println!("Similarity search results:");
  for result in search_results.iter() {
    println!("ID: {}, Text: {}, Distance: {}", result.id, result.text, result.distance);
  }

  let messages = vec![Message {
    role: "user".to_string(),
    content: format!(
      "Response Language: Persian, Context: {}, Query: {search_text}",
      search_results.into_iter().map(|f| f.text).join("\n"),
    ),
  }];
  println!("\n\n🔴🔴🔴🔴🔴🔴🔴  Message:\n{messages:#?}\n🔴🔴🔴🔴🔴🔴\n\n");
  let request = CompletionRequest {
    response_format: HashMap::new(),
    temperature: 1,
    model: "llama3-8b-8192".to_string(),
    messages,
  };
  let json_response = client
    .post("https://api.groq.com/openai/v1/chat/completions")
    .header(header::AUTHORIZATION, format!("Bearer {}", api_key))
    .json(&request)
    .send()
    .await
    .unwrap()
    .json::<Value>()
    .await
    .unwrap();
  let text_response = json_response["choices"][0]["message"]["content"].as_str().unwrap();

  println!("{text_response}");
}
