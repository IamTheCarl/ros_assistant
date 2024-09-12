import sys
import os

# Needed for PyTest to import things from our application.
sys.path.insert(0, os.path.abspath(os.path.join(
    os.path.dirname(__file__), '../home_assistant_bridge')))
