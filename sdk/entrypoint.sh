#!/bin/bash
set -e

echo "============================================"
echo "  NexCore Evaluator Container"
echo "  NexVigilant — nexvigilant.com"
echo "============================================"
echo ""
echo "  API:       http://localhost:3030"
echo "  Docs:      http://localhost:3030/docs"
echo "  Health:    http://localhost:3030/health"
echo ""
echo "  Python SDK examples (run inside container):"
echo "    /app/.venv/bin/python /app/examples/quickstart.py"
echo "    /app/.venv/bin/python /app/examples/faers_pipeline.py"
echo ""
echo "  Or from host:"
echo "    pip install nexcore-sdk"
echo "    python -c \"from nexcore_sdk import NexCoreClient; print(NexCoreClient().health())\""
echo ""
echo "============================================"
echo ""

exec nexcore-api "$@"
