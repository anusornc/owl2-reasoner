#!/usr/bin/env python3

"""
Benchmark Setup Script for LUBM and SP2B Integration
Sets up the benchmark infrastructure and downloads necessary data
"""

import os
import sys
import urllib.request
import subprocess
import json
import argparse
from pathlib import Path
from typing import Dict, List, Optional
import tempfile
import shutil
import random

class BenchmarkSetup:
    """Setup LUBM and SP2B benchmarks"""

    def __init__(self, base_dir: str = "benchmarking"):
        self.base_dir = Path(base_dir)
        self.benchmarks_dir = self.base_dir / "benchmarks"
        self.results_dir = self.base_dir / "results"
        self.config_file = self.base_dir / "config.json"

        # Create directories
        self.benchmarks_dir.mkdir(exist_ok=True)
        self.results_dir.mkdir(exist_ok=True)

        # Setup paths
        self.lubm_dir = self.benchmarks_dir / "lubm"
        self.sp2b_dir = self.benchmarks_dir / "sp2b"
        self.bioportal_dir = self.benchmarks_dir / "bioportal"
        self.scalability_dir = self.benchmarks_dir / "scalability"

        # Create subdirectories
        for dir_path in [self.lubm_dir, self.sp2b_dir, self.bioportal_dir, self.scalability_dir]:
            dir_path.mkdir(exist_ok=True)

    def setup_lubm(self) -> bool:
        """Setup LUBM benchmark"""
        print("🔧 Setting up LUBM benchmark...")

        try:
            # Create LUBM subdirectories
            (self.lubm_dir / "data").mkdir(exist_ok=True)
            (self.lubm_dir / "queries").mkdir(exist_ok=True)
            (self.lubm_dir / "generator").mkdir(exist_ok=True)

            # Download LUBM base ontology
            self._download_lubm_ontology()

            # Setup LUBM queries from Pellet examples
            self._setup_lubm_queries()

            # Setup LUBM generator
            self._setup_lubm_generator()

            print("✅ LUBM benchmark setup completed!")
            return True

        except Exception as e:
            print(f"❌ LUBM setup failed: {e}")
            return False

    def setup_sp2b(self) -> bool:
        """Setup SP2B benchmark"""
        print("🔧 Setting up SP2B benchmark...")

        try:
            # Create SP2B subdirectories
            (self.sp2b_dir / "data").mkdir(exist_ok=True)
            (self.sp2b_dir / "queries").mkdir(exist_ok=True)
            (self.sp2b_dir / "generator").mkdir(exist_ok=True)

            # Setup SP2B queries adapted for OWL2 reasoning
            self._setup_sp2b_queries()

            # Setup SP2B generator (placeholder)
            self._setup_sp2b_generator()

            print("✅ SP2B benchmark setup completed!")
            return True

        except Exception as e:
            print(f"❌ SP2B setup failed: {e}")
            return False

    def setup_bioportal(self) -> bool:
        """Setup BioPortal ontologies (placeholder)"""
        print("🔧 Setting up BioPortal benchmark...")

        try:
            # Create BioPortal subdirectories
            (self.bioportal_dir / "ontologies").mkdir(exist_ok=True)
            (self.bioportal_dir / "queries").mkdir(exist_ok=True)

            # Placeholder for BioPortal setup
            print("⚠️  BioPortal setup requires API key - skipping download")
            print("   To enable BioPortal testing, set BIOPORTAL_API_KEY environment variable")

            return True

        except Exception as e:
            print(f"❌ BioPortal setup failed: {e}")
            return False

    def setup_scalability(self) -> bool:
        """Setup scalability testing"""
        print("🔧 Setting up scalability benchmark...")

        try:
            # Create scalability subdirectories
            (self.scalability_dir / "ontologies").mkdir(exist_ok=True)
            (self.scalability_dir / "queries").mkdir(exist_ok=True)

            # Generate scalability test ontologies
            self._generate_scalability_ontologies()

            print("✅ Scalability benchmark setup completed!")
            return True

        except Exception as e:
            print(f"❌ Scalability setup failed: {e}")
            return False

    def _download_lubm_ontology(self):
        """Download LUBM base ontology"""
        print("   📥 Downloading LUBM base ontology...")

        # LUBM base ontology URL
        lubm_url = "http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl"
        local_path = self.lubm_dir / "data" / "univ-bench.owl"

        if not local_path.exists():
            try:
                urllib.request.urlretrieve(lubm_url, local_path)
                print(f"   ✅ Downloaded to: {local_path}")
            except Exception as e:
                print(f"   ⚠️  Download failed: {e}")
                print("   Creating placeholder ontology...")
                self._create_placeholder_lubm_ontology(local_path)
        else:
            print(f"   ✅ Already exists: {local_path}")

    def _create_placeholder_lubm_ontology(self, path: Path):
        """Create placeholder LUBM ontology when download fails"""
        lubm_ontology = """<?xml version="1.0"?>
<rdf:RDF
    xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
    xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#"
    xmlns:owl="http://www.w3.org/2002/07/owl#"
    xmlns:ub="http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl#"
    xml:base="http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl">

    <owl:Ontology rdf:about=""/>

    <!-- Classes -->
    <owl:Class rdf:about="http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl#University">
        <rdfs:subClassOf rdf:resource="http://www.w3.org/2002/07/owl#Thing"/>
    </owl:Class>

    <owl:Class rdf:about="http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl#Department">
        <rdfs:subClassOf rdf:resource="http://www.w3.org/2002/07/owl#Thing"/>
    </owl:Class>

    <owl:Class rdf:about="http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl#Professor">
        <rdfs:subClassOf rdf:resource="http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl#Faculty"/>
    </owl:Class>

    <owl:Class rdf:about="http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl#Student">
        <rdfs:subClassOf rdf:resource="http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl#Person"/>
    </owl:Class>

    <owl:Class rdf:about="http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl#Course">
        <rdfs:subClassOf rdf:resource="http://www.w3.org/2002/07/owl#Thing"/>
    </owl:Class>

    <owl:Class rdf:about="http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl#Person">
        <rdfs:subClassOf rdf:resource="http://www.w3.org/2002/07/owl#Thing"/>
    </owl:Class>

    <owl:Class rdf:about="http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl#Faculty">
        <rdfs:subClassOf rdf:resource="http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl#Person"/>
    </owl:Class>

    <!-- Properties -->
    <owl:ObjectProperty rdf:about="http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl#worksFor">
        <rdfs:domain rdf:resource="http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl#Person"/>
        <rdfs:range rdf:resource="http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl#Organization"/>
    </owl:ObjectProperty>

    <owl:ObjectProperty rdf:about="http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl#takesCourse">
        <rdfs:domain rdf:resource="http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl#Student"/>
        <rdfs:range rdf:resource="http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl#Course"/>
    </owl:ObjectProperty>

    <owl:ObjectProperty rdf:about="http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl#memberOf">
        <rdfs:domain rdf:resource="http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl#Person"/>
        <rdfs:range rdf:resource="http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl#Organization"/>
    </owl:ObjectProperty>

</rdf:RDF>"""

        with open(path, 'w') as f:
            f.write(lubm_ontology)

    def _setup_lubm_queries(self):
        """Setup LUBM queries from Pellet examples"""
        print("   📝 Setting up LUBM queries...")

        # Query paths from Pellet distribution
        pellet_examples_dir = Path("established_reasoners/pellet-2.3.1/examples/data")

        queries = {
            "query1": {
                "description": "Persons working for organizations",
                "file": "lubm-query.sparql"
            },
            "query2": {
                "description": "Students taking courses",
                "file": "lubm-query2.sparql"
            },
            "query3": {
                "description": "Faculty members",
                "file": "lubm-query3.sparql"
            },
            "query4": {
                "description": "Department members",
                "file": "lubm-query4.sparql"
            },
            "query5": {
                "description": "University members",
                "file": "lubm-query5.sparql"
            }
        }

        for query_name, query_info in queries.items():
            source_file = pellet_examples_dir / query_info["file"]

            if source_file.exists():
                # Copy from Pellet examples
                dest_file = self.lubm_dir / "queries" / f"{query_name}.sparql"
                shutil.copy2(source_file, dest_file)
                print(f"   ✅ Copied {query_name} from Pellet examples")
            else:
                # Create placeholder query
                self._create_placeholder_lubm_query(query_name, query_info["description"])

    def _create_placeholder_lubm_query(self, query_name: str, description: str):
        """Create placeholder LUBM query when source is not available"""
        dest_file = self.lubm_dir / "queries" / f"{query_name}.sparql"

        if query_name == "query1":
            query_content = """PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX owl: <http://www.w3.org/2002/07/owl#>
PREFIX ub: <http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl#>

SELECT ?person
WHERE {
    ?person a [
        owl:intersectionOf (
            ub:Person
            [
                owl:onProperty ub:worksFor ;
                owl:someValuesFrom ub:Organization
            ]
        )
    ] .
}"""
        elif query_name == "query2":
            query_content = """PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX owl: <http://www.w3.org/2002/07/owl#>
PREFIX ub: <http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl#>

SELECT ?person
WHERE {
    ?person a [
        owl:intersectionOf (
            ub:Student
            [
                owl:onProperty ub:takesCourse ;
                owl:minCardinality 1
            ]
        )
    ] .
}"""
        else:
            # Generic placeholder for other queries
            query_content = f"""# Placeholder for {query_name}: {description}
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX owl: <http://www.w3.org/2002/07/owl#>
PREFIX ub: <http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl#>

# Query: {description}
SELECT ?x WHERE {{
    ?x a ub:Entity .
}}
LIMIT 10
"""

        with open(dest_file, 'w') as f:
            f.write(query_content)

        print(f"   ✅ Created placeholder {query_name}")

    def _setup_lubm_generator(self):
        """Setup LUBM data generator"""
        print("   🔧 Setting up LUBM generator...")

        # Create a simple Python generator script
        generator_script = """#!/usr/bin/env python3

import sys
import random
from pathlib import Path
from rdflib import Graph, URIRef, Literal, Namespace
from rdflib.namespace import RDF, RDFS, OWL, XSD

def generate_lubm_data(num_universities: int, output_dir: Path):
    \"\"\"Generate LUBM data with specified number of universities\"\"\"

    # Create namespace
    UB = Namespace("http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl#")

    # Create graph
    g = Graph()

    # Add base ontology elements
    g.bind("ub", UB)
    g.bind("rdf", RDF)
    g.bind("rdfs", RDFS)
    g.bind("owl", OWL)

    # Generate data for each university
    for univ_id in range(num_universities):
        university_uri = UB[f"University{univ_id}"]

        # Add university
        g.add((university_uri, RDF.type, UB.University))

        # Generate departments (5-10 per university)
        num_departments = random.randint(5, 10)
        for dept_id in range(num_departments):
            dept_uri = UB[f"Department{univ_id}_{dept_id}"]
            g.add((dept_uri, RDF.type, UB.Department))
            g.add((dept_uri, UB.subOrganizationOf, university_uri))

            # Generate faculty (3-8 per department)
            num_faculty = random.randint(3, 8)
            for fac_id in range(num_faculty):
                fac_uri = UB[f"Professor{univ_id}_{dept_id}_{fac_id}"]
                g.add((fac_uri, RDF.type, UB.Professor))
                g.add((fac_uri, UB.worksFor, dept_uri))
                g.add((fac_uri, UB.memberOf, university_uri))

            # Generate students (20-50 per department)
            num_students = random.randint(20, 50)
            for stud_id in range(num_students):
                stud_uri = UB[f"Student{univ_id}_{dept_id}_{stud_id}"]
                g.add((stud_uri, RDF.type, UB.Student))
                g.add((stud_uri, UB.memberOf, university_uri))

                # Generate courses (2-5 per student)
                num_courses = random.randint(2, 5)
                for course_id in range(num_courses):
                    course_uri = UB[f"Course{univ_id}_{dept_id}_{stud_id}_{course_id}"]
                    g.add((course_uri, RDF.type, UB.Course))
                    g.add((stud_uri, UB.takesCourse, course_uri))

    # Save the generated data
    output_file = output_dir / f"university{num_universities}.owl"
    g.serialize(destination=str(output_file), format='xml')
    print(f"Generated: {output_file} ({len(g)} triples)")

if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Usage: python lubm_generator.py <num_universities> <output_dir>")
        sys.exit(1)

    num_universities = int(sys.argv[1])
    output_dir = Path(sys.argv[2])
    output_dir.mkdir(exist_ok=True)

    generate_lubm_data(num_universities, output_dir)
"""

        generator_file = self.lubm_dir / "generator" / "lubm_generator.py"
        with open(generator_file, 'w') as f:
            f.write(generator_script)

        # Make executable
        generator_file.chmod(0o755)

        print("   ✅ LUBM generator setup completed")

    def _setup_sp2b_queries(self):
        """Setup SP2B queries adapted for OWL2 reasoning"""
        print("   📝 Setting up SP2B queries...")

        queries = {
            "sp2b_query_1": {
                "description": "Social network reasoning - friends of friends",
                "reasoning_aspect": "transitive reasoning",
                "query": """PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX foaf: <http://xmlns.com/foaf/0.1/>
PREFIX owl: <http://www.w3.org/2002/07/owl#>

SELECT DISTINCT ?person WHERE {
    ?person foaf:knows ?friend .
    ?friend foaf:knows ?friend_of_friend .
    FILTER(?person != ?friend_of_friend)
}"""
            },
            "sp2b_query_2": {
                "description": "Interest classification with type inference",
                "reasoning_aspect": "type inference",
                "query": """PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX foaf: <http://xmlns.com/foaf/0.1/>
PREFIX dc: <http://purl.org/dc/elements/1.1/>

SELECT ?person ?interest_type WHERE {
    ?person foaf:interest ?interest .
    ?interest rdf:type ?interest_type .
    ?interest dc:title ?title .
}"""
            },
            "sp2b_query_3": {
                "description": "Organization hierarchy reasoning",
                "reasoning_aspect": "hierarchical reasoning",
                "query": """PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX org: <http://www.w3.org/ns/org#>
PREFIX foaf: <http://xmlns.com/foaf/0.1/>

SELECT ?person ?organization WHERE {
    ?person foaf:member ?organization .
    ?organization org:subOrganizationOf ?parent_organization .
}"""
            }
        }

        for query_name, query_info in queries.items():
            query_file = self.sp2b_dir / "queries" / f"{query_name}.sparql"

            query_content = f"""# {query_info['description']}
# Reasoning Aspect: {query_info['reasoning_aspect']}

{query_info['query']}
"""

            with open(query_file, 'w') as f:
                f.write(query_content)

        print(f"   ✅ Created {len(queries)} SP2B queries")

    def _setup_sp2b_generator(self):
        """Setup SP2B data generator"""
        print("   🔧 Setting up SP2B generator...")

        # Create a simple SP2B generator script
        generator_script = """#!/usr/bin/env python3

import sys
import random
from pathlib import Path
from rdflib import Graph, URIRef, Literal, Namespace
from rdflib.namespace import RDF, RDFS, OWL, XSD

def generate_sp2b_data(scale_factor: int, output_dir: Path):
    \"\"\"Generate SP2B social network data with specified scale factor\"\"\"

    # Define namespaces
    FOAF = Namespace("http://xmlns.com/foaf/0.1/")
    ORG = Namespace("http://www.w3.org/ns/org/")
    DC = Namespace("http://purl.org/dc/elements/1.1/")

    # Create graph
    g = Graph()
    g.bind("foaf", FOAF)
    g.bind("org", ORG)
    g.bind("dc", DC)

    # Calculate numbers based on scale factor
    num_people = scale_factor * 1000
    num_organizations = scale_factor * 100
    num_interests = scale_factor * 50

    # Generate people
    people = []
    for i in range(num_people):
        person_uri = FOAF[f"person_{i}"]
        people.append(person_uri)
        g.add((person_uri, RDF.type, FOAF.Person))
        g.add((person_uri, FOAF.name, Literal(f"Person {i}")))

    # Generate organizations
    organizations = []
    for i in range(num_organizations):
        org_uri = ORG[f"organization_{i}"]
        organizations.append(org_uri)
        g.add((org_uri, RDF.type, ORG.Organization))
        g.add((org_uri, FOAF.name, Literal(f"Organization {i}")))

    # Generate interests
    interests = []
    for i in range(num_interests):
        interest_uri = FOAF[f"interest_{i}"]
        interests.append(interest_uri)
        g.add((interest_uri, RDF.type, FOAF.Interest))
        g.add((interest_uri, DC.title, Literal(f"Interest {i}")))

    # Generate social connections (knows relationships)
    for i, person in enumerate(people):
        # Each person knows 5-15 other people
        num_connections = random.randint(5, min(15, len(people) - 1))
        connections = random.sample([p for p in people if p != person], num_connections)

        for connection in connections:
            g.add((person, FOAF.knows, connection))

    # Generate organization memberships
    for i, person in enumerate(people):
        # Each person belongs to 1-3 organizations
        num_memberships = random.randint(1, min(3, len(organizations)))
        memberships = random.sample(organizations, num_memberships)

        for organization in memberships:
            g.add((person, FOAF.member, organization))

    # Generate interests
    for i, person in enumerate(people):
        # Each person has 3-10 interests
        num_person_interests = random.randint(3, min(10, len(interests)))
        person_interests = random.sample(interests, num_person_interests)

        for interest in person_interests:
            g.add((person, FOAF.interest, interest))

    # Generate organization hierarchy
    for i, org in enumerate(organizations):
        # 20% chance of being a sub-organization
        if random.random() < 0.2 and i > 0:
            parent_org = random.choice(organizations[:i])
            g.add((org, ORG.subOrganizationOf, parent_org))

    # Save the generated data
    output_file = output_dir / f"sp2b_scale_{scale_factor}.ttl"
    g.serialize(destination=str(output_file), format='turtle')
    print(f"Generated: {output_file} ({len(g)} triples)")

if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Usage: python sp2b_generator.py <scale_factor> <output_dir>")
        sys.exit(1)

    scale_factor = int(sys.argv[1])
    output_dir = Path(sys.argv[2])
    output_dir.mkdir(exist_ok=True)

    generate_sp2b_data(scale_factor, output_dir)
"""

        generator_file = self.sp2b_dir / "generator" / "sp2b_generator.py"
        with open(generator_file, 'w') as f:
            f.write(generator_script)

        # Make executable
        generator_file.chmod(0o755)

        print("   ✅ SP2B generator setup completed")

    def _generate_scalability_ontologies(self):
        """Generate ontologies for scalability testing"""
        print("   📊 Generating scalability test ontologies...")

        # Scale definitions
        scales = {
            "small": {"entities": 1000, "axioms": 5000},
            "medium": {"entities": 10000, "axioms": 50000},
            "large": {"entities": 100000, "axioms": 500000}
        }

        for scale_name, scale_info in scales.items():
            self._generate_scalability_ontology(scale_name, scale_info)

    def _generate_scalability_ontology(self, scale_name: str, scale_info: dict):
        """Generate a single scalability ontology"""
        print(f"   📝 Generating {scale_name} scale ontology...")

        try:
            from rdflib import Graph, URIRef, Literal, Namespace
            from rdflib.namespace import RDF, RDFS, OWL, XSD

            # Create namespace
            EX = Namespace("http://example.org/scalability#")

            # Create graph
            g = Graph()
            g.bind("ex", EX)

            num_entities = scale_info["entities"]
            num_axioms = scale_info["axioms"]

            # Generate classes
            num_classes = num_entities // 10
            for i in range(num_classes):
                class_uri = EX[f"Class_{i}"]
                g.add((class_uri, RDF.type, OWL.Class))

            # Generate properties
            num_properties = num_entities // 20
            for i in range(num_properties):
                prop_uri = EX[f"property_{i}"]
                g.add((prop_uri, RDF.type, OWL.ObjectProperty))

            # Generate individuals
            for i in range(num_entities):
                individual_uri = EX[f"individual_{i}"]
                g.add((individual_uri, RDF.type, EX[f"Class_{i % num_classes}"]))

            # Generate axioms
            for i in range(num_axioms):
                axiom_type = random.choice(["subclass", "equivalent", "disjoint"])

                if axiom_type == "subclass":
                    class1 = EX[f"Class_{random.randint(0, num_classes-1)}"]
                    class2 = EX[f"Class_{random.randint(0, num_classes-1)}"]
                    if class1 != class2:
                        g.add((class1, RDFS.subClassOf, class2))

                elif axiom_type == "equivalent":
                    class1 = EX[f"Class_{random.randint(0, num_classes-1)}"]
                    class2 = EX[f"Class_{random.randint(0, num_classes-1)}"]
                    if class1 != class2:
                        g.add((class1, OWL.equivalentClass, class2))

                elif axiom_type == "disjoint":
                    class1 = EX[f"Class_{random.randint(0, num_classes-1)}"]
                    class2 = EX[f"Class_{random.randint(0, num_classes-1)}"]
                    if class1 != class2:
                        g.add((class1, OWL.disjointWith, class2))

            # Save the ontology
            output_file = self.scalability_dir / "ontologies" / f"scalability_{scale_name}.owl"
            g.serialize(destination=str(output_file), format='xml')
            print(f"   ✅ Generated {scale_name}: {output_file} ({len(g)} triples)")

        except ImportError:
            print("   ⚠️  rdflib not available, creating placeholder files")

            # Create placeholder files
            output_file = self.scalability_dir / "ontologies" / f"scalability_{scale_name}.owl"
            placeholder_content = f"""<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#"
         xmlns:owl="http://www.w3.org/2002/07/owl#"
         xmlns:ex="http://example.org/scalability#">

    <owl:Ontology rdf:about=""/>

    <!-- Placeholder for {scale_name} scale ontology -->
    <!-- Would contain {scale_info['entities']} entities and {scale_info['axioms']} axioms -->

</rdf:RDF>"""

            with open(output_file, 'w') as f:
                f.write(placeholder_content)

            print(f"   ✅ Created placeholder {scale_name}")

    def generate_test_data(self) -> bool:
        """Generate test data for all benchmarks"""
        print("🔄 Generating test data...")

        try:
            # Generate LUBM test data
            print("   📊 Generating LUBM test data...")
            lubm_data_dir = self.lubm_dir / "data"
            for univ_count in [1, 10]:
                print(f"      Generating {univ_count} university dataset...")
                result = subprocess.run([
                    sys.executable,
                    str(self.lubm_dir / "generator" / "lubm_generator.py"),
                    str(univ_count),
                    str(lubm_data_dir)
                ], capture_output=True, text=True)

                if result.returncode == 0:
                    print(f"      ✅ Generated LUBM {univ_count} university dataset")
                else:
                    print(f"      ⚠️  LUBM generation failed: {result.stderr}")

            # Generate SP2B test data
            print("   📊 Generating SP2B test data...")
            sp2b_data_dir = self.sp2b_dir / "data"
            for scale_factor in [1, 10]:
                print(f"      Generating SP2B scale {scale_factor} dataset...")
                result = subprocess.run([
                    sys.executable,
                    str(self.sp2b_dir / "generator" / "sp2b_generator.py"),
                    str(scale_factor),
                    str(sp2b_data_dir)
                ], capture_output=True, text=True)

                if result.returncode == 0:
                    print(f"      ✅ Generated SP2B scale {scale_factor} dataset")
                else:
                    print(f"      ⚠️  SP2B generation failed: {result.stderr}")

            print("✅ Test data generation completed!")
            return True

        except Exception as e:
            print(f"❌ Test data generation failed: {e}")
            return False

    def create_config_file(self):
        """Create configuration file"""
        print("📝 Creating configuration file...")

        config = {
            "benchmarks": {
                "lubm": {
                    "enabled": True,
                    "university_counts": [1, 10],
                    "queries": ["query1", "query2", "query3", "query4", "query5"],
                    "iterations": 3,
                    "data_dir": str(self.lubm_dir / "data"),
                    "query_dir": str(self.lubm_dir / "queries")
                },
                "sp2b": {
                    "enabled": True,
                    "scale_factors": [1, 10],
                    "queries": ["sp2b_query_1", "sp2b_query_2", "sp2b_query_3"],
                    "iterations": 3,
                    "data_dir": str(self.sp2b_dir / "data"),
                    "query_dir": str(self.sp2b_dir / "queries")
                },
                "scalability": {
                    "enabled": True,
                    "scales": ["small", "medium", "large"],
                    "iterations": 3,
                    "data_dir": str(self.scalability_dir / "ontologies")
                },
                "bioportal": {
                    "enabled": False,
                    "api_key": os.getenv("BIOPORTAL_API_KEY"),
                    "ontologies": ["GO", "SNOMEDCT", "DOID"],
                    "iterations": 3
                }
            },
            "reasoners": {
                "rust_owl2": {
                    "name": "Rust OWL2 Reasoner",
                    "command": "cargo run --example",
                    "working_dir": "../../",
                    "classification_cmd": "cargo run --example classification_check --",
                    "consistency_cmd": "cargo run --example consistency_check --",
                    "query_cmd": "cargo run --example query_check --"
                },
                "elk": {
                    "name": "ELK Reasoner",
                    "command": "java -jar elk.jar",
                    "classification_cmd": "java -jar elk.jar -c",
                    "consistency_cmd": "java -jar elk.jar -s"
                },
                "hermit": {
                    "name": "HermiT Reasoner",
                    "command": "java -jar hermit.jar",
                    "classification_cmd": "java -jar hermit.jar -c",
                    "consistency_cmd": "java -jar hermit.jar -k"
                }
            },
            "output": {
                "results_dir": str(self.results_dir),
                "report_format": ["markdown", "json"],
                "include_charts": True
            }
        }

        with open(self.config_file, 'w') as f:
            json.dump(config, f, indent=2)

        print(f"✅ Configuration file created: {self.config_file}")

    def run_full_setup(self) -> bool:
        """Run complete setup process"""
        print("🚀 Starting comprehensive benchmark setup...")
        print("=" * 50)

        success = True

        # Setup each benchmark type
        setup_functions = [
            ("LUBM", self.setup_lubm),
            ("SP2B", self.setup_sp2b),
            ("BioPortal", self.setup_bioportal),
            ("Scalability", self.setup_scalability)
        ]

        for name, setup_func in setup_functions:
            print(f"\n🔧 Setting up {name}...")
            if not setup_func():
                success = False
                print(f"❌ {name} setup failed")
            else:
                print(f"✅ {name} setup completed")

        # Generate test data
        print(f"\n📊 Generating test data...")
        if not self.generate_test_data():
            success = False
            print("❌ Test data generation failed")
        else:
            print("✅ Test data generation completed")

        # Create configuration file
        print(f"\n📝 Creating configuration...")
        self.create_config_file()
        print("✅ Configuration completed")

        # Summary
        print("\n" + "=" * 50)
        if success:
            print("🎉 Benchmark setup completed successfully!")
            print("\n📁 Setup directories:")
            print(f"   - LUBM: {self.lubm_dir}")
            print(f"   - SP2B: {self.sp2b_dir}")
            print(f"   - BioPortal: {self.bioportal_dir}")
            print(f"   - Scalability: {self.scalability_dir}")
            print(f"   - Results: {self.results_dir}")
            print(f"   - Config: {self.config_file}")
            print("\n🚀 Next steps:")
            print("   1. Review and update config.json with your reasoner paths")
            print("   2. Run: python enhanced_benchmark_framework.py")
            print("   3. Check results in the benchmark_results/ directory")
        else:
            print("⚠️  Benchmark setup completed with some issues")
            print("   Please review the setup log and fix any issues")

        return success

def main():
    """Main setup function"""
    parser = argparse.ArgumentParser(description="Setup LUBM and SP2B benchmarks")
    parser.add_argument("--base-dir", default="benchmarking", help="Base directory for benchmarks")
    parser.add_argument("--skip-data-generation", action="store_true", help="Skip test data generation")

    args = parser.parse_args()

    # Initialize setup
    setup = BenchmarkSetup(args.base_dir)

    # Run setup
    success = setup.run_full_setup()

    sys.exit(0 if success else 1)

if __name__ == "__main__":
    main()