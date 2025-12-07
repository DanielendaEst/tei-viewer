#!/bin/bash

# sync_projects.sh
# Syncs project folders from tei-viewer/projects to public/projects
# This ensures all XML files, images, and manifest.json are available at runtime

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECTS_SRC="${SCRIPT_DIR}/projects"
PROJECTS_DEST="${SCRIPT_DIR}/public/projects"

echo "Syncing projects from ${PROJECTS_SRC} to ${PROJECTS_DEST}..."

# Create public/projects directory if it doesn't exist
mkdir -p "${PROJECTS_DEST}"

# Loop through each directory in projects/
for project_dir in "${PROJECTS_SRC}"/*; do
    if [ -d "${project_dir}" ]; then
        project_name="$(basename "${project_dir}")"
        echo "  Syncing project: ${project_name}"

        # Create destination directory
        dest_dir="${PROJECTS_DEST}/${project_name}"
        mkdir -p "${dest_dir}"

        # Copy XML files
        if ls "${project_dir}"/*.xml 1> /dev/null 2>&1; then
            cp -v "${project_dir}"/*.xml "${dest_dir}/"
        fi

        # Copy manifest.json if it exists
        if [ -f "${project_dir}/manifest.json" ]; then
            cp -v "${project_dir}/manifest.json" "${dest_dir}/"
        else
            echo "    Warning: No manifest.json found for ${project_name}"
        fi

        # Copy commentary.html if it exists
        if [ -f "${project_dir}/commentary.html" ]; then
            cp -v "${project_dir}/commentary.html" "${dest_dir}/"
        else
            echo "    Warning: No commentary.html found for ${project_name}"
        fi

        # Copy images directory if it exists
        if [ -d "${project_dir}/images" ]; then
            mkdir -p "${dest_dir}/images"
            cp -rv "${project_dir}/images/"* "${dest_dir}/images/" 2>/dev/null || echo "    No images to copy"
        fi

        # Copy README.md if it exists
        if [ -f "${project_dir}/README.md" ]; then
            cp -v "${project_dir}/README.md" "${dest_dir}/"
        fi
    fi
done

echo "Project sync complete!"
echo ""
echo "Projects synced:"
ls -1 "${PROJECTS_DEST}"
