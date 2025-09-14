use std::collections::{HashMap, HashSet, VecDeque};

// Copy the evolved algorithm
pub struct EvolvedReasoningEngine {
    classes: HashSet<String>,
    properties: HashSet<String>,
    subclass_relations: HashMap<String, Vec<String>>,
    operations_count: u64,
    total_time_ns: u64,
}

impl EvolvedReasoningEngine {
    pub fn new() -> Self {
        EvolvedReasoningEngine {
            classes: HashSet::new(),
            properties: HashSet::new(),
            subclass_relations: HashMap::new(),
            operations_count: 0,
            total_time_ns: 0,
        }
    }

    pub fn add_class(&mut self, class: String) {
        self.classes.insert(class);
    }

    pub fn add_property(&mut self, property: String) {
        self.properties.insert(property);
    }

    pub fn add_subclass_relation(&mut self, sub: String, sup: String) {
        self.subclass_relations.entry(sub.clone()).or_insert_with(Vec::new).push(sup);
    }

    pub fn is_subclass_of(&mut self, sub_class: &str, super_class: &str) -> bool {
        let start = std::time::Instant::now();
        let result = self.is_subclass_of_basic(sub_class, super_class);
        let elapsed = start.elapsed();
        self.operations_count += 1;
        self.total_time_ns += elapsed.as_nanos() as u64;
        result
    }

    // Evolved BFS implementation
    fn is_subclass_of_basic(&self, sub_class: &str, super_class: &str) -> bool {
        if sub_class == super_class {
            return true;
        }

        let mut queue: VecDeque<&str> = VecDeque::new();
        let mut visited: HashSet<&str> = HashSet::new();

        queue.push_back(sub_class);
        visited.insert(sub_class);

        while let Some(current_class) = queue.pop_front() {
            if let Some(supers) = self.subclass_relations.get(current_class) {
                for sup in supers {
                    if sup == super_class {
                        return true;
                    }
                    if visited.insert(sup) {
                        queue.push_back(sup);
                    }
                }
            }
        }
        false
    }
}

// Copy the original algorithm
pub struct OriginalReasoningEngine {
    classes: HashSet<String>,
    properties: HashSet<String>,
    subclass_relations: HashMap<String, Vec<String>>,
    operations_count: u64,
    total_time_ns: u64,
}

impl OriginalReasoningEngine {
    pub fn new() -> Self {
        OriginalReasoningEngine {
            classes: HashSet::new(),
            properties: HashSet::new(),
            subclass_relations: HashMap::new(),
            operations_count: 0,
            total_time_ns: 0,
        }
    }

    pub fn add_class(&mut self, class: String) {
        self.classes.insert(class);
    }

    pub fn add_property(&mut self, property: String) {
        self.properties.insert(property);
    }

    pub fn add_subclass_relation(&mut self, sub: String, sup: String) {
        self.subclass_relations.entry(sub.clone()).or_insert_with(Vec::new).push(sup);
    }

    pub fn is_subclass_of(&mut self, sub_class: &str, super_class: &str) -> bool {
        let start = std::time::Instant::now();
        let result = self.is_subclass_of_basic(sub_class, super_class);
        let elapsed = start.elapsed();
        self.operations_count += 1;
        self.total_time_ns += elapsed.as_nanos() as u64;
        result
    }

    // Original O(n²) recursive implementation
    fn is_subclass_of_basic(&self, sub_class: &str, super_class: &str) -> bool {
        if sub_class == super_class {
            return true;
        }

        if let Some(supers) = self.subclass_relations.get(sub_class) {
            for sup in supers {
                if sup == super_class {
                    return true;
                }
                if self.is_subclass_of_basic(sup, super_class) {
                    return true;
                }
            }
        }
        false
    }
}

fn main() {
    // Create a complex ontology that will show the difference
    let mut evolved_engine = EvolvedReasoningEngine::new();
    let mut original_engine = OriginalReasoningEngine::new();

    // Build a deep hierarchy that's problematic for recursive algorithms
    let mut classes = Vec::new();
    for i in 0..100 {
        classes.push(format!("class_{}", i));
    }

    // Create a deep chain: class_0 -> class_1 -> ... -> class_99
    for i in 0..99 {
        evolved_engine.add_subclass_relation(classes[i].clone(), classes[i + 1].clone());
        original_engine.add_subclass_relation(classes[i].clone(), classes[i + 1].clone());
    }

    // Add some branches to make it more realistic
    for i in (0..100).step_by(10) {
        let branch_name = format!("branch_{}", i);
        evolved_engine.add_subclass_relation(classes[i].clone(), branch_name.clone());
        original_engine.add_subclass_relation(classes[i].clone(), branch_name.clone());

        for j in 1..5 {
            let sub_branch = format!("{}_sub_{}", branch_name, j);
            evolved_engine.add_subclass_relation(branch_name.clone(), sub_branch.clone());
            original_engine.add_subclass_relation(branch_name.clone(), sub_branch.clone());
        }
    }

    // Test cases that will be challenging for the recursive algorithm
    let test_cases = vec![
        // Deep chain tests (problematic for recursion)
        ("class_0", "class_99", true),
        ("class_10", "class_95", true),
        ("class_5", "class_85", true),

        // Branch tests
        ("branch_0", "branch_0_sub_3", true),
        ("class_20", "branch_20_sub_2", true),

        // Negative tests
        ("class_99", "class_0", false),
        ("branch_0_sub_1", "class_10", false),
    ];

    // Collect results for evolved algorithm
    let start_total = std::time::Instant::now();
    let mut correct_count = 0;
    let mut evolved_tests = Vec::new();

    for (sub, sup, expected) in &test_cases {
        let start = std::time::Instant::now();
        let result = evolved_engine.is_subclass_of(sub, sup);
        let elapsed = start.elapsed();

        let is_correct = result == *expected;
        if is_correct {
            correct_count += 1;
        }

        evolved_tests.push(format!("{{\"sub\": \"{}\", \"sup\": \"{}\", \"expected\": {}, \"result\": {}, \"correct\": {}, \"time_ns\": {}}}",
                sub, sup, expected, result, is_correct, elapsed.as_nanos()));
    }

    let evolved_total = start_total.elapsed();
    let evolved_correct = correct_count;

    // Collect results for original algorithm
    let start_total = std::time::Instant::now();
    let mut correct_count = 0;
    let mut original_tests = Vec::new();

    for (sub, sup, expected) in &test_cases {
        let start = std::time::Instant::now();
        let result = original_engine.is_subclass_of(sub, sup);
        let elapsed = start.elapsed();

        let is_correct = result == *expected;
        if is_correct {
            correct_count += 1;
        }

        original_tests.push(format!("{{\"sub\": \"{}\", \"sup\": \"{}\", \"expected\": {}, \"result\": {}, \"correct\": {}, \"time_ns\": {}}}",
                sub, sup, expected, result, is_correct, elapsed.as_nanos()));
    }

    let original_total = start_total.elapsed();
    let original_correct = correct_count;

    // Output JSON
    println!("{{\"test_results\": [");
    println!("  {{\"algorithm\": \"evolved_bfs\", \"total_time_ns\": {}, \"correct_tests\": {}, \"total_tests\": {}, \"tests\": [{}]}},",
            evolved_total.as_nanos(), evolved_correct, test_cases.len(), evolved_tests.join(","));
    println!("  {{\"algorithm\": \"original_recursive\", \"total_time_ns\": {}, \"correct_tests\": {}, \"total_tests\": {}, \"tests\": [{}]}}",
            original_total.as_nanos(), original_correct, test_cases.len(), original_tests.join(","));
    println!("]}}");
}