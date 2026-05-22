// import dependencies into the code
use ollama_rs::Ollama;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use tauri::{ipc::Channel, State};
use ollama_rs::generation::chat::{ChatMessage, request::ChatMessageRequest};
use futures_util::StreamExt;

//Intialize Ollama client and define application State
struct AppState {
    ollama: Mutex<Ollama>,
}

#[derive(Serialize)]
struct ChatResponse{
    message: String
}

#[derive(Deserialize)]
struct ChatRequest {
    model: String,
    messages:Vec<ChatMessage>,
}

// Tauri command to get the list of available models from ollama
#[tauri::command]
async fn get_models(state:State<'_, AppState>) -> Result<Vec<String>, String> {
    let models ={
      let client = state.ollama.lock().await;
      client.list_local_models()
      .await
      .map_err(|e| format!("failed to list models: {:?}", e))?
    };
    Ok(models.into_iter().map(|m| m.name.clone()).collect())
    }

#[tauri::command]
async fn chat(
    request: ChatRequest,
    on_stream: Channel<ChatResponse>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = state.ollama.lock().await;
    let chat_request = ChatMessageRequest::new(request.model, request.messages);
    let mut stream = client
        .send_chat_messages_stream(chat_request)
        .await
        .map_err(|e| format!("Failed to send chat stream: {:?}", e))?;

    while let Some(response) = stream.next().await {
        let response = response.map_err(|e| format!("Stream error: {:?}", e))?;  
    
        let chat_response = ChatResponse{
            message: response.message.content,
        };
        on_stream.send(chat_response).map_err(|e| e.to_string())?;
    }
    Ok(()) 

}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_models, chat])
        .manage (AppState {
            ollama: Mutex::new(Ollama::default()),
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

