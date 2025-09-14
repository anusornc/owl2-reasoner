# 🚀 OpenEvolve Real-time Evolution Visualization

Interactive web dashboard for monitoring algorithm evolution progress with real-time updates and beautiful visualizations.

## 📊 Features

### 🎯 Real-time Monitoring
- **Live evolution tracking** - Monitor progress as it happens
- **Performance metrics** - Score, correctness, performance tracking
- **Island status** - Multi-island evolution monitoring
- **Auto-refresh** - Dashboard updates every 30 seconds

### 📈 Interactive Visualizations
- **Performance charts** - Evolution progress over time
- **Feature space heatmap** - MAP-Elites coverage visualization
- **Timeline view** - Iteration-by-iteration breakdown
- **Statistics dashboard** - Key metrics at a glance

### 🎨 Modern UI
- **Responsive design** - Works on desktop and mobile
- **Beautiful styling** - Modern gradient design with smooth animations
- **Real-time indicators** - Status lights and progress indicators
- **Interactive elements** - Hover effects and smooth transitions

## 🚀 Quick Start

### Option 1: Start Dashboard with Demo Data
```bash
# Start dashboard with sample evolution data
python3 evolution_visualization.py --create-demo

# Then start the dashboard
python3 evolution_visualization.py --port 8080
```

### Option 2: Monitor Real Evolution
```bash
# Start real-time monitoring with dashboard
python3 monitor_evolution.py --port 8080

# Dashboard only mode (no monitoring)
python3 monitor_evolution.py --dashboard-only --port 8080
```

### Option 3: Monitor Existing Evolution Data
```bash
# Point to your OpenEvolve output directory
python3 monitor_evolution.py --output-dir /path/to/your/openevolve_output --port 8080
```

## 📋 Prerequisites

Install required Python packages:
```bash
pip install matplotlib numpy
```

## 🌐 Dashboard Features

### Main Dashboard View
![Dashboard Overview](https://via.placeholder.com/800x400?text=Interactive+Evolution+Dashboard)

#### 📊 Statistics Grid
- **Current Iteration**: Active evolution iteration
- **Total Programs**: Number of programs evaluated
- **Best Solutions**: Count of improved solutions found
- **Active Islands**: Number of evolution islands running

#### 📈 Performance Charts
- **Evolution Progress**: Line chart showing best score over iterations
- **Feature Space**: Heatmap showing MAP-Elites coverage
- **Real-time Updates**: Charts update as evolution progresses

#### ⏱️ Timeline View
- **Iteration History**: Last 10 iterations with detailed metrics
- **Performance Metrics**: Score, correctness, performance, time
- **Status Indicators**: Visual representation of iteration success

## 🔧 Configuration Options

### Command Line Arguments
```bash
python3 monitor_evolution.py [OPTIONS]

Options:
  --port PORT          Dashboard port (default: 8080)
  --output-dir DIR     Evolution output directory (default: openevolve_output)
  --dashboard-only     Start dashboard without monitoring
  --create-demo        Create demo evolution data
```

### Environment Variables
```bash
# Set custom port
export OPENEVOLVE_DASHBOARD_PORT=9000

# Set output directory
export OPENEVOLVE_OUTPUT_DIR=/path/to/evolution/output
```

## 📊 Data Structure

### Evolution Data Format
```json
{
  "iterations": [
    {
      "iteration": 1,
      "best_score": 0.85,
      "correctness": 0.95,
      "performance": 7500,
      "time_ns": 125,
      "timestamp": "2025-09-14T12:00:00"
    }
  ],
  "best_programs": [
    {
      "id": "prog_001",
      "generation": 1,
      "score": 0.85,
      "timestamp": "2025-09-14T12:00:00"
    }
  ],
  "islands": {
    "island_0": {"programs": 25, "best_score": 0.95},
    "island_1": {"programs": 18, "best_score": 0.87}
  },
  "current_iteration": 10,
  "status": "running"
}
```

## 🔄 Real-time Monitoring

### Log File Monitoring
The monitor automatically:
1. **Discovers log files** in `openevolve_output/logs/`
2. **Parses evolution metrics** from log entries
3. **Updates dashboard** in real-time
4. **Tracks progress** across all iterations

### Supported Log Formats
```
2025-09-14 12:00:00,123 - INFO - Iteration 1: Program abc123... completed in 15.2s
2025-09-14 12:00:00,124 - INFO - Metrics: score=0.8500, correctness=0.9500, performance=7500.00
2025-09-14 12:00:00,125 - INFO - New best program def456...
```

## 🎨 Customization

### Styling
Edit the CSS in `evolution_visualization.py`:
```python
# Change color scheme
background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);

# Modify chart styles
plt.style.use('seaborn-v0_8')
```

### Data Processing
Add custom metrics parsing in `monitor_evolution.py`:
```python
def parse_log_line(self, line):
    # Add your custom parsing logic here
    custom_match = re.search(r"Custom metric: ([\d.]+)", line)
    if custom_match:
        return {"type": "custom", "value": float(custom_match.group(1))}
```

## 🌐 Accessing the Dashboard

### Web Browser
Open your browser and navigate to:
```
http://localhost:8080
```

### Mobile Access
The dashboard is responsive and works on mobile devices:
```
http://your-server-ip:8080
```

## 🔧 Troubleshooting

### Common Issues

#### Port Already in Use
```bash
# Use a different port
python3 monitor_evolution.py --port 8081
```

#### Missing Dependencies
```bash
# Install required packages
pip install matplotlib numpy
```

#### No Evolution Data
```bash
# Create demo data
python3 evolution_visualization.py --create-demo
```

#### Log File Not Found
```bash
# Check log directory
ls -la openevolve_output/logs/

# Run evolution first to generate logs
python3 openevolve-run.py program.py evaluator.py
```

## 🚀 Production Deployment

### Using Gunicorn (Advanced)
```bash
# Install Gunicorn
pip install gunicorn

# Create WSGI app
# (Create app.py with Flask/Django application)

# Run with Gunicorn
gunicorn --bind 0.0.0.0:8080 app:app
```

### Docker Deployment
```dockerfile
FROM python:3.11-slim

WORKDIR /app
COPY requirements.txt .
RUN pip install -r requirements.txt

COPY *.py .
EXPOSE 8080

CMD ["python", "monitor_evolution.py", "--port", "8080"]
```

### Nginx Reverse Proxy
```nginx
server {
    listen 80;
    server_name your-domain.com;

    location / {
        proxy_pass http://localhost:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

## 📈 Performance Tips

### Large Evolution Runs
- **Increase log polling interval**: Modify `time.sleep(0.1)` in monitor
- **Use database backend**: For storing large amounts of evolution data
- **Implement pagination**: For timeline view with many iterations

### Memory Optimization
- **Limit stored iterations**: Keep only last N iterations in memory
- **Use data compression**: For long-running evolution processes
- **Implement cleanup**: Remove old visualization data

## 🤝 Contributing

### Adding New Visualizations
1. **Create new chart method** in `EvolutionVisualizer` class
2. **Update HTML template** to include new chart
3. **Add data parsing** for new metrics
4. **Update documentation** with new features

### Bug Reports
Please report issues with:
- Python version and OS
- Complete error messages
- Steps to reproduce
- Expected vs actual behavior

## 📄 License

This visualization system is part of OpenEvolve and follows the same license terms.

---

**Built with ❤️ for the OpenEvolve evolutionary computing framework**