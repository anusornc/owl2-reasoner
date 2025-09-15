#!/usr/bin/env python3

import sys
import random
from pathlib import Path
from rdflib import Graph, URIRef, Literal, Namespace
from rdflib.namespace import RDF, RDFS, OWL, XSD

def generate_lubm_data(num_universities: int, output_dir: Path):
    """Generate LUBM data with specified number of universities"""

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
