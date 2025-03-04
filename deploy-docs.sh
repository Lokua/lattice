#!/bin/bash
set -e

echo "Starting documentation deployment..."

# Save the current branch name
current_branch=$(git branch --show-current)
echo "Current branch: $current_branch"

# Generate docs for both the main app and derives
echo "Generating documentation for main app and derives crate..."
cargo doc --release --no-deps --document-private-items -p lattice -p derives || {
   echo "Error: Documentation generation failed"
   exit 1
}
echo "Documentation generated successfully"

echo "Creating temporary directory for docs..."
temp_dir=$(mktemp -d)
echo "Temporary directory created at: $temp_dir"

# More selective copying - exclude binary and implementation files
echo "Copying documentation to temporary directory..."
rsync -av --exclude '*.bin' \
         --exclude '*.impl' \
         --exclude 'implementors' \
         --exclude '*.desc' \
         target/doc/* "$temp_dir/"

# Clean up .DS_Store files before switching branches
echo "Cleaning up .DS_Store files..."
find . -name ".DS_Store" -delete

# Create the new branch
echo "Creating new gh-pages-temp branch..."
git checkout --orphan gh-pages-temp

# Clear the working directory completely
echo "Clearing working directory..."
git rm -rf .
git clean -fd

# Copy the docs back
echo "Copying documentation back from temporary directory..."
cp -r "$temp_dir"/* .

# Create a proper root index.html that redirects to the main crate
echo "Creating root index.html with proper redirect..."
cat > index.html << EOF
<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8">
    <title>Redirecting to lattice documentation</title>
    <meta http-equiv="refresh" content="0; URL=lattice/">
    <link rel="canonical" href="lattice/">
  </head>
  <body>
    <p>Redirecting to <a href="lattice/">lattice documentation</a>...</p>
  </body>
</html>
EOF

# Add .nojekyll file to prevent GitHub Pages from trying to process the site with Jekyll
echo "Adding .nojekyll file..."
touch .nojekyll

# Clean up temporary directory 
rm -rf "$temp_dir"
echo "Temporary directory cleaned up"

# Add and commit
echo "Committing documentation..."
git add .
git commit -m "Update documentation"

# Replace gh-pages branch
echo "Updating gh-pages branch..."
git branch -D gh-pages || true
git branch -m gh-pages
echo "Pushing to GitHub..."
git push -f origin gh-pages

# Return to original branch and clean up
echo "Returning to original branch: $current_branch"
git checkout "$current_branch"
git clean -fd  # Clean up any leftover files

# Remove all empty directories, processing deepest first
find . -depth -type d -empty -delete 

echo "Documentation deployment complete!"
echo "Please ensure GitHub Pages is configured to deploy from the gh-pages branch in your repository settings." 