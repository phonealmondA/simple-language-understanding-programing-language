# Tauri Pattern Learning Integration

## Overview

The Quantum Pattern Learning system is now fully integrated into the Tauri GUI application, with a dedicated visualization page for monitoring and analyzing learned patterns in real-time.

## Features

### 1. Backend Integration ✅

**New Tauri Commands:**
- `get_pattern_learning_data()` - Retrieves all cached patterns with full details
- `get_pattern_stats()` - Returns aggregated statistics by pattern type

**Location:** `src/tauri_commands.rs:422-592`

**Data Structures:**
```rust
pub struct PatternLearningData {
    patterns: Vec<PatternInfo>,
    total_patterns: usize,
    most_used_pattern: Option<String>,
    avg_success_rate: f64,
}

pub struct PatternInfo {
    name: String,
    pattern_type: String,
    success_rate: f64,
    avg_iterations: f64,
    execution_time_ms: f64,
    problem_signature: String,
    times_used: u32,
    timestamp: u64,
}
```

### 2. Frontend Visualization ✅

**New Page:** `ui/pattern-learning.html`

**Features:**
- **Statistics Dashboard** - Total patterns, average success rate, most used pattern
- **Pattern List** - Detailed view of all cached patterns with metrics
- **Performance Analysis** - Average execution time, iterations, fastest pattern
- **Type Breakdown** - Statistics grouped by pattern type
- **Real-time Charts** - Visual bar chart of pattern distribution by type

**Layout:**
```
┌─────────────────────────────────────┐
│         Header & Back Button        │
├─────────────────┬───────────────────┤
│   Statistics    │  Pattern List     │
│   Overview      │  (Scrollable)     │
├─────────────────┼───────────────────┤
│  Performance    │  Type Breakdown   │
│  Analysis       │  (By Category)    │
└─────────────────┴───────────────────┘
```

### 3. Navigation

**Access Pattern Learning Page:**

From the main IDE (`index.html`), add this button to the Controls section:

```html
<button onclick="window.location.href='pattern-learning.html'"
        style="background: linear-gradient(135deg, #FF9800 0%, #F57C00 100%);">
    🧠 Pattern Learning
</button>
```

Or simply open `http://localhost:1420/pattern-learning.html` directly when Tauri is running.

## Usage

### Running the Tauri App

```bash
# Development mode (with hot-reload)
npm run tauri dev

# Or
cd src-tauri
cargo tauri dev
```

### Viewing Pattern Learning

1. Launch the Tauri application
2. Run some .slut programs from the main IDE page
3. Navigate to the Pattern Learning page:
   - Click the "🧠 Pattern Learning" button (if added)
   - Or open `pattern-learning.html` in a new window
4. View real-time pattern statistics and learning progress

### Real-Time Updates

The pattern learning page automatically refreshes data when:
- Page loads initially
- "🔄 Refresh Data" button is clicked
- After running programs (will auto-update on next refresh)

## Data Flow

```
.slut Program Execution
        ↓
Pattern Learning Activated (target > 100 OR inputs >= 3)
        ↓
Multiple Patterns Tested in Parallel (Rayon)
        ↓
Best Pattern Cached to quantum_consciousness_cache.json
        ↓
Tauri Commands Read Cache
        ↓
Frontend Displays Visualization
```

## API Reference

### JavaScript (Frontend)

```javascript
const { invoke } = window.__TAURI__.tauri;

// Get all pattern learning data
const data = await invoke('get_pattern_learning_data');
// Returns: { patterns: [...], total_patterns, most_used_pattern, avg_success_rate }

// Get aggregated statistics
const stats = await invoke('get_pattern_stats');
// Returns: { total_patterns, avg_success_rate, most_used, by_type: {...} }
```

### Rust (Backend)

```rust
use tauri::State;

// In your command handler
#[tauri::command]
pub fn get_pattern_learning_data(
    app: AppHandle,
    _state: State<'_, AppState>,
) -> Result<PatternLearningData, String> {
    // Implementation reads from cache/quantum_consciousness_cache.json
}
```

## Visualization Components

### 1. Statistics Cards

Displays:
- **Total Patterns** - Number of cached strategies
- **Avg Success Rate** - Overall pattern accuracy
- **Most Used** - Most frequently applied pattern

### 2. Pattern List

For each pattern shows:
- Pattern name and type
- Success rate percentage
- Average execution time (ms)
- Times used count

Sorted by usage frequency (most used first).

### 3. Performance Chart

Bar chart showing:
- Distribution of patterns by type
- Height represents count
- Hover for details

### 4. Type Breakdown

Statistics grouped by pattern type:
- CountLoop, WhileLoop, RangeLoop, etc.
- Average success rate per type
- Average execution time per type

## Cache Format

Patterns are stored in `cache/quantum_consciousness_cache.json`:

```json
{
  "control_flow_patterns": {
    "target:250_inputs:3_range:3-25_complexity:Medium": {
      "pattern_type": "Hybrid",
      "structure": "cached_lookup_first",
      "success_rate": 100.0,
      "avg_iterations": 0.0,
      "execution_time_ms": 8.3,
      "problem_signature": "target:250_inputs:3...",
      "timestamp": 1699234567890,
      "times_used": 5
    }
  }
}
```

## Styling

The pattern learning page uses the same design system as the main IDE:

- **Color Scheme:** Purple gradient (#667eea → #764ba2)
- **Accents:** Green for success (#4CAF50), Blue for info (#2196F3)
- **Background:** Dark (#1a1a2e)
- **Cards:** Semi-transparent panels with glow effects

## Future Enhancements

### Planned Features:

1. **Real-Time Event Stream**
   - Listen for pattern test events during execution
   - Show live progress as patterns are tested in parallel

2. **Historical Trends**
   - Graph showing pattern performance over time
   - Track how patterns improve with more usage

3. **Pattern Comparison**
   - Side-by-side comparison of different pattern types
   - Performance benchmarking

4. **Export/Import**
   - Export successful patterns for sharing
   - Import patterns from other projects

5. **Interactive Charts**
   - Click on chart bars to filter pattern list
   - Drill-down into specific pattern types

6. **Pattern Testing**
   - Test patterns manually from the UI
   - Simulate different problem scenarios

## Troubleshooting

### Pattern Learning Page Shows No Data

**Problem:** Empty state displayed even after running programs.

**Solutions:**
1. Check that `cache/quantum_consciousness_cache.json` exists
2. Verify pattern learning was activated (target > 100 OR inputs >= 3)
3. Run programs from the main IDE first
4. Click "🔄 Refresh Data" button

### Commands Not Found

**Problem:** `invoke('get_pattern_learning_data')` fails.

**Solutions:**
1. Ensure Tauri commands are registered in `src-tauri/src/main.rs`
2. Rebuild the Tauri app: `cargo tauri dev`
3. Check console for error messages

### Styling Issues

**Problem:** Page looks broken or unstyled.

**Solutions:**
1. Ensure `pattern-learning.html` is in the `ui/` directory
2. Clear browser cache
3. Verify Tauri is serving files from `ui/` directory

## File Structure

```
.slut-programing/
├── src/
│   └── tauri_commands.rs          [Pattern learning commands]
├── src-tauri/
│   └── src/
│       └── main.rs                 [Command registration]
├── ui/
│   ├── index.html                  [Main IDE]
│   ├── pattern-learning.html       [Pattern visualization page]
│   └── js/
│       └── (existing scripts)
└── cache/
    └── quantum_consciousness_cache.json  [Pattern storage]
```

## Performance Notes

- **Cache Loading:** O(n) where n = number of cached patterns
- **Statistics Calculation:** Performed on-demand when page loads
- **Memory Usage:** Minimal - only loads data when requested
- **Update Frequency:** Manual refresh (not auto-polling to save resources)

## Integration Checklist

- [x] Tauri commands created (`get_pattern_learning_data`, `get_pattern_stats`)
- [x] Commands registered in `src-tauri/src/main.rs`
- [x] Pattern learning HTML page created
- [x] JavaScript API integration
- [x] Statistics dashboard
- [x] Pattern list view
- [x] Performance charts
- [x] Type breakdown view
- [ ] Navigation button in main IDE
- [ ] Real-time event streaming
- [ ] Historical trend charts
- [ ] Export/import functionality

## Example Usage

### From Main IDE

```javascript
// Run a program that triggers pattern learning
async function runComplexProgram() {
    await invoke('run_file', {
        filePath: 'examples/pattern_learning_simple.slut'
    });

    // Pattern will be cached automatically
    // View results in pattern-learning.html
}
```

### From Pattern Learning Page

```javascript
// Refresh data after running programs
async function refreshData() {
    const data = await invoke('get_pattern_learning_data');

    console.log(`Total patterns: ${data.total_patterns}`);
    console.log(`Avg success: ${data.avg_success_rate}%`);
    console.log(`Most used: ${data.most_used_pattern}`);

    updateUI(data);
}
```

## Contributing

To add new visualizations:

1. Add data to Tauri commands if needed
2. Update `pattern-learning.html` with new sections
3. Add JavaScript functions to process/display data
4. Update this documentation

## License

Same as main project.

---

**Pattern Learning in Action:**
```
Run 1: Tests 6 patterns → Caches winner (45ms)
Run 2: Reuses cache → Faster execution (8ms) ⚡
Run 3: Pattern improves → Even faster (3ms) ⚡⚡
```

The system gets smarter with every run! 🧠
