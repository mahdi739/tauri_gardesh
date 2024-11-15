#[derive(Debug, Serialize)]
struct Message {
  role: String,
  content: String,
}

#[derive(Debug, Serialize)]
struct CompletionRequest {
  model: String,
  messages: Vec<Message>,
}
#[tokio::main]
fn main(){
     let messages = vec![Message {
      role: "user".to_string(),
      content: format!("Response Language: Persian, Context: {text}, Query: {query}"),
    }];
    println!("\n\n🔴🔴🔴🔴🔴🔴🔴  Message:\n{messages:#?}\n🔴🔴🔴🔴🔴🔴\n\n");
    let request = CompletionRequest { model: "llama3-8b-8192".to_string(), messages };
    let response = client
      .post("https://api.groq.com/openai/v1/chat/completions")
      .header(header::AUTHORIZATION, format!("Bearer {}", api_key))
      .json(&request)
      .send()
      .await
      .unwrap()
      .text()
      .await
      .unwrap();
    println!("{response}");
}