use tauri::{State, AppHandle, Manager};
use std::sync::Mutex;
use std::path::PathBuf;
use anyhow::Result;

use crate::QuantumTranspiler;

/// Shared state that the UI can access
pub struct AppState {
    pub transpiler: Mutex<Option<QuantumTranspiler>>,
    pub is_running: Mutex<bool>,
    pub current_file: Mutex<Option<String>>,
    pub observation_count: Mutex<u32>,
    pub last_accuracy: Mutex<f64>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            transpiler: Mutex::new(None),
            is_running: Mutex::new(false),
            current_file: Mutex::new(None),
            observation_count: Mutex::new(0),
            last_accuracy: Mutex::new(0.0),
        }
    }
}

/// Get the cache directory for the application
/// Uses ./cache/ folder in project directory (ignored by .taurignore)
fn get_cache_directory(_app: &AppHandle) -> Result<PathBuf, String> {
    // Use current working directory + cache subfolder
    let current_dir = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?;

    let cache_dir = current_dir.join("cache");

    // Create directory if it doesn't exist
    if !cache_dir.exists() {
        std::fs::create_dir_all(&cache_dir)
            .map_err(|e| format!("Failed to create cache directory: {}", e))?;
        println!(">> Created cache directory: {}", cache_dir.display());
    }

    Ok(cache_dir)
}

/// Statistics returned to the frontend
#[derive(serde::Serialize, Default)]
pub struct CacheStats {
    pub variable_count: usize,
    pub solution_count: usize,
    pub last_accuracy: f64,
    pub observation_count: u32,
}

/// History entry for accuracy over time
#[derive(serde::Serialize, Clone)]
pub struct AccuracyHistoryEntry {
    pub equation: String,
    pub result: f64,
    pub accuracy: f64,
    pub timestamp: u64,
}

/// Cache history data
#[derive(serde::Serialize)]
pub struct CacheHistory {
    pub attempts: Vec<AccuracyHistoryEntry>,
    pub target: Option<f64>,
}

/// Console message structure for event emission
#[derive(serde::Serialize, Clone)]
pub struct ConsoleMessage {
    pub message: String,
    pub timestamp: u64,
    pub level: String,
}

/// Helper function to emit console output to the UI
fn emit_console(app: &AppHandle, message: String, level: &str) {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let console_msg = ConsoleMessage {
        message,
        timestamp,
        level: level.to_string(),
    };

    // Emit the event to the frontend
    let _ = app.emit_all("console-output", console_msg);
}

/// Command to run a .slut file once
#[tauri::command]
pub async fn run_file(
    file_path: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<String, String> {
    use std::io::{self, Write};
    use std::sync::{Arc, Mutex as StdMutex};

    let file_name = PathBuf::from(&file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&file_path)
        .to_string();

    emit_console(&app, format!("Starting execution: {}", file_name), "info");

    // Create a buffer to capture stdout
    let output_buffer = Arc::new(StdMutex::new(Vec::<u8>::new()));
    let buffer_clone = output_buffer.clone();

    // Capture stdout by redirecting prints
    // Note: This is a simplified approach; in production you'd use a proper stdout capture mechanism

    let mut transpiler_guard = state.transpiler.lock().unwrap();

    // Initialize transpiler if needed
    if transpiler_guard.is_none() {
        // Get cache directory from Tauri app data
        let cache_dir = get_cache_directory(&app)?;
        let cache_msg = format!("Using cache directory: {}", cache_dir.display());
        println!("{}", cache_msg);
        emit_console(&app, cache_msg, "info");

        match QuantumTranspiler::new_with_cache_dir(cache_dir) {
            Ok(mut trans) => {
                // Set up console callback to emit to Tauri IDE
                let app_clone = app.clone();
                trans.set_console_callback(std::sync::Arc::new(move |message, level| {
                    emit_console(&app_clone, message, level);
                }));

                emit_console(&app, "Transpiler initialized successfully".to_string(), "success");
                *transpiler_guard = Some(trans);
            }
            Err(e) => {
                let err_msg = format!("Failed to initialize transpiler: {}", e);
                emit_console(&app, err_msg.clone(), "error");
                return Err(err_msg);
            }
        }
    }

    let transpiler = transpiler_guard.as_mut().unwrap();

    // Execute the file
    emit_console(&app, format!("Executing {}...", file_name), "info");

    // Since we can't easily capture println! output, we'll return the full console output
    // by reading what the transpiler prints
    match transpiler.execute_file(&PathBuf::from(&file_path)) {
        Ok(_) => {
            // Increment observation count
            let mut obs_count = state.observation_count.lock().unwrap();
            *obs_count += 1;

            // Update current file
            let mut current_file = state.current_file.lock().unwrap();
            *current_file = Some(file_path.clone());

            let success_msg = format!("✓ Execution complete for observation {}", *obs_count);
            emit_console(&app, success_msg.clone(), "success");

            // Return the success message
            Ok(success_msg)
        }
        Err(e) => {
            let err_msg = format!("Execution error: {}", e);
            emit_console(&app, err_msg.clone(), "error");
            Err(err_msg)
        }
    }
}

/// Command to get current cache stats - ALWAYS reads from cache file
#[tauri::command]
pub fn get_cache_stats(app: AppHandle, state: State<'_, AppState>) -> Result<CacheStats, String> {
    use serde_json::Value;

    let cache_dir = get_cache_directory(&app)?;
    let cache_file = cache_dir.join("quantum_consciousness_cache.json");

    if !cache_file.exists() {
        return Ok(CacheStats::default());
    }

    // Read directly from cache file instead of in-memory state
    let contents = std::fs::read_to_string(&cache_file)
        .map_err(|e| format!("Failed to read cache file: {}", e))?;

    let cache_data: Value = serde_json::from_str(&contents)
        .map_err(|e| format!("Failed to parse cache JSON: {}", e))?;

    // Count variables
    let variable_count = cache_data
        .get("variables")
        .and_then(|v| v.as_object())
        .map(|obj| obj.len())
        .unwrap_or(0);

    // Count solutions
    let solution_count = cache_data
        .get("math_solutions")
        .and_then(|v| v.as_object())
        .map(|obj| obj.len())
        .unwrap_or(0);

    // Get last accuracy from variable_attempts
    let last_accuracy = cache_data
        .get("variable_attempts")
        .and_then(|v| v.get("result"))
        .and_then(|r| r.as_array())
        .and_then(|arr| arr.last())
        .and_then(|attempt| attempt.get("accuracy"))
        .and_then(|a| a.as_f64())
        .unwrap_or(0.0);

    // Count observation attempts
    let observation_count = cache_data
        .get("variable_attempts")
        .and_then(|v| v.get("result"))
        .and_then(|r| r.as_array())
        .map(|arr| arr.len() as u32)
        .unwrap_or(0);

    Ok(CacheStats {
        variable_count,
        solution_count,
        last_accuracy,
        observation_count,
    })
}

/// Command to run until solved (with max attempts)
#[tauri::command]
pub async fn run_until_solved(
    file_path: String,
    max_attempts: u32,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<String, String> {
    // Set running flag
    *state.is_running.lock().unwrap() = true;

    emit_console(&app, format!("Starting loop mode (max {} attempts)", max_attempts), "info");

    let mut attempts = 0;
    let mut best_accuracy = 0.0;

    while attempts < max_attempts && best_accuracy < 100.0 {
        // Check if user stopped it
        if !*state.is_running.lock().unwrap() {
            let stop_msg = format!("Stopped after {} attempts", attempts);
            emit_console(&app, stop_msg.clone(), "info");
            return Ok(stop_msg);
        }

        // Run the file
        emit_console(&app, format!("Attempt {}/{}", attempts + 1, max_attempts), "info");

        match run_file(file_path.clone(), state.clone(), app.clone()).await {
            Ok(msg) => {
                println!("{}", msg);
            }
            Err(e) => {
                *state.is_running.lock().unwrap() = false;
                emit_console(&app, format!("Error: {}", e), "error");
                return Err(e);
            }
        }

        // Get updated stats
        let stats = get_cache_stats(app.clone(), state.clone()).map_err(|e| e.to_string())?;
        best_accuracy = stats.last_accuracy;

        // Update accuracy in state
        *state.last_accuracy.lock().unwrap() = best_accuracy;

        attempts += 1;

        emit_console(&app, format!("Current accuracy: {:.1}%", best_accuracy), "info");

        // Check if solved
        if best_accuracy >= 100.0 {
            let success_msg = format!("🎉 Solved in {} attempts with {:.1}% accuracy!", attempts, best_accuracy);
            emit_console(&app, success_msg.clone(), "success");
            break;
        }

        // Small delay between attempts
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    // Clear running flag
    *state.is_running.lock().unwrap() = false;

    let final_msg = format!("Completed {} attempts. Best accuracy: {:.1}%", attempts, best_accuracy);
    emit_console(&app, final_msg.clone(), "success");
    Ok(final_msg)
}

/// Command to stop running execution
#[tauri::command]
pub fn stop_execution(state: State<'_, AppState>) -> Result<(), String> {
    *state.is_running.lock().unwrap() = false;
    Ok(())
}

/// Command to reset the transpiler state
#[tauri::command]
pub fn reset_transpiler(state: State<'_, AppState>, app: AppHandle) -> Result<(), String> {
    let mut transpiler_guard = state.transpiler.lock().unwrap();

    // Optionally clear the cache file
    if let Ok(cache_dir) = get_cache_directory(&app) {
        let cache_file = cache_dir.join("quantum_consciousness_cache.json");
        if cache_file.exists() {
            let _ = std::fs::remove_file(&cache_file);
            println!(">> Cleared cache file: {}", cache_file.display());
        }
    }

    *transpiler_guard = None;

    *state.observation_count.lock().unwrap() = 0;
    *state.last_accuracy.lock().unwrap() = 0.0;
    *state.current_file.lock().unwrap() = None;
    *state.is_running.lock().unwrap() = false;

    Ok(())
}

/// Command to get the current working directory
#[tauri::command]
pub fn get_working_directory() -> Result<String, String> {
    std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| format!("Failed to get working directory: {}", e))
}

/// Command to get cache history from the JSON file
#[tauri::command]
pub fn get_cache_history(app: AppHandle) -> Result<CacheHistory, String> {
    use serde_json::Value;

    let cache_dir = get_cache_directory(&app)?;
    let cache_file = cache_dir.join("quantum_consciousness_cache.json");

    if !cache_file.exists() {
        return Ok(CacheHistory {
            attempts: vec![],
            target: None,
        });
    }

    // Read the cache file
    let contents = std::fs::read_to_string(&cache_file)
        .map_err(|e| format!("Failed to read cache file: {}", e))?;

    // Parse JSON
    let cache_data: Value = serde_json::from_str(&contents)
        .map_err(|e| format!("Failed to parse cache JSON: {}", e))?;

    // Extract target number from variables
    let target = cache_data
        .get("variables")
        .and_then(|v| v.get("targetNum"))
        .and_then(|t| t.get("value"))
        .and_then(|val| val.get("Number"))
        .and_then(|n| n.as_f64());

    // Extract variable attempts (focus on "result" variable)
    let mut attempts = Vec::new();

    if let Some(var_attempts) = cache_data.get("variable_attempts") {
        if let Some(result_attempts) = var_attempts.get("result") {
            if let Some(arr) = result_attempts.as_array() {
                for entry in arr {
                    if let (Some(eq), Some(res), Some(acc), Some(ts)) = (
                        entry.get("equation").and_then(|e| e.as_str()),
                        entry.get("result").and_then(|r| r.as_f64()),
                        entry.get("accuracy").and_then(|a| a.as_f64()),
                        entry.get("timestamp").and_then(|t| t.as_u64()),
                    ) {
                        attempts.push(AccuracyHistoryEntry {
                            equation: eq.to_string(),
                            result: res,
                            accuracy: acc,
                            timestamp: ts,
                        });
                    }
                }
            }
        }
    }

    Ok(CacheHistory { attempts, target })
}

/// Command to clear in-memory state (call on window close or manual reset)
#[tauri::command]
pub fn clear_memory_state(state: State<'_, AppState>) -> Result<(), String> {
    println!(">> Clearing in-memory state");

    *state.transpiler.lock().unwrap() = None;
    *state.observation_count.lock().unwrap() = 0;
    *state.last_accuracy.lock().unwrap() = 0.0;
    *state.current_file.lock().unwrap() = None;
    *state.is_running.lock().unwrap() = false;

    Ok(())
}

// ============================================================================
// PATTERN LEARNING COMMANDS
// ============================================================================

/// Pattern learning data for frontend visualization
#[derive(serde::Serialize, Clone)]
pub struct PatternLearningData {
    pub patterns: Vec<PatternInfo>,
    pub total_patterns: usize,
    pub most_used_pattern: Option<String>,
    pub avg_success_rate: f64,
}

/// Individual pattern information
#[derive(serde::Serialize, Clone)]
pub struct PatternInfo {
    pub name: String,
    pub pattern_type: String,
    pub success_rate: f64,
    pub avg_iterations: f64,
    pub execution_time_ms: f64,
    pub problem_signature: String,
    pub times_used: u32,
    pub timestamp: u64,
}

/// Command to get all cached patterns
#[tauri::command]
pub fn get_pattern_learning_data(
    app: AppHandle,
    _state: State<'_, AppState>,
) -> Result<PatternLearningData, String> {
    let cache_dir = get_cache_directory(&app)?;
    let cache_path = cache_dir.join("quantum_consciousness_cache.json");

    // If no cache exists, return empty data
    if !cache_path.exists() {
        return Ok(PatternLearningData {
            patterns: vec![],
            total_patterns: 0,
            most_used_pattern: None,
            avg_success_rate: 0.0,
        });
    }

    // Read cache
    let cache_content = std::fs::read_to_string(&cache_path)
        .map_err(|e| format!("Failed to read cache: {}", e))?;

    let cache_data: serde_json::Value = serde_json::from_str(&cache_content)
        .map_err(|e| format!("Failed to parse cache: {}", e))?;

    // Extract patterns
    let mut patterns = Vec::new();

    if let Some(patterns_obj) = cache_data.get("control_flow_patterns") {
        if let Some(patterns_map) = patterns_obj.as_object() {
            for (_key, pattern_data) in patterns_map {
                if let (
                    Some(structure),
                    Some(pattern_type),
                    Some(success_rate),
                    Some(avg_iterations),
                    Some(execution_time_ms),
                    Some(problem_signature),
                    Some(times_used),
                    Some(timestamp),
                ) = (
                    pattern_data.get("structure").and_then(|s| s.as_str()),
                    pattern_data.get("pattern_type"),
                    pattern_data.get("success_rate").and_then(|s| s.as_f64()),
                    pattern_data.get("avg_iterations").and_then(|a| a.as_f64()),
                    pattern_data.get("execution_time_ms").and_then(|e| e.as_f64()),
                    pattern_data.get("problem_signature").and_then(|p| p.as_str()),
                    pattern_data.get("times_used").and_then(|t| t.as_u64()),
                    pattern_data.get("timestamp").and_then(|t| t.as_u64()),
                ) {
                    // Convert pattern_type enum to string
                    let pattern_type_str = match pattern_type.as_str() {
                        Some(s) => s.to_string(),
                        None => format!("{:?}", pattern_type),
                    };

                    patterns.push(PatternInfo {
                        name: structure.to_string(),
                        pattern_type: pattern_type_str,
                        success_rate,
                        avg_iterations,
                        execution_time_ms,
                        problem_signature: problem_signature.to_string(),
                        times_used: times_used as u32,
                        timestamp,
                    });
                }
            }
        }
    }

    // Calculate statistics
    let total_patterns = patterns.len();
    let avg_success_rate = if total_patterns > 0 {
        patterns.iter().map(|p| p.success_rate).sum::<f64>() / total_patterns as f64
    } else {
        0.0
    };

    let most_used_pattern = patterns
        .iter()
        .max_by_key(|p| p.times_used)
        .map(|p| p.name.clone());

    Ok(PatternLearningData {
        patterns,
        total_patterns,
        most_used_pattern,
        avg_success_rate,
    })
}

/// Pattern test result for real-time visualization
#[derive(serde::Serialize, Clone)]
pub struct PatternTestEvent {
    pub variant_name: String,
    pub pattern_type: String,
    pub correctness: f64,
    pub execution_time_ms: f64,
    pub iterations: u32,
    pub is_best: bool,
}

/// Command to get pattern learning statistics summary
#[tauri::command]
pub fn get_pattern_stats(
    app: AppHandle,
    _state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let data = get_pattern_learning_data(app, _state)?;

    // Group patterns by type
    let mut by_type: std::collections::HashMap<String, Vec<&PatternInfo>> =
        std::collections::HashMap::new();

    for pattern in &data.patterns {
        by_type
            .entry(pattern.pattern_type.clone())
            .or_insert_with(Vec::new)
            .push(pattern);
    }

    // Calculate stats by type
    let mut type_stats = serde_json::Map::new();
    for (ptype, patterns) in by_type {
        let count = patterns.len();
        let avg_success = patterns.iter().map(|p| p.success_rate).sum::<f64>() / count as f64;
        let avg_time = patterns.iter().map(|p| p.execution_time_ms).sum::<f64>() / count as f64;

        let mut stats = serde_json::Map::new();
        stats.insert("count".to_string(), serde_json::json!(count));
        stats.insert("avg_success_rate".to_string(), serde_json::json!(avg_success));
        stats.insert("avg_time_ms".to_string(), serde_json::json!(avg_time));

        type_stats.insert(ptype, serde_json::Value::Object(stats));
    }

    Ok(serde_json::json!({
        "total_patterns": data.total_patterns,
        "avg_success_rate": data.avg_success_rate,
        "most_used": data.most_used_pattern,
        "by_type": type_stats,
    }))
}
