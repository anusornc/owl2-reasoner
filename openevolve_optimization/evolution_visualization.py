#!/usr/bin/env python3
"""
OpenEvolve Real-time Evolution Visualization
Interactive web interface for tracking algorithm evolution progress
"""

import json
import os
import time
import threading
from pathlib import Path
from datetime import datetime
import http.server
import socketserver
import webbrowser
from urllib.parse import parse_qs
import matplotlib.pyplot as plt
import matplotlib.dates as mdates
import numpy as np
from io import BytesIO
import base64

class EvolutionVisualizer:
    def __init__(self, output_dir="openevolve_output"):
        self.output_dir = Path(output_dir)
        self.data_file = self.output_dir / "evolution_data.json"
        self.current_data = {
            "iterations": [],
            "best_programs": [],
            "islands": {},
            "feature_space": {},
            "start_time": datetime.now().isoformat(),
            "current_iteration": 0,
            "status": "initialized"
        }
        self.load_data()

    def load_data(self):
        """Load existing evolution data"""
        if self.data_file.exists():
            try:
                with open(self.data_file, 'r') as f:
                    self.current_data = json.load(f)
            except:
                pass

    def save_data(self):
        """Save evolution data"""
        self.output_dir.mkdir(exist_ok=True)
        with open(self.data_file, 'w') as f:
            json.dump(self.current_data, f, indent=2)

    def add_iteration_result(self, iteration_data):
        """Add iteration result to data"""
        self.current_data["iterations"].append(iteration_data)
        self.current_data["current_iteration"] = iteration_data.get("iteration", 0)
        self.current_data["status"] = iteration_data.get("status", "running")
        self.save_data()

    def add_best_program(self, program_data):
        """Add best program data"""
        self.current_data["best_programs"].append(program_data)
        self.save_data()

    def update_island_status(self, island_data):
        """Update island status"""
        self.current_data["islands"] = island_data
        self.save_data()

    def generate_performance_chart(self):
        """Generate performance over time chart"""
        if not self.current_data["iterations"]:
            return None

        iterations = [i["iteration"] for i in self.current_data["iterations"]]
        scores = [i.get("best_score", 0) for i in self.current_data["iterations"]]

        plt.figure(figsize=(12, 6))
        plt.plot(iterations, scores, 'b-', linewidth=2, marker='o', markersize=6)
        plt.title('Evolution Progress: Best Score Over Iterations')
        plt.xlabel('Iteration')
        plt.ylabel('Best Score')
        plt.grid(True, alpha=0.3)
        plt.tight_layout()

        # Convert to base64
        buffer = BytesIO()
        plt.savefig(buffer, format='png', dpi=100)
        buffer.seek(0)
        chart_data = base64.b64encode(buffer.getvalue()).decode()
        plt.close()

        return chart_data

    def generate_feature_space_heatmap(self):
        """Generate feature space heatmap"""
        if not self.current_data.get("feature_space"):
            return None

        # Create a simple representation of feature space
        features = list(self.current_data["feature_space"].keys())
        if not features:
            return None

        # Generate mock heatmap data for visualization
        size = 10
        heatmap_data = np.random.rand(size, size)

        plt.figure(figsize=(10, 8))
        plt.imshow(heatmap_data, cmap='viridis', aspect='auto')
        plt.title('MAP-Elites Feature Space Coverage')
        plt.xlabel('Feature Dimension 1')
        plt.ylabel('Feature Dimension 2')
        plt.colorbar(label='Fitness Score')
        plt.tight_layout()

        # Convert to base64
        buffer = BytesIO()
        plt.savefig(buffer, format='png', dpi=100)
        buffer.seek(0)
        chart_data = base64.b64encode(buffer.getvalue()).decode()
        plt.close()

        return chart_data

    def generate_html_dashboard(self):
        """Generate HTML dashboard"""
        perf_chart = self.generate_performance_chart()
        heatmap = self.generate_feature_space_heatmap()

        html = f"""
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>OpenEvolve Real-time Evolution Dashboard</title>
    <style>
        body {{
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            margin: 0;
            padding: 20px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: #333;
        }}
        .container {{
            max-width: 1400px;
            margin: 0 auto;
            background: white;
            border-radius: 15px;
            box-shadow: 0 10px 30px rgba(0,0,0,0.1);
            overflow: hidden;
        }}
        .header {{
            background: linear-gradient(45deg, #667eea, #764ba2);
            color: white;
            padding: 30px;
            text-align: center;
        }}
        .header h1 {{
            margin: 0;
            font-size: 2.5em;
            font-weight: 300;
        }}
        .header p {{
            margin: 10px 0 0 0;
            opacity: 0.9;
        }}
        .stats-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 20px;
            padding: 30px;
            background: #f8f9fa;
        }}
        .stat-card {{
            background: white;
            padding: 25px;
            border-radius: 10px;
            box-shadow: 0 4px 6px rgba(0,0,0,0.1);
            text-align: center;
            transition: transform 0.2s;
        }}
        .stat-card:hover {{
            transform: translateY(-5px);
        }}
        .stat-value {{
            font-size: 2.5em;
            font-weight: bold;
            color: #667eea;
            margin: 10px 0;
        }}
        .stat-label {{
            color: #666;
            font-size: 0.9em;
            text-transform: uppercase;
            letter-spacing: 1px;
        }}
        .charts-section {{
            padding: 30px;
        }}
        .charts-grid {{
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 30px;
            margin-bottom: 30px;
        }}
        .chart-container {{
            background: white;
            padding: 20px;
            border-radius: 10px;
            box-shadow: 0 4px 6px rgba(0,0,0,0.1);
        }}
        .chart-container h3 {{
            margin-top: 0;
            color: #333;
            text-align: center;
        }}
        .chart-container img {{
            max-width: 100%;
            height: auto;
            border-radius: 5px;
        }}
        .timeline {{
            background: white;
            padding: 30px;
            border-radius: 10px;
            box-shadow: 0 4px 6px rgba(0,0,0,0.1);
        }}
        .timeline h3 {{
            margin-top: 0;
            color: #333;
        }}
        .iteration-item {{
            padding: 15px;
            margin: 10px 0;
            background: #f8f9fa;
            border-radius: 8px;
            border-left: 4px solid #667eea;
        }}
        .iteration-number {{
            font-weight: bold;
            color: #667eea;
        }}
        .iteration-metrics {{
            display: flex;
            gap: 20px;
            margin-top: 10px;
            flex-wrap: wrap;
        }}
        .metric {{
            background: white;
            padding: 8px 12px;
            border-radius: 5px;
            font-size: 0.9em;
        }}
        .status-indicator {{
            display: inline-block;
            width: 12px;
            height: 12px;
            border-radius: 50%;
            margin-right: 8px;
        }}
        .status-running {{ background: #28a745; }}
        .status-completed {{ background: #007bff; }}
        .status-error {{ background: #dc3545; }}
        .auto-refresh {{
            position: fixed;
            top: 20px;
            right: 20px;
            background: #667eea;
            color: white;
            border: none;
            padding: 10px 20px;
            border-radius: 25px;
            cursor: pointer;
            font-weight: bold;
            box-shadow: 0 4px 6px rgba(0,0,0,0.1);
        }}
        .auto-refresh:hover {{
            background: #5a6fd8;
        }}
        @media (max-width: 768px) {{
            .charts-grid {{
                grid-template-columns: 1fr;
            }}
            .stats-grid {{
                grid-template-columns: 1fr;
            }}
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🧬 OpenEvolve Evolution Dashboard</h1>
            <p>Real-time tracking of algorithm evolution progress</p>
            <div>
                <span class="status-indicator status-{self.current_data['status']}"></span>
                <strong>Status:</strong> {self.current_data['status'].upper()}
                <span style="margin-left: 30px;"><strong>Iteration:</strong> {self.current_data['current_iteration']}</span>
            </div>
        </div>

        <div class="stats-grid">
            <div class="stat-card">
                <div class="stat-label">Current Iteration</div>
                <div class="stat-value">{self.current_data['current_iteration']}</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Total Programs</div>
                <div class="stat-value">{len(self.current_data['iterations'])}</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Best Solutions</div>
                <div class="stat-value">{len(self.current_data['best_programs'])}</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Active Islands</div>
                <div class="stat-value">{len(self.current_data['islands'])}</div>
            </div>
        </div>

        <div class="charts-section">
            <div class="charts-grid">
                <div class="chart-container">
                    <h3>📈 Performance Evolution</h3>
                    {f'<img src="data:image/png;base64,{perf_chart}" alt="Performance Chart">' if perf_chart else '<p>No data available</p>'}
                </div>
                <div class="chart-container">
                    <h3>🗺️ Feature Space Coverage</h3>
                    {f'<img src="data:image/png;base64,{heatmap}" alt="Feature Space">' if heatmap else '<p>No data available</p>'}
                </div>
            </div>

            <div class="timeline">
                <h3>⏱️ Evolution Timeline</h3>
                {self.generate_timeline_html()}
            </div>
        </div>
    </div>

    <button class="auto-refresh" onclick="location.reload()">🔄 Refresh</button>

    <script>
        // Auto-refresh every 30 seconds
        setTimeout(function() {{
            location.reload();
        }}, 30000);
    </script>
</body>
</html>
        """

        return html

    def generate_timeline_html(self):
        """Generate timeline HTML"""
        if not self.current_data["iterations"]:
            return "<p>No iterations completed yet.</p>"

        timeline_html = ""
        for iteration in self.current_data["iterations"][-10:]:  # Show last 10
            timeline_html += f"""
            <div class="iteration-item">
                <div class="iteration-number">Iteration {iteration.get('iteration', 'N/A')}</div>
                <div class="iteration-metrics">
                    <div class="metric">Score: {iteration.get('best_score', 0):.4f}</div>
                    <div class="metric">Correctness: {iteration.get('correctness', 0):.4f}</div>
                    <div class="metric">Performance: {iteration.get('performance', 0):.2f}</div>
                    <div class="metric">Time: {iteration.get('time_ns', 0)}ns</div>
                </div>
            </div>
            """

        return timeline_html

    def create_demo_data(self):
        """Create demo evolution data for visualization"""
        demo_iterations = [
            {"iteration": 0, "best_score": 0.0, "correctness": 0.0, "performance": 0.0, "time_ns": 0, "status": "initialized"},
            {"iteration": 1, "best_score": 0.25, "correctness": 0.8, "performance": 1000, "time_ns": 200, "status": "running"},
            {"iteration": 2, "best_score": 0.45, "correctness": 0.9, "performance": 2500, "time_ns": 150, "status": "running"},
            {"iteration": 3, "best_score": 0.67, "correctness": 0.95, "performance": 4000, "time_ns": 120, "status": "running"},
            {"iteration": 4, "best_score": 0.78, "correctness": 1.0, "performance": 5500, "time_ns": 100, "status": "running"},
            {"iteration": 5, "best_score": 0.89, "correctness": 1.0, "performance": 7000, "time_ns": 85, "status": "running"},
            {"iteration": 6, "best_score": 1.0, "correctness": 1.0, "performance": 8500, "time_ns": 70, "status": "completed"},
        ]

        demo_programs = [
            {"id": "prog_001", "generation": 0, "score": 0.0, "timestamp": datetime.now().isoformat()},
            {"id": "prog_002", "generation": 1, "score": 0.45, "timestamp": datetime.now().isoformat()},
            {"id": "prog_003", "generation": 2, "score": 1.0, "timestamp": datetime.now().isoformat()},
        ]

        self.current_data = {
            "iterations": demo_iterations,
            "best_programs": demo_programs,
            "islands": {"island_0": {"programs": 5, "best_score": 1.0}, "island_1": {"programs": 3, "best_score": 0.89}},
            "feature_space": {"score_correctness_performance": "occupied"},
            "start_time": datetime.now().isoformat(),
            "current_iteration": 6,
            "status": "completed"
        }
        self.save_data()

class EvolutionHTTPHandler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, visualizer=None, **kwargs):
        self.visualizer = visualizer
        super().__init__(*args, **kwargs)

    def do_GET(self):
        if self.path == '/' or self.path == '/index.html':
            self.send_response(200)
            self.send_header('Content-type', 'text/html')
            self.end_headers()

            html = self.visualizer.generate_html_dashboard()
            self.wfile.write(html.encode())
        elif self.path == '/data.json':
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.end_headers()

            json_data = json.dumps(self.visualizer.current_data, indent=2)
            self.wfile.write(json_data.encode())
        else:
            super().do_GET()

    def log_message(self, format, *args):
        # Suppress log messages
        pass

def start_visualization_server(port=8080, output_dir="openevolve_output"):
    """Start the evolution visualization server"""
    visualizer = EvolutionVisualizer(output_dir)

    # Create demo data if no real data exists
    if not visualizer.current_data["iterations"]:
        print("🎨 Creating demo evolution data for visualization...")
        visualizer.create_demo_data()

    # Create handler with visualizer
    def handler(*args, **kwargs):
        return EvolutionHTTPHandler(*args, visualizer=visualizer, **kwargs)

    # Start server
    with socketserver.TCPServer(("", port), handler) as httpd:
        print(f"🚀 Evolution Dashboard started!")
        print(f"📊 Open your browser and go to: http://localhost:{port}")
        print(f"📂 Evolution data directory: {output_dir}")
        print(f"⏹️  Press Ctrl+C to stop the server")
        print(f"🔄 Dashboard auto-refreshes every 30 seconds")
        print("=" * 60)

        try:
            # Open browser automatically
            webbrowser.open(f'http://localhost:{port}')
            httpd.serve_forever()
        except KeyboardInterrupt:
            print(f"\n🛑 Visualization server stopped.")

def main():
    """Main function to start the visualization"""
    import argparse

    parser = argparse.ArgumentParser(description="OpenEvolve Evolution Visualization")
    parser.add_argument("--port", type=int, default=8080, help="Port for web server")
    parser.add_argument("--output-dir", default="openevolve_output", help="Evolution output directory")
    parser.add_argument("--create-demo", action="store_true", help="Create demo data")

    args = parser.parse_args()

    if args.create_demo:
        visualizer = EvolutionVisualizer(args.output_dir)
        visualizer.create_demo_data()
        print(f"✅ Demo data created in {args.output_dir}")
        return

    start_visualization_server(args.port, args.output_dir)

if __name__ == "__main__":
    main()