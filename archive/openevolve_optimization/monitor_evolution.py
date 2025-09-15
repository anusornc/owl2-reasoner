#!/usr/bin/env python3
"""
Real-time OpenEvolve Monitoring Script
Monitors evolution progress and updates visualization dashboard
"""

import json
import time
import threading
import subprocess
from pathlib import Path
from datetime import datetime
import re
from evolution_visualization import EvolutionVisualizer

class EvolutionMonitor:
    def __init__(self, output_dir="openevolve_output"):
        self.output_dir = Path(output_dir)
        self.visualizer = EvolutionVisualizer(output_dir)
        self.running = False
        self.log_file = None
        self.find_log_file()

    def find_log_file(self):
        """Find the most recent evolution log file"""
        logs_dir = self.output_dir / "logs"
        if logs_dir.exists():
            log_files = list(logs_dir.glob("openevolve_*.log"))
            if log_files:
                self.log_file = max(log_files, key=lambda x: x.stat().st_mtime)
                print(f"📄 Found log file: {self.log_file}")

    def parse_log_line(self, line):
        """Parse a log line and extract evolution metrics"""
        # Parse iteration completion
        iteration_match = re.search(r"Iteration (\d+): Program ([a-f0-9-]+).*completed in ([\d.]+)s", line)
        if iteration_match:
            return {
                "type": "iteration_complete",
                "iteration": int(iteration_match.group(1)),
                "program_id": iteration_match.group(2),
                "time_seconds": float(iteration_match.group(3))
            }

        # Parse metrics
        metrics_match = re.search(r"Metrics: score=([\d.]+).*correctness=([\d.]+).*performance=([\d.]+).*avg_time_ns=([\d.]+)", line)
        if metrics_match:
            return {
                "type": "metrics",
                "score": float(metrics_match.group(1)),
                "correctness": float(metrics_match.group(2)),
                "performance": float(metrics_match.group(3)),
                "avg_time_ns": float(metrics_match.group(4))
            }

        # Parse new best solution
        best_match = re.search(r"New best program ([a-f0-9-]+)", line)
        if best_match:
            return {
                "type": "new_best",
                "program_id": best_match.group(1)
            }

        # Parse island status
        island_match = re.search(r"Island (\d+): (\d+) programs, best=([\d.]+)", line)
        if island_match:
            return {
                "type": "island_status",
                "island_id": int(island_match.group(1)),
                "programs": int(island_match.group(2)),
                "best_score": float(island_match.group(3))
            }

        return None

    def read_existing_log_data(self):
        """Read and parse existing log data"""
        if not self.log_file or not self.log_file.exists():
            return

        print(f"📖 Parsing existing log data from {self.log_file}")

        with open(self.log_file, 'r') as f:
            current_iteration = None
            current_metrics = None

            for line in f:
                parsed = self.parse_log_line(line.strip())
                if parsed:
                    if parsed["type"] == "iteration_complete":
                        current_iteration = parsed["iteration"]
                    elif parsed["type"] == "metrics" and current_iteration:
                        current_metrics = parsed
                        # Add to visualizer data
                        iteration_data = {
                            "iteration": current_iteration,
                            "best_score": current_metrics["score"],
                            "correctness": current_metrics["correctness"],
                            "performance": current_metrics["performance"],
                            "time_ns": current_metrics["avg_time_ns"],
                            "timestamp": datetime.now().isoformat()
                        }
                        self.visualizer.add_iteration_result(iteration_data)

                        if current_metrics["score"] > 0.8:  # Good solutions
                            program_data = {
                                "id": f"prog_{current_iteration}",
                                "generation": current_iteration // 5,
                                "score": current_metrics["score"],
                                "timestamp": datetime.now().isoformat()
                            }
                            self.visualizer.add_best_program(program_data)

                        current_iteration = None
                        current_metrics = None

        print(f"✅ Parsed {len(self.visualizer.current_data['iterations'])} iterations")

    def monitor_log_file(self):
        """Monitor log file for new evolution data"""
        if not self.log_file:
            print("⚠️  No log file found")
            return

        print(f"👀 Starting real-time monitoring of {self.log_file}")

        try:
            with open(self.log_file, 'r') as f:
                # Go to end of file
                f.seek(0, 2)
                current_iteration = None
                current_metrics = None

                while self.running:
                    line = f.readline()
                    if line:
                        parsed = self.parse_log_line(line.strip())
                        if parsed:
                            if parsed["type"] == "iteration_complete":
                                current_iteration = parsed["iteration"]
                                print(f"🔄 Iteration {current_iteration} completed")
                            elif parsed["type"] == "metrics" and current_iteration:
                                current_metrics = parsed
                                # Add to visualizer data
                                iteration_data = {
                                    "iteration": current_iteration,
                                    "best_score": current_metrics["score"],
                                    "correctness": current_metrics["correctness"],
                                    "performance": current_metrics["performance"],
                                    "time_ns": current_metrics["avg_time_ns"],
                                    "timestamp": datetime.now().isoformat()
                                }
                                self.visualizer.add_iteration_result(iteration_data)

                                print(f"📊 Iteration {current_iteration}: Score={current_metrics['score']:.4f}, "
                                      f"Correctness={current_metrics['correctness']:.4f}, "
                                      f"Performance={current_metrics['performance']:.2f}")

                                if current_metrics["score"] > 0.8:
                                    program_data = {
                                        "id": f"prog_{current_iteration}",
                                        "generation": current_iteration // 5,
                                        "score": current_metrics["score"],
                                        "timestamp": datetime.now().isoformat()
                                    }
                                    self.visualizer.add_best_program(program_data)

                                current_iteration = None
                                current_metrics = None

                            elif parsed["type"] == "new_best":
                                print(f"🎉 New best solution found: {parsed['program_id']}")

                            elif parsed["type"] == "island_status":
                                print(f"🏝️  Island {parsed['island_id']}: {parsed['programs']} programs, best={parsed['best_score']:.4f}")
                    else:
                        time.sleep(0.1)  # Wait for new content

        except Exception as e:
            print(f"❌ Error monitoring log file: {e}")

    def start_monitoring(self):
        """Start monitoring evolution"""
        self.running = True

        # First, read existing data
        self.read_existing_log_data()

        # Start real-time monitoring
        monitor_thread = threading.Thread(target=self.monitor_log_file)
        monitor_thread.daemon = True
        monitor_thread.start()

        return monitor_thread

    def stop_monitoring(self):
        """Stop monitoring"""
        self.running = False

def main():
    """Main function"""
    import argparse

    parser = argparse.ArgumentParser(description="OpenEvolve Real-time Monitor")
    parser.add_argument("--output-dir", default="openevolve_output", help="Evolution output directory")
    parser.add_argument("--dashboard-only", action="store_true", help="Start dashboard without monitoring")
    parser.add_argument("--port", type=int, default=8080, help="Dashboard port")

    args = parser.parse_args()

    print("🧬 OpenEvolve Real-time Evolution Monitor")
    print("=" * 50)

    if args.dashboard_only:
        # Start dashboard only
        from evolution_visualization import start_visualization_server
        start_visualization_server(args.port, args.output_dir)
    else:
        # Start monitoring and dashboard
        monitor = EvolutionMonitor(args.output_dir)

        try:
            # Start monitoring
            monitor_thread = monitor.start_monitoring()

            # Start dashboard in a separate process
            import multiprocessing
            dashboard_process = multiprocessing.Process(
                target=start_visualization_server,
                args=(args.port, args.output_dir)
            )
            dashboard_process.start()

            print(f"🚀 Monitoring system started!")
            print(f"📊 Dashboard: http://localhost:{args.port}")
            print(f"📂 Monitoring directory: {args.output_dir}")
            print(f"⏹️  Press Ctrl+C to stop")

            # Keep main thread alive
            while True:
                time.sleep(1)

        except KeyboardInterrupt:
            print(f"\n🛑 Stopping monitoring system...")
            monitor.stop_monitoring()
            if 'dashboard_process' in locals():
                dashboard_process.terminate()
            print("✅ Monitoring stopped")

if __name__ == "__main__":
    main()