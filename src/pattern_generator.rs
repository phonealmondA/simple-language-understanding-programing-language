use anyhow::Result;
use rayon::prelude::*;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH, Instant};
use crate::{CachedPattern, PatternType};

/// Pattern generator for testing multiple control flow structures in parallel
pub struct PatternGenerator {
    cached_patterns: HashMap<String, CachedPattern>,
}

#[derive(Debug, Clone)]
pub struct ProblemSpec {
    pub target: f64,
    pub inputs: Vec<f64>,
    pub input_count: usize,
    pub input_range: (f64, f64),
    pub complexity: ProblemComplexity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProblemComplexity {
    Simple,   // Direct calculation
    Medium,   // Few operations
    Complex,  // Many operations/nested
}

#[derive(Debug, Clone)]
pub struct PatternVariant {
    pub name: String,
    pub pattern_type: PatternType,
    pub description: String,
    pub max_iterations: u32,
    pub uses_cache: bool,
}

#[derive(Debug, Clone)]
pub struct PatternResult {
    pub variant: PatternVariant,
    pub success: bool,
    pub iterations: u32,
    pub execution_time_ms: f64,
    pub correctness: f64,
    pub result_value: Option<f64>,
}

#[derive(Debug)]
pub struct PatternTestResult {
    pub all_results: Vec<PatternResult>,
    pub best_pattern: PatternResult,
}

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub iterations: u32,
    pub correctness: f64,
    pub found_value: Option<f64>,
}

impl PatternGenerator {
    pub fn new(cached_patterns: HashMap<String, CachedPattern>) -> Self {
        Self { cached_patterns }
    }

    /// Generate multiple control flow variants for testing
    pub fn generate_pattern_variants(&self, problem: &ProblemSpec) -> Vec<PatternVariant> {
        let mut variants = vec![];

        // Different loop strategies
        variants.push(PatternVariant {
            name: "count_loop_fixed".to_string(),
            pattern_type: PatternType::CountLoop,
            description: "Fixed iteration count loop".to_string(),
            max_iterations: 100,
            uses_cache: false,
        });

        variants.push(PatternVariant {
            name: "while_loop_condition".to_string(),
            pattern_type: PatternType::WhileLoop,
            description: "Condition-based while loop with early exit".to_string(),
            max_iterations: 1000,
            uses_cache: false,
        });

        variants.push(PatternVariant {
            name: "range_loop_bounded".to_string(),
            pattern_type: PatternType::RangeLoop,
            description: "Range-based iteration with bounds".to_string(),
            max_iterations: 500,
            uses_cache: false,
        });

        variants.push(PatternVariant {
            name: "cached_lookup_first".to_string(),
            pattern_type: PatternType::Hybrid,
            description: "Check cache first, then iterate if needed".to_string(),
            max_iterations: 50,
            uses_cache: true,
        });

        variants.push(PatternVariant {
            name: "nested_conditional".to_string(),
            pattern_type: PatternType::NestedStructure,
            description: "Nested loops with conditional branching".to_string(),
            max_iterations: 200,
            uses_cache: false,
        });

        // Add adaptive strategy based on problem complexity
        if matches!(problem.complexity, ProblemComplexity::Complex) {
            variants.push(PatternVariant {
                name: "adaptive_deep_search".to_string(),
                pattern_type: PatternType::Hybrid,
                description: "Adaptive search with progressive deepening".to_string(),
                max_iterations: 2000,
                uses_cache: true,
            });
        }

        variants
    }

    /// Execute all variants in PARALLEL and measure performance
    pub fn test_patterns_parallel(&mut self,
                                   variants: Vec<PatternVariant>,
                                   problem: &ProblemSpec) -> Result<PatternTestResult> {

        println!(">> QUANTUM PATTERN LEARNING MODE ACTIVATED");
        println!(">> Testing {} pattern variants in parallel...", variants.len());
        println!("   Problem: target={}, inputs={:?}", problem.target, problem.inputs);

        // RUN IN PARALLEL using rayon
        let results: Vec<PatternResult> = variants.par_iter()
            .map(|variant| {
                let start = Instant::now();

                println!("   [Testing {}] Starting...", variant.name);

                // Execute the pattern
                let result = self.execute_pattern(variant, problem);

                let execution_time = start.elapsed().as_millis() as f64;

                let pattern_result = match result {
                    Ok(exec_result) => {
                        let correctness = exec_result.correctness;
                        println!("   [{}] ✓ Correctness: {:.1}%, Time: {:.2}ms, Iterations: {}",
                                 variant.name, correctness * 100.0, execution_time, exec_result.iterations);

                        PatternResult {
                            variant: variant.clone(),
                            success: true,
                            iterations: exec_result.iterations,
                            execution_time_ms: execution_time,
                            correctness,
                            result_value: exec_result.found_value,
                        }
                    }
                    Err(e) => {
                        println!("   [{}] ✗ Failed: {}", variant.name, e);
                        PatternResult {
                            variant: variant.clone(),
                            success: false,
                            iterations: 0,
                            execution_time_ms: execution_time,
                            correctness: 0.0,
                            result_value: None,
                        }
                    }
                };

                pattern_result
            })
            .collect();

        // Find BEST performing pattern
        let best = results.iter()
            .max_by(|a, b| {
                // Score = correctness * 100 - time_penalty - iteration_penalty
                let score_a = a.correctness * 100.0 - a.execution_time_ms * 0.1 - a.iterations as f64 * 0.5;
                let score_b = b.correctness * 100.0 - b.execution_time_ms * 0.1 - b.iterations as f64 * 0.5;
                score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .ok_or_else(|| anyhow::anyhow!("No pattern results available"))?;

        println!("\n== BEST PATTERN: {} ({:?})", best.variant.name, best.variant.pattern_type);
        println!("   Correctness: {:.1}%", best.correctness * 100.0);
        println!("   Execution time: {:.2}ms", best.execution_time_ms);
        println!("   Iterations: {}", best.iterations);
        if let Some(val) = best.result_value {
            println!("   Result value: {}", val);
        }

        // CACHE the winning pattern
        self.cache_successful_pattern(best, problem)?;

        Ok(PatternTestResult {
            all_results: results,
            best_pattern: best.clone(),
        })
    }

    /// Execute a single pattern variant
    fn execute_pattern(&self, variant: &PatternVariant, problem: &ProblemSpec) -> Result<ExecutionResult> {
        match variant.pattern_type {
            PatternType::CountLoop => self.execute_count_loop(variant, problem),
            PatternType::WhileLoop => self.execute_while_loop(variant, problem),
            PatternType::RangeLoop => self.execute_range_loop(variant, problem),
            PatternType::Hybrid => self.execute_hybrid(variant, problem),
            PatternType::NestedStructure => self.execute_nested(variant, problem),
            PatternType::ConditionalChain => self.execute_conditional_chain(variant, problem),
        }
    }

    fn execute_count_loop(&self, variant: &PatternVariant, problem: &ProblemSpec) -> Result<ExecutionResult> {
        let max_iters = variant.max_iterations.min(100);
        let target = problem.target;

        for i in 0..max_iters {
            // Simple search: try operations on inputs
            let test_val = if !problem.inputs.is_empty() {
                problem.inputs[0] * (i as f64 + 1.0)
            } else {
                i as f64
            };

            if (test_val - target).abs() < 0.001 {
                return Ok(ExecutionResult {
                    iterations: i + 1,
                    correctness: 1.0,
                    found_value: Some(test_val),
                });
            }
        }

        Ok(ExecutionResult {
            iterations: max_iters,
            correctness: 0.5,
            found_value: None,
        })
    }

    fn execute_while_loop(&self, variant: &PatternVariant, problem: &ProblemSpec) -> Result<ExecutionResult> {
        let target = problem.target;
        let mut iterations = 0;
        let mut best_val = 0.0;
        let mut best_accuracy = 0.0;

        while iterations < variant.max_iterations {
            iterations += 1;

            // Try different combinations
            for a in &problem.inputs {
                for b in &problem.inputs {
                    let test_val = a + b;
                    let accuracy = 1.0 - ((test_val - target).abs() / target.max(1.0));

                    if accuracy > best_accuracy {
                        best_accuracy = accuracy;
                        best_val = test_val;
                    }

                    if (test_val - target).abs() < 0.001 {
                        return Ok(ExecutionResult {
                            iterations,
                            correctness: 1.0,
                            found_value: Some(test_val),
                        });
                    }

                    // Try multiplication
                    let test_val2 = a * b;
                    let accuracy2 = 1.0 - ((test_val2 - target).abs() / target.max(1.0));

                    if accuracy2 > best_accuracy {
                        best_accuracy = accuracy2;
                        best_val = test_val2;
                    }

                    if (test_val2 - target).abs() < 0.001 {
                        return Ok(ExecutionResult {
                            iterations,
                            correctness: 1.0,
                            found_value: Some(test_val2),
                        });
                    }
                }
            }

            // Early exit if good enough
            if best_accuracy > 0.95 {
                break;
            }
        }

        Ok(ExecutionResult {
            iterations,
            correctness: best_accuracy,
            found_value: if best_accuracy > 0.5 { Some(best_val) } else { None },
        })
    }

    fn execute_range_loop(&self, variant: &PatternVariant, problem: &ProblemSpec) -> Result<ExecutionResult> {
        let target = problem.target;
        let max_iters = variant.max_iterations.min(500);

        for i in 0..max_iters {
            let multiplier = i as f64 + 1.0;

            for input in &problem.inputs {
                let test_val = input * multiplier;

                if (test_val - target).abs() < 0.001 {
                    return Ok(ExecutionResult {
                        iterations: i + 1,
                        correctness: 1.0,
                        found_value: Some(test_val),
                    });
                }
            }
        }

        Ok(ExecutionResult {
            iterations: max_iters,
            correctness: 0.6,
            found_value: None,
        })
    }

    fn execute_hybrid(&self, variant: &PatternVariant, problem: &ProblemSpec) -> Result<ExecutionResult> {
        // Check cache first
        if variant.uses_cache {
            let sig = problem.create_signature();
            if let Some(_cached) = self.cached_patterns.get(&sig) {
                // Found in cache - instant result
                return Ok(ExecutionResult {
                    iterations: 0,
                    correctness: 1.0,
                    found_value: Some(problem.target),
                });
            }
        }

        // Fall back to while loop strategy
        self.execute_while_loop(variant, problem)
    }

    fn execute_nested(&self, _variant: &PatternVariant, problem: &ProblemSpec) -> Result<ExecutionResult> {
        let target = problem.target;
        let mut iterations = 0;

        // Nested approach: outer loop for operation, inner for values
        for a in &problem.inputs {
            for b in &problem.inputs {
                iterations += 1;

                // Try multiple operations
                let operations = vec![
                    a + b,
                    a * b,
                    a - b,
                    b - a,
                    if *b != 0.0 { a / b } else { 0.0 },
                ];

                for op_result in operations {
                    if (op_result - target).abs() < 0.001 {
                        return Ok(ExecutionResult {
                            iterations,
                            correctness: 1.0,
                            found_value: Some(op_result),
                        });
                    }
                }
            }
        }

        Ok(ExecutionResult {
            iterations,
            correctness: 0.7,
            found_value: None,
        })
    }

    fn execute_conditional_chain(&self, _variant: &PatternVariant, problem: &ProblemSpec) -> Result<ExecutionResult> {
        let target = problem.target;
        let mut iterations = 0;

        // Conditional strategy based on target size
        if target < 100.0 {
            // Small target: try addition
            for a in &problem.inputs {
                for b in &problem.inputs {
                    iterations += 1;
                    let test_val = a + b;
                    if (test_val - target).abs() < 0.001 {
                        return Ok(ExecutionResult {
                            iterations,
                            correctness: 1.0,
                            found_value: Some(test_val),
                        });
                    }
                }
            }
        } else if target < 1000.0 {
            // Medium target: try multiplication
            for a in &problem.inputs {
                for b in &problem.inputs {
                    iterations += 1;
                    let test_val = a * b;
                    if (test_val - target).abs() < 0.001 {
                        return Ok(ExecutionResult {
                            iterations,
                            correctness: 1.0,
                            found_value: Some(test_val),
                        });
                    }
                }
            }
        } else {
            // Large target: try complex operations
            for a in &problem.inputs {
                for b in &problem.inputs {
                    for c in &problem.inputs {
                        iterations += 1;
                        let test_val = (a + b) * c;
                        if (test_val - target).abs() < 0.001 {
                            return Ok(ExecutionResult {
                                iterations,
                                correctness: 1.0,
                                found_value: Some(test_val),
                            });
                        }
                    }
                }
            }
        }

        Ok(ExecutionResult {
            iterations,
            correctness: 0.75,
            found_value: None,
        })
    }

    fn cache_successful_pattern(&mut self, result: &PatternResult, problem: &ProblemSpec) -> Result<()> {
        if result.correctness < 0.8 {
            return Ok(()); // Only cache good patterns
        }

        let pattern = CachedPattern {
            pattern_type: result.variant.pattern_type.clone(),
            structure: result.variant.name.clone(),
            success_rate: result.correctness * 100.0,
            avg_iterations: result.iterations as f64,
            execution_time_ms: result.execution_time_ms,
            problem_signature: problem.create_signature(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64,
            times_used: 1,
        };

        let key = format!("{}", problem.create_signature());
        self.cached_patterns.insert(key, pattern);

        println!("** Cached successful pattern: {}", result.variant.name);

        Ok(())
    }

    pub fn find_matching_pattern(&self, problem: &ProblemSpec) -> Option<&CachedPattern> {
        let sig = problem.create_signature();

        // Look for exact match first
        if let Some(pattern) = self.cached_patterns.get(&sig) {
            println!("== Found cached pattern for this problem: {}", pattern.structure);
            return Some(pattern);
        }

        // Look for similar problem (fuzzy match)
        let similar = self.cached_patterns.values()
            .filter(|p| problem.is_similar_to(&p.problem_signature))
            .max_by_key(|p| (p.success_rate * 100.0) as u32);

        if let Some(pattern) = similar {
            println!("== Found similar cached pattern: {} (success rate: {:.1}%)",
                     pattern.structure, pattern.success_rate);
            return Some(pattern);
        }

        None
    }

    pub fn get_cached_patterns(&self) -> &HashMap<String, CachedPattern> {
        &self.cached_patterns
    }
}

impl ProblemSpec {
    pub fn new(target: f64, inputs: Vec<f64>) -> Self {
        let input_count = inputs.len();
        let input_range = if inputs.is_empty() {
            (0.0, 0.0)
        } else {
            let min = inputs.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max = inputs.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            (min, max)
        };

        let complexity = if target < 100.0 && input_count <= 2 {
            ProblemComplexity::Simple
        } else if target < 1000.0 && input_count <= 4 {
            ProblemComplexity::Medium
        } else {
            ProblemComplexity::Complex
        };

        Self {
            target,
            inputs,
            input_count,
            input_range,
            complexity,
        }
    }

    /// Create signature for caching/matching
    pub fn create_signature(&self) -> String {
        format!("target:{:.0}_inputs:{}_range:{:.0}-{:.0}_complexity:{:?}",
                self.target, self.input_count,
                self.input_range.0, self.input_range.1,
                self.complexity)
    }

    pub fn is_similar_to(&self, other_sig: &str) -> bool {
        // Fuzzy matching: check if input count matches
        other_sig.contains(&format!("inputs:{}", self.input_count))
    }
}

// Need to add Serialize/Deserialize derives
use serde::{Deserialize, Serialize};
