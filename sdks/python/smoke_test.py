import sys
try:
    import agentkern
    print("✅ AgentKern Python SDK loaded successfully")
    print(f"Version: {agentkern.__version__}")
except ImportError as e:
    print(f"❌ Failed to import agentkern: {e}")
    sys.exit(1)
except Exception as e:
    print(f"❌ Unexpected error: {e}")
    sys.exit(1)
