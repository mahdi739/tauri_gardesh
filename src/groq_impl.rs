#![allow(unused)]
use glob::glob;
use lopdf::Document;
use ndarray::Array2;
use reqwest::Client;
use serde::Serialize;
use std::collections::VecDeque;
use std::error::Error;
use std::fs;
use std::io::prelude::*;
use std::path::Path;

#[derive(Serialize)]
struct Message {
  role: String,
  content: String,
}

#[derive(Serialize)]
struct CompletionRequest {
  model: String,
  messages: Vec<Message>,
}

async fn extract_text_from_pdf(pdf_path: &str) -> Result<String, Box<dyn Error>> {
  let document = Document::load(pdf_path)?;
  let mut text = String::new();
  for (page_id, _page) in document.get_pages() {
    let page_content = document.extract_text(&[page_id])?;
    text.push_str(&page_content);
  }
  Ok(text)
}

async fn load_and_extract_texts_from_pdfs(
  data_folder: &str,
) -> Result<Vec<String>, Box<dyn Error>> {
  let mut texts = Vec::new();
  for entry in glob(&format!("{}/*.pdf", data_folder))?.filter_map(Result::ok) {
    match extract_text_from_pdf(entry.to_str().unwrap()).await {
      Ok(text) => texts.push(text),
      Err(e) => eprintln!("Error processing {}: {}", entry.display(), e),
    }
  }
  Ok(texts)
}

fn split_text_into_chunks(text: &str, chunk_size: usize) -> Vec<String> {
  text
    .chars()
    .collect::<Vec<char>>()
    .chunks(chunk_size)
    .map(|chunk| chunk.iter().collect())
    .collect()
}

async fn generate_response(
  client: &Client,
  chunks: Vec<String>,
  query: &str,
  context_history: Option<Vec<String>>,
  model: &str,
) -> Result<String, Box<dyn Error>> {
  let context = if let Some(history) = context_history {
    format!("{} {}", history.join(" "), chunks.join(" "))
  } else {
    chunks.join(" ")
  };

  let messages = vec![Message {
    role: "user".to_string(),
    content: format!("Context: {} Query: {}", context, query),
  }];
  let request = CompletionRequest { model: model.to_string(), messages };

  let response = client
    .post("https://api.groq.com/openai/v1/chat/completions")
    .json(&request)
    .send()
    .await?
    .text()
    .await?;

  Ok(response)
}

fn maintain_conversational_context(
  response: String,
  mut context_history: VecDeque<String>,
  max_context_length: usize,
) -> VecDeque<String> {
  if context_history.len() >= max_context_length {
    context_history.pop_front();
  }
  context_history.push_back(response);
  context_history
}

async fn rag_pipeline(
  client: &Client,
  data_folder: &str,
  query: &str,
  mut context_history: VecDeque<String>,
) -> Result<String, Box<dyn Error>> {
  let texts = load_and_extract_texts_from_pdfs(data_folder).await?;
  let chunks: Vec<String> =
    texts.iter().flat_map(|text| split_text_into_chunks(text, 512)).collect();

  println!("Loaded and Extracted Texts from {} PDFs", texts.len());

  let relevant_texts = retrieve_relevant_chunks(&chunks, query, 5).await;

  let response = generate_response(
    client,
    relevant_texts,
    query,
    Some(context_history.iter().map(|s| s.clone()).collect()),
    "llama3-8b-8192",
  )
  .await?;
  Ok(response)
}

async fn retrieve_relevant_chunks(chunks: &[String], query: &str, top_k: usize) -> Vec<String> {
  let mut corpus = vec![query.to_string()];
  corpus.extend_from_slice(chunks);

  // Simulate TF-IDF vectorization and cosine similarity
  let vectorizer =
    corpus.iter().map(|text| text.split_whitespace().collect::<Vec<&str>>()).collect::<Vec<_>>();
  let vectors: Vec<Array2<f64>> =
    vectorizer.iter().map(|_v| ndarray::Array::zeros((1, 1))).collect(); // Placeholder

  // Placeholder cosine similarity calculation
  let cosine_matrix = vec![vec![1.0; vectors.len()]; vectors.len()];
  let similarity_scores = &cosine_matrix[0][1..];

  let mut ranked_indices: Vec<usize> = (0..similarity_scores.len()).collect();
  ranked_indices.sort_by(|&a, &b| similarity_scores[b].partial_cmp(&similarity_scores[a]).unwrap());

  ranked_indices.iter().take(top_k).map(|&idx| chunks[idx].clone()).collect()
}

async fn main() {
  let client = Client::new();
  let data_folder = "data/";
  let context_history: VecDeque<String> = VecDeque::new();

  interactive_cli(&client, data_folder, context_history).await.unwrap();
}

async fn interactive_cli(
  client: &Client,
  data_folder: &str,
  mut context_history: VecDeque<String>,
) -> Result<(), Box<dyn Error>> {
  use tokio::io::{self, AsyncBufReadExt};

  println!("Welcome to the RAG-powered conversational assistant! Type /bye to exit.");
  let stdin = io::BufReader::new(io::stdin());
  let mut lines = stdin.lines();

  while let Some(line) = lines.next_line().await? {
    let user_input = line.trim();
    if user_input.eq_ignore_ascii_case("/bye") {
      println!("Goodbye!");
      break;
    }

    let response = rag_pipeline(client, data_folder, user_input, context_history.clone()).await?;
    context_history = maintain_conversational_context(response.clone(), context_history, 2);

    println!(">> groq: {}", response);
  }

  Ok(())
}
