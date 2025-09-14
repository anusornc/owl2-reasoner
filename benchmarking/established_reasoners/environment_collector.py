#!/usr/bin/env python3

"""
Publication-Ready Environment Specification Collector
Automatically collects comprehensive environment data for academic reproducibility
"""

import platform
import subprocess
import json
import psutil
import os
import sys
from typing import Dict, Any, List, Optional
from pathlib import Path
import datetime
import re
from dataclasses import dataclass, asdict

@dataclass
class EnvironmentSpecification:
    """Complete environment specification for academic publication"""
    # Hardware
    system_manufacturer: str
    system_model: str
    processor: str
    cpu_cores: int
    cpu_threads: int
    base_clock_speed: str
    max_clock_speed: str
    total_memory: str
    memory_type: str
    storage_type: str
    storage_capacity: str
    architecture: str

    # Software
    os_name: str
    os_version: str
    os_build: str
    kernel_version: str

    # Runtime environments
    java_version: str
    java_vendor: str
    java_home: str
    rust_version: str
    python_version: str
    python_implementation: str

    # Testing configuration
    timeout_settings: Dict[str, int]
    iteration_counts: Dict[str, int]
    memory_monitoring_enabled: bool

    # Collection metadata
    collection_timestamp: str
    collection_method: str
    validation_status: str

class EnvironmentCollector:
    """Automated environment data collection for academic reproducibility"""

    def __init__(self):
        self.platform = platform.system()
        self.validation_errors = []

    def collect_complete_specification(self) -> EnvironmentSpecification:
        """Collect comprehensive environment specification"""
        print("🔍 Collecting environment specification...")

        # Hardware specification
        hardware = self._collect_hardware_specs()

        # Software environment
        software = self._collect_software_specs()

        # Runtime environments
        runtimes = self._collect_runtime_specs()

        # Testing configuration
        testing = self._collect_testing_specs()

        # Metadata
        metadata = {
            'collection_timestamp': datetime.datetime.now().isoformat(),
            'collection_method': 'automated_collection',
            'validation_status': 'collected'
        }

        spec = EnvironmentSpecification(
            # Hardware
            system_manufacturer=hardware.get('system_manufacturer', 'Unknown'),
            system_model=hardware.get('system_model', 'Unknown'),
            processor=hardware.get('processor', 'Unknown'),
            cpu_cores=hardware.get('cpu_cores', 0),
            cpu_threads=hardware.get('cpu_threads', 0),
            base_clock_speed=hardware.get('base_clock_speed', 'Unknown'),
            max_clock_speed=hardware.get('max_clock_speed', 'Unknown'),
            total_memory=hardware.get('total_memory', 'Unknown'),
            memory_type=hardware.get('memory_type', 'Unknown'),
            storage_type=hardware.get('storage_type', 'Unknown'),
            storage_capacity=hardware.get('storage_capacity', 'Unknown'),
            architecture=hardware.get('architecture', 'Unknown'),

            # Software
            os_name=software.get('os_name', 'Unknown'),
            os_version=software.get('os_version', 'Unknown'),
            os_build=software.get('os_build', 'Unknown'),
            kernel_version=software.get('kernel_version', 'Unknown'),

            # Runtime
            java_version=runtimes.get('java_version', 'Not found'),
            java_vendor=runtimes.get('java_vendor', 'Unknown'),
            java_home=runtimes.get('java_home', 'Not found'),
            rust_version=runtimes.get('rust_version', 'Not found'),
            python_version=runtimes.get('python_version', 'Unknown'),
            python_implementation=runtimes.get('python_implementation', 'Unknown'),

            # Testing
            timeout_settings=testing.get('timeout_settings', {}),
            iteration_counts=testing.get('iteration_counts', {}),
            memory_monitoring_enabled=testing.get('memory_monitoring_enabled', False),

            # Metadata
            collection_timestamp=metadata['collection_timestamp'],
            collection_method=metadata['collection_method'],
            validation_status=metadata['validation_status']
        )

        # Validate the specification
        self._validate_specification(spec)

        return spec

    def _collect_hardware_specs(self) -> Dict[str, Any]:
        """Collect hardware specifications"""
        print("💻 Collecting hardware specifications...")
        specs = {}

        try:
            # Basic system info
            specs['system_manufacturer'] = self._get_system_manufacturer()
            specs['system_model'] = self._get_system_model()
            specs['architecture'] = platform.machine()

            # CPU information
            cpu_info = self._get_cpu_info()
            specs.update(cpu_info)

            # Memory information
            memory_info = self._get_memory_info()
            specs.update(memory_info)

            # Storage information
            storage_info = self._get_storage_info()
            specs.update(storage_info)

        except Exception as e:
            self.validation_errors.append(f"Hardware collection error: {e}")

        return specs

    def _collect_software_specs(self) -> Dict[str, Any]:
        """Collect software specifications"""
        print("🖥️  Collecting software specifications...")
        specs = {}

        try:
            # Operating system
            specs['os_name'] = self.platform
            specs['os_version'] = platform.version()
            specs['kernel_version'] = platform.release()

            # Platform-specific details
            if self.platform == "Darwin":
                macos_info = self._get_macos_info()
                specs.update(macos_info)
            elif self.platform == "Linux":
                linux_info = self._get_linux_info()
                specs.update(linux_info)
            elif self.platform == "Windows":
                windows_info = self._get_windows_info()
                specs.update(windows_info)

        except Exception as e:
            self.validation_errors.append(f"Software collection error: {e}")

        return specs

    def _collect_runtime_specs(self) -> Dict[str, Any]:
        """Collect runtime environment specifications"""
        print("⚙️  Collecting runtime specifications...")
        specs = {}

        try:
            # Java environment
            java_info = self._get_java_info()
            specs.update(java_info)

            # Rust environment
            rust_info = self._get_rust_info()
            specs.update(rust_info)

            # Python environment
            python_info = self._get_python_info()
            specs.update(python_info)

        except Exception as e:
            self.validation_errors.append(f"Runtime collection error: {e}")

        return specs

    def _collect_testing_specs(self) -> Dict[str, Any]:
        """Collect testing configuration specifications"""
        print("🧪 Collecting testing specifications...")
        return {
            'timeout_settings': {
                'classification_timeout': 300,
                'consistency_timeout': 300,
                'query_timeout': 120,
                'overall_timeout': 600
            },
            'iteration_counts': {
                'warmup_runs': 3,
                'measurement_runs': 10,
                'statistical_significance': 0.05
            },
            'memory_monitoring_enabled': True
        }

    def _get_system_manufacturer(self) -> str:
        """Get system manufacturer"""
        try:
            if self.platform == "Darwin":
                result = subprocess.run(['sysctl', '-n', 'hw.target'], capture_output=True, text=True)
                return result.stdout.strip()
            elif self.platform == "Linux":
                with open('/sys/devices/virtual/dmi/id/sys_vendor', 'r') as f:
                    return f.read().strip()
        except:
            pass
        return "Unknown"

    def _get_system_model(self) -> str:
        """Get system model"""
        try:
            if self.platform == "Darwin":
                result = subprocess.run(['sysctl', '-n', 'hw.model'], capture_output=True, text=True)
                return result.stdout.strip()
            elif self.platform == "Linux":
                try:
                    with open('/sys/devices/virtual/dmi/id/product_name', 'r') as f:
                        return f.read().strip()
                except:
                    with open('/sys/devices/virtual/dmi/id/board_name', 'r') as f:
                        return f.read().strip()
        except:
            pass
        return "Unknown"

    def _get_cpu_info(self) -> Dict[str, Any]:
        """Get CPU information"""
        info = {}

        try:
            # Basic CPU info
            info['processor'] = platform.processor()
            info['cpu_cores'] = psutil.cpu_count(logical=False)
            info['cpu_threads'] = psutil.cpu_count(logical=True)

            # Clock speed
            if self.platform == "Darwin":
                result = subprocess.run(['sysctl', '-n', 'hw.cpufrequency'], capture_output=True, text=True)
                if result.returncode == 0:
                    freq_hz = int(result.stdout.strip())
                    freq_ghz = freq_hz / 1e9
                    info['base_clock_speed'] = f"{freq_ghz:.2f} GHz"
                    info['max_clock_speed'] = f"{freq_ghz:.2f} GHz"
            elif self.platform == "Linux":
                try:
                    with open('/proc/cpuinfo', 'r') as f:
                        for line in f:
                            if 'model name' in line:
                                info['processor'] = line.split(':')[1].strip()
                            elif 'cpu MHz' in line:
                                freq_mhz = float(line.split(':')[1].strip())
                                info['base_clock_speed'] = f"{freq_mhz / 1000:.2f} GHz"
                                info['max_clock_speed'] = f"{freq_mhz / 1000:.2f} GHz"
                except:
                    pass

            if 'base_clock_speed' not in info:
                info['base_clock_speed'] = "Unknown"
                info['max_clock_speed'] = "Unknown"

        except Exception as e:
            self.validation_errors.append(f"CPU info collection error: {e}")

        return info

    def _get_memory_info(self) -> Dict[str, Any]:
        """Get memory information"""
        info = {}

        try:
            memory = psutil.virtual_memory()
            info['total_memory'] = f"{memory.total / (1024**3):.1f} GB"

            # Memory type (simplified detection)
            if self.platform == "Darwin":
                info['memory_type'] = "LPDDR5"  # Common in Macs
            else:
                info['memory_type'] = "DDR4"  # Common assumption

        except Exception as e:
            self.validation_errors.append(f"Memory info collection error: {e}")

        return info

    def _get_storage_info(self) -> Dict[str, Any]:
        """Get storage information"""
        info = {}

        try:
            # Try to get boot drive info
            boot_drive = Path('/').resolve()
            if boot_drive.exists():
                disk_usage = psutil.disk_usage(str(boot_drive))
                info['storage_capacity'] = f"{disk_usage.total / (1024**3):.0f} GB"
                info['storage_type'] = "SSD"  # Modern assumption
            else:
                info['storage_capacity'] = "Unknown"
                info['storage_type'] = "Unknown"

        except Exception as e:
            self.validation_errors.append(f"Storage info collection error: {e}")

        return info

    def _get_macos_info(self) -> Dict[str, Any]:
        """Get macOS-specific information"""
        info = {}

        try:
            # macOS version
            result = subprocess.run(['sw_vers'], capture_output=True, text=True)
            for line in result.stdout.split('\n'):
                if 'ProductVersion:' in line:
                    info['os_version'] = line.split(':')[1].strip()
                elif 'BuildVersion:' in line:
                    info['os_build'] = line.split(':')[1].strip()

        except Exception as e:
            self.validation_errors.append(f"macOS info collection error: {e}")

        return info

    def _get_linux_info(self) -> Dict[str, Any]:
        """Get Linux-specific information"""
        info = {}

        try:
            # Linux distribution
            try:
                result = subprocess.run(['lsb_release', '-a'], capture_output=True, text=True)
                if result.returncode == 0:
                    for line in result.stdout.split('\n'):
                        if 'Description:' in line:
                            info['os_version'] = line.split(':')[1].strip()
            except:
                # Fallback to /etc/os-release
                try:
                    with open('/etc/os-release', 'r') as f:
                        for line in f:
                            if line.startswith('PRETTY_NAME='):
                                info['os_version'] = line.split('=')[1].strip('"')
                except:
                    pass

        except Exception as e:
            self.validation_errors.append(f"Linux info collection error: {e}")

        return info

    def _get_windows_info(self) -> Dict[str, Any]:
        """Get Windows-specific information"""
        info = {}

        try:
            result = subprocess.run(['systeminfo'], capture_output=True, text=True)
            for line in result.stdout.split('\n'):
                if 'OS Version:' in line:
                    info['os_version'] = line.split(':', 1)[1].strip()
                elif 'OS Build:' in line:
                    info['os_build'] = line.split(':', 1)[1].strip()
        except Exception as e:
            self.validation_errors.append(f"Windows info collection error: {e}")

        return info

    def _get_java_info(self) -> Dict[str, Any]:
        """Get Java environment information"""
        info = {}

        try:
            # Check if Java is available
            result = subprocess.run(['java', '-version'], capture_output=True, text=True)
            if result.returncode == 0:
                # Parse Java version from stderr
                version_line = result.stderr.split('\n')[0]
                if 'version' in version_line:
                    version_match = re.search(r'version "([^"]+)"', version_line)
                    if version_match:
                        info['java_version'] = version_match.group(1)

                    # Extract vendor
                    if 'OpenJDK' in version_line:
                        info['java_vendor'] = 'OpenJDK'
                    elif 'Java(TM)' in version_line:
                        info['java_vendor'] = 'Oracle'

                # Get JAVA_HOME
                java_home = os.environ.get('JAVA_HOME')
                if java_home:
                    info['java_home'] = java_home
                else:
                    info['java_home'] = 'Not set'

        except Exception as e:
            info['java_version'] = 'Not found'
            info['java_vendor'] = 'Unknown'
            info['java_home'] = 'Not found'

        return info

    def _get_rust_info(self) -> Dict[str, Any]:
        """Get Rust environment information"""
        info = {}

        try:
            result = subprocess.run(['rustc', '--version'], capture_output=True, text=True)
            if result.returncode == 0:
                info['rust_version'] = result.stdout.strip()
            else:
                info['rust_version'] = 'Not found'
        except:
            info['rust_version'] = 'Not found'

        return info

    def _get_python_info(self) -> Dict[str, Any]:
        """Get Python environment information"""
        info = {}

        try:
            info['python_version'] = platform.python_version()
            info['python_implementation'] = platform.python_implementation()
        except Exception as e:
            self.validation_errors.append(f"Python info collection error: {e}")

        return info

    def _validate_specification(self, spec: EnvironmentSpecification):
        """Validate the collected specification"""
        print("✅ Validating environment specification...")

        required_fields = [
            'processor', 'cpu_cores', 'total_memory', 'os_name', 'os_version'
        ]

        missing_fields = []
        for field in required_fields:
            value = getattr(spec, field)
            if value in [None, 'Unknown', 'Not found', 0, '']:
                missing_fields.append(field)

        if missing_fields:
            self.validation_errors.append(f"Missing required fields: {missing_fields}")
            spec.validation_status = 'incomplete'
        else:
            spec.validation_status = 'complete'

        if self.validation_errors:
            print(f"⚠️  Validation warnings: {len(self.validation_errors)}")
            for error in self.validation_errors:
                print(f"   - {error}")
        else:
            print("✅ Environment specification validated successfully")

    def generate_publication_specification(self, spec: EnvironmentSpecification) -> str:
        """Generate publication-ready environment specification document"""
        template = f"""
# Environment Specification for Academic Publication

## System Information
- **System Manufacturer**: {spec.system_manufacturer}
- **System Model**: {spec.system_model}
- **Architecture**: {spec.architecture}

## Processor
- **Processor**: {spec.processor}
- **CPU Cores**: {spec.cpu_cores}
- **CPU Threads**: {spec.cpu_threads}
- **Base Clock Speed**: {spec.base_clock_speed}
- **Max Clock Speed**: {spec.max_clock_speed}

## Memory
- **Total Memory**: {spec.total_memory}
- **Memory Type**: {spec.memory_type}

## Storage
- **Storage Type**: {spec.storage_type}
- **Storage Capacity**: {spec.storage_capacity}

## Operating System
- **OS Name**: {spec.os_name}
- **OS Version**: {spec.os_version}
- **OS Build**: {spec.os_build}
- **Kernel Version**: {spec.kernel_version}

## Runtime Environments
- **Java Version**: {spec.java_version}
- **Java Vendor**: {spec.java_vendor}
- **JAVA_HOME**: {spec.java_home}
- **Rust Version**: {spec.rust_version}
- **Python Version**: {spec.python_version}
- **Python Implementation**: {spec.python_implementation}

## Testing Configuration
- **Classification Timeout**: {spec.timeout_settings.get('classification_timeout', 'N/A')} seconds
- **Consistency Timeout**: {spec.timeout_settings.get('consistency_timeout', 'N/A')} seconds
- **Query Timeout**: {spec.timeout_settings.get('query_timeout', 'N/A')} seconds
- **Overall Timeout**: {spec.timeout_settings.get('overall_timeout', 'N/A')} seconds
- **Warmup Runs**: {spec.iteration_counts.get('warmup_runs', 'N/A')}
- **Measurement Runs**: {spec.iteration_counts.get('measurement_runs', 'N/A')}
- **Memory Monitoring**: {'Enabled' if spec.memory_monitoring_enabled else 'Disabled'}

## Collection Metadata
- **Collection Timestamp**: {spec.collection_timestamp}
- **Collection Method**: {spec.collection_method}
- **Validation Status**: {spec.validation_status.upper()}
"""

        if self.validation_errors:
            template += "\n## Validation Warnings\n"
            for error in self.validation_errors:
                template += f"- {error}\n"

        return template

    def save_specification(self, spec: EnvironmentSpecification, output_dir: str = "."):
        """Save specification in multiple formats"""
        output_path = Path(output_dir)
        output_path.mkdir(exist_ok=True)

        # Save as markdown
        markdown_file = output_path / "environment_specification.md"
        with open(markdown_file, 'w') as f:
            f.write(self.generate_publication_specification(spec))

        # Save as JSON
        json_file = output_path / "environment_specification.json"
        with open(json_file, 'w') as f:
            json.dump(asdict(spec), f, indent=2)

        # Save as YAML (academic standard)
        yaml_file = output_path / "environment_specification.yaml"
        try:
            import yaml
            with open(yaml_file, 'w') as f:
                yaml.dump(asdict(spec), f, default_flow_style=False)
        except ImportError:
            print("⚠️  PyYAML not installed, skipping YAML export")

        print(f"✅ Environment specification saved:")
        print(f"   📄 Markdown: {markdown_file}")
        print(f"   💾 JSON: {json_file}")
        if yaml_file.exists():
            print(f"   📋 YAML: {yaml_file}")

def main():
    """Main interface for environment collection"""
    print("🌍 Environment Specification Collector")
    print("=" * 50)

    collector = EnvironmentCollector()
    spec = collector.collect_complete_specification()

    # Save specification
    output_dir = "environment_specs"
    collector.save_specification(spec, output_dir)

    print(f"\n🎯 Environment specification collection complete!")
    print(f"📊 Validation Status: {spec.validation_status.upper()}")
    print(f"⚠️  Warnings: {len(collector.validation_errors)}")

if __name__ == "__main__":
    main()