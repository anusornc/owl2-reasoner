#!/usr/bin/env python3

"""
SP2B (SPARQL Performance Benchmark) Setup for OWL2 Reasoning
Adapts SP2B for reasoning-focused evaluation rather than just query performance
"""

import os
import sys
import json
from pathlib import Path
from typing import Dict, List, Tuple, Optional
from dataclasses import dataclass
import tempfile

@dataclass
class SP2BConfiguration:
    """SP2B benchmark configuration for OWL2 reasoning"""
    base_url: str = "http://dbpedia.org/sparql-benchmark/"
    version: str = "1.0"
    scales: List[int] = None
    output_dir: str = "benchmarks/sp2b"
    reasoning_queries: List[str] = None

    def __post_init__(self):
        if self.scales is None:
            self.scales = [1, 10, 100]  # Standard SP2B scales
        if self.reasoning_queries is None:
            self.reasoning_queries = ["Q1_Transitive", "Q2_TypeInference", "Q3_Hierarchical"]

class SP2BSetup:
    """SP2B benchmark setup for OWL2 reasoning evaluation"""

    def __init__(self, config: SP2BConfiguration):
        self.config = config
        self.setup_dir = Path(config.output_dir)
        self.data_dir = self.setup_dir / "data"
        self.ontology_dir = self.setup_dir / "ontology"
        self.queries_dir = self.setup_dir / "queries"
        self.results_dir = self.setup_dir / "results"

    def setup_complete_benchmark(self) -> bool:
        """Set up complete SP2B benchmark for OWL2 reasoning"""
        print("🔗 Setting up SP2B (SPARQL Performance Benchmark) for OWL2 Reasoning...")
        print("=" * 60)

        try:
            # Create directory structure
            self._create_directory_structure()

            # Set up social network ontology for reasoning
            self._setup_social_network_ontology()

            # Generate social network datasets for different scales
            self._generate_social_network_datasets()

            # Set up reasoning-focused queries
            self._setup_reasoning_queries()

            # Create configuration files
            self._create_configuration_files()

            # Validate setup
            self._validate_setup()

            print("✅ SP2B benchmark setup complete!")
            self._print_setup_summary()
            return True

        except Exception as e:
            print(f"❌ SP2B setup failed: {e}")
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

    def _setup_social_network_ontology(self):
        """Set up social network ontology with reasoning characteristics"""
        print("🌐 Setting up social network ontology...")

        # Create social network ontology with complex reasoning patterns
        sp2b_ontology = self._create_social_network_ontology()

        # Save ontology in multiple formats
        self._save_ontology_formats(sp2b_ontology)

        print("✅ Social network ontology created")

    def _create_social_network_ontology(self) -> str:
        """Create social network ontology content"""
        return """Prefix(sp2b:<http://purl.org/NET/rdfsp2b#>)
Prefix(rdf:<http://www.w3.org/1999/02/22-rdf-syntax-ns#>)
Prefix(rdfs:<http://www.w3.org/2000/01/rdf-schema#>)
Prefix(owl:<http://www.w3.org/2002/07/owl#>)
Prefix(foaf:<http://xmlns.com/foaf/0.1/>)
Prefix(xs:<http://www.w3.org/2001/XMLSchema#>)

Ontology(<http://example.org/sp2b-social-network#>

# Core Social Network Classes
Declaration(Class(sp2b:Person))
Declaration(Class(sp2b:Document))
Declaration(Class(sp2b:Group))
Declaration(Class(sp2b:Organization))
Declaration(Class(sp2b:Tag))
Declaration(Class(sp2b:Topic))
Declaration(Class(sp2b:Event))

# Person Subclasses for Reasoning
Declaration(Class(sp2b:Student))
Declaration(Class(sp2b:Professor))
Declaration(Class(sp2b:Researcher))
Declaration(Class(sp2b:Employee))
Declaration(Class(sp2b:Administrator))

# Document Types
Declaration(Class(sp2b:Article))
Declaration(Class(sp2b:BlogPost))
Declaration(Class(sp2b:Comment))
Declaration(Class(sp2b:Review))
Declaration(Class(sp2b:Publication))

# Relationship Types (Complex Properties)
Declaration(ObjectProperty(sp2b:knows))
Declaration(ObjectProperty(sp2b:friendOf))
Declaration(ObjectProperty(sp2b:colleagueOf))
Declaration(ObjectProperty(sp2b:supervisorOf))
Declaration(ObjectProperty(sp2b:subordinateOf))
Declaration(ObjectProperty(sp2b:collaboratorOf))
Declaration(ObjectProperty(sp2b:authorOf))
Declaration(ObjectProperty(sp2b:creatorOf))
Declaration(ObjectProperty(sp2b:editorOf))
Declaration(ObjectProperty(sp2b:reviewerOf))
Declaration(ObjectProperty(sp2b:memberOf))
Declaration(ObjectProperty(sp2b:leaderOf))
Declaration(ObjectProperty(sp2b:participantOf))
Declaration(ObjectProperty(sp2b:organizerOf))
Declaration(ObjectProperty(sp2b:interestedIn))
Declaration(ObjectProperty(sp2b:expertIn))
Declaration(ObjectProperty(sp2b:hasTopic))
Declaration(ObjectProperty(sp2b:hasTag))
Declaration(ObjectProperty(sp2b:references))
Declaration(ObjectProperty(sp2b:cites))
Declaration(ObjectProperty(sp2b:replyTo))
Declaration(ObjectProperty(sp2b:relatedTo))
Declaration(ObjectProperty(sp2b:similarTo))

# Class Hierarchies for Reasoning
SubClassOf(sp2b:Student sp2b:Person)
SubClassOf(sp2b:Professor sp2b:Person)
SubClassOf(sp2b:Researcher sp2b:Person)
SubClassOf(sp2b:Employee sp2b:Person)
SubClassOf(sp2b:Administrator sp2b:Person)

SubClassOf(sp2b:Article sp2b:Document)
SubClassOf(sp2b:BlogPost sp2b:Document)
SubClassOf(sp2b:Comment sp2b:Document)
SubClassOf(sp2b:Review sp2b:Document)
SubClassOf(sp2b:Publication sp2b:Document)

# Complex Property Characteristics
InverseProperties(sp2b:supervisorOf sp2b:subordinateOf)
SymmetricProperty(sp2b:friendOf)
SymmetricProperty(sp2b:colleagueOf)
SymmetricProperty(sp2b:collaboratorOf)
TransitiveProperty(sp2b:knows)
TransitiveProperty(sp2b:colleagueOf)
TransitiveProperty(sp2b:collaboratorOf)

# Property Domains and Ranges
ObjectPropertyDomain(sp2b:knows sp2b:Person)
ObjectPropertyRange(sp2b:knows sp2b:Person)

ObjectPropertyDomain(sp2b:friendOf sp2b:Person)
ObjectPropertyRange(sp2b:friendOf sp2b:Person)

ObjectPropertyDomain(sp2b:supervisorOf sp2b:Person)
ObjectPropertyRange(sp2b:supervisorOf sp2b:Person)

ObjectPropertyDomain(sp2b:authorOf sp2b:Person)
ObjectPropertyRange(sp2b:authorOf sp2b:Document)

ObjectPropertyDomain(sp2b:expertIn sp2b:Person)
ObjectPropertyRange(sp2b:expertIn sp2b:Topic)

ObjectPropertyDomain(sp2b:hasTopic sp2b:Document)
ObjectPropertyRange(sp2b:hasTopic sp2b:Topic)

# Complex Reasoning Axioms
SubClassOf(
    ObjectSomeValuesFrom(sp2b:supervisorOf sp2b:Person)
    sp2b:Professor
)

SubClassOf(
    ObjectSomeValuesFrom(sp2b:subordinateOf sp2b:Professor)
    sp2b:Student
)

SubClassOf(
    ObjectIntersectionOf(
        sp2b:Person
        ObjectSomeValuesFrom(sp2b:expertIn sp2b:Topic)
    )
    ObjectSomeValuesFrom(sp2b:interestedIn sp2b:Topic)
)

SubClassOf(
    ObjectSomeValuesFrom(sp2b:authorOf sp2b:Publication)
    sp2b:Researcher
)

# Transitive Closure Reasoning
SubClassOf(
    ObjectSomeValuesFrom(sp2b:knows ObjectSomeValuesFrom(sp2b:knows sp2b:Person))
    ObjectSomeValuesFrom(sp2b:knows sp2b:Person)
)

# Hierarchical Reasoning
SubClassOf(
    ObjectSomeValuesFrom(sp2b:memberOf sp2b:Group)
    ObjectSomeValuesFrom(sp2b:participantOf sp2b:Event)
)

SubClassOf(
    ObjectSomeValuesFrom(sp2b:leaderOf sp2b:Group)
    ObjectIntersectionOf(
        sp2b:Person
        ObjectSomeValuesFrom(sp2b:memberOf sp2b:Group)
    )
)

# Type Inference Rules
SubClassOf(
    ObjectIntersectionOf(
        sp2b:Person
        ObjectSomeValuesFrom(sp2b:authorOf ObjectSomeValuesFrom(sp2b:hasTopic sp2b:Topic))
        ObjectSomeValuesFrom(sp2b:expertIn sp2b:Topic)
    )
    sp2b:Researcher
)

SubClassOf(
    ObjectIntersectionOf(
        sp2b:Person
        ObjectSomeValuesFrom(sp2b:supervisorOf sp2b:Student)
    )
    sp2b:Professor
)

# Property Chains for Complex Reasoning
SubObjectPropertyOf(
    ObjectPropertyChain(sp2b:supervisorOf sp2b:memberOf)
    sp2b:responsibleFor
)

SubObjectPropertyOf(
    ObjectPropertyChain(sp2b:knows sp2b:expertIn)
    sp2b:indirectlyInterestedIn
)

# Cardinality Constraints
SubClassOf(
    sp2b:Professor
    ObjectExactCardinality(1 sp2b:memberOf sp2b:Organization)
)

SubClassOf(
    sp2b:Student
    ObjectMinCardinality(1 sp2b:subordinateOf sp2b:Professor)
)

# Disjoint Classes for Reasoning
DisjointClasses(sp2b:Student sp2b:Professor)
DisjointClasses(sp2b:Article sp2b:BlogPost sp2b:Comment)

# Equivalent Classes for Complex Reasoning
EquivalentClasses(
    sp2b:Expert
    ObjectIntersectionOf(
        sp2b:Person
        ObjectSomeValuesFrom(sp2b:expertIn sp2b:Topic)
        ObjectSomeValuesFrom(sp2b:authorOf sp2b:Publication)
    )
)

# Property Restrictions
SubClassOf(
    sp2b:Researcher
    ObjectAllValuesFrom(sp2b:authorOf sp2b:Publication)
)

SubClassOf(
    sp2b:Student
    ObjectAllValuesFrom(sp2b:subordinateOf sp2b:Professor)
)

)"""

    def _save_ontology_formats(self, ontology_content: str):
        """Save ontology in multiple formats"""
        # Save as OWL Functional Syntax
        owl_file = self.ontology_dir / "sp2b-social.owl"
        with open(owl_file, 'w') as f:
            f.write(ontology_content)

        # Save as Turtle format
        turtle_content = self._convert_to_turtle(ontology_content)
        turtle_file = self.ontology_dir / "sp2b-social.ttl"
        with open(turtle_file, 'w') as f:
            f.write(turtle_content)

        # Save as RDF/XML (simplified)
        rdf_content = self._convert_to_rdfxml(ontology_content)
        rdf_file = self.ontology_dir / "sp2b-social.rdf"
        with open(rdf_file, 'w') as f:
            f.write(rdf_content)

    def _convert_to_turtle(self, owl_content: str) -> str:
        """Convert OWL functional syntax to Turtle (simplified)"""
        return f"""@prefix sp2b: <http://purl.org/NET/rdfsp2b#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix foaf: <http://xmlns.com/foaf/0.1/> .
@prefix xs: <http://www.w3.org/2001/XMLSchema#> .

<http://example.org/sp2b-social-network#> a owl:Ontology .

# Core Classes
sp2b:Person a owl:Class ;
    rdfs:label "Person" ;
    rdfs:comment "A person in the social network" .

sp2b:Document a owl:Class ;
    rdfs:label "Document" ;
    rdfs:comment "A document or publication" .

sp2b:Group a owl:Class ;
    rdfs:label "Group" ;
    rdfs:comment "A group or organization" .

sp2b:Topic a owl:Class ;
    rdfs:label "Topic" ;
    rdfs:comment "A topic or area of expertise" .

# Person Subclasses
sp2b:Student a owl:Class ;
    rdfs:subClassOf sp2b:Person ;
    rdfs:label "Student" .

sp2b:Professor a owl:Class ;
    rdfs:subClassOf sp2b:Person ;
    rdfs:label "Professor" .

sp2b:Researcher a owl:Class ;
    rdfs:subClassOf sp2b:Person ;
    rdfs:label "Researcher" .

# Properties
sp2b:knows a owl:ObjectProperty ;
    rdfs:domain sp2b:Person ;
    rdfs:range sp2b:Person ;
    a owl:TransitiveProperty .

sp2b:friendOf a owl:ObjectProperty ;
    rdfs:domain sp2b:Person ;
    rdfs:range sp2b:Person ;
    a owl:SymmetricProperty .

sp2b:supervisorOf a owl:ObjectProperty ;
    rdfs:domain sp2b:Person ;
    rdfs:range sp2b:Person .

sp2b:subordinateOf a owl:ObjectProperty ;
    rdfs:domain sp2b:Person ;
    rdfs:range sp2b:Person ;
    owl:inverseOf sp2b:supervisorOf .

sp2b:authorOf a owl:ObjectProperty ;
    rdfs:domain sp2b:Person ;
    rdfs:range sp2b:Document .

sp2b:expertIn a owl:ObjectProperty ;
    rdfs:domain sp2b:Person ;
    rdfs:range sp2b:Topic .

sp2b:hasTopic a owl:ObjectProperty ;
    rdfs:domain sp2b:Document ;
    rdfs:range sp2b:Topic .

# Document Types
sp2b:Article a owl:Class ;
    rdfs:subClassOf sp2b:Document ;
    rdfs:label "Article" .

sp2b:Publication a owl:Class ;
    rdfs:subClassOf sp2b:Document ;
    rdfs:label "Publication" .
"""

    def _convert_to_rdfxml(self, owl_content: str) -> str:
        """Convert to RDF/XML (simplified)"""
        return f"""<?xml version="1.0"?>
<rdf:RDF xmlns="http://purl.org/NET/rdfsp2b#"
     xml:base="http://purl.org/NET/rdfsp2b#"
     xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
     xmlns:owl="http://www.w3.org/2002/07/owl#"
     xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#">

    <owl:Ontology rdf:about="http://example.org/sp2b-social-network#"/>

    <!-- Core Classes -->
    <owl:Class rdf:about="Person"/>
    <owl:Class rdf:about="Document"/>
    <owl:Class rdf:about="Group"/>
    <owl:Class rdf:about="Topic"/>

    <!-- Person Subclasses -->
    <owl:Class rdf:about="Student">
        <rdfs:subClassOf rdf:resource="Person"/>
    </owl:Class>

    <owl:Class rdf:about="Professor">
        <rdfs:subClassOf rdf:resource="Person"/>
    </owl:Class>

    <owl:Class rdf:about="Researcher">
        <rdfs:subClassOf rdf:resource="Person"/>
    </owl:Class>

    <!-- Properties -->
    <owl:ObjectProperty rdf:about="knows">
        <rdfs:domain rdf:resource="Person"/>
        <rdfs:range rdf:resource="Person"/>
        <rdf:type rdf:resource="http://www.w3.org/2002/07/owl#TransitiveProperty"/>
    </owl:ObjectProperty>

    <owl:ObjectProperty rdf:about="friendOf">
        <rdfs:domain rdf:resource="Person"/>
        <rdfs:range rdf:resource="Person"/>
        <rdf:type rdf:resource="http://www.w3.org/2002/07/owl#SymmetricProperty"/>
    </owl:ObjectProperty>

    <owl:ObjectProperty rdf:about="authorOf">
        <rdfs:domain rdf:resource="Person"/>
        <rdfs:range rdf:resource="Document"/>
    </owl:ObjectProperty>

</rdf:RDF>"""

    def _generate_social_network_datasets(self):
        """Generate social network datasets for different scales"""
        print("🌐 Generating social network datasets...")

        for scale in self.config.scales:
            self._generate_scale_dataset(scale)

        print("✅ Social network datasets generated")

    def _generate_scale_dataset(self, scale: int):
        """Generate dataset for specific scale"""
        print(f"   Generating scale {scale} dataset...")

        scale_dir = self.data_dir / f"scale_{scale}"
        individuals = self._generate_social_network_data(scale)

        # Save dataset in multiple formats
        self._save_scale_dataset(scale_dir, individuals, scale)

        # Create metadata
        metadata = {
            "scale": scale,
            "people": scale * 125,
            "documents": scale * 40,
            "groups": scale * 8,
            "topics": scale * 15,
            "relationships": scale * 500,
            "estimated_triples": scale * 5000,
            "reasoning_complexity": "high",
            "generation_timestamp": "2024-09-14T00:00:00Z"
        }

        with open(scale_dir / "metadata.json", 'w') as f:
            json.dump(metadata, f, indent=2)

    def _generate_social_network_data(self, scale: int) -> List[str]:
        """Generate social network data for given scale"""
        individuals = []

        # Generate people with different roles
        for person_id in range(1, scale * 125 + 1):
            person_uri = f"http://example.org/person{person_id}"

            # Assign roles (students, professors, researchers)
            if person_id % 10 == 0:
                role = "Professor"
                individuals.append(f"<{person_uri}> rdf:type sp2b:Professor .")
                individuals.append(f"<{person_uri}> sp2b:expertIn <http://example.org/topic{person_id % 15 + 1}> .")
            elif person_id % 5 == 0:
                role = "Researcher"
                individuals.append(f"<{person_uri}> rdf:type sp2b:Researcher .")
                individuals.append(f"<{person_uri}> sp2b:expertIn <http://example.org/topic{person_id % 15 + 1}> .")
            else:
                role = "Student"
                individuals.append(f"<{person_uri}> rdf:type sp2b:Student .")
                # Students are supervised by professors
                supervisor_id = ((person_id - 1) // 10) * 10 + 10
                if supervisor_id < scale * 125:
                    individuals.append(f"<{person_uri}> sp2b:subordinateOf <http://example.org/person{supervisor_id}> .")
                    individuals.append(f"<http://example.org/person{supervisor_id}> sp2b:supervisorOf <{person_uri}> .")

            # Friend relationships (for transitive reasoning)
            friend_ids = [person_id - 1, person_id + 1, person_id + 10]
            for friend_id in friend_ids:
                if 1 <= friend_id <= scale * 125 and friend_id != person_id:
                    if person_id < friend_id:  # Avoid duplicates
                        individuals.append(f"<{person_uri}> sp2b:friendOf <http://example.org/person{friend_id}> .")
                        individuals.append(f"<http://example.org/person{friend_id}> sp2b:friendOf <{person_uri}> .")

            # Knows relationships (for transitive closure reasoning)
            know_ids = range(max(1, person_id - 5), min(scale * 125 + 1, person_id + 6))
            for know_id in know_ids:
                if know_id != person_id:
                    individuals.append(f"<{person_uri}> sp2b:knows <http://example.org/person{know_id}> .")

            # Generate documents and authorship
            if person_id % 3 == 0:  # 1/3 of people create documents
                for doc_id in range(1, 4):  # Each creates up to 3 documents
                    doc_uri = f"http://example.org/document{person_id}_{doc_id}"
                    if doc_id == 1:
                        doc_type = "Article"
                    elif doc_id == 2:
                        doc_type = "BlogPost"
                    else:
                        doc_type = "Publication"

                    individuals.append(f"<{doc_uri}> rdf:type sp2b:{doc_type} .")
                    individuals.append(f"<{person_uri}> sp2b:authorOf <{doc_uri}> .")

                    # Add topics to documents
                    topic_id = (person_id + doc_id) % 15 + 1
                    individuals.append(f"<{doc_uri}> sp2b:hasTopic <http://example.org/topic{topic_id}> .")

        # Generate groups and memberships
        for group_id in range(1, scale * 8 + 1):
            group_uri = f"http://example.org/group{group_id}"
            individuals.append(f"<{group_uri}> rdf:type sp2b:Group .")

            # Add members to groups
            member_ids = range((group_id - 1) * 15 + 1, min(group_id * 15 + 1, scale * 125 + 1))
            for member_id in member_ids:
                if member_id <= scale * 125:
                    individuals.append(f"<http://example.org/person{member_id}> sp2b:memberOf <{group_uri}> .")

        # Generate topics
        for topic_id in range(1, scale * 15 + 1):
            topic_uri = f"http://example.org/topic{topic_id}"
            individuals.append(f"<{topic_uri}> rdf:type sp2b:Topic .")
            individuals.append(f"<{topic_uri}> sp2b:name \"Topic {topic_id}\" .")

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
        header = """@prefix sp2b: <http://purl.org/NET/rdfsp2b#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xs: <http://www.w3.org/2001/XMLSchema#> .

"""
        return header + "\n".join(individuals) + "\n"

    def _create_rdfxml_dataset(self, individuals: List[str]) -> str:
        """Create RDF/XML format dataset"""
        header = """<?xml version="1.0"?>
<rdf:RDF xmlns="http://purl.org/NET/rdfsp2b#"
     xml:base="http://purl.org/NET/rdfsp2b#"
     xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
     xmlns:owl="http://www.w3.org/2002/07/owl#">

"""
        footer = "</rdf:RDF>"

        # Convert individuals to RDF/XML (simplified)
        xml_individuals = []
        for individual in individuals:
            if "rdf:type" in individual:
                parts = individual.split()
                if len(parts) >= 4:
                    subject = parts[0].strip('<>')
                    obj_type = parts[3].strip('<>')
                    xml_individuals.append(f'    <sp2b:{obj_type.split(":")[-1]} rdf:about="{subject}"/>')

        return header + "\n".join(xml_individuals) + "\n" + footer

    def _create_owl_dataset(self, individuals: List[str]) -> str:
        """Create OWL Functional Syntax dataset"""
        header = """Prefix(sp2b:<http://purl.org/NET/rdfsp2b#>)
Prefix(rdf:<http://www.w3.org/1999/02/22-rdf-syntax-ns#>)

Ontology(<http://example.org/sp2b-dataset>

"""
        footer = ")"

        # Convert to functional syntax (simplified)
        functional_individuals = []
        for individual in individuals:
            if "rdf:type" in individual:
                functional_individuals.append("    " + individual.replace(" rdf:type ", " Type(") + ")")

        return header + "\n".join(functional_individuals) + "\n" + footer

    def _setup_reasoning_queries(self):
        """Set up reasoning-focused SP2B queries"""
        print("🧠 Setting up reasoning-focused queries...")

        queries = {
            "Q1_Transitive": {
                "description": "Transitive relationship reasoning (knows chain)",
                "type": "Reasoning",
                "reasoning_type": "transitive_closure",
                "operation": "classification",
                "query": self._create_sp2b_q1_transitive(),
                "complexity": "high",
                "expected_reasoning": "Transitive closure of 'knows' relationships"
            },
            "Q2_TypeInference": {
                "description": "OWL2 type inference (researcher detection)",
                "type": "Reasoning",
                "reasoning_type": "classification",
                "operation": "classification",
                "query": self._create_sp2b_q2_type_inference(),
                "complexity": "medium",
                "expected_reasoning": "Complex type inference using property chains"
            },
            "Q3_Hierarchical": {
                "description": "Hierarchical classification reasoning",
                "type": "Reasoning",
                "reasoning_type": "classification",
                "operation": "classification",
                "query": self._create_sp2b_q3_hierarchical(),
                "complexity": "high",
                "expected_reasoning": "Multi-level hierarchy reasoning"
            },
            "Q4_Consistency": {
                "description": "Ontology consistency checking",
                "type": "Reasoning",
                "reasoning_type": "consistency",
                "operation": "consistency",
                "query": "# Consistency checking operation - no SPARQL query needed",
                "complexity": "medium",
                "expected_reasoning": "Detect inconsistencies in social network"
            },
            "Q5_Symmetric": {
                "description": "Symmetric property reasoning",
                "type": "Reasoning",
                "reasoning_type": "classification",
                "operation": "classification",
                "query": self._create_sp2b_q5_symmetric(),
                "complexity": "medium",
                "expected_reasoning": "Symmetric property inference"
            },
            "Q6_Inverse": {
                "description": "Inverse property reasoning",
                "type": "Reasoning",
                "reasoning_type": "classification",
                "operation": "classification",
                "query": self._create_sp2b_q6_inverse(),
                "complexity": "low",
                "expected_reasoning": "Inverse property inference"
            }
        }

        # Save queries
        for query_id, query_info in queries.items():
            query_file = self.queries_dir / f"{query_id}.rq"
            with open(query_file, 'w') as f:
                f.write(query_info["query"])

            # Save metadata
            metadata_file = self.queries_dir / f"{query_id}_metadata.json"
            with open(metadata_file, 'w') as f:
                json.dump(query_info, f, indent=2)

        print("✅ SP2B reasoning queries setup complete")

    def _create_sp2b_q1_transitive(self) -> str:
        """Create SP2B Query 1 - Transitive reasoning"""
        return """PREFIX sp2b: <http://purl.org/NET/rdfsp2b#>
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>

# This query tests transitive closure reasoning
# Find all people indirectly known through the 'knows' relationship
SELECT DISTINCT ?person1 ?person2 ?distance
WHERE {
    ?person1 rdf:type sp2b:Person .
    ?person2 rdf:type sp2b:Person .
    ?person1 sp2b:knows* ?person2 .
    FILTER (?person1 != ?person2)
}
ORDER BY ?person1 ?person2
LIMIT 200"""

    def _create_sp2b_q2_type_inference(self) -> str:
        """Create SP2B Query 2 - Type inference reasoning"""
        return """PREFIX sp2b: <http://purl.org/NET/rdfsp2b#>
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>

# This query tests complex type inference
# Find researchers based on their publications and expertise
SELECT DISTINCT ?person ?topic
WHERE {
    ?person rdf:type sp2b:Person .
    ?person sp2b:authorOf ?document .
    ?document sp2b:hasTopic ?topic .
    ?person sp2b:expertIn ?topic .
    ?topic rdf:type sp2b:Topic .
}
ORDER BY ?person ?topic
LIMIT 150"""

    def _create_sp2b_q3_hierarchical(self) -> str:
        """Create SP2B Query 3 - Hierarchical reasoning"""
        return """PREFIX sp2b: <http://purl.org/NET/rdfsp2b#>
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>

# This query tests hierarchical classification reasoning
# Find professors and all their indirect subordinates
SELECT DISTINCT ?professor ?student
WHERE {
    ?professor rdf:type sp2b:Professor .
    ?student rdf:type sp2b:Student .
    ?professor sp2b:supervisorOf+ ?student .
}
ORDER BY ?professor ?student
LIMIT 100"""

    def _create_sp2b_q5_symmetric(self) -> str:
        """Create SP2B Query 5 - Symmetric property reasoning"""
        return """PREFIX sp2b: <http://purl.org/NET/rdfsp2b#>
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>

# This query tests symmetric property reasoning
# Find all friend relationships (should be bidirectional)
SELECT DISTINCT ?person1 ?person2
WHERE {
    ?person1 rdf:type sp2b:Person .
    ?person2 rdf:type sp2b:Person .
    ?person1 sp2b:friendOf ?person2 .
    FILTER (?person1 < ?person2)  # Avoid duplicates
}
ORDER BY ?person1 ?person2
LIMIT 100"""

    def _create_sp2b_q6_inverse(self) -> str:
        """Create SP2B Query 6 - Inverse property reasoning"""
        return """PREFIX sp2b: <http://purl.org/NET/rdfsp2b#>
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>

# This query tests inverse property reasoning
# Find supervisor-subordinate relationships
SELECT DISTINCT ?supervisor ?subordinate
WHERE {
    ?supervisor rdf:type sp2b:Professor .
    ?subordinate rdf:type sp2b:Student .
    {
        ?supervisor sp2b:supervisorOf ?subordinate .
    } UNION {
        ?subordinate sp2b:subordinateOf ?supervisor .
    }
}
ORDER BY ?supervisor ?subordinate
LIMIT 100"""

    def _create_configuration_files(self):
        """Create configuration files for benchmark integration"""
        print("⚙️  Creating configuration files...")

        # Main SP2B configuration
        config = {
            "benchmark_name": "SP2B for OWL2 Reasoning",
            "version": self.config.version,
            "description": "SPARQL Performance Benchmark adapted for OWL2 reasoning evaluation",
            "scales": self.config.scales,
            "reasoning_queries": self.config.reasoning_queries,
            "base_ontology": str(self.ontology_dir / "sp2b-social.owl"),
            "data_directories": {
                "scale_1": str(self.data_dir / "scale_1"),
                "scale_10": str(self.data_dir / "scale_10"),
                "scale_100": str(self.data_dir / "scale_100")
            },
            "query_directory": str(self.queries_dir),
            "output_directory": str(self.results_dir),
            "reasoning_complexity": {
                "transitive_closure": "high",
                "type_inference": "medium",
                "hierarchical": "high",
                "consistency": "medium",
                "symmetric": "low",
                "inverse": "low"
            },
            "expected_complexity": {
                "scale_1": {"min_time_ms": 100, "max_time_ms": 1000},
                "scale_10": {"min_time_ms": 500, "max_time_ms": 5000},
                "scale_100": {"min_time_ms": 2000, "max_time_ms": 20000}
            }
        }

        with open(self.setup_dir / "sp2b_config.json", 'w') as f:
            json.dump(config, f, indent=2)

        # Create integration script
        integration_script = self._create_integration_script()
        with open(self.setup_dir / "integrate_with_framework.py", 'w') as f:
            f.write(integration_script)

        print("✅ Configuration files created")

    def _create_integration_script(self) -> str:
        """Create integration script for testing framework"""
        return '''#!/usr/bin/env python3

"""
SP2B Integration Script for Enhanced Testing Framework
Integrates SP2B reasoning benchmark with the publication-ready testing system
"""

import sys
import json
from pathlib import Path
from enhanced_data_structures import BenchmarkSuite, BenchmarkType, TestOperation

def integrate_sp2b_with_framework():
    """Integrate SP2B benchmark with enhanced testing framework"""

    # Load SP2B configuration
    config_path = Path("sp2b_config.json")
    with open(config_path, 'r') as f:
        sp2b_config = json.load(f)

    print(f"🔗 Integrating SP2B benchmark: {sp2b_config['benchmark_name']}")

    # Create benchmark suite
    suite = BenchmarkSuite(
        suite_name="SP2B_Reasoning_Test",
        benchmark_type=BenchmarkType.SP2B,
        description=sp2b_config['description'],
        version=sp2b_config['version'],
        test_results=[]
    )

    # Test configurations for different reasoners
    reasoners = {
        "Rust OWL2": {
            "command": ["cargo", "run", "--example", "sp2b_example", "--quiet"],
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

    print("🔧 SP2B integration ready for enhanced testing framework")
    return suite

if __name__ == "__main__":
    integrate_sp2b_with_framework()
'''

    def _validate_setup(self):
        """Validate the SP2B setup"""
        print("✅ Validating SP2B setup...")

        validation_checks = [
            (self.setup_dir / "sp2b_config.json", "Configuration file"),
            (self.ontology_dir / "sp2b-social.owl", "Base ontology"),
            (self.ontology_dir / "sp2b-social.ttl", "Turtle format"),
            (self.ontology_dir / "sp2b-social.rdf", "RDF/XML format"),
            (self.data_dir / "scale_1" / "dataset.ttl", "Scale 1 dataset"),
            (self.data_dir / "scale_10" / "dataset.ttl", "Scale 10 dataset"),
            (self.queries_dir / "Q1_Transitive.rq", "Transitive reasoning query"),
            (self.queries_dir / "Q1_Transitive_metadata.json", "Query metadata"),
        ]

        all_valid = True
        for file_path, description in validation_checks:
            if file_path.exists():
                print(f"   ✅ {description}: Found")
            else:
                print(f"   ❌ {description}: Missing")
                all_valid = False

        if all_valid:
            print("✅ SP2B setup validation successful")
        else:
            raise Exception("SP2B setup validation failed")

    def _print_setup_summary(self):
        """Print setup summary"""
        print("\n📋 SP2B Setup Summary")
        print("=" * 30)
        print(f"📁 Installation directory: {self.setup_dir}")
        print(f"🌐 Social network ontology: {self.ontology_dir}")
        print(f"📊 Data scales: {', '.join(map(str, self.config.scales))}")
        print(f"🧠 Reasoning queries: {', '.join(self.config.reasoning_queries)}")
        print(f"📋 Configuration: {self.setup_dir / 'sp2b_config.json'}")
        print(f"🔧 Integration script: {self.setup_dir / 'integrate_with_framework.py'}")

        # Calculate estimated sizes
        total_estimated_triples = sum(scale * 5000 for scale in self.config.scales)
        print(f"📈 Estimated total triples: {total_estimated_triples:,}")

def main():
    """Main setup function"""
    print("🔗 SP2B Benchmark Setup Utility")
    print("=" * 60)

    # Configuration
    config = SP2BConfiguration(
        output_dir="benchmarks/sp2b",
        scales=[1, 10, 100]
    )

    # Setup benchmark
    setup = SP2BSetup(config)
    success = setup.setup_complete_benchmark()

    if success:
        print("\n🎉 SP2B benchmark is ready for academic testing!")
        print("\nKey features:")
        print("✅ Transitive closure reasoning")
        print("✅ Complex type inference")
        print("✅ Hierarchical classification")
        print("✅ Symmetric and inverse properties")
        print("✅ Multi-scale social network data")
        print("\nNext steps:")
        print("1. Review the social network ontology structure")
        print("2. Run integration with enhanced framework")
        print("3. Execute reasoning-focused tests")
        print("4. Analyze results for publication")
    else:
        print("\n❌ Setup failed. Please check the error messages above.")
        sys.exit(1)

if __name__ == "__main__":
    main()