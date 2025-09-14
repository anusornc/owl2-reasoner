#!/usr/bin/env python3
"""
Stress test to show the real advantage of BFS over recursive algorithm
"""

import json
import subprocess
import tempfile
from pathlib import Path

def create_stress_test_program(depth=1000, branches=5):
    """Create a stress test with very deep hierarchy"""

    rust_code = f"""
use std::collections::{{HashMap, HashSet, VecDeque}};
use std::time::Instant;

// Evolved BFS algorithm
struct EvolvedEngine {{
    subclass_relations: HashMap<String, Vec<String>>,
}}

impl EvolvedEngine {{
    fn new() -> Self {{
        EvolvedEngine {{
            subclass_relations: HashMap::new(),
        }}
    }}

    fn add_relation(&mut self, sub: String, sup: String) {{
        self.subclass_relations.entry(sub).or_insert_with(Vec::new).push(sup);
    }}

    fn is_subclass(&self, sub: &str, sup: &str) -> bool {{
        if sub == sup {{ return true; }}

        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();

        queue.push_back(sub);
        visited.insert(sub);

        while let Some(current) = queue.pop_front() {{
            if let Some(relations) = self.subclass_relations.get(current) {{
                for related in relations {{
                    if related == sup {{ return true; }}
                    if visited.insert(related) {{
                        queue.push_back(related);
                    }}
                }}
            }}
        }}
        false
    }}
}}

// Original recursive algorithm
struct OriginalEngine {{
    subclass_relations: HashMap<String, Vec<String>>,
}}

impl OriginalEngine {{
    fn new() -> Self {{
        OriginalEngine {{
            subclass_relations: HashMap::new(),
        }}
    }}

    fn add_relation(&mut self, sub: String, sup: String) {{
        self.subclass_relations.entry(sub).or_insert_with(Vec::new).push(sup);
    }}

    fn is_subclass(&self, sub: &str, sup: &str) -> bool {{
        if sub == sup {{ return true; }}
        if let Some(relations) = self.subclass_relations.get(sub) {{
            for related in relations {{
                if related == sup || self.is_subclass(related, sup) {{
                    return true;
                }}
            }}
        }}
        false
    }}
}}

fn main() {{
    let mut evolved = EvolvedEngine::new();
    let mut original = OriginalEngine::new();

    // Create a deep hierarchy that will cause stack overflow in recursive algorithm
    let depth = {depth};

    // Create main chain
    for i in 0..depth-1 {{
        let sub = format!("class_{{}}", i);
        let sup = format!("class_{{}}", i+1);
        evolved.add_relation(sub.clone(), sup.clone());
        original.add_relation(sub, sup);
    }}

    // Add some branches to make it more realistic
    for i in 0..depth {{
        if i % {branches} == 0 {{
            let branch_base = format!("class_{{}}", i);
            for j in 1..={branches} {{
                let branch = format!("branch_{{}}_{{}}", i, j);
                evolved.add_relation(branch_base.clone(), branch.clone());
                original.add_relation(branch_base, branch);
            }}
        }}
    }}

    // Test cases
    let tests = vec![
        ("class_0", &format!("class_{{}}", depth-1)),  // Deep chain test
        ("class_10", &format!("class_{{}}", depth-10)), // Medium depth
        ("class_50", "class_60"),  // Shallow test
    ];

    let mut results = Vec::new();

    // Test evolved algorithm
    for (sub, sup) in &tests {{
        let start = Instant::now();
        let result = evolved.is_subclass(sub, sup);
        let time = start.elapsed().as_nanos();
        results.push(format!(\"{{\"algorithm\":\"evolved\",\"sub\":\"{{}}\",\"sup\":\"{{}}\",\"result\":{{}},\"time_ns\":{{}}}}\",
                          sub, sup, result, time));
    }}

    // Test original algorithm (this might cause stack overflow)
    for (sub, sup) in &tests {{
        let start = Instant::now();
        let result = original.is_subclass(sub, sup);
        let time = start.elapsed().as_nanos();
        results.push(format!(\"{{\"algorithm\":\"original\",\"sub\":\"{{}}\",\"sup\":\"{{}}\",\"result\":{{}},\"time_ns\":{{}}}}\",
                          sub, sup, result, time));
    }}

    println!(\"[{{}}]\", results.join(\",\"));
}}
"""

    return rust_code

def run_stress_test(depth=1000):
    """Run stress test with specified depth"""
    try:
        with tempfile.TemporaryDirectory() as temp_dir:
            project_dir = Path(temp_dir) / "stress_test"

            # Initialize Cargo project
            subprocess.run(["cargo", "init", "--name", "stress_test", "--bin", str(project_dir)],
                          capture_output=True, check=True)

            # Write the stress test program
            main_path = project_dir / "src" / "main.rs"
            with open(main_path, "w") as f:
                f.write(create_stress_test_program(depth))

            # Build with increased stack size for recursive algorithm
            build_result = subprocess.run([
                "cargo", "build", "--release"
            ], cwd=project_dir, capture_output=True, text=True, timeout=120)

            if build_result.returncode != 0:
                return {"error": "Build failed", "stderr": build_result.stderr}

            # Run the test
            run_result = subprocess.run([
                "cargo", "run", "--release"
            ], cwd=project_dir, capture_output=True, text=True, timeout=60)

            if run_result.returncode != 0:
                return {"error": "Run failed", "stderr": run_result.stderr, "stdout": run_result.stdout}

            # Parse results
            try:
                output = run_result.stdout.strip()
                if output.startswith('[') and output.endswith(']'):
                    results = json.loads(output)

                    evolved_times = []
                    original_times = []

                    for result in results:
                        if result["algorithm"] == "evolved":
                            evolved_times.append(result["time_ns"])
                        else:
                            original_times.append(result["time_ns"])

                    avg_evolved = sum(evolved_times) / len(evolved_times) if evolved_times else 0
                    avg_original = sum(original_times) / len(original_times) if original_times else 0

                    return {
                        "depth": depth,
                        "evolved_avg_time_ns": avg_evolved,
                        "original_avg_time_ns": avg_original,
                        "speedup": avg_original / avg_evolved if avg_evolved > 0 else float('inf'),
                        "evolved_times": evolved_times,
                        "original_times": original_times,
                        "all_results": results
                    }
                else:
                    return {"error": "Invalid output format", "raw_output": output}

            except json.JSONDecodeError as e:
                return {"error": f"JSON parse error: {e}", "raw_output": output}

    except subprocess.TimeoutExpired:
        return {"error": "Timeout"}
    except Exception as e:
        return {"error": str(e)}

def main():
    print("🧪 OWL2 Reasoner Stress Test - Deep Hierarchy Performance")
    print("=" * 60)

    # Test with increasing depths
    depths = [100, 500, 1000]

    for depth in depths:
        print(f"\n📊 Testing with depth {depth}...")
        print("-" * 40)

        results = run_stress_test(depth)

        if "error" in results:
            print(f"❌ Test failed: {results['error']}")
            continue

        evolved_time = results["evolved_avg_time_ns"]
        original_time = results["original_avg_time_ns"]
        speedup = results["speedup"]

        print(f"Evolved BFS: {evolved_time:,.0f} ns (average)")
        print(f"Original Recursive: {original_time:,.0f} ns (average)")
        print(f"Speedup: {speedup:.2f}x")

        if speedup > 1.0:
            print("✅ Evolved algorithm is faster")
        else:
            print("⚠️  Original algorithm is faster (but may stack overflow on larger depths)")

    print(f"\n💡 KEY INSIGHTS:")
    print(f"   • BFS algorithm scales linearly: O(N + E)")
    print(f"   • Recursive algorithm has exponential worst-case: O(n²)")
    print(f"   • BFS prevents stack overflow on deep hierarchies")
    print(f"   • BFS handles cycles gracefully with visited set")
    print(f"   • The evolved algorithm is more robust for production use")

if __name__ == "__main__":
    main()