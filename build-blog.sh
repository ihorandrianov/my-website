#!/bin/bash
set -e

echo "Building blog..."
cd blog
npm run build
echo "Blog built to static/blog"
