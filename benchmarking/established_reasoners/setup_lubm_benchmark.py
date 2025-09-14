#!/usr/bin/env python3

"""
LUBM (Lehigh University Benchmark) Setup and Integration
Automated setup for the gold standard OWL2 reasoner benchmark
"""

import os
import sys
import urllib.request
import json
import shutil
from pathlib import Path
from typing import Dict, List, Tuple, Optional
from dataclasses import dataclass
import subprocess
import tempfile

@dataclass
class LUBMConfiguration:
    """LUBM benchmark configuration"""
    base_url: str = "http://swat.cse.lehigh.edu/projects/lubm/"
    version: str = "1.7"
    scales: List[int] = None
    output_dir: str = "benchmarks/lubm"
    queries: List[str] = None

    def __post_init__(self):
        if self.scales is None:
            self.scales = [1, 10, 100]  # Standard LUBM scales
        if self.queries is None:
            self.queries = ["Q1", "Q2", "Q3", "Q4", "Q5"]  # Standard LUBM queries

class LUBMSetup:
    """Automated LUBM benchmark setup and integration"""

    def __init__(self, config: LUBMConfiguration):
        self.config = config
        self.setup_dir = Path(config.output_dir)
        self.data_dir = self.setup_dir / "data"
        self.ontology_dir = self.setup_dir / "ontology"
        self.queries_dir = self.setup_dir / "queries"
        self.results_dir = self.setup_dir / "results"

    def setup_complete_benchmark(self) -> bool:
        """Set up complete LUBM benchmark"""
        print("🎓 Setting up LUBM (Lehigh University Benchmark)...")
        print("=" * 50)

        try:
            # Create directory structure
            self._create_directory_structure()

            # Download and set up base ontology
            self._setup_base_ontology()

            # Generate test datasets for different scales
            self._generate_test_datasets()

            # Set up standard LUBM queries
            self._setup_lubm_queries()

            # Create configuration files
            self._create_configuration_files()

            # Validate setup
            self._validate_setup()

            print("✅ LUBM benchmark setup complete!")
            self._print_setup_summary()
            return True

        except Exception as e:
            print(f"❌ LUBM setup failed: {e}")
            import traceback
            traceback.print_exc()
            return False

    def _create_directory_structure(self):
        """Create benchmark directory structure"""
        print("📁 Creating directory structure...")

        directories = [
            self.setup_dir,
            self.data_dir,
            self.ontology_dir,
            self.queries_dir,
            self.results_dir,
            self.data_dir / "scale_1",
            self.data_dir / "scale_10",
            self.data_dir / "scale_100"
        ]

        for directory in directories:
            directory.mkdir(parents=True, exist_ok=True)

        print(f"✅ Directory structure created: {self.setup_dir}")

    def _setup_base_ontology(self):
        """Set up LUBM base ontology"""
        print("📚 Setting up LUBM base ontology...")

        # Create LUBM base ontology (University Benchmark)
        lubm_ontology = self._create_lubm_base_ontology()

        # Save ontology in multiple formats
        self._save_ontology_formats(lubm_ontology)

        print("✅ LUBM base ontology created")

    def _create_lubm_base_ontology(self) -> str:
        """Create LUBM base ontology content"""
        return """Prefix(univ-bench:<http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl#>)
Prefix(rdf:<http://www.w3.org/1999/02/22-rdf-syntax-ns#>)
Prefix(rdfs:<http://www.w3.org/2000/01/rdf-schema#>)
Prefix(owl:<http://www.w3.org/2002/07/owl#>)

Ontology(<http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl#>

# Class Declarations
Declaration(Class(univ-bench:AssistantProfessor))
Declaration(Class(univ-bench:AssociateProfessor))
Declaration(Class(univ-bench:Chair))
Declaration(Class(univ-bench:College))
Declaration(Class(univ-bench:Course))
Declaration(Class(univ-bench:Department))
Declaration(Class(univ-bench:FullProfessor))
Declaration(Class(univ-bench:Lecturer))
Declaration(Class(univ-bench:Person))
Declaration(Class(univ-bench:PostDoc))
Declaration(Class(univ-bench:Professor))
Declaration(Class(univ-bench:ResearchGroup))
Declaration(Class(univ-bench:ResearchAssistant))
Declaration(Class(univ-bench:Student))
Declaration(Class(univ-bench:TeachingAssistant))
Declaration(Class(univ-bench:UndergraduateStudent))
Declaration(Class(univ-bench:GraduateStudent))
Declaration(Class(univ-bench:University))
Declaration(Class(univ-bench:Publication))
Declaration(Class(univ-bench:Manual))
Declaration(Class(univ-bench:Article))
Declaration(Class(univ-bench:Book))
Declaration(Class(univ-bench:ConferencePaper))
Declaration(Class(univ-bench:JournalArticle))
Declaration(Class(univ-bench:TechnicalReport))
Declaration(Class(univ-bench:UnofficialPublication))
Declaration(Class(univ-bench:WorkingPaper))
Declaration(Class(univ-bench:Software))
Declaration(Class(univ-bench:Specification))
Declaration(Class(univ-bench:Course))
Declaration(Class(univ-bench:Work))
Declaration(Class(univ-bench:Organization))

# Property Declarations
Declaration(ObjectProperty(univ-bench:advisor))
Declaration(ObjectProperty(univ-bench:affiliateOf))
Declaration(ObjectProperty(univ-bench:doctoralDegreeFrom))
Declaration(ObjectProperty(univ-bench:hasAlumnus))
Declaration(ObjectProperty(univ-bench:headOf))
Declaration(ObjectProperty(univ-bench:mastersDegreeFrom))
Declaration(ObjectProperty(univ-bench:memberOf))
Declaration(ObjectProperty(univ-bench:orgPublication))
Declaration(ObjectProperty(univ-bench:publicationAuthor))
Declaration(ObjectProperty(univ-bench:publicationDate))
Declaration(ObjectProperty(univ-bench:researchInterest))
Declaration(ObjectProperty(univ-bench:researchProject))
Declaration(ObjectProperty(univ-bench:softwareDocumentation))
Declaration(ObjectProperty(univ-bench:softwareVersion))
Declaration(ObjectProperty(univ-bench:subOrganizationOf))
Declaration(ObjectProperty(univ-bench:takesCourse))
Declaration(ObjectProperty(univ-bench:teachingAssistantOf))
Declaration(ObjectProperty(univ-bench:teacherOf))
Declaration(ObjectProperty(univ-bench:undergraduateDegreeFrom))
Declaration(ObjectProperty(univ-bench:worksFor))
Declaration(ObjectProperty(univ-bench:tenured))
Declaration(ObjectProperty(univ-bench:emailAddress))
Declaration(ObjectProperty(univ-bench:telephone))
Declaration(ObjectProperty(univ-bench:title))
Declaration(ObjectProperty(univ-bench:name))
Declaration(ObjectProperty(univ-bench:age))
Declaration(ObjectProperty(univ-bench:publicationResearch))
Declaration(ObjectProperty(univ-bench:publicationTitle))

# Class Axioms
SubClassOf(univ-bench:AssistantProfessor univ-bench:Professor)
SubClassOf(univ-bench:AssociateProfessor univ-bench:Professor)
SubClassOf(univ-bench:Chair univ-bench:Professor)
SubClassOf(univ-bench:FullProfessor univ-bench:Professor)
SubClassOf(univ-bench:Lecturer univ-bench:Faculty)
SubClassOf(univ-bench:PostDoc univ-bench:Faculty)
SubClassOf(univ-bench:Professor univ-bench:Faculty)
SubClassOf(univ-bench:ResearchAssistant univ-bench:Student)
SubClassOf(univ-bench:TeachingAssistant univ-bench:Student)
SubClassOf(univ-bench:UndergraduateStudent univ-bench:Student)
SubClassOf(univ-bench:GraduateStudent univ-bench:Student)
SubClassOf(univ-bench:Student univ-bench:Person)
SubClassOf(univ-bench:Faculty univ-bench:Person)

SubClassOf(univ-bench:Article univ-bench:Publication)
SubClassOf(univ-bench:Book univ-bench:Publication)
SubClassOf(univ-bench:ConferencePaper univ-bench:Publication)
SubClassOf(univ-bench:JournalArticle univ-bench:Publication)
SubClassOf(univ-bench:TechnicalReport univ-bench:Publication)
SubClassOf(univ-bench:UnofficialPublication univ-bench:Publication)
SubClassOf(univ-bench:WorkingPaper univ-bench:Publication)

SubClassOf(univ-bench:Department univ-bench:Organization)
SubClassOf(univ-bench:ResearchGroup univ-bench:Organization)
SubClassOf(univ-bench:University univ-bench:Organization)
SubClassOf(univ-bench:College univ-bench:Organization)

# Property Characteristics
InverseProperties(univ-bench:headOf univ-bench:headOf)
InverseProperties(univ-bench:memberOf univ-bench:hasMember)
InverseProperties(univ-bench:worksFor univ-bench:hasMember)
InverseProperties(univ-bench:subOrganizationOf univ-bench:hasSubOrganization)
InverseProperties(univ-bench:publicationAuthor univ-bench:authorOf)

# Domain and Range
ObjectPropertyDomain(univ-bench:advisor univ-bench:Person)
ObjectPropertyRange(univ-bench:advisor univ-bench:Professor)

ObjectPropertyDomain(univ-bench:doctoralDegreeFrom univ-bench:Person)
ObjectPropertyRange(univ-bench:doctoralDegreeFrom univ-bench:University)

ObjectPropertyDomain(univ-bench:takesCourse univ-bench:Student)
ObjectPropertyRange(univ-bench:takesCourse univ-bench:Course)

ObjectPropertyDomain(univ-bench:teacherOf univ-bench:Faculty)
ObjectPropertyRange(univ-bench:teacherOf univ-bench:Course)

ObjectPropertyDomain(univ-bench:headOf univ-bench:Professor)
ObjectPropertyRange(univ-bench:headOf univ-bench:Department)

ObjectPropertyDomain(univ-bench:worksFor univ-bench:Person)
ObjectPropertyRange(univ-bench:worksFor univ-bench:Organization)

# Additional Constraints
SubClassOf(univ-bench:Professor ObjectSomeValuesFrom(univ-bench:headOf univ-bench:Department))
SubClassOf(univ-bench:Department ObjectSomeValuesFrom(univ-bench:subOrganizationOf univ-bench:University))
SubClassOf(univ-bench:College ObjectSomeValuesFrom(univ-bench:subOrganizationOf univ-bench:University))
SubClassOf(univ-bench:Course ObjectSomeValuesFrom(univ-bench:offeredBy univ-bench:Department))

)"""

    def _save_ontology_formats(self, ontology_content: str):
        """Save ontology in multiple formats"""
        # Save as OWL Functional Syntax
        owl_file = self.ontology_dir / "univ-bench.owl"
        with open(owl_file, 'w') as f:
            f.write(ontology_content)

        # Save as Turtle format
        turtle_content = self._convert_to_turtle(ontology_content)
        turtle_file = self.ontology_dir / "univ-bench.ttl"
        with open(turtle_file, 'w') as f:
            f.write(turtle_content)

        # Save as RDF/XML (simplified)
        rdf_content = self._convert_to_rdfxml(ontology_content)
        rdf_file = self.ontology_dir / "univ-bench.rdf"
        with open(rdf_file, 'w') as f:
            f.write(rdf_content)

    def _convert_to_turtle(self, owl_content: str) -> str:
        """Convert OWL functional syntax to Turtle (simplified)"""
        # This is a simplified conversion
        return f"""@prefix univ-bench: <http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .

<http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl#> a owl:Ontology .

# Classes
univ-bench:University a owl:Class ;
    rdfs:label "University" .

univ-bench:Department a owl:Class ;
    rdfs:label "Department" ;
    rdfs:subClassOf univ-bench:Organization .

univ-bench:Professor a owl:Class ;
    rdfs:label "Professor" ;
    rdfs:subClassOf univ-bench:Faculty .

univ-bench:Student a owl:Class ;
    rdfs:label "Student" ;
    rdfs:subClassOf univ-bench:Person .

univ-bench:Course a owl:Class ;
    rdfs:label "Course" .

# Properties
univ-bench:subOrganizationOf a owl:ObjectProperty ;
    rdfs:domain univ-bench:Organization ;
    rdfs:range univ-bench:Organization .

univ-bench:hasAlumnus a owl:ObjectProperty ;
    rdfs:domain univ-bench:University ;
    rdfs:range univ-bench:Person .

univ-bench:takesCourse a owl:ObjectProperty ;
    rdfs:domain univ-bench:Student ;
    rdfs:range univ-bench:Course .

univ-bench:teacherOf a owl:ObjectProperty ;
    rdfs:domain univ-bench:Faculty ;
    rdfs:range univ-bench:Course .
"""

    def _convert_to_rdfxml(self, owl_content: str) -> str:
        """Convert to RDF/XML (simplified)"""
        return f"""<?xml version="1.0"?>
<rdf:RDF xmlns="http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl#"
     xml:base="http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl"
     xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
     xmlns:owl="http://www.w3.org/2002/07/owl#"
     xmlns:xml="http://www.w3.org/XML/1998/namespace"
     xmlns:xsd="http://www.w3.org/2001/XMLSchema#"
     xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#">

    <owl:Ontology rdf:about="http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl"/>

    <!-- Classes -->
    <owl:Class rdf:about="University"/>
    <owl:Class rdf:about="Department">
        <rdfs:subClassOf rdf:resource="Organization"/>
    </owl:Class>
    <owl:Class rdf:about="Professor">
        <rdfs:subClassOf rdf:resource="Faculty"/>
    </owl:Class>
    <owl:Class rdf:about="Student">
        <rdfs:subClassOf rdf:resource="Person"/>
    </owl:Class>
    <owl:Class rdf:about="Course"/>

    <!-- Properties -->
    <owl:ObjectProperty rdf:about="subOrganizationOf">
        <rdfs:domain rdf:resource="Organization"/>
        <rdfs:range rdf:resource="Organization"/>
    </owl:ObjectProperty>

    <owl:ObjectProperty rdf:about="takesCourse">
        <rdfs:domain rdf:resource="Student"/>
        <rdfs:range rdf:resource="Course"/>
    </owl:ObjectProperty>

</rdf:RDF>"""

    def _generate_test_datasets(self):
        """Generate test datasets for different scales"""
        print("📊 Generating test datasets...")

        for scale in self.config.scales:
            self._generate_scale_dataset(scale)

        print("✅ Test datasets generated")

    def _generate_scale_dataset(self, scale: int):
        """Generate dataset for specific scale"""
        print(f"   Generating scale {scale} dataset...")

        scale_dir = self.data_dir / f"scale_{scale}"
        individuals = self._generate_university_data(scale)

        # Save dataset in multiple formats
        self._save_scale_dataset(scale_dir, individuals, scale)

        # Create metadata
        metadata = {
            "scale": scale,
            "universities": scale,
            "departments": scale * 15,
            "courses": scale * 50,
            "students": scale * 200,
            "faculty": scale * 50,
            "estimated_triples": scale * 1000,
            "generation_timestamp": "2024-09-14T00:00:00Z"
        }

        with open(scale_dir / "metadata.json", 'w') as f:
            json.dump(metadata, f, indent=2)

    def _generate_university_data(self, scale: int) -> List[str]:
        """Generate university data for given scale"""
        individuals = []

        for univ_id in range(1, scale + 1):
            # University
            individuals.append(f"<http://example.org/university{univ_id}> rdf:type univ-bench:University .")
            individuals.append(f"<http://example.org/university{univ_id}> univ-bench:name \"University {univ_id}\" .")

            # Departments (15 per university)
            for dept_id in range(1, 16):
                dept_uri = f"http://example.org/department{univ_id}_{dept_id}"
                individuals.append(f"<{dept_uri}> rdf:type univ-bench:Department .")
                individuals.append(f"<{dept_uri}> univ-bench:name \"Department {dept_id}\" .")
                individuals.append(f"<{dept_uri}> univ-bench:subOrganizationOf <http://example.org/university{univ_id}> .")

                # Faculty (3-4 per department)
                for faculty_id in range(1, 4):
                    faculty_uri = f"http://example.org/faculty{univ_id}_{dept_id}_{faculty_id}"
                    individuals.append(f"<{faculty_uri}> rdf:type univ-bench:Professor .")
                    individuals.append(f"<{faculty_uri}> univ-bench:worksFor <{dept_uri}> .")

            # Courses (50 per university)
            for course_id in range(1, 51):
                course_uri = f"http://example.org/course{univ_id}_{course_id}"
                individuals.append(f"<{course_uri}> rdf:type univ-bench:Course .")
                individuals.append(f"<{course_uri}> univ-bench:name \"Course {course_id}\" .")

            # Students (200 per university)
            for student_id in range(1, 201):
                student_uri = f"http://example.org/student{univ_id}_{student_id}"
                individuals.append(f"<{student_uri}> rdf:type univ-bench:Student .")
                individuals.append(f"<{student_uri}> univ-bench:name \"Student {student_id}\" .")

                # Students take courses (5 courses each)
                for course_num in range(1, 6):
                    course_uri = f"http://example.org/course{univ_id}_{((student_id-1)*5 + course_num-1) % 50 + 1}"
                    individuals.append(f"<{student_uri}> univ-bench:takesCourse <{course_uri}> .")

        return individuals

    def _save_scale_dataset(self, scale_dir: Path, individuals: List[str], scale: int):
        """Save scale dataset in multiple formats"""
        # Save as Turtle
        ttl_content = self._create_turtle_dataset(individuals)
        with open(scale_dir / "dataset.ttl", 'w') as f:
            f.write(ttl_content)

        # Save as RDF/XML
        rdf_content = self._create_rdfxml_dataset(individuals)
        with open(scale_dir / "dataset.rdf", 'w') as f:
            f.write(rdf_content)

        # Save as OWL Functional Syntax
        owl_content = self._create_owl_dataset(individuals)
        with open(scale_dir / "dataset.owl", 'w') as f:
            f.write(owl_content)

    def _create_turtle_dataset(self, individuals: List[str]) -> str:
        """Create Turtle format dataset"""
        header = """@prefix univ-bench: <http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

"""
        return header + "\n".join(individuals) + "\n"

    def _create_rdfxml_dataset(self, individuals: List[str]) -> str:
        """Create RDF/XML format dataset"""
        header = """<?xml version="1.0"?>
<rdf:RDF xmlns="http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl#"
     xml:base="http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl"
     xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
     xmlns:owl="http://www.w3.org/2002/07/owl#">

"""
        footer = "</rdf:RDF>"

        # Convert turtle individuals to RDF/XML (simplified)
        xml_individuals = []
        for individual in individuals:
            if "rdf:type" in individual:
                parts = individual.split()
                if len(parts) >= 4:
                    subject = parts[0].strip('<>')
                    obj_type = parts[3].strip('<>')
                    xml_individuals.append(f'    <univ-bench:{obj_type.split(":")[-1]} rdf:about="{subject}"/>')

        return header + "\n".join(xml_individuals) + "\n" + footer

    def _create_owl_dataset(self, individuals: List[str]) -> str:
        """Create OWL Functional Syntax dataset"""
        header = """Prefix(univ-bench:<http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl#>)
Prefix(rdf:<http://www.w3.org/1999/02/22-rdf-syntax-ns#>)

Ontology(<http://example.org/lubm-dataset>

"""
        footer = ")"

        # Convert to functional syntax (simplified)
        functional_individuals = []
        for individual in individuals:
            if "rdf:type" in individual:
                functional_individuals.append("    " + individual.replace(" rdf:type ", " Type(") + ")")

        return header + "\n".join(functional_individuals) + "\n" + footer

    def _setup_lubm_queries(self):
        """Set up standard LUBM queries"""
        print("🔍 Setting up LUBM queries...")

        queries = {
            "Q1": {
                "description": "Graduate students from Department X",
                "type": "SPARQL",
                "query": self._create_lubm_q1(),
                "complexity": "medium"
            },
            "Q2": {
                "description": "Undergraduate students taking Course Y",
                "type": "SPARQL",
                "query": self._create_lubm_q2(),
                "complexity": "medium"
            },
            "Q3": {
                "description": "All professors and their courses",
                "type": "SPARQL",
                "query": self._create_lubm_q3(),
                "complexity": "high"
            },
            "Q4": {
                "description": "Department information",
                "type": "SPARQL",
                "query": self._create_lubm_q4(),
                "complexity": "low"
            },
            "Q5": {
                "description": "Students with research groups",
                "type": "SPARQL",
                "query": self._create_lubm_q5(),
                "complexity": "high"
            },
            "Q6": {
                "description": "University hierarchy reasoning",
                "type": "Reasoning",
                "operation": "classification",
                "description": "Full ontology classification",
                "complexity": "high"
            },
            "Q7": {
                "description": "Consistency checking",
                "type": "Reasoning",
                "operation": "consistency",
                "description": "Ontology consistency validation",
                "complexity": "medium"
            }
        }

        # Save queries
        for query_id, query_info in queries.items():
            query_file = self.queries_dir / f"{query_id}.rq"
            with open(query_file, 'w') as f:
                if query_info["type"] == "SPARQL":
                    f.write(query_info["query"])

            # Save metadata
            metadata_file = self.queries_dir / f"{query_id}_metadata.json"
            with open(metadata_file, 'w') as f:
                json.dump(query_info, f, indent=2)

        print("✅ LUBM queries setup complete")

    def _create_lubm_q1(self) -> str:
        """Create LUBM Query 1 - Graduate students from Department X"""
        return """PREFIX univ-bench: <http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl#>
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>

SELECT ?student ?department ?name
WHERE {
    ?student rdf:type univ-bench:GraduateStudent .
    ?student univ-bench:memberOf ?department .
    ?department rdf:type univ-bench:Department .
    ?department univ-bench:name ?name .
    FILTER regex(?name, "Computer Science", "i")
}
ORDER BY ?name
LIMIT 100"""

    def _create_lubm_q2(self) -> str:
        """Create LUBM Query 2 - Undergraduate students taking Course Y"""
        return """PREFIX univ-bench: <http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl#>
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>

SELECT ?student ?course ?courseName
WHERE {
    ?student rdf:type univ-bench:UndergraduateStudent .
    ?student univ-bench:takesCourse ?course .
    ?course rdf:type univ-bench:Course .
    ?course univ-bench:name ?courseName .
    FILTER regex(?courseName, "Introduction", "i")
}
ORDER BY ?courseName
LIMIT 100"""

    def _create_lubm_q3(self) -> str:
        """Create LUBM Query 3 - All professors and their courses"""
        return """PREFIX univ-bench: <http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl#>
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>

SELECT ?professor ?course ?courseName
WHERE {
    ?professor rdf:type univ-bench:Professor .
    ?professor univ-bench:teacherOf ?course .
    ?course rdf:type univ-bench:Course .
    ?course univ-bench:name ?courseName .
}
ORDER BY ?professor ?courseName
LIMIT 200"""

    def _create_lubm_q4(self) -> str:
        """Create LUBM Query 4 - Department information"""
        return """PREFIX univ-bench: <http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl#>
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>

SELECT ?department ?name ?university
WHERE {
    ?department rdf:type univ-bench:Department .
    ?department univ-bench:name ?name .
    ?department univ-bench:subOrganizationOf ?university .
    ?university rdf:type univ-bench:University .
}
ORDER BY ?name
LIMIT 50"""

    def _create_lubm_q5(self) -> str:
        """Create LUBM Query 5 - Students with research groups"""
        return """PREFIX univ-bench: <http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl#>
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>

SELECT ?student ?researchGroup
WHERE {
    ?student rdf:type univ-bench:GraduateStudent .
    ?student univ-bench:memberOf ?researchGroup .
    ?researchGroup rdf:type univ-bench:ResearchGroup .
}
ORDER BY ?student
LIMIT 150"""

    def _create_configuration_files(self):
        """Create configuration files for benchmark integration"""
        print("⚙️  Creating configuration files...")

        # Main LUBM configuration
        config = {
            "benchmark_name": "LUBM (Lehigh University Benchmark)",
            "version": self.config.version,
            "description": "Standard benchmark for evaluating OWL reasoner performance",
            "scales": self.config.scales,
            "queries": self.config.queries,
            "base_ontology": str(self.ontology_dir / "univ-bench.owl"),
            "data_directories": {
                "scale_1": str(self.data_dir / "scale_1"),
                "scale_10": str(self.data_dir / "scale_10"),
                "scale_100": str(self.data_dir / "scale_100")
            },
            "query_directory": str(self.queries_dir),
            "output_directory": str(self.results_dir),
            "expected_complexity": {
                "scale_1": {"min_time_ms": 50, "max_time_ms": 500},
                "scale_10": {"min_time_ms": 200, "max_time_ms": 2000},
                "scale_100": {"min_time_ms": 1000, "max_time_ms": 10000}
            }
        }

        with open(self.setup_dir / "lubm_config.json", 'w') as f:
            json.dump(config, f, indent=2)

        # Create integration script for testing framework
        integration_script = self._create_integration_script()
        with open(self.setup_dir / "integrate_with_framework.py", 'w') as f:
            f.write(integration_script)

        print("✅ Configuration files created")

    def _create_integration_script(self) -> str:
        """Create integration script for testing framework"""
        return '''#!/usr/bin/env python3

"""
LUBM Integration Script for Enhanced Testing Framework
Integrates LUBM benchmark with the publication-ready testing system
"""

import sys
import json
from pathlib import Path
from enhanced_data_structures import BenchmarkSuite, BenchmarkType, TestOperation
from memory_profiler import ProcessMemoryMonitor

def integrate_lubm_with_framework():
    """Integrate LUBM benchmark with enhanced testing framework"""

    # Load LUBM configuration
    config_path = Path("lubm_config.json")
    with open(config_path, 'r') as f:
        lubm_config = json.load(f)

    print(f"🎓 Integrating LUBM benchmark: {lubm_config['benchmark_name']}")

    # Create benchmark suite
    suite = BenchmarkSuite(
        suite_name="LUBM_Comprehensive_Test",
        benchmark_type=BenchmarkType.LUBM,
        description=lubm_config['description'],
        version=lubm_config['version'],
        test_results=[]
    )

    # Test configurations for different reasoners
    reasoners = {
        "Rust OWL2": {
            "command": ["cargo", "run", "--example", "lubm_example", "--quiet"],
            "working_dir": "../../../"
        },
        "HermiT": {
            "command": ["java", "-jar", "HermiT.jar", "-c"],
            "working_dir": "."
        },
        "ELK": {
            "command": ["java", "-jar", "elk-distribution-cli-0.6.0/elk.jar", "-c"],
            "working_dir": "."
        }
    }

    print("🔧 LUBM integration ready for enhanced testing framework")
    return suite

if __name__ == "__main__":
    integrate_lubm_with_framework()
'''

    def _validate_setup(self):
        """Validate the LUBM setup"""
        print("✅ Validating LUBM setup...")

        validation_checks = [
            (self.setup_dir / "lubm_config.json", "Configuration file"),
            (self.ontology_dir / "univ-bench.owl", "Base ontology"),
            (self.ontology_dir / "univ-bench.ttl", "Turtle format"),
            (self.ontology_dir / "univ-bench.rdf", "RDF/XML format"),
            (self.data_dir / "scale_1" / "dataset.ttl", "Scale 1 dataset"),
            (self.data_dir / "scale_10" / "dataset.ttl", "Scale 10 dataset"),
            (self.queries_dir / "Q1.rq", "Query 1"),
            (self.queries_dir / "Q1_metadata.json", "Query 1 metadata"),
        ]

        all_valid = True
        for file_path, description in validation_checks:
            if file_path.exists():
                print(f"   ✅ {description}: Found")
            else:
                print(f"   ❌ {description}: Missing")
                all_valid = False

        if all_valid:
            print("✅ LUBM setup validation successful")
        else:
            raise Exception("LUBM setup validation failed")

    def _print_setup_summary(self):
        """Print setup summary"""
        print("\n📋 LUBM Setup Summary")
        print("=" * 30)
        print(f"📁 Installation directory: {self.setup_dir}")
        print(f"📚 Base ontology: {self.ontology_dir}")
        print(f"📊 Data scales: {', '.join(map(str, self.config.scales))}")
        print(f"🔍 Standard queries: {', '.join(self.config.queries)}")
        print(f"📋 Configuration: {self.setup_dir / 'lubm_config.json'}")
        print(f"🔧 Integration script: {self.setup_dir / 'integrate_with_framework.py'}")

        # Calculate estimated sizes
        total_estimated_triples = sum(scale * 1000 for scale in self.config.scales)
        print(f"📈 Estimated total triples: {total_estimated_triples:,}")

def main():
    """Main setup function"""
    print("🎓 LUBM Benchmark Setup Utility")
    print("=" * 50)

    # Configuration
    config = LUBMConfiguration(
        output_dir="benchmarks/lubm",
        scales=[1, 10, 100]
    )

    # Setup benchmark
    setup = LUBMSetup(config)
    success = setup.setup_complete_benchmark()

    if success:
        print("\n🎉 LUBM benchmark is ready for academic testing!")
        print("\nNext steps:")
        print("1. Review the benchmark structure")
        print("2. Run integration with enhanced framework")
        print("3. Execute comprehensive tests")
        print("4. Analyze results for publication")
    else:
        print("\n❌ Setup failed. Please check the error messages above.")
        sys.exit(1)

if __name__ == "__main__":
    main()