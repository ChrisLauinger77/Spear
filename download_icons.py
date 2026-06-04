import urllib.request
import os

# Make all custom icons white (#FFFFFF)
icon_colors = {
    "google.svg": "#FFFFFF",
    "github.svg": "#FFFFFF",
    "duckduckgo.svg": "#FFFFFF",
    "youtube.svg": "#FFFFFF",
    "calculator.svg": "#FFFFFF",
    "terminal.svg": "#FFFFFF",
}

icons = {
    "google.svg": "https://raw.githubusercontent.com/simple-icons/simple-icons/develop/icons/google.svg",
    "github.svg": "https://raw.githubusercontent.com/simple-icons/simple-icons/develop/icons/github.svg",
    "duckduckgo.svg": "https://raw.githubusercontent.com/simple-icons/simple-icons/develop/icons/duckduckgo.svg",
    "youtube.svg": "https://raw.githubusercontent.com/simple-icons/simple-icons/develop/icons/youtube.svg",
    "calculator.svg": "https://raw.githubusercontent.com/FortAwesome/Font-Awesome/6.x/svgs/solid/calculator.svg",
    "terminal.svg": "https://raw.githubusercontent.com/FortAwesome/Font-Awesome/6.x/svgs/solid/terminal.svg",
}

dest_dir = "icons"
os.makedirs(dest_dir, exist_ok=True)

for filename, url in icons.items():
    dest_path = os.path.join(dest_dir, filename)
    print(f"Downloading {url} to {dest_path}...")
    try:
        # Download the file content as text
        req = urllib.request.Request(url, headers={'User-Agent': 'Mozilla/5.0'})
        with urllib.request.urlopen(req) as response:
            content = response.read().decode('utf-8')
        
        # Inject fill color attribute
        color = icon_colors.get(filename)
        if color:
            if "<svg" in content:
                content = content.replace("<svg", f'<svg fill="{color}"')
        
        # Write modified content
        with open(dest_path, "w", encoding="utf-8") as f:
            f.write(content)
            
        print(f"Success & Colored: {filename}")
    except Exception as e:
        print(f"Failed to download/color {filename}: {e}")
