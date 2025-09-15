#!/usr/bin/env python3

import sys
import random
from pathlib import Path
from rdflib import Graph, URIRef, Literal, Namespace
from rdflib.namespace import RDF, RDFS, OWL, XSD

def generate_sp2b_data(scale_factor: int, output_dir: Path):
    """Generate SP2B social network data with specified scale factor"""

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
