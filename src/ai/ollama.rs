// A minimal client for a locally running Ollama server.
//
// Everything here is optional at runtime: if no server is listening, callers
// fall back to the deterministic classifier rather than failing. TLS is
// deliberately compiled out (ureq's default features are off), so this client
// can only ever talk to a plain-HTTP local endpoint.

use std::time::Duration;

use serde::Deserialize;

pub const DEFAULT_HOST: &str = "http://localhost:11434";
pub const DEFAULT_MODEL: &str = "llama3.2:3b";

#[derive(Debug)]
pub enum AiError {
    /// Nothing is listening, or the server could not be reached.
    Unavailable(String),
    /// The server answered, but not with something we could use.
    Malformed(String),
}

impl std::fmt::Display for AiError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiError::Unavailable(detail) => write!(formatter, "{detail}"),
            AiError::Malformed(detail) => write!(formatter, "{detail}"),
        }
    }
}

pub struct Ollama {
    agent: ureq::Agent,
    host: String,
    model: String,
}

#[derive(Deserialize)]
struct TagsResponse {
    #[serde(default)]
    models: Vec<TagEntry>,
}

#[derive(Deserialize)]
struct TagEntry {
    #[serde(default)]
    name: String,
}

#[derive(Deserialize)]
struct GenerateResponse {
    #[serde(default)]
    response: String,
}

impl Ollama {
    /// Reads `OLLAMA_HOST` and `REORGANIZE_MODEL`, falling back to the usual
    /// local defaults.
    pub fn from_env() -> Ollama {
        let host = std::env::var("OLLAMA_HOST").unwrap_or_else(|_| DEFAULT_HOST.to_string());
        let model = std::env::var("REORGANIZE_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_string());

        Ollama::new(host, model)
    }

    pub fn new(host: String, model: String) -> Ollama {
        let config = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(60)))
            .build();

        Ollama {
            agent: config.into(),
            host: normalize_host(&host),
            model,
        }
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    /// Names of the models the server has pulled.
    pub fn installed_models(&self) -> Result<Vec<String>, AiError> {
        let url = format!("{}/api/tags", self.host);

        let tags: TagsResponse = self
            .agent
            .get(&url)
            .call()
            .map_err(|error| AiError::Unavailable(error.to_string()))?
            .body_mut()
            .read_json()
            .map_err(|error| AiError::Malformed(error.to_string()))?;

        Ok(tags.models.into_iter().map(|entry| entry.name).collect())
    }

    /// Whether the configured model is one the server can actually run.
    pub fn has_model(&self, installed: &[String]) -> bool {
        let wanted = self.model.as_str();

        installed.iter().any(|name| {
            name == wanted
                // `ollama pull llama3.2` reports itself as `llama3.2:latest`,
                // so an untagged name matches its tagged form. A name that
                // does specify a tag has to match it exactly.
                || (!wanted.contains(':') && name.split(':').next() == Some(wanted))
        })
    }

    /// Sends a single non-streaming prompt and returns the raw completion.
    // Exercised by the tests; the rule-generation work is its first caller
    // in the binary itself.
    #[allow(dead_code)]
    pub fn generate(&self, prompt: &str) -> Result<String, AiError> {
        let url = format!("{}/api/generate", self.host);

        let request = serde_json::json!({
            "model": self.model,
            "prompt": prompt,
            "stream": false,
            // Classification wants the same answer every time, not variety.
            "options": { "temperature": 0 },
        });

        let generated: GenerateResponse = self
            .agent
            .post(&url)
            .send_json(&request)
            .map_err(|error| AiError::Unavailable(error.to_string()))?
            .body_mut()
            .read_json()
            .map_err(|error| AiError::Malformed(error.to_string()))?;

        Ok(generated.response)
    }
}

/// `OLLAMA_HOST` is often set as a bare `host:port`.
fn normalize_host(host: &str) -> String {
    let trimmed = host.trim().trim_end_matches('/');

    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else {
        format!("http://{trimmed}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    /// Serves one canned JSON reply on a free port, then closes.
    ///
    /// Reading the full request before replying matters for POST: answering
    /// mid-body would reset the connection instead of exercising the client.
    fn stub_server(status: &str, body: &'static str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let status = status.to_string();

        thread::spawn(move || {
            let Ok((mut stream, _)) = listener.accept() else {
                return;
            };

            let mut request = Vec::new();
            let mut chunk = [0u8; 1024];

            loop {
                let Ok(read) = stream.read(&mut chunk) else {
                    return;
                };

                if read == 0 {
                    break;
                }

                request.extend_from_slice(&chunk[..read]);

                let text = String::from_utf8_lossy(&request);

                let Some(headers_end) = text.find("\r\n\r\n") else {
                    continue;
                };

                let expected_body = text[..headers_end]
                    .lines()
                    .find_map(|line| {
                        let (name, value) = line.split_once(':')?;

                        if name.eq_ignore_ascii_case("content-length") {
                            value.trim().parse::<usize>().ok()
                        } else {
                            None
                        }
                    })
                    .unwrap_or(0);

                if request.len() >= headers_end + 4 + expected_body {
                    break;
                }
            }

            let response = format!(
                "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );

            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
        });

        format!("http://{address}")
    }

    /// An address with nothing listening on it.
    fn closed_port() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();

        drop(listener);

        format!("http://{address}")
    }

    #[test]
    fn lists_installed_models() {
        let host = stub_server(
            "200 OK",
            r#"{"models":[{"name":"llama3.2:3b"},{"name":"qwen2.5:3b"}]}"#,
        );

        let ollama = Ollama::new(host, "llama3.2:3b".to_string());
        let models = ollama.installed_models().unwrap();

        assert_eq!(models, vec!["llama3.2:3b", "qwen2.5:3b"]);
    }

    #[test]
    fn a_server_with_no_models_lists_nothing() {
        let host = stub_server("200 OK", r#"{"models":[]}"#);

        let ollama = Ollama::new(host, "llama3.2:3b".to_string());

        assert!(ollama.installed_models().unwrap().is_empty());
    }

    #[test]
    fn generate_returns_the_response_field() {
        let host = stub_server("200 OK", r#"{"model":"x","response":"Documents","done":true}"#);

        let ollama = Ollama::new(host, "llama3.2:3b".to_string());

        assert_eq!(ollama.generate("where does this go?").unwrap(), "Documents");
    }

    // The whole design depends on this being an error rather than a hang or a
    // panic: no server means fall back to the deterministic classifier.
    #[test]
    fn an_unreachable_server_is_unavailable() {
        let ollama = Ollama::new(closed_port(), "llama3.2:3b".to_string());

        assert!(matches!(
            ollama.installed_models(),
            Err(AiError::Unavailable(_))
        ));
    }

    #[test]
    fn a_non_json_reply_is_malformed() {
        let host = stub_server("200 OK", "this is not json");

        let ollama = Ollama::new(host, "llama3.2:3b".to_string());

        assert!(matches!(
            ollama.installed_models(),
            Err(AiError::Malformed(_))
        ));
    }

    #[test]
    fn a_server_error_is_not_mistaken_for_a_result() {
        let host = stub_server("500 Internal Server Error", r#"{"error":"boom"}"#);

        let ollama = Ollama::new(host, "llama3.2:3b".to_string());

        assert!(ollama.installed_models().is_err());
    }

    #[test]
    fn an_untagged_model_matches_its_tagged_form() {
        let ollama = Ollama::new(DEFAULT_HOST.to_string(), "llama3.2".to_string());

        assert!(ollama.has_model(&["llama3.2:latest".to_string()]));
        assert!(ollama.has_model(&["llama3.2".to_string()]));
    }

    #[test]
    fn a_tagged_model_must_match_its_tag() {
        let ollama = Ollama::new(DEFAULT_HOST.to_string(), "llama3.2:3b".to_string());

        assert!(ollama.has_model(&["llama3.2:3b".to_string()]));
        assert!(!ollama.has_model(&["llama3.2:latest".to_string()]));
        assert!(!ollama.has_model(&["qwen2.5:3b".to_string()]));
        assert!(!ollama.has_model(&[]));
    }

    #[test]
    fn a_bare_host_and_port_gets_a_scheme() {
        assert_eq!(normalize_host("127.0.0.1:11434"), "http://127.0.0.1:11434");
        assert_eq!(normalize_host("  localhost:11434  "), "http://localhost:11434");
    }

    #[test]
    fn an_explicit_scheme_and_trailing_slash_are_respected() {
        assert_eq!(normalize_host("http://box:11434/"), "http://box:11434");
        assert_eq!(normalize_host("https://box:11434"), "https://box:11434");
    }
}
