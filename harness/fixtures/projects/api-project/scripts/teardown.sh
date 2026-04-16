#!/bin/bash
echo "Cleaning up api-project fixture environment"
rm -rf workspace/*.tmp 2>/dev/null || true
rm -rf api-configs/*.tmp 2>/dev/null || true