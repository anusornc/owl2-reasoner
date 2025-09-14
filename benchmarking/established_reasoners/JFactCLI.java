import org.semanticweb.owlapi.apibinding.OWLManager;
import org.semanticweb.owlapi.model.*;
import org.semanticweb.owlapi.reasoner.*;
import org.semanticweb.owlapi.util.ShortFormProvider;
import org.semanticweb.owlapi.util.SimpleShortFormProvider;
import uk.ac.manchester.cs.jfact.JFactFactory;

import java.io.File;
import java.util.concurrent.TimeUnit;

public class JFactCLI {
    public static void main(String[] args) {
        if (args.length < 2) {
            System.out.println("JFact CLI Wrapper for OWL2 Reasoner Benchmark");
            System.out.println("Usage: java JFactCLI <operation> <ontology-file>");
            System.out.println("Operations:");
            System.out.println("  classify    - Classify the ontology");
            System.out.println("  consistent  - Check ontology consistency");
            System.exit(1);
        }

        String operation = args[0].toLowerCase();
        String ontologyFile = args[1];

        try {
            // Validate input file
            File file = new File(ontologyFile);
            if (!file.exists()) {
                System.err.println("Error: Ontology file not found: " + ontologyFile);
                System.exit(1);
            }

            // Start timing
            long startTime = System.nanoTime();

            // Create OWLAPI manager and load ontology
            OWLOntologyManager manager = OWLManager.createOWLOntologyManager();
            OWLOntology ontology = manager.loadOntologyFromOntologyDocument(file);

            // Create JFact reasoner
            OWLReasonerFactory reasonerFactory = new JFactFactory();
            OWLReasoner reasoner = reasonerFactory.createReasoner(ontology);

            // Perform requested operation
            switch (operation) {
                case "classify":
                    performClassification(reasoner, ontology);
                    break;
                case "consistent":
                    performConsistencyCheck(reasoner);
                    break;
                default:
                    System.err.println("Error: Unknown operation: " + operation);
                    System.exit(1);
            }

            // Calculate and display execution time
            long endTime = System.nanoTime();
            long durationMs = TimeUnit.NANOSECONDS.toMillis(endTime - startTime);
            System.out.println("JFact execution time: " + durationMs + " ms");

        } catch (OWLOntologyCreationException e) {
            System.err.println("Error loading ontology: " + e.getMessage());
            e.printStackTrace();
            System.exit(1);
        } catch (Exception e) {
            System.err.println("Error: " + e.getMessage());
            e.printStackTrace();
            System.exit(1);
        }
    }

    private static void performClassification(OWLReasoner reasoner, OWLOntology ontology) {
        System.out.println("Classifying ontology with JFact...");

        // Precompute inferences (this triggers the classification)
        reasoner.precomputeInferences();

        // Get some classification statistics
        ShortFormProvider shortFormProvider = new SimpleShortFormProvider();

        // Count classes
        long classCount = ontology.classesInSignature().count();
        System.out.println("Classes in signature: " + classCount);

        // Count consistent classes (non-unsatisfiable)
        long consistentClassCount = ontology.classesInSignature()
            .filter(cls -> !reasoner.isSatisfiable(cls))
            .count();
        System.out.println("Unsatisfiable classes: " + consistentClassCount);

        // Show hierarchy information for first few classes
        ontology.classesInSignature().limit(5).forEach(cls -> {
            String className = shortFormProvider.getShortForm(cls);
            NodeSet<OWLClass> superClasses = reasoner.getSuperClasses(cls, false);
            NodeSet<OWLClass> subClasses = reasoner.getSubClasses(cls, false);

            System.out.println("Class: " + className);
            System.out.println("  Super classes: " + superClasses.entities().count());
            System.out.println("  Sub classes: " + subClasses.entities().count());
        });

        System.out.println("Classification completed successfully.");
    }

    private static void performConsistencyCheck(OWLReasoner reasoner) {
        System.out.println("Checking ontology consistency with JFact...");

        // Check consistency
        boolean isConsistent = reasoner.isConsistent();

        if (isConsistent) {
            System.out.println("Ontology is CONSISTENT");
        } else {
            System.out.println("Ontology is INCONSISTENT");

            // Try to find unsatisfiable classes
            System.out.println("Checking for unsatisfiable classes...");
            reasoner.getRootOntology().classesInSignature()
                .filter(cls -> !reasoner.isSatisfiable(cls))
                .findFirst()
                .ifPresent(unsatisfiableClass -> {
                    System.out.println("Found unsatisfiable class: " +
                        new SimpleShortFormProvider().getShortForm(unsatisfiableClass));
                });
        }

        System.out.println("Consistency check completed.");
    }
}