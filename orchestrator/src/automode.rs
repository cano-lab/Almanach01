use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing;

// ─── Data Structures ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelManifest {
    pub id: String,
    pub friendly_name: String,  // e.g. "Kimi K2.7"
    pub description: String,    // e.g. "Strong coding and long-context model from China"
    pub provider: String,       // "kimi", "augure", "fugu", "lm_studio"
    pub model_string: String,   // the API identifier
    pub endpoint: String,       // "cloud" | "local"
    pub context_window: i64,
    pub max_output_tokens: i64,
    pub vision: bool,
    pub languages: Vec<String>, // ["fr", "en"]
    pub tool_use: bool,
    pub structured_output: bool,
    pub cost_tier: String,      // "free" | "low" | "medium" | "high"
    pub latency_tier: String,   // "fast" | "medium" | "slow"
    pub strengths: Vec<String>, // soft tags: ["gentle-explainer", "franco-ontarian"]
    pub enabled: bool,
    // orchestrator info: if this model is itself an orchestrator
    pub is_orchestrator: bool,
    pub orchestrator_type: Option<String>, // "kimi", "augure", "fugu", "local"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleManifest {
    pub id: String,
    pub name: String,
    pub description: String,
    pub requires_vision: bool,
    pub required_languages: Vec<String>,
    pub min_context_window: i64,
    pub temperature: f32,
    pub default_sensitivity: String, // "low" | "high"
    pub preferred_strengths: Vec<String>,
    pub recommended_models: Vec<String>, // explicit role -> model mapping (ids)
    pub fallback_policy: String, // "relax_soft" | "escalate_to_teacher"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomodeConfig {
    pub enabled: bool,
    pub router_model: String,       // e.g. "augure-nano"
    pub router_provider: String,    // e.g. "augure"
    pub models: Vec<ModelManifest>,
    pub roles: Vec<RoleManifest>,
    pub default_role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRecord {
    pub role: String,
    pub chosen_model: Option<String>, // None if escalation
    pub chosen_orchestrator: Option<String>, // "kimi", "augure", "fugu", "local"
    pub eligible: Vec<String>,
    pub filtered_out: Vec<FilteredOut>,
    pub method: String, // "deterministic" | "llm_tiebreak" | "fallback" | "escalation"
    pub reason: String,
    pub sensitivity: String,
    pub llm_used: bool,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilteredOut {
    pub model: String,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct RouteRequest {
    pub role_id: String,
    pub payload_summary: String, // non-sensitive task descriptor
    pub sensitivity: Option<String>, // override role default
    pub grade_level: Option<String>,
}

// ─── Router ───────────────────────────────────────────────────────────────────

pub struct AutomodeRouter {
    config: AutomodeConfig,
    api_keys: Arc<RwLock<HashMap<String, String>>>,
}

impl AutomodeRouter {
    pub fn new(config: AutomodeConfig, api_keys: Arc<RwLock<HashMap<String, String>>>) -> Self {
        Self { config, api_keys }
    }

    pub async fn route(&self, req: RouteRequest) -> DecisionRecord {
        let timestamp = chrono::Utc::now().to_rfc3339();
        
        // Find the role
        let role = match self.config.roles.iter().find(|r| r.id == req.role_id) {
            Some(r) => r.clone(),
            None => {
                return DecisionRecord {
                    role: req.role_id.clone(),
                    chosen_model: None,
                    chosen_orchestrator: None,
                    eligible: vec![],
                    filtered_out: vec![],
                    method: "escalation".to_string(),
                    reason: format!("Role '{}' not found in manifest", req.role_id),
                    sensitivity: req.sensitivity.clone().unwrap_or_else(|| "low".to_string()),
                    llm_used: false,
                    timestamp,
                };
            }
        };

        let sensitivity = req.sensitivity.clone().unwrap_or_else(|| role.default_sensitivity.clone());
        let pool: Vec<ModelManifest> = self.config.models.iter().filter(|m| m.enabled).cloned().collect();

        // ── HARD FILTER (code, no LLM) ─────────────────────────────────────
        let mut eligible: Vec<ModelManifest> = vec![];
        let mut filtered_out: Vec<FilteredOut> = vec![];

        for m in &pool {
            let mut reasons = vec![];

            if role.requires_vision && !m.vision {
                reasons.push("missing vision capability".to_string());
            }
            for lang in &role.required_languages {
                if !m.languages.contains(lang) {
                    reasons.push(format!("missing language: {}", lang));
                }
            }
            if m.context_window < role.min_context_window {
                reasons.push(format!("context too small: {} < {}", m.context_window, role.min_context_window));
            }
            if sensitivity == "high" && m.endpoint != "local" {
                reasons.push("not local (high sensitivity)".to_string());
            }

            if reasons.is_empty() {
                eligible.push(m.clone());
            } else {
                filtered_out.push(FilteredOut {
                    model: m.id.clone(),
                    reason: reasons.join(", "),
                });
            }
        }

        // ── CASE: 0 eligible ─────────────────────────────────────────────────
        if eligible.is_empty() {
            return self.handle_zero_eligible(&role, &req, &pool, &filtered_out, &sensitivity, timestamp).await;
        }

        // ── CASE: 1 eligible → deterministic ─────────────────────────────────
        if eligible.len() == 1 {
            let chosen = &eligible[0];
            return DecisionRecord {
                role: role.id.clone(),
                chosen_model: Some(chosen.model_string.clone()),
                chosen_orchestrator: chosen.orchestrator_type.clone(),
                eligible: eligible.iter().map(|m| m.id.clone()).collect(),
                filtered_out,
                method: "deterministic".to_string(),
                reason: "Only model meeting the requirements".to_string(),
                sensitivity: sensitivity.clone(),
                llm_used: false,
                timestamp,
            };
        }

        // ── CASE: >1 eligible → LLM tie-break ────────────────────────────────
        self.llm_tiebreak(&role, &req, &eligible, &filtered_out, &sensitivity, timestamp).await
    }

    async fn handle_zero_eligible(
        &self,
        role: &RoleManifest,
        req: &RouteRequest,
        _pool: &[ModelManifest],
        filtered_out: &[FilteredOut],
        sensitivity: &str,
        timestamp: String,
    ) -> DecisionRecord {
        match role.fallback_policy.as_str() {
            "relax_soft" => {
                // Relax min_context_window only; never relax vision/language/sensitivity
                let mut relaxed_eligible: Vec<ModelManifest> = vec![];
                for m in &self.config.models {
                    if !m.enabled { continue; }
                    let mut ok = true;
                    if role.requires_vision && !m.vision { ok = false; }
                    for lang in &role.required_languages {
                        if !m.languages.contains(lang) { ok = false; }
                    }
                    if sensitivity == "high" && m.endpoint != "local" { ok = false; }
                    if ok {
                        relaxed_eligible.push(m.clone());
                    }
                }

                if relaxed_eligible.is_empty() {
                    DecisionRecord {
                        role: role.id.clone(),
                        chosen_model: None,
                        chosen_orchestrator: None,
                        eligible: vec![],
                        filtered_out: filtered_out.to_vec(),
                        method: "escalation".to_string(),
                        reason: "No models meet even relaxed requirements".to_string(),
                        sensitivity: sensitivity.to_string(),
                        llm_used: false,
                        timestamp,
                    }
                } else if relaxed_eligible.len() == 1 {
                    let chosen = &relaxed_eligible[0];
                    DecisionRecord {
                        role: role.id.clone(),
                        chosen_model: Some(chosen.model_string.clone()),
                        chosen_orchestrator: chosen.orchestrator_type.clone(),
                        eligible: relaxed_eligible.iter().map(|m| m.id.clone()).collect(),
                        filtered_out: filtered_out.to_vec(),
                        method: "fallback".to_string(),
                        reason: "Only model after relaxing context-window requirement".to_string(),
                        sensitivity: sensitivity.to_string(),
                        llm_used: false,
                        timestamp,
                    }
                } else {
                    self.llm_tiebreak(role, req, &relaxed_eligible, filtered_out, sensitivity, timestamp).await
                }
            }
            _ => {
                // escalate_to_teacher
                let unmet = filtered_out.iter()
                    .map(|f| format!("{}: {}", f.model, f.reason))
                    .collect::<Vec<_>>()
                    .join("; ");
                DecisionRecord {
                    role: role.id.clone(),
                    chosen_model: None,
                    chosen_orchestrator: None,
                    eligible: vec![],
                    filtered_out: filtered_out.to_vec(),
                    method: "escalation".to_string(),
                    reason: format!("No enabled model meets requirements: {}", unmet),
                    sensitivity: sensitivity.to_string(),
                    llm_used: false,
                    timestamp,
                }
            }
        }
    }

    async fn llm_tiebreak(
        &self,
        role: &RoleManifest,
        req: &RouteRequest,
        eligible: &[ModelManifest],
        filtered_out: &[FilteredOut],
        sensitivity: &str,
        timestamp: String,
    ) -> DecisionRecord {
        // Build router prompt
        let model_profiles: Vec<String> = eligible.iter().map(|m| {
            format!(
                "- {}: strengths=[{}], latency={}, cost={}",
                m.id,
                m.strengths.join(", "),
                m.latency_tier,
                m.cost_tier
            )
        }).collect();

        let prompt = format!(
            "You are a model router for a Grade 6-8 AI classroom.\n\
            Task: {}\n\
            Role: {} — {}\n\
            Preferred strengths: {}\n\
            Eligible models:\n{}\n\
            Choose the best model for this task. Return strict JSON:\n\
            {{\"model_id\": \"<one of eligible ids>\", \"reason\": \"<plain language, <=20 words>\"}}",
            req.payload_summary,
            role.name,
            role.description,
            role.preferred_strengths.join(", "),
            model_profiles.join("\n")
        );

        // Call the router model (augure-nano or configured)
        let router_model = &self.config.router_model;
        let router_provider = &self.config.router_provider;
        
        let choice = match self.call_router_model(&prompt, router_provider, router_model).await {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("Router LLM failed: {}. Using heuristic.", e);
                return self.heuristic_pick(role, eligible, filtered_out, sensitivity, timestamp);
            }
        };

        // Validate choice is eligible
        let chosen = match eligible.iter().find(|m| m.id == choice.model_id) {
            Some(m) => m.clone(),
            None => {
                tracing::warn!("Router LLM chose invalid model '{}'. Using heuristic.", choice.model_id);
                return self.heuristic_pick(role, eligible, filtered_out, sensitivity, timestamp);
            }
        };

        DecisionRecord {
            role: role.id.clone(),
            chosen_model: Some(chosen.model_string.clone()),
            chosen_orchestrator: chosen.orchestrator_type.clone(),
            eligible: eligible.iter().map(|m| m.id.clone()).collect(),
            filtered_out: filtered_out.to_vec(),
            method: "llm_tiebreak".to_string(),
            reason: choice.reason,
            sensitivity: sensitivity.to_string(),
            llm_used: true,
            timestamp,
        }
    }

    fn heuristic_pick(
        &self,
        role: &RoleManifest,
        eligible: &[ModelManifest],
        filtered_out: &[FilteredOut],
        sensitivity: &str,
        timestamp: String,
    ) -> DecisionRecord {
        // Score: preferred strengths matches, then latency, then cost
        let mut scored: Vec<(i64, &ModelManifest)> = eligible.iter().map(|m| {
            let strength_matches = m.strengths.iter()
                .filter(|s| role.preferred_strengths.contains(*s))
                .count() as i64;
            let latency_score = match m.latency_tier.as_str() {
                "fast" => 3,
                "medium" => 2,
                "slow" => 1,
                _ => 0,
            };
            let cost_score = match m.cost_tier.as_str() {
                "free" => 3,
                "low" => 2,
                "medium" => 1,
                _ => 0,
            };
            let total = strength_matches * 100 + latency_score * 10 + cost_score;
            (total, m)
        }).collect();

        scored.sort_by(|a, b| b.0.cmp(&a.0)); // highest first

        let chosen = scored.first().map(|(_, m)| *m).unwrap_or(&eligible[0]).clone();

        DecisionRecord {
            role: role.id.clone(),
            chosen_model: Some(chosen.model_string.clone()),
            chosen_orchestrator: chosen.orchestrator_type.clone(),
            eligible: eligible.iter().map(|m| m.id.clone()).collect(),
            filtered_out: filtered_out.to_vec(),
            method: "llm_tiebreak".to_string(),
            reason: "Heuristic: best strength match + latency + cost".to_string(),
            sensitivity: sensitivity.to_string(),
            llm_used: false, // heuristic, no LLM
            timestamp,
        }
    }

    async fn call_router_model(
        &self,
        prompt: &str,
        provider: &str,
        model: &str,
    ) -> Result<RouterChoice, Box<dyn std::error::Error + Send + Sync>> {
        // Get API key
        let api_key = {
            let keys = self.api_keys.read().await;
            keys.get(provider)
                .cloned()
                .ok_or_else(|| format!("No API key for router provider: {}", provider))?
        };

        // Build endpoint
        let endpoint = match provider {
            "augure" => "https://api.augureai.ca/v1/chat/completions",
            "kimi" => "https://api.moonshot.cn/v1/chat/completions",
            "fugu" => "https://api.sakana.ai/v1/chat/completions",
            "openai" => "https://api.openai.com/v1/chat/completions",
            _ => return Err(format!("Unknown router provider: {}", provider).into()),
        };

        let client = reqwest::Client::new();
        let body = serde_json::json!({
            "model": model,
            "messages": [
                {"role": "system", "content": "You are a model router. Return only strict JSON."},
                {"role": "user", "content": prompt}
            ],
            "max_tokens": 256,
            "temperature": 0.1,
        });

        let response = client
            .post(endpoint)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Router request failed: {}", e))?;

        let resp_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse router response: {}", e))?;

        let content = resp_json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or("No content in router response")?;

        // Parse JSON from content (model may wrap in markdown)
        let json_str = if content.contains("```json") {
            content.split("```json").nth(1).unwrap_or(content)
                .split("```").next().unwrap_or(content)
                .trim()
        } else if content.contains("```") {
            content.split("```").nth(1).unwrap_or(content)
                .split("```").next().unwrap_or(content)
                .trim()
        } else {
            content.trim()
        };

        let choice: RouterChoice = serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse router JSON: {}. Raw: {}", e, json_str))?;

        Ok(choice)
    }
}

#[derive(Debug, Clone, Deserialize)]
struct RouterChoice {
    model_id: String,
    reason: String,
}

// ─── Config Loading ─────────────────────────────────────────────────────────

use std::path::Path;

pub fn load_automode_config(data_dir: &Path) -> anyhow::Result<AutomodeConfig> {
    let config_path = data_dir.join("automode.json");
    if config_path.exists() {
        let contents = std::fs::read_to_string(&config_path)?;
        let config: AutomodeConfig = serde_json::from_str(&contents)?;
        Ok(config)
    } else {
        // Return default config with no models/roles
        Ok(AutomodeConfig {
            enabled: false,
            router_model: "augure-nano".to_string(),
            router_provider: "augure".to_string(),
            models: vec![],
            roles: vec![],
            default_role: "explainer".to_string(),
        })
    }
}

pub fn save_automode_config(data_dir: &Path, config: &AutomodeConfig) -> anyhow::Result<()> {
    let config_path = data_dir.join("automode.json");
    let contents = serde_json::to_string_pretty(config)?;
    std::fs::write(&config_path, contents)?;
    Ok(())
}
