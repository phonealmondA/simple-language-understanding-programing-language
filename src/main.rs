use anyhow::Result;
use clap::Parser;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use std::io::{self, Write};
use tracing::{info, warn};
use tracing_subscriber;

mod function_builder;
mod function_executor;
mod math_engine;
mod equation_solver;
mod variable_manager;
mod interactive_engine;
mod condition_evaluator;
mod loop_executor;
mod memory;
mod pattern_generator;

use function_builder::FunctionBuilder;
use function_executor::FunctionExecutor;
use math_engine::MathEngine;
use variable_manager::VariableManager;
use interactive_engine::InteractiveEngine;
use condition_evaluator::ConditionEvaluator;
use loop_executor::LoopExecutor;
use pattern_generator::{PatternGenerator, ProblemSpec};

#[derive(Parser)]
#[command(name = "quantum")]
#[command(about = "Quantum Consciousness Programming Language Transpiler")]
struct Args {
    
    file: Option<PathBuf>,
    
    #[arg(short, long, default_value = "1")]
    observations: u32,
    
    #[arg(short, long)]
    interactive: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct QuantumCache {
    templates: HashMap<String, CachedTemplate>,
    variables: HashMap<String, StoredVariable>,
    quantum_states: HashMap<String, CollapsedState>,
    variable_attempts: HashMap<String, Vec<VariableAttempt>>,
    built_functions: HashMap<String, BuiltFunction>,
    math_solutions: HashMap<String, MathSolution>,
    function_results: HashMap<String, FunctionResult>,
    // NEW: Pattern learning fields
    #[serde(default)]
    control_flow_patterns: HashMap<String, CachedPattern>,
    #[serde(default)]
    function_strategies: HashMap<String, FunctionStrategy>,
    #[serde(default)]
    algorithm_performances: HashMap<String, AlgorithmMetrics>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StoredVariable {
    pub name: String,
    pub value: VariableValue,
    pub timestamp: u64,
    pub source_equation: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum VariableValue {
    Number(f64),
    String(String),
    Boolean(bool),
    FunctionResult(String), 
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionResult {
    pub function_name: String,
    pub result: VariableValue,
    pub execution_time: f64,
    pub timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CachedTemplate {
    name: String,
    func_type: String,
    parameter_count: usize,
    timestamp: u64,
    is_built: bool,
    file_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuiltFunction {
    pub name: String,
    pub variants: Vec<FunctionVariant>,
    pub timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionVariant {
    pub parameter_count: usize,
    pub parameter_pattern: String,
    pub rust_function_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CollapsedState {
    result: f64,
    equation: String,
    accuracy: f64,
    timestamp: u64,
    calculation_time: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VariableAttempt {
    equation: String,
    result: f64,
    timestamp: u64,
    accuracy: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MathSolution {
    pub result: f64,
    pub equation: String,
    pub accuracy: f64,
    pub timestamp: u64,
    pub attempts: u32,
}

// NEW: Pattern learning structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedPattern {
    pub pattern_type: PatternType,
    pub structure: String,
    pub success_rate: f64,
    pub avg_iterations: f64,
    pub execution_time_ms: f64,
    pub problem_signature: String,
    pub timestamp: u64,
    pub times_used: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    CountLoop,
    RangeLoop,
    WhileLoop,
    ConditionalChain,
    NestedStructure,
    Hybrid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionStrategy {
    pub strategy_name: String,
    pub approach: Vec<String>,
    pub success_cases: Vec<String>,
    pub avg_performance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmMetrics {
    pub algorithm_name: String,
    pub iterations_taken: u32,
    pub memory_used: usize,
    pub execution_time_ms: f64,
    pub correctness_score: f64,
}

fn main() -> Result<()> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .init();

    let args = Args::parse();

    // If interactive mode requested, run CLI interactive engine
    if args.interactive {
        info!("** Quantum Consciousness Interactive Mode **");
        info!(">> Starting interactive mathematical reasoning engine");

        let mut interactive_engine = InteractiveEngine::new()?;
        interactive_engine.run_interactive_session()?;

        return Ok(());
    }

    // If file argument provided, run in CLI mode
    if let Some(file_path) = args.file {
        info!("** Quantum Consciousness Observer (Rust Edition)");
        info!(">> Building programs with variable storage, string interpolation, and function hierarchy");
        info!(">> Executing: {:?}", file_path);

        let mut transpiler = QuantumTranspiler::new()?;

        for i in 1..=args.observations {
            if args.observations > 1 {
                info!("== OBSERVATION {} ==", i);
            }

            transpiler.execute_file(&file_path)?;

            if i < args.observations {
                std::thread::sleep(std::time::Duration::from_secs(2));
            }
        }

        info!("** Complete!");
        return Ok(());
    }

    // No file and not interactive - show usage
    eprintln!("Error: No file specified");
    eprintln!();
    eprintln!("Usage:");
    eprintln!("  quantum <file.slut>              Run a .slut file");
    eprintln!("  quantum --interactive            Start interactive mode");
    eprintln!();
    eprintln!("To run the GUI, use: cd src-tauri && cargo tauri dev");

    std::process::exit(1);
}

pub struct QuantumTranspiler {
    cache: QuantumCache,
    execution_count: u32,
    function_builder: FunctionBuilder,
    function_executor: FunctionExecutor,
    pub math_engine: MathEngine,
    pub variable_manager: VariableManager,
    condition_evaluator: ConditionEvaluator,
    loop_executor: LoopExecutor,
    current_class_name: String,
    pattern_generator: PatternGenerator,
}

impl QuantumTranspiler {
    pub fn new() -> Result<Self> {
        // Ensure cache directory exists at initialization
        let cache_dir = PathBuf::from("cache");
        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir)?;
            info!("** Created cache directory: {}", cache_dir.display());
        } else {
            info!("** Using existing cache directory: {}", cache_dir.display());
        }

        let cache = Self::load_cache().unwrap_or_else(|_| {
            info!("** Starting with fresh quantum consciousness cache");
            info!("** Cache location: cache/quantum_consciousness_cache.json");
            QuantumCache {
                templates: HashMap::new(),
                variables: HashMap::new(),
                quantum_states: HashMap::new(),
                variable_attempts: HashMap::new(),
                built_functions: HashMap::new(),
                math_solutions: HashMap::new(),
                function_results: HashMap::new(),
                control_flow_patterns: HashMap::new(),
                function_strategies: HashMap::new(),
                algorithm_performances: HashMap::new(),
            }
        });

        if !cache.templates.is_empty() || !cache.built_functions.is_empty() || !cache.math_solutions.is_empty() {
            info!("** Loaded previous quantum states, built functions, variables, and math solutions from cache");
        }
        
        let function_builder = FunctionBuilder::new()?;
        let function_executor = FunctionExecutor::new()?;
        let math_engine = MathEngine::new(cache.math_solutions.clone(), cache.variable_attempts.clone());
        let variable_manager = VariableManager::new(cache.variables.clone());
        let condition_evaluator = ConditionEvaluator::new();
        let loop_executor = LoopExecutor::new();
        let pattern_generator = PatternGenerator::new(cache.control_flow_patterns.clone());

        if !cache.control_flow_patterns.is_empty() {
            info!("** Loaded {} cached control flow patterns", cache.control_flow_patterns.len());
        }

        Ok(Self {
            cache,
            execution_count: 0,
            function_builder,
            function_executor,
            math_engine,
            variable_manager,
            condition_evaluator,
            loop_executor,
            current_class_name: String::new(),
            pattern_generator,
        })
    }
    
    fn load_cache() -> Result<QuantumCache> {
        // Load from ./cache/ directory
        let cache_path = PathBuf::from("cache").join("quantum_consciousness_cache.json");
        let content = fs::read_to_string(&cache_path)?;
        let cache: QuantumCache = serde_json::from_str(&content)?;
        Ok(cache)
    }

    fn save_cache(&mut self) -> Result<()> {
        // Update cache with latest data from all engines
        self.cache.math_solutions = self.math_engine.get_solutions();
        self.cache.variable_attempts = self.math_engine.get_variable_attempts();
        self.cache.variables = self.variable_manager.get_all_variables();
        self.cache.control_flow_patterns = self.pattern_generator.get_cached_patterns().clone();

        // Ensure cache directory exists
        let cache_dir = PathBuf::from("cache");
        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir)?;
            info!("** Created cache directory: {}", cache_dir.display());
        }

        // Save JSON to ./cache/ directory
        let content = serde_json::to_string_pretty(&self.cache)?;
        let cache_path = cache_dir.join("quantum_consciousness_cache.json");
        fs::write(&cache_path, content)?;

        // Also save binary format to ./cache/ directory
        use memory::BinaryCache;
        let binary_cache_path = cache_dir.join("quantum_cache.bin");
        let binary_cache_path_str = binary_cache_path.to_string_lossy().to_string();

        if let Ok(binary_cache) = BinaryCache::from_hashmap_with_path(
            self.cache.math_solutions.clone(),
            &binary_cache_path_str
        ) {
            if let Err(e) = binary_cache.save_to_disk() {
                warn!("!! Failed to save binary cache: {}", e);
            } else {
                info!("** Binary cache saved to: {}", binary_cache_path_str);
            }
        }

        Ok(())
    }
    
    pub fn execute_file(&mut self, file_path: &PathBuf) -> Result<()> {
        // CRITICAL: Reload cache before each execution to pick up previous run's learning
        self.reload_cache()?;

        let source = fs::read_to_string(file_path)?;
        self.parse_and_execute(&source)?;
        self.save_cache()?;
        Ok(())
    }

    /// Reload cache from disk before execution to ensure continuity
    fn reload_cache(&mut self) -> Result<()> {
        if let Ok(cache) = Self::load_cache() {
            info!("** Reloading cache from previous runs...");

            // Update the in-memory cache
            self.cache = cache;

            // Sync with the engines
            self.math_engine = MathEngine::new(
                self.cache.math_solutions.clone(),
                self.cache.variable_attempts.clone()
            );
            self.variable_manager = VariableManager::new(self.cache.variables.clone());

            info!("** Cache reloaded: {} variables, {} solutions",
                  self.cache.variables.len(),
                  self.cache.math_solutions.len());
        }
        Ok(())
    }
    
    fn parse_and_execute(&mut self, source: &str) -> Result<()> {
        info!(">> Building program from your intentions...");

        self.extract_all_classes(source)?;

        let main_regex = Regex::new(r"\*\s*<main>\s*(\w+)\s*\{[^}]*\^\s*observe_execution\s*\{([\s\S]*?)\}\s*\}")?;

        if let Some(captures) = main_regex.captures(source) {
            let class_name = &captures[1];
            let body = &captures[2];

            info!(">> Quantum consciousness activated for: {}", class_name);
            self.current_class_name = class_name.to_string();
            self.execute_main_body(body, class_name)?;
            info!("** Program built and executed successfully!");
        } else {
            warn!("!! No main class found in source");
        }

        Ok(())
    }
    
    fn extract_all_classes(&mut self, source: &str) -> Result<()> {
        let class_regex = Regex::new(r"\*\s*(?:<main>\s*)?(\w+)\s*(?:\(\[([^\]]*)\]\))?\s*\{[\s\S]*?\^\s*observe_execution\s*\{([\s\S]*?)\}\s*\}")?;
        
        for captures in class_regex.captures_iter(source) {
            let class_name = &captures[1];
            let _parameters = captures.get(2).map(|m| m.as_str()).unwrap_or("");
            let body = &captures[3];
            
            if !captures[0].contains("<main>") {
                
                println!(">> Discovered function class: {}", class_name);
                self.cache.function_results.insert(
                    class_name.to_string(),
                    FunctionResult {
                        function_name: class_name.to_string(),
                        result: VariableValue::String(body.to_string()),
                        execution_time: 0.0,
                        timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64,
                    }
                );
            }
        }
        
        Ok(())
    }

    /// Execute pattern learning for a target-solving problem
    /// This tests multiple control flow patterns in parallel
    fn execute_pattern_learning(&mut self, target: f64, inputs: Vec<f64>, var_name: &str) -> Result<f64> {
        info!("");
        info!("🧠 QUANTUM PATTERN LEARNING ACTIVATED");
        info!(">> Target: {}, Inputs: {:?}", target, inputs);

        // Create problem specification
        let problem = ProblemSpec::new(target, inputs.clone());

        // Check if we have a cached pattern for this problem
        if let Some(cached_pattern) = self.pattern_generator.find_matching_pattern(&problem) {
            info!("⚡ Using cached pattern: {} (success rate: {:.1}%)",
                   cached_pattern.structure, cached_pattern.success_rate);
            info!("   Previous performance: {:.2}ms, {} iterations",
                   cached_pattern.execution_time_ms, cached_pattern.avg_iterations as u32);

            // Use existing math engine to solve (it's already optimized)
            let solution = self.math_engine.solve_target(target, &inputs, var_name, &self.current_class_name)?;
            return Ok(solution.result);
        }

        // No cached pattern - generate and test variants in parallel
        info!(">> No cached pattern found - testing multiple strategies in parallel...");
        let variants = self.pattern_generator.generate_pattern_variants(&problem);

        info!(">> Generated {} pattern variants", variants.len());
        for variant in &variants {
            info!("   - {} ({:?}): {}", variant.name, variant.pattern_type, variant.description);
        }

        // Test all patterns in parallel
        let test_result = self.pattern_generator.test_patterns_parallel(variants, &problem)?;

        info!("");
        info!("✓ PATTERN LEARNING COMPLETE");
        info!("   Best pattern: {} ({:?})",
               test_result.best_pattern.variant.name,
               test_result.best_pattern.variant.pattern_type);
        info!("   Performance: {:.1}% correct, {:.2}ms, {} iterations",
               test_result.best_pattern.correctness * 100.0,
               test_result.best_pattern.execution_time_ms,
               test_result.best_pattern.iterations);

        // Return the result from the best pattern
        if let Some(result) = test_result.best_pattern.result_value {
            Ok(result)
        } else {
            // Fallback to regular math engine
            info!("   Falling back to standard math engine");
            let solution = self.math_engine.solve_target(target, &inputs, var_name, &self.current_class_name)?;
            Ok(solution.result)
        }
    }

    fn execute_main_body(&mut self, body: &str, class_name: &str) -> Result<()> {
        let lines = body.lines().collect::<Vec<&str>>();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                i += 1;
                continue;
            }

            // Check if this is the start of a loop statement
            if line.starts_with("loop") && line.contains("<>") {
                // Collect the entire loop statement across multiple lines
                let mut full_statement = String::new();
                let mut brace_count = 0;
                let mut in_loop = false;

                while i < lines.len() {
                    let current_line = lines[i].trim();

                    if current_line.is_empty() {
                        i += 1;
                        continue;
                    }

                    // Track braces
                    brace_count += current_line.chars().filter(|&c| c == '{').count() as i32;
                    brace_count -= current_line.chars().filter(|&c| c == '}').count() as i32;

                    // Add line to statement
                    if full_statement.is_empty() {
                        full_statement.push_str(current_line);
                    } else {
                        full_statement.push('\n');
                        full_statement.push_str(current_line);
                    }

                    if brace_count > 0 {
                        in_loop = true;
                    }

                    i += 1;

                    // Break when we've closed all braces
                    if in_loop && brace_count == 0 {
                        break;
                    }
                }

                // Execute the complete loop statement
                self.execute_statement(&full_statement, class_name)?;
                continue;
            }

            // Check if this is the start of a selection statement
            if line.starts_with("if") && line.contains("<>") {
                // Collect the entire selection statement across multiple lines
                let mut full_statement = String::new();
                let mut brace_count = 0;
                let mut in_selection = false;
                let mut first_line = true;

                while i < lines.len() {
                    let current_line = lines[i].trim();

                    if current_line.is_empty() {
                        i += 1;
                        continue;
                    }

                    // Track braces BEFORE adding to statement
                    let has_open_brace = current_line.contains('{');
                    brace_count += current_line.chars().filter(|&c| c == '{').count() as i32;
                    brace_count -= current_line.chars().filter(|&c| c == '}').count() as i32;

                    // Add line to statement, preserving structure
                    if first_line {
                        // First line: "if <> (...) ... {"
                        full_statement.push_str(current_line);
                        first_line = false;
                    } else if has_open_brace {
                        // Line with opening brace
                        full_statement.push(' ');
                        full_statement.push_str(current_line);
                    } else if current_line == "<>" {
                        // Delimiter line - preserve with newline
                        full_statement.push('\n');
                        full_statement.push_str(current_line);
                        full_statement.push('\n');
                    } else if current_line.starts_with("<elif>") || current_line.starts_with("<else>") {
                        // Condition lines
                        full_statement.push(' ');
                        full_statement.push_str(current_line);
                    } else {
                        // Regular statement line
                        full_statement.push('\n');
                        full_statement.push_str(current_line);
                    }

                    if brace_count > 0 {
                        in_selection = true;
                    }

                    i += 1;

                    // Break when we've closed all braces
                    if in_selection && brace_count == 0 {
                        break;
                    }
                }

                // Execute the complete selection statement
                self.execute_statement(&full_statement, class_name)?;
            } else {
                // Regular single-line statement
                self.execute_statement(line, class_name)?;
                i += 1;
            }
        }
        Ok(())
    }
    
    fn execute_statement(&mut self, statement: &str, class_name: &str) -> Result<()> {
        // Check for break statement - must be checked FIRST
        if statement.trim() == "break" {
            self.loop_executor.signal_break();
            return Ok(());
        }

        // Check for continue statement
        if statement.trim() == "continue" {
            self.loop_executor.signal_continue();
            return Ok(());
        }

        // Check for selection statement (if/elif/else)
        let selection_regex = Regex::new(
            r"if\s*<>\s*\(([^)]+)\)((?:\s*<elif>\s*\([^)]+\))*)\s*<else>\s*\(([^)]+)\)\s*\{([\s\S]*?)\}"
        )?;

        if let Some(captures) = selection_regex.captures(statement) {
            let if_condition = &captures[1];
            let elif_part = &captures[2];
            let else_condition = &captures[3];
            let full_body = &captures[4];

            // Parse elif conditions
            let elif_regex = Regex::new(r"<elif>\s*\(([^)]+)\)")?;
            let elif_conditions: Vec<String> = elif_regex
                .captures_iter(elif_part)
                .map(|c| c[1].to_string())
                .collect();

            // Build complete conditions list
            let mut conditions = vec![if_condition.to_string()];
            conditions.extend(elif_conditions);
            conditions.push(else_condition.to_string());

            // Split body by standalone <> delimiter (not part of assignments)
            // A standalone <> appears at the start of a line with optional whitespace
            let bodies: Vec<String> = {
                let mut body_blocks = Vec::new();
                let mut current_block = String::new();

                for line in full_body.lines() {
                    let trimmed = line.trim();

                    // Check if this line is ONLY the delimiter
                    if trimmed == "<>" {
                        // Save current block and start new one
                        if !current_block.trim().is_empty() {
                            body_blocks.push(current_block.trim().to_string());
                        }
                        current_block.clear();
                    } else if !trimmed.is_empty() {
                        // Add line to current block
                        if !current_block.is_empty() {
                            current_block.push('\n');
                        }
                        current_block.push_str(trimmed);
                    }
                }

                // Don't forget the last block
                if !current_block.trim().is_empty() {
                    body_blocks.push(current_block.trim().to_string());
                }

                body_blocks
            };

            // Verify we have matching conditions and bodies
            if conditions.len() != bodies.len() {
                println!("!! Selection error: {} conditions but {} body blocks",
                        conditions.len(), bodies.len());
                return Ok(());
            }

            return self.execute_selection_statement(conditions, bodies, class_name);
        }

        // Check for count loop - PHASE 1
        let count_loop_regex = Regex::new(
            r"loop\s*<>\s*count\s*\(\s*([^)]+)\s*\)\s*\{([\s\S]*?)\}"
        )?;

        if let Some(captures) = count_loop_regex.captures(statement) {
            let count_expr = &captures[1];
            let body = &captures[2];

            return self.execute_count_loop(count_expr, body, class_name);
        }

        // Check for range loop - PHASE 2
        let range_loop_regex = Regex::new(
            r"loop\s*<>\s*range\s*\(\s*([^,]+)\s*,\s*([^)]+)\s*\)\s*as\s+(\w+)\s*\{([\s\S]*?)\}"
        )?;

        if let Some(captures) = range_loop_regex.captures(statement) {
            let start_expr = &captures[1];
            let end_expr = &captures[2];
            let loop_var = &captures[3];
            let body = &captures[4];

            return self.execute_range_loop(start_expr, end_expr, loop_var, body, class_name);
        }

        // Check for while loop - PHASE 3
        let while_loop_regex = Regex::new(
            r"loop\s*<>\s*while\s*\(\s*([^)]+)\s*\)\s*\{([\s\S]*?)\}"
        )?;

        if let Some(captures) = while_loop_regex.captures(statement) {
            let condition = &captures[1];
            let body = &captures[2];

            return self.execute_while_loop(condition, body, class_name);
        }

        let speak_interpolation_regex = Regex::new(r#"speak\s*\(\s*"([^"]*)"\s*\)"#)?;
        if let Some(captures) = speak_interpolation_regex.captures(statement) {
            let message = &captures[1];
            let interpolated = self.interpolate_string(message)?;
            println!("{}", interpolated);
            return Ok(());
        }
        
        
        let user_input_regex = Regex::new(r#"(\w+)\s*<>\s*userIn\s*\(\s*"([^"]*)"\s*\)"#)?;
        if let Some(captures) = user_input_regex.captures(statement) {
            let var_name = &captures[1];
            let prompt = &captures[2];
            return self.execute_user_input_assignment(var_name, prompt);
        }
        
        let var_function_regex = Regex::new(r"(\w+)\s*<>\s*(\w+)\s*\(\s*\)")?;
        if let Some(captures) = var_function_regex.captures(statement) {
            let var_name = &captures[1];
            let function_name = &captures[2];
            return self.execute_function_call_assignment(var_name, function_name, class_name);
        }
        
        let var_expression_regex = Regex::new(r"(\w+)\s*<>\s*(.+)")?;
        if let Some(captures) = var_expression_regex.captures(statement) {
            let var_name = &captures[1];
            let expression = &captures[2];
            return self.execute_variable_assignment(var_name, expression, class_name);
        }
        
        let target_math_regex = Regex::new(r"(\w+)\s*\(\s*\[\s*([^\]]+)\s*\]\s*\)\s*<>\s*randomChoice\s*\(\s*\[\s*([^\]]*)\s*\]\s*\)")?;
        if let Some(captures) = target_math_regex.captures(statement) {
            let var_name = &captures[1];
            let target_str = &captures[2];
            let inputs_str = &captures[3];
            return self.solve_target_math(var_name, target_str, inputs_str, class_name);
        }
        
        let poly_synthesis_regex = Regex::new(r"(\w+)\s*\(\s*([^)]*)\s*\)\s*<>\s*function\s*\(\s*(\w+)\s*\)")?;
        if let Some(captures) = poly_synthesis_regex.captures(statement) {
            let func_name = &captures[1];
            let params = &captures[2];
            let func_type = &captures[3];
            return self.synthesize_polymorphic_function(func_name, params, func_type);
        }
        
        let poly_exec_regex = Regex::new(r#"(\w+)\s*\(\s*([^)]+)\s*\)\s*\(\s*"((?:[^"\\]|\\.)*)"\s*\)"#)?;
        if let Some(captures) = poly_exec_regex.captures(statement) {
            let func_name = &captures[1];
            let params = &captures[2];
            let body = &captures[3];
            return self.execute_polymorphic_function(func_name, params, body);
        }
        
        
        let woof_regex = Regex::new(r"woof\s+(\w+)")?;
        if let Some(captures) = woof_regex.captures(statement) {
            let var_name = &captures[1];
            return self.output_variable(var_name);
        }
        
        Ok(())
    }
    
    fn execute_user_input_assignment(&mut self, var_name: &str, prompt: &str) -> Result<()> {
        print!("{}: ", prompt);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();


        let value = if let Ok(num) = input.parse::<f64>() {
            VariableValue::Number(num)
        } else {
            VariableValue::String(input.to_string())
        };

        self.variable_manager.store_variable(
            var_name,
            value,
            Some(format!("userIn(\"{}\")", prompt)),
        )?;

        Ok(())
    }
    
    fn interpolate_string(&self, message: &str) -> Result<String> {
        let var_regex = Regex::new(r"~(\w+)~")?;
        let mut result = message.to_string();
        
        for captures in var_regex.captures_iter(message) {
            let var_name = &captures[1];
            let placeholder = &captures[0];
            
            if let Some(variable) = self.variable_manager.get_variable(var_name) {
                let value_str = match &variable.value {
                    VariableValue::Number(n) => n.to_string(),
                    VariableValue::String(s) => s.clone(),
                    VariableValue::Boolean(b) => b.to_string(),
                    VariableValue::FunctionResult(f) => format!("[Function: {}]", f),
                };
                result = result.replace(placeholder, &value_str);
            } else {
                result = result.replace(placeholder, &format!("[undefined: {}]", var_name));
            }
        }
        
        Ok(result)
    }
    
    fn execute_function_call_assignment(&mut self, var_name: &str, function_name: &str, _class_name: &str) -> Result<()> {
        if let Some(_function_result) = self.cache.function_results.get(function_name) {
            let function_result = self.execute_function_body(function_name)?;

            self.variable_manager.store_variable(
                var_name,
                function_result,
                Some(format!("{}()", function_name)),
            )?;

        } else {
            println!("!! Function {} not found", function_name);
        }

        Ok(())
    }
    
    fn execute_function_body(&mut self, function_name: &str) -> Result<VariableValue> {

        if let Some(function_result) = self.cache.function_results.get(function_name) {
            let body = match &function_result.result {
                VariableValue::String(body_str) => body_str.clone(),
                _ => return Ok(VariableValue::String(format!("Invalid function: {}", function_name))),
            };
            
            let mut function_return_value = VariableValue::Number(0.0);
            
            for line in body.lines() {
                let line = line.trim();
                if line.is_empty() { continue; }
                
                let woof_regex = Regex::new(r"woof\s+(\w+)")?;
                if let Some(captures) = woof_regex.captures(line) {
                    let return_var = &captures[1];
                    
                    if let Some(variable) = self.variable_manager.get_variable(return_var) {
                        function_return_value = variable.value.clone();
                        println!("-- Function {} returning: {:?}", function_name, function_return_value);
                        break;
                    } else {
                        println!("!! Return variable '{}' not found in function {}", return_var, function_name);
                    }
                } else {
                    
                    self.execute_statement(line, function_name)?;
                }
            }
            
            Ok(function_return_value)
        } else {
            Err(anyhow::anyhow!("Function {} not found", function_name))
        }
    }
    
    fn execute_variable_assignment(&mut self, var_name: &str, expression: &str, _class_name: &str) -> Result<()> {
        // Silent execution (removed verbose logging)

        if expression.starts_with("calc(") && expression.ends_with(")") {

            let inner = &expression[5..expression.len()-1];
            let params: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();

            if params.len() >= 2 {
                let mut resolved_params = Vec::new();

                for param in &params {
                    if let Ok(num) = param.parse::<f64>() {

                        resolved_params.push(num);
                    } else if let Some(variable) = self.variable_manager.get_variable(param) {

                        if let VariableValue::Number(n) = variable.value {
                            resolved_params.push(n);
                        } else {
                            println!("!! Variable '{}' is not numeric", param);
                            return Ok(());
                        }
                    } else {
                        println!("!! Could not resolve parameter: {}", param);
                        return Ok(());
                    }
                }

                let result = if resolved_params.len() == 2 {
                    resolved_params[0] + resolved_params[1]
                } else if resolved_params.len() == 3 {
                    resolved_params[0] + resolved_params[1] + resolved_params[2]
                } else {
                    resolved_params.iter().sum()
                };

                self.variable_manager.store_variable(
                    var_name,
                    VariableValue::Number(result),
                    Some(format!("calc({})", inner)),
                )?;

            } else {
                println!("!! calc() requires at least 2 parameters");
            }
        } else if expression.starts_with("randomChoice(") {

            let choice_regex = Regex::new(r"randomChoice\s*\(\s*\[\s*([^\]]*)\s*\]\s*\)")?;
            if let Some(captures) = choice_regex.captures(expression) {
                let choices_str = &captures[1];
                let choice_parts: Vec<&str> = choices_str.split(',').map(|s| s.trim()).collect();

                let mut resolved_choices = Vec::new();

                for choice in &choice_parts {
                    if let Ok(num) = choice.parse::<f64>() {

                        resolved_choices.push(VariableValue::Number(num));
                    } else if let Some(variable) = self.variable_manager.get_variable(choice) {

                        resolved_choices.push(variable.value.clone());
                    } else {

                        resolved_choices.push(VariableValue::String(choice.trim_matches('"').to_string()));
                    }
                }

                if !resolved_choices.is_empty() {
                    use rand::Rng;
                    let mut rng = rand::thread_rng();
                    let chosen = &resolved_choices[rng.gen_range(0..resolved_choices.len())];

                    self.variable_manager.store_variable(
                        var_name,
                        chosen.clone(),
                        Some(format!("randomChoice({})", choices_str)),
                    )?;
                }
            }
        } else {

            let value = if let Ok(num) = expression.parse::<f64>() {
                VariableValue::Number(num)
            } else if expression == "true" || expression == "false" {
                VariableValue::Boolean(expression == "true")
            } else {
                VariableValue::String(expression.trim_matches('"').to_string())
            };

            self.variable_manager.store_variable(var_name, value, None)?;
        }
        
        Ok(())
    }
    
    fn output_variable(&self, var_name: &str) -> Result<()> {
        if let Some(variable) = self.variable_manager.get_variable(var_name) {
            match &variable.value {
                VariableValue::Number(n) => println!("Final result: {}", n),
                VariableValue::String(s) => println!("Final result: {}", s),
                VariableValue::Boolean(b) => println!("Final result: {}", b),
                VariableValue::FunctionResult(f) => println!("Final result: [Function: {}]", f),
            }
            
            if let Some(eq) = &variable.source_equation {
                println!("   Source: {}", eq);
            }
        } else {
            println!("!! Variable '{}' not found", var_name);
        }
        Ok(())
    }
    
    fn solve_target_math(&mut self, var_name: &str, target_str: &str, inputs_str: &str, class_name: &str) -> Result<()> {

        let target: f64 = if let Ok(num) = target_str.parse::<f64>() {
            num
        } else if let Some(variable) = self.variable_manager.get_variable(target_str) {
            match &variable.value {
                VariableValue::Number(n) => {
                    println!("-- Resolved target variable '{}' = {}", target_str, n);
                    *n
                },
                _ => {
                    println!("!! Target variable '{}' is not numeric", target_str);
                    return Ok(());
                }
            }
        } else {
            println!("!! Could not resolve target: {}", target_str);
            return Ok(());
        };

        let inputs = self.variable_manager.resolve_expression_inputs_with_target(inputs_str, Some(target));

        println!(">> Target-seeking quantum mathematics for variable '{}': target={}, inputs={:?}",
                var_name, target, inputs);

        // Enable pattern learning for complex problems (3+ inputs or targets > 100)
        let use_pattern_learning = inputs.len() >= 3 || target > 100.0;

        let result = if use_pattern_learning {
            info!(">> Pattern learning enabled (complex problem detected)");
            self.execute_pattern_learning(target, inputs, var_name)?
        } else {
            let solution = self.math_engine.solve_target(target, &inputs, var_name, class_name)?;
            solution.result
        };

        // Store the result
        let solution_eq = format!("pattern_learned_solution({})", target);
        self.variable_manager.store_variable(
            var_name,
            VariableValue::Number(result),
            Some(solution_eq.clone()),
        )?;

        println!("== Solution found: {} (value: {})", solution_eq, result);
        println!("-- Variable '{}' stored with value: {}", var_name, result);

        Ok(())
    }
    
    fn synthesize_polymorphic_function(&mut self, name: &str, params: &str, func_type: &str) -> Result<()> {
        let param_count = if params.trim().is_empty() { 0 } else { params.split(',').count() };
        let cache_key = format!("{}_{}_{}", name, func_type, param_count);
        
        println!(">> Synthesizing {} function: {} (supports {} parameters)", func_type, name, param_count);
        
        if let Some(template) = self.cache.templates.get(&cache_key) {
            if template.is_built {
                println!("== Using previously built function: {}", name);
                return Ok(());
            }
        }
        
        println!(">> Generating Rust code for function: {}", name);
        let built_function = self.function_builder.build_function(name, func_type, param_count)?;
        
        let template = CachedTemplate {
            name: name.to_string(),
            func_type: func_type.to_string(),
            parameter_count: param_count,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64,
            is_built: true,
            file_path: Some(format!("functions/src/{}.rs", name.to_lowercase())),
        };
        
        self.cache.templates.insert(cache_key, template);
        self.cache.built_functions.insert(name.to_string(), built_function);
        
        println!("** Function successfully built and cached: {}", name);
        Ok(())
    }
    
    fn execute_polymorphic_function(&mut self, func_name: &str, params: &str, body: &str) -> Result<()> {
        let param_list: Vec<&str> = params.split(',').map(|p| p.trim()).collect();

        println!(">> Executing built function: {}({}) with {} parameters",
                func_name, params, param_list.len());

        if let Some(built_function) = self.cache.built_functions.get(func_name) {
            self.function_executor.execute_function(built_function, &param_list, body)?;
        } else {
            println!("!! Function {} not found in built functions - needs synthesis first", func_name);
        }

        Ok(())
    }

    /// Execute a selection statement (if/elif/else)
    ///
    /// Evaluates conditions in order and executes the first matching branch
    fn execute_selection_statement(
        &mut self,
        conditions: Vec<String>,
        bodies: Vec<String>,
        class_name: &str
    ) -> Result<()> {
        println!(">> Evaluating selection statement with {} branches", conditions.len());

        // Get current variables for condition evaluation
        let variables = self.variable_manager.get_all_variables();

        // Evaluate each condition in order
        for (i, condition) in conditions.iter().enumerate() {
            let result = self.condition_evaluator.evaluate(condition, &variables)?;

            if result {
                println!("-- Condition {} evaluated to true: {}", i, condition);
                println!("-- Executing branch {}", i);

                // Execute the corresponding body block
                let body = &bodies[i];
                self.execute_body_block(body, class_name)?;

                return Ok(()); // Exit after first true condition
            } else {
                println!("-- Condition {} evaluated to false: {}", i, condition);
            }
        }

        // If we get here, something went wrong (else should always be true)
        println!("!! Warning: No condition matched (else should be true)");
        Ok(())
    }

    /// Execute statements within a body block
    ///
    /// Parses and executes multiple statements that may be separated by
    /// newlines or spaces
    fn execute_body_block(&mut self, body: &str, class_name: &str) -> Result<()> {
        // Silently execute body block (removed verbose logging)

        // Split by newlines first to handle multi-line bodies
        let lines: Vec<&str> = body.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect();

        // Handle multi-line statements (loops, selections)
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim();

            // Skip empty lines
            if line.is_empty() {
                i += 1;
                continue;
            }

            // Check if this is the start of a loop statement
            if line.starts_with("loop") && line.contains("<>") {
                let mut full_statement = String::new();
                let mut brace_count = 0;
                let mut in_loop = false;

                while i < lines.len() {
                    let current_line = lines[i].trim();

                    if current_line.is_empty() {
                        i += 1;
                        continue;
                    }

                    brace_count += current_line.chars().filter(|&c| c == '{').count() as i32;
                    brace_count -= current_line.chars().filter(|&c| c == '}').count() as i32;

                    if full_statement.is_empty() {
                        full_statement.push_str(current_line);
                    } else {
                        full_statement.push('\n');
                        full_statement.push_str(current_line);
                    }

                    if brace_count > 0 {
                        in_loop = true;
                    }

                    i += 1;

                    if in_loop && brace_count == 0 {
                        break;
                    }
                }

                self.execute_statement(&full_statement, class_name)?;

                // Check if we should exit early (continue or break)
                if self.loop_executor.should_skip_iteration() {
                    println!("   >> Skipping rest of iteration (continue)");
                    return Ok(());
                }
                continue;
            }

            // Regular single-line statement
            self.execute_statement(line, class_name)?;

            // Check if we should exit early (continue or break)
            if self.loop_executor.should_skip_iteration() {
                println!("   >> Skipping rest of iteration (continue)");
                return Ok(());
            }

            i += 1;
        }

        if lines.is_empty() {
            // Try splitting by spaces if no newlines
            let parts: Vec<&str> = body.split_whitespace()
                .filter(|s| !s.is_empty())
                .collect();

            // Reconstruct statements - this is a simple approach
            // that works for most cases
            let mut current_statement = String::new();
            let mut in_string = false;
            let mut paren_depth = 0;

            for part in parts {
                // Track string literals
                if part.contains('"') {
                    in_string = !in_string;
                }

                // Track parentheses
                paren_depth += part.chars().filter(|&c| c == '(').count() as i32;
                paren_depth -= part.chars().filter(|&c| c == ')').count() as i32;

                if current_statement.is_empty() {
                    current_statement = part.to_string();
                } else {
                    current_statement.push(' ');
                    current_statement.push_str(part);
                }

                // Execute when we have a complete statement
                if !in_string && paren_depth == 0 {
                    self.execute_statement(&current_statement, class_name)?;
                    current_statement.clear();
                }
            }

            // Execute any remaining statement
            if !current_statement.is_empty() {
                self.execute_statement(&current_statement, class_name)?;
            }
        }

        Ok(())
    }

    /// Execute a count-based loop
    fn execute_count_loop(
        &mut self,
        count_expr: &str,
        body: &str,
        class_name: &str
    ) -> Result<()> {

        // Resolve count expression (could be literal or variable)
        let count = if let Ok(num) = count_expr.trim().parse::<u32>() {
            // Direct number
            num
        } else if let Some(var) = self.variable_manager.get_variable(count_expr.trim()) {
            // Variable reference
            match &var.value {
                VariableValue::Number(n) => {
                    if *n >= 0.0 && n.fract() == 0.0 {
                        println!("-- Resolved count variable '{}' = {}", count_expr, n);
                        *n as u32
                    } else {
                        println!("!! Count must be a non-negative integer, got {}", n);
                        return Ok(());
                    }
                }
                _ => {
                    println!("!! Count variable '{}' is not numeric", count_expr);
                    return Ok(());
                }
            }
        } else {
            // Try evaluating as expression using math_engine
            let variables = self.variable_manager.get_all_variables();
            let mut var_map = HashMap::new();
            for (name, stored_var) in variables {
                var_map.insert(name, stored_var.value);
            }

            match self.math_engine.solve_expression(count_expr, &var_map) {
                Ok(result) if result >= 0.0 && result.fract() == 0.0 => {
                    println!("-- Evaluated count expression '{}' = {}", count_expr, result);
                    result as u32
                }
                Ok(result) => {
                    println!("!! Count expression result must be non-negative integer, got {}", result);
                    return Ok(());
                }
                Err(e) => {
                    println!("!! Could not resolve count expression '{}': {}", count_expr, e);
                    return Ok(());
                }
            }
        };

        // Execute the loop manually to avoid borrow checker issues
        self.loop_executor.loop_depth += 1;

        for _i in 0..count {
            self.loop_executor.should_continue = false;

            // Execute body
            self.execute_body_block(body, class_name)?;

            // Check for break
            if self.loop_executor.should_break {
                self.loop_executor.should_break = false;
                break;
            }
        }

        self.loop_executor.loop_depth -= 1;
        Ok(())
    }

    /// Execute a range-based loop with iterator variable
    fn execute_range_loop(
        &mut self,
        start_expr: &str,
        end_expr: &str,
        loop_var_name: &str,
        body: &str,
        class_name: &str
    ) -> Result<()> {

        // Helper to resolve expression to integer
        let resolve_to_int = |transpiler: &mut Self, expr: &str| -> Result<i32> {
            if let Ok(num) = expr.trim().parse::<i32>() {
                Ok(num)
            } else if let Some(var) = transpiler.variable_manager.get_variable(expr.trim()) {
                match &var.value {
                    VariableValue::Number(n) => Ok(*n as i32),
                    _ => Err(anyhow::anyhow!("Variable '{}' is not numeric", expr))
                }
            } else {
                let variables = transpiler.variable_manager.get_all_variables();
                let mut var_map = HashMap::new();
                for (name, stored_var) in variables {
                    var_map.insert(name, stored_var.value);
                }

                match transpiler.math_engine.solve_expression(expr, &var_map) {
                    Ok(result) => Ok(result as i32),
                    Err(e) => Err(anyhow::anyhow!("Could not evaluate '{}': {}", expr, e))
                }
            }
        };

        // Resolve start and end
        let start = resolve_to_int(self, start_expr)?;
        let end = resolve_to_int(self, end_expr)?;

        // Execute the loop manually to avoid borrow checker issues
        self.loop_executor.loop_depth += 1;

        for i in start..end {
            self.loop_executor.should_continue = false;

            // Store loop variable before executing body
            self.variable_manager.store_variable(
                loop_var_name,
                VariableValue::Number(i as f64),
                Some(format!("loop iterator")),
            )?;

            // Execute body
            self.execute_body_block(body, class_name)?;

            // Check for break
            if self.loop_executor.should_break {
                self.loop_executor.should_break = false;
                break;
            }
        }

        self.loop_executor.loop_depth -= 1;
        Ok(())
    }

    /// Execute a while loop with condition
    fn execute_while_loop(
        &mut self,
        condition: &str,
        body: &str,
        class_name: &str
    ) -> Result<()> {
        // Safety limit to prevent infinite loops
        const MAX_ITERATIONS: u32 = 10000;

        // Clone strings for closures
        let condition_str = condition.to_string();
        let body_str = body.to_string();
        let class_name_str = class_name.to_string();

        // We need to handle the borrow checker carefully here
        // Store whether we should continue in a variable outside the closures
        let mut should_continue = true;
        let mut iteration_count = 0;

        while should_continue && iteration_count < MAX_ITERATIONS {
            // Check condition
            let variables = self.variable_manager.get_all_variables();
            let condition_result = self.condition_evaluator.evaluate(&condition_str, &variables)?;

            if !condition_result {
                should_continue = false;
                break;
            }

            // Execute body
            if !self.loop_executor.should_skip_iteration() {
                self.execute_body_block(&body_str, &class_name_str)?;
            }

            // Check for break
            if self.loop_executor.should_break {
                self.loop_executor.should_break = false;
                break;
            }

            // Reset continue flag
            self.loop_executor.should_continue = false;
            iteration_count += 1;
        }

        if iteration_count >= MAX_ITERATIONS {
            println!("!! While loop hit max iterations ({})", MAX_ITERATIONS);
        }

        Ok(())
    }
}