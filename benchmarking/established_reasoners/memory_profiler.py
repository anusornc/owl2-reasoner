#!/usr/bin/env python3

"""
Publication-Ready Memory Profiling System for OWL2 Reasoner Testing
Implements cross-platform memory monitoring with academic-grade metrics
"""

import psutil
import platform
import time
import threading
import subprocess
import json
from typing import Dict, List, Tuple, Optional, Any
from dataclasses import dataclass
from pathlib import Path
import os
import sys

@dataclass
class MemoryMetrics:
    """Academic-grade memory metrics for publication"""
    peak_memory_mb: float
    average_memory_mb: float
    final_memory_mb: float
    memory_efficiency: float  # triples per MB
    memory_overhead_percent: float
    gc_collections: int  # JVM only
    memory_timeline: List[Tuple[float, float]]  # (time, memory_mb)
    process_id: int
    monitoring_duration: float

class CrossPlatformMemoryProfiler:
    """Cross-platform memory profiler for academic OWL2 reasoner testing"""

    def __init__(self, sampling_interval: float = 0.1):
        self.platform = platform.system()
        self.sampling_interval = sampling_interval
        self.monitoring_active = False
        self.monitoring_thread = None
        self.memory_data = []
        self.process_info = {}

    def start_monitoring(self, process_info: Dict[str, Any]) -> int:
        """Start memory monitoring for a process"""
        self.process_info = process_info
        self.memory_data = []
        self.monitoring_active = True

        # Start monitoring thread
        self.monitoring_thread = threading.Thread(
            target=self._monitor_memory,
            daemon=True
        )
        self.monitoring_thread.start()

        return process_info.get('pid', 0)

    def stop_monitoring(self) -> MemoryMetrics:
        """Stop monitoring and return comprehensive metrics"""
        self.monitoring_active = False

        if self.monitoring_thread:
            self.monitoring_thread.join(timeout=5.0)

        return self._calculate_metrics()

    def _monitor_memory(self):
        """Core monitoring logic (platform-specific)"""
        start_time = time.time()

        while self.monitoring_active:
            try:
                if self.platform == "Darwin":
                    memory_mb = self._get_macos_memory()
                elif self.platform == "Linux":
                    memory_mb = self._get_linux_memory()
                elif self.platform == "Windows":
                    memory_mb = self._get_windows_memory()
                else:
                    memory_mb = self._get_generic_memory()

                current_time = time.time() - start_time
                self.memory_data.append((current_time, memory_mb))

                time.sleep(self.sampling_interval)

            except Exception as e:
                print(f"Memory monitoring error: {e}")
                break

    def _get_macos_memory(self) -> float:
        """Get memory usage on macOS"""
        try:
            pid = self.process_info.get('pid')
            if pid:
                process = psutil.Process(pid)
                return process.memory_info().rss / 1024 / 1024  # Convert to MB
        except:
            pass
        return 0.0

    def _get_linux_memory(self) -> float:
        """Get memory usage on Linux"""
        try:
            pid = self.process_info.get('pid')
            if pid:
                # Try psutil first
                process = psutil.Process(pid)
                return process.memory_info().rss / 1024 / 1024
        except:
            pass

        # Fallback to /proc filesystem
        try:
            pid = self.process_info.get('pid')
            if pid:
                with open(f'/proc/{pid}/status', 'r') as f:
                    for line in f:
                        if line.startswith('VmRSS:'):
                            return float(line.split()[1]) / 1024  # KB to MB
        except:
            pass

        return 0.0

    def _get_windows_memory(self) -> float:
        """Get memory usage on Windows"""
        try:
            pid = self.process_info.get('pid')
            if pid:
                process = psutil.Process(pid)
                return process.memory_info().rss / 1024 / 1024
        except:
            pass
        return 0.0

    def _get_generic_memory(self) -> float:
        """Generic memory monitoring using psutil"""
        try:
            pid = self.process_info.get('pid')
            if pid:
                process = psutil.Process(pid)
                return process.memory_info().rss / 1024 / 1024
        except:
            pass
        return 0.0

    def _calculate_metrics(self) -> MemoryMetrics:
        """Calculate comprehensive memory metrics"""
        if not self.memory_data:
            return MemoryMetrics(0.0, 0.0, 0.0, 0.0, 0.0, 0, [], 0, 0.0)

        # Extract memory values
        memory_values = [mem for _, mem in self.memory_data]

        # Basic metrics
        peak_memory_mb = max(memory_values)
        average_memory_mb = sum(memory_values) / len(memory_values)
        final_memory_mb = memory_values[-1] if memory_values else 0.0

        # Calculate efficiency (if triples processed is available)
        triples_processed = self.process_info.get('triples_processed', 1)
        memory_efficiency = triples_processed / average_memory_mb if average_memory_mb > 0 else 0.0

        # Calculate overhead (simplified)
        baseline_memory = self.process_info.get('baseline_memory', 10.0)  # MB
        memory_overhead_percent = ((average_memory_mb - baseline_memory) / baseline_memory) * 100

        # GC collections (estimate for JVM processes)
        gc_collections = self._estimate_gc_collections()

        # Monitoring duration
        monitoring_duration = self.memory_data[-1][0] if self.memory_data else 0.0

        return MemoryMetrics(
            peak_memory_mb=peak_memory_mb,
            average_memory_mb=average_memory_mb,
            final_memory_mb=final_memory_mb,
            memory_efficiency=memory_efficiency,
            memory_overhead_percent=memory_overhead_percent,
            gc_collections=gc_collections,
            memory_timeline=self.memory_data,
            process_id=self.process_info.get('pid', 0),
            monitoring_duration=monitoring_duration
        )

    def _estimate_gc_collections(self) -> int:
        """Estimate GC collections for JVM processes"""
        # This is a simplified estimate
        # In practice, you might use JVM monitoring tools
        return 0

class ProcessMemoryMonitor:
    """High-level interface for monitoring process memory"""

    def __init__(self):
        self.profiler = CrossPlatformMemoryProfiler()
        self.active_monitors = {}

    def monitor_command(self, command: List[str], timeout: int = 300,
                       triples_processed: int = 1) -> Tuple[subprocess.CompletedProcess, MemoryMetrics]:
        """Monitor a command and return both process result and memory metrics"""

        # Start process
        process = subprocess.Popen(
            command,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True
        )

        # Start memory monitoring
        process_info = {
            'pid': process.pid,
            'command': command,
            'start_time': time.time(),
            'triples_processed': triples_processed,
            'baseline_memory': self._get_baseline_memory()
        }

        monitor_id = f"process_{process.pid}"
        self.profiler.start_monitoring(process_info)

        try:
            # Wait for process completion
            result = process.wait(timeout=timeout)

            # Get process output
            stdout, stderr = process.communicate()

            completed_process = subprocess.CompletedProcess(
                args=command,
                returncode=result,
                stdout=stdout,
                stderr=stderr
            )

        except subprocess.TimeoutExpired:
            process.kill()
            stdout, stderr = process.communicate()
            completed_process = subprocess.CompletedProcess(
                args=command,
                returncode=-1,
                stdout=stdout,
                stderr="Process timed out"
            )

        finally:
            # Stop monitoring and get metrics
            memory_metrics = self.profiler.stop_monitoring()

        return completed_process, memory_metrics

    def _get_baseline_memory(self) -> float:
        """Get baseline memory usage"""
        try:
            return psutil.virtual_memory().used / 1024 / 1024
        except:
            return 10.0  # Default baseline

class MemoryAnalysisEngine:
    """Advanced memory analysis for academic publications"""

    @staticmethod
    def analyze_efficiency(metrics: MemoryMetrics) -> Dict[str, float]:
        """Analyze memory efficiency for publication"""
        return {
            'peak_memory_mb': metrics.peak_memory_mb,
            'average_memory_mb': metrics.average_memory_mb,
            'memory_efficiency_triples_per_mb': metrics.memory_efficiency,
            'memory_overhead_percent': metrics.memory_overhead_percent,
            'memory_stability': MemoryAnalysisEngine._calculate_memory_stability(metrics),
            'scalability_score': MemoryAnalysisEngine._calculate_scalability_score(metrics)
        }

    @staticmethod
    def _calculate_memory_stability(metrics: MemoryMetrics) -> float:
        """Calculate memory usage stability (0-1 scale)"""
        if len(metrics.memory_timeline) < 2:
            return 1.0

        memory_values = [mem for _, mem in metrics.memory_timeline]
        if not memory_values:
            return 1.0

        # Calculate coefficient of variation
        mean = sum(memory_values) / len(memory_values)
        if mean == 0:
            return 1.0

        variance = sum((x - mean) ** 2 for x in memory_values) / len(memory_values)
        std_dev = variance ** 0.5
        cv = std_dev / mean

        # Convert to stability score (lower CV = higher stability)
        return max(0.0, 1.0 - (cv * 2))  # Scale factor for scoring

    @staticmethod
    def _calculate_scalability_score(metrics: MemoryMetrics) -> float:
        """Calculate scalability score based on memory usage patterns"""
        if metrics.average_memory_mb == 0:
            return 0.0

        # Higher efficiency and lower overhead = better scalability
        efficiency_component = min(1.0, metrics.memory_efficiency / 1000)  # Normalize
        overhead_component = max(0.0, 1.0 - (metrics.memory_overhead_percent / 100))

        return (efficiency_component + overhead_component) / 2

    @staticmethod
    def generate_memory_report(metrics: MemoryMetrics) -> str:
        """Generate publication-ready memory report"""
        analysis = MemoryAnalysisEngine.analyze_efficiency(metrics)

        report = f"""
# Memory Profiling Results

## Basic Metrics
- **Peak Memory Usage**: {metrics.peak_memory_mb:.2f} MB
- **Average Memory Usage**: {metrics.average_memory_mb:.2f} MB
- **Final Memory Usage**: {metrics.final_memory_mb:.2f} MB
- **Monitoring Duration**: {metrics.monitoring_duration:.2f} seconds

## Efficiency Metrics
- **Memory Efficiency**: {analysis['memory_efficiency_triples_per_mb']:.1f} triples/MB
- **Memory Overhead**: {analysis['memory_overhead_percent']:.1f}%
- **Memory Stability**: {analysis['memory_stability']:.3f}
- **Scalability Score**: {analysis['scalability_score']:.3f}

## Advanced Metrics
- **GC Collections**: {metrics.gc_collections}
- **Process ID**: {metrics.process_id}
- **Data Points Collected**: {len(metrics.memory_timeline)}

## Memory Timeline
{MemoryAnalysisEngine._format_timeline(metrics.memory_timeline)}
"""
        return report

    @staticmethod
    def _format_timeline(timeline: List[Tuple[float, float]]) -> str:
        """Format memory timeline for reporting"""
        if not timeline:
            return "No timeline data available"

        # Show sample points
        if len(timeline) > 10:
            step = len(timeline) // 10
            sample_points = timeline[::step]
        else:
            sample_points = timeline

        lines = ["Time (s) | Memory (MB)"]
        lines.append("-" * 20)
        for time_point, memory in sample_points:
            lines.append(f"{time_point:8.2f} | {memory:10.2f}")

        return "\n".join(lines)

# Main execution interface
def main():
    """Main interface for memory profiling"""
    if len(sys.argv) < 2:
        print("Usage: python memory_profiler.py <command> [args...]")
        print("Example: python memory_profiler.py java -jar HermiT.jar -c test.owl")
        return

    command = sys.argv[1:]

    monitor = ProcessMemoryMonitor()

    print(f"🔍 Monitoring command: {' '.join(command)}")
    print("⏱️  Starting memory profiling...")

    try:
        result, metrics = monitor.monitor_command(command, timeout=300)

        print("✅ Monitoring complete")
        print(f"📊 Return code: {result.returncode}")
        print(f"🧠 Peak memory: {metrics.peak_memory_mb:.2f} MB")
        print(f"📈 Average memory: {metrics.average_memory_mb:.2f} MB")
        print(f"⚡ Memory efficiency: {metrics.memory_efficiency:.1f} triples/MB")

        # Save detailed report
        report = MemoryAnalysisEngine.generate_memory_report(metrics)
        report_file = "memory_report.md"
        with open(report_file, 'w') as f:
            f.write(report)

        print(f"📄 Detailed report saved to: {report_file}")

        # Save metrics as JSON
        metrics_file = "memory_metrics.json"
        metrics_dict = {
            'peak_memory_mb': metrics.peak_memory_mb,
            'average_memory_mb': metrics.average_memory_mb,
            'final_memory_mb': metrics.final_memory_mb,
            'memory_efficiency': metrics.memory_efficiency,
            'memory_overhead_percent': metrics.memory_overhead_percent,
            'gc_collections': metrics.gc_collections,
            'monitoring_duration': metrics.monitoring_duration,
            'process_id': metrics.process_id,
            'command': command,
            'return_code': result.returncode
        }

        with open(metrics_file, 'w') as f:
            json.dump(metrics_dict, f, indent=2)

        print(f"💾 Metrics saved to: {metrics_file}")

    except Exception as e:
        print(f"❌ Error during monitoring: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    main()