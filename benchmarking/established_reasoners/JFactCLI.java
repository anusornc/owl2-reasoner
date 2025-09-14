import java.io.File;
import java.util.Arrays;
import org.semanticweb.owlapi.apibinding.OWLManager;
import org.semanticweb.owlapi.model.OWLOntology;
import org.semanticweb.owlapi.model.OWLOntologyManager;
import org.semanticweb.owlapi.reasoner.OWLReasoner;
import uk.ac.manchester.cs.jfact.JFactFactory;

public class JFactCLI {
    public static void main(String[] args) {
        if (args.length < 2) {
            System.out.println("Usage: java JFactCLI <operation> <ontology-file>");
            System.out.println("Operations: classify, consistent");
            System.exit(1);
        }

        String operation = args[0];
        String ontologyFile = args[1];

        try {
            // Create OWL ontology manager
            OWLOntologyManager manager = OWLManager.createOWLOntologyManager();

            // Load ontology
            System.out.println("Loading ontology from " + ontologyFile + "...");
            OWLOntology ontology = manager.loadOntologyFromOntologyDocument(new File(ontologyFile));
            System.out.println("Ontology loaded successfully");

            // Create JFact reasoner
            System.out.println("Initializing JFact reasoner...");
            JFactFactory reasonerFactory = new JFactFactory();
            OWLReasoner reasoner = reasonerFactory.createReasoner(ontology);

            // Perform operation
            if (operation.equals("classify")) {
                System.out.println("Performing classification...");
                reasoner.precomputeInferences();
                System.out.println("Classification completed");
            } else if (operation.equals("consistent")) {
                System.out.println("Checking consistency...");
                boolean isConsistent = reasoner.isConsistent();
                System.out.println("Ontology is " + (isConsistent ? "" : "in") + "consistent");
            } else {
                System.out.println("Unknown operation: " + operation);
                System.exit(1);
            }

        } catch (Exception e) {
            System.out.println("Error: " + e.getMessage());
            e.printStackTrace();
            System.exit(1);
        }
    }
}