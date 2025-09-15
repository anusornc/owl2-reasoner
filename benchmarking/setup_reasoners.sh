#!/bin/bash

# Setup script for OWL2 reasoner benchmarking
# This script helps download and set up established OWL2 reasoners

set -e

echo "🚀 Setting up OWL2 Reasoner Benchmarking Environment"
echo "=================================================="

# Create directory structure
mkdir -p established_reasoners
cd established_reasoners

echo "📁 Created directory: $(pwd)"

# Function to download with fallback
download_with_fallback() {
    local name="$1"
    local url1="$2"
    local url2="$3"
    local output="$4"

    echo "📥 Downloading $name..."

    if curl -L -f -o "$output" "$url1" 2>/dev/null; then
        echo "✅ Successfully downloaded $name from $url1"
        return 0
    fi

    echo "⚠️  Primary URL failed, trying fallback..."
    if curl -L -f -o "$output" "$url2" 2>/dev/null; then
        echo "✅ Successfully downloaded $name from $url2"
        return 0
    fi

    echo "❌ Failed to download $name"
    return 1
}

# Try to download ELK
echo ""
echo "🔍 Setting up ELK Reasoner..."

# Try multiple sources for ELK
download_with_fallback \
    "ELK Reasoner" \
    "https://repo1.maven.org/maven2/org/liveontologies/elk-reasoner/0.4.3/elk-reasoner-0.4.3-standalone.jar" \
    "https://search.maven.org/remotecontent?filepath=org/liveontologies/elk-reasoner/0.4.3/elk-reasoner-0.4.3-standalone.jar" \
    "elk.jar"

# Try to download HermiT
echo ""
echo "🔍 Setting up HermiT Reasoner..."

download_with_fallback \
    "HermiT Reasoner" \
    "https://repo1.maven.org/maven2/net/sourceforge/hermit/hermit/1.4.3.456/hermit-1.4.3.456.jar" \
    "https://search.maven.org/remotecontent?filepath=net/sourceforge/hermit/hermit/1.4.3.456/hermit-1.4.3.456.jar" \
    "hermit.jar"

# Try to download JFact
echo ""
echo "🔍 Setting up JFact Reasoner..."

download_with_fallback \
    "JFact Reasoner" \
    "https://repo1.maven.org/maven2/uk/ac/manchester/cs/jfact/jfact/1.6.4/jfact-1.6.4.jar" \
    "https://search.maven.org/remotecontent?filepath=uk/ac/manchester/cs/jfact/jfact/1.6.4/jfact-1.6.4.jar" \
    "jfact.jar"

# Try to download Pellet
echo ""
echo "🔍 Setting up Pellet Reasoner..."

download_with_fallback \
    "Pellet Reasoner" \
    "https://repo1.maven.org/maven2/com/clarkparsia/pellet/pellet-cli/2.4.2/pellet-cli-2.4.2.jar" \
    "https://search.maven.org/remotecontent?filepath=com/clarkparsia/pellet/pellet-cli/2.4.2/pellet-cli-2.4.2.jar" \
    "pellet.jar"

# Check what we successfully downloaded
echo ""
echo "📋 Download Summary:"
echo "=================="

for jar in *.jar; do
    if [ -f "$jar" ]; then
        size=$(du -h "$jar" | cut -f1)
        echo "✅ $jar ($size)"

        # Test if it's a valid JAR file
        if file "$jar" | grep -q "Java archive"; then
            echo "   🟢 Valid Java archive"
        else
            echo "   🔴 Invalid or corrupted file"
        fi
    fi
done

# Create test script
echo ""
echo "📝 Creating test script..."

cat > test_reasoners.sh << 'EOF'
#!/bin/bash

echo "🧪 Testing OWL2 Reasoners..."
echo "========================="

# Test each downloaded reasoner
for jar in *.jar; do
    if [ -f "$jar" ] && [ -s "$jar" ]; then
        echo ""
        echo "🔍 Testing $jar..."

        # Check if Java is available
        if ! command -v java &> /dev/null; then
            echo "❌ Java not available"
            continue
        fi

        # Test basic functionality
        case "$jar" in
            *elk*)
                echo "📋 Testing ELK..."
                java -jar "$jar" --help 2>/dev/null || echo "   ℹ️  ELK may not have --help option"
                ;;
            *hermit*)
                echo "📋 Testing HermiT..."
                java -jar "$jar" --help 2>/dev/null || echo "   ℹ️  HermiT may not have --help option"
                ;;
            *jfact*)
                echo "📋 Testing JFact..."
                java -jar "$jar" --help 2>/dev/null || echo "   ℹ️  JFact may not have --help option"
                ;;
            *pellet*)
                echo "📋 Testing Pellet..."
                java -jar "$jar" --help 2>/dev/null || echo "   ℹ️  Pellet may not have --help option"
                ;;
            *)
                echo "   ℹ️  Unknown reasoner type"
                ;;
        esac
    fi
done

echo ""
echo "✅ Reasoner testing completed!"
EOF

chmod +x test_reasoners.sh

# Create manual download instructions
echo ""
echo "📚 MANUAL DOWNLOAD INSTRUCTIONS"
echo "==============================="

cat > MANUAL_DOWNLOAD.md << 'EOF'
# Manual Download Instructions for OWL2 Reasoners

If the automatic download failed, you can manually download the reasoners:

## ELK Reasoner
- **Official Site**: https://www.cs.ox.ac.uk/isg/tools/ELK/
- **GitHub**: https://github.com/liveontologies/elk
- **Maven Central**: Search for "elk-reasoner"

## HermiT Reasoner
- **Official Site**: https://www.cs.man.ac.uk/~horrocks/Hermit/
- **GitHub**: https://github.com/phenoscape/HermiT
- **Maven Central**: Search for "hermit"

## JFact Reasoner
- **GitHub**: https://github.com/sszuev/jfact
- **Maven Central**: Search for "jfact"

## Pellet Reasoner
- **GitHub**: https://github.com/stardog-union/pellet
- **Maven Central**: Search for "pellet"

## Alternative Sources
1. **OWL API**: Many reasoners are available through the OWL API
2. **Protege Plugin**: Some reasoners are available as Protege plugins
3. **Academic Sources**: Check university research group websites

## Testing Downloads
After downloading, place the .jar files in this directory and run:
```bash
./test_reasoners.sh
```
EOF

echo ""
echo "🎯 Setup completed!"
echo ""
echo "Next steps:"
echo "1. Run: ./test_reasoners.sh"
echo "2. Check MANUAL_DOWNLOAD.md if any downloads failed"
echo "3. Run the benchmark: cd .. && python benchmark_framework.py"
echo ""
echo "📁 Current directory: $(pwd)"
echo "📁 Files downloaded:"
ls -la *.jar 2>/dev/null || echo "   No jar files found"